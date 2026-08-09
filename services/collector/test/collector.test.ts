import assert from "node:assert/strict";
import { afterEach, describe, it } from "node:test";
import { decode, encode } from "@msgpack/msgpack";
import {
  MockCollectorBackend,
  NotConfiguredBackend,
  type CollectorBackend,
} from "../src/backend.js";
import {
  CONTENT_TYPE,
  createDefaultSnapshot,
  type InputState,
  type SimulateFailure,
  type SimulateResponse,
  type SimulateSuccess,
} from "../src/protocol.js";
import {
  createCollectorServer,
  listen,
  type RunningCollectorServer,
} from "../src/server.js";
import type {
  RecordingResponse,
  RecordingStartRequest,
  RecordingStatus,
} from "../src/recording.js";
import type { GymObservation } from "../src/gym.js";

const runningServers: RunningCollectorServer[] = [];

afterEach(async () => {
  await Promise.all(runningServers.splice(0).map((server) => server.close()));
});

describe("collector HTTP service", () => {
  it("reports backend readiness without claiming the game is configured", async () => {
    const running = await start(new NotConfiguredBackend());
    const response = await fetch(`${running.url}/health`);
    const body = (await response.json()) as Record<string, unknown>;

    assert.equal(response.status, 200);
    assert.equal(body.status, "ok");
    assert.equal(body.ready, false);
    assert.equal(body.backend, "not-configured");
  });

  it("round-trips a valid MessagePack request through the explicit mock backend", async () => {
    const running = await start(new MockCollectorBackend());
    const response = await post(running, validRequest(2));
    const body = (await decodeResponse(response)) as SimulateSuccess;

    assert.equal(response.status, 200);
    assert.equal(response.headers.get("content-type"), CONTENT_TYPE);
    assert.equal(body.success, true);
    assert.equal(body.states.length, 3);
    assert.equal(body.states[2]?._collector_mock, true);
    assert.equal(body.states[2]?._frame, 2);
  });

  it("returns NOT_CONFIGURED when no real collector adapter exists", async () => {
    const running = await start(new NotConfiguredBackend());
    const response = await post(running, validRequest(1));
    const body = (await decodeResponse(response)) as SimulateFailure;

    assert.equal(response.status, 503);
    assert.equal(body.success, false);
    assert.equal(body.code, "NOT_CONFIGURED");
  });

  it("rejects non-MessagePack media types", async () => {
    const running = await start(new MockCollectorBackend());
    const response = await fetch(`${running.url}/api/simulate`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: "{}",
    });
    const body = (await decodeResponse(response)) as SimulateFailure;

    assert.equal(response.status, 415);
    assert.equal(body.code, "UNSUPPORTED_MEDIA_TYPE");
  });

  it("rejects malformed MessagePack", async () => {
    const running = await start(new MockCollectorBackend());
    const response = await fetch(`${running.url}/api/simulate`, {
      method: "POST",
      headers: { "content-type": CONTENT_TYPE },
      body: Uint8Array.of(0xc1),
    });
    const body = (await decodeResponse(response)) as SimulateFailure;

    assert.equal(response.status, 400);
    assert.equal(body.code, "INVALID_REQUEST");
  });

  it("validates axes, booleans, and the frames/input count", async () => {
    const running = await start(new MockCollectorBackend());
    const request = validRequest(1);
    const invalidInput = request.inputs[0] as unknown as Record<
      string,
      unknown
    >;
    invalidInput.move_x = 2;
    const response = await post(running, request);
    const body = (await decodeResponse(response)) as SimulateFailure;

    assert.equal(response.status, 400);
    assert.equal(body.code, "INVALID_REQUEST");
    assert.match(body.error, /move_x/);
  });

  it("defaults the additive crouch-dash input to false", async () => {
    const running = await start(new MockCollectorBackend());
    const request = validRequest(1);
    delete (request.inputs[0] as Partial<InputState>).crouch_dash_pressed;
    const response = await post(running, request);
    const body = (await decodeResponse(response)) as SimulateSuccess;

    assert.equal(response.status, 200);
    assert.equal(body.success, true);
  });

  it("converts a hung backend into a bounded timeout response", async () => {
    const backend: CollectorBackend = {
      name: "hanging-test-backend",
      async collect() {
        return await new Promise(() => undefined);
      },
    };
    const running = await start(backend, 20);
    const response = await post(running, validRequest(1));
    const body = (await decodeResponse(response)) as SimulateFailure;

    assert.equal(response.status, 504);
    assert.equal(body.code, "BACKEND_TIMEOUT");
  });

  it("rejects backend results with the wrong state count", async () => {
    const backend: CollectorBackend = {
      name: "invalid-test-backend",
      async collect() {
        return [createDefaultSnapshot()];
      },
    };
    const running = await start(backend);
    const response = await post(running, validRequest(1));
    const body = (await decodeResponse(response)) as SimulateFailure;

    assert.equal(response.status, 502);
    assert.equal(body.code, "BACKEND_ERROR");
    assert.match(body.error, /expected 2/);
  });

  it("forwards only constrained recording fields and strips output paths", async () => {
    let forwarded: RecordingStartRequest | undefined;
    const backend: CollectorBackend = {
      name: "recording-test-backend",
      async collect() {
        return [createDefaultSnapshot()];
      },
      async recordingStart(request) {
        forwarded = request;
        return recordingStatus();
      },
    };
    const running = await start(backend);
    const response = await postRecording(running, "start", {
      capture_token: "a".repeat(32),
      scenario_id: "dash-tech.2-4",
      start_state_index: 0,
      end_state_index: 10,
      timeout_ms: 30_000,
      recording_root: "C:/attacker-controlled",
      output_path: "../../escape.mp4",
    });
    const body = (await decodeResponse(response)) as RecordingResponse;

    assert.equal(response.status, 200);
    assert.equal(body.success, true);
    assert.deepEqual(forwarded, {
      capture_token: "a".repeat(32),
      scenario_id: "dash-tech.2-4",
      start_state_index: 0,
      end_state_index: 10,
      timeout_ms: 30_000,
    });
  });

  it("rejects recording token and scenario path injection", async () => {
    const running = await start(new MockCollectorBackend());
    const response = await postRecording(running, "start", {
      capture_token: "../bad",
      scenario_id: "../escape",
      start_state_index: 0,
      end_state_index: 1,
      timeout_ms: 30_000,
    });
    const body = (await decodeResponse(response)) as SimulateFailure;

    assert.equal(response.status, 400);
    assert.equal(body.code, "INVALID_REQUEST");
  });

  it("does not pretend an unconfigured backend supports recording", async () => {
    const running = await start(new MockCollectorBackend());
    const response = await postRecording(running, "status", {
      capture_token: "b".repeat(32),
    });
    const body = (await decodeResponse(response)) as SimulateFailure;

    assert.equal(response.status, 503);
    assert.equal(body.code, "RECORDING_NOT_CONFIGURED");
  });

  it("exposes reset and batched step for a persistent gym backend", async () => {
    const episodeId = "b".repeat(32);
    const observation = gymObservation(episodeId);
    const backend: CollectorBackend = {
      name: "gym-test-backend",
      async collect() {
        return [createDefaultSnapshot()];
      },
      async gymReset(request) {
        assert.equal(request.area_sid, "Example/Map");
        assert.equal(request.seed, 123456789);
        assert.equal(request.skip_transitions, true);
        assert.equal(request.fast_mode, true);
        assert.equal(request.include_player_states, false);
        return {
          observation,
          player_states: [observation.player],
          frames_executed: 0,
        };
      },
      async gymStep(request) {
        assert.equal(request.episode_id, episodeId);
        assert.equal(request.inputs.length, 2);
        return {
          observation: { ...observation, episode_frame: 2 },
          player_states: [observation.player, observation.player],
          frames_executed: 2,
        };
      },
    };
    const running = await start(backend);
    const reset = await postGym(running, "reset", {
      area_sid: "Example/Map",
      room: "start",
      seed: 123456789,
      fast_mode: true,
      include_player_states: false,
    });
    const resetBody = (await decodeResponse(reset)) as any;
    assert.equal(reset.status, 200);
    assert.equal(resetBody.observation.room_geometry.tile_size, 8);

    const step = await postGym(running, "step", {
      episode_id: episodeId,
      inputs: [validRequest(1).inputs[0], validRequest(1).inputs[0]],
    });
    const stepBody = (await decodeResponse(step)) as any;
    assert.equal(step.status, 200);
    assert.equal(stepBody.frames_executed, 2);
    assert.equal(stepBody.observation.episode_frame, 2);
  });

  it("rejects malformed gym ids and unsupported gym backends", async () => {
    const running = await start(new MockCollectorBackend());
    const malformed = await postGym(running, "step", {
      episode_id: "bad",
      inputs: [validRequest(1).inputs[0]],
    });
    assert.equal(malformed.status, 400);

    const unsupported = await postGym(running, "reset", { area_id: 1 });
    const body = (await decodeResponse(unsupported)) as SimulateFailure;
    assert.equal(unsupported.status, 503);
    assert.equal(body.code, "GYM_NOT_CONFIGURED");
  });

  it("rejects reset seeds outside the signed 32-bit protocol range", async () => {
    const running = await start(new MockCollectorBackend());
    for (const seed of [0x8000_0000, -0x8000_0001, 1.5, "1"]) {
      const response = await postGym(running, "reset", { area_id: 1, seed });
      const body = (await decodeResponse(response)) as SimulateFailure;
      assert.equal(response.status, 400);
      assert.equal(body.code, "INVALID_REQUEST");
    }
  });
});

function gymObservation(episodeId: string): GymObservation {
  return {
    episode_id: episodeId,
    episode_frame: 0,
    area_id: 1,
    area_sid: "Example/Map",
    room: "start",
    fast_mode: false,
    player: createDefaultSnapshot(),
    room_geometry: {
      tile_size: 8,
      bounds: [0, 0, 16, 8],
      tile_origin: [0, 0],
      width: 2,
      height: 1,
      solids: ["10"],
    },
    entities: [],
    terminated: false,
    truncated: false,
    success: false,
  };
}

function recordingStatus(): RecordingStatus {
  return {
    state: "active",
    scenario_id: "dash-tech.2-4",
    start_state_index: 0,
    end_state_index: 10,
    latest_state_index: -1,
    render_frame_count: 0,
    final_state_presented: false,
    repeated_presentation_count: 0,
    unpresented_update_ranges: [],
  };
}

async function start(
  backend: CollectorBackend,
  timeoutMs = 500,
): Promise<RunningCollectorServer> {
  const server = createCollectorServer({ backend, timeoutMs });
  const running = await listen(server, 0, "127.0.0.1", backend);
  runningServers.push(running);
  return running;
}

function validRequest(frames: number) {
  const input: InputState = {
    move_x: 1,
    move_y: 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: false,
    crouch_dash_pressed: false,
    grab_held: false,
    talk_pressed: false,
  };
  return {
    map: Uint8Array.of(0x43, 0x45, 0x4c),
    inputs: Array.from({ length: frames }, () => ({ ...input })),
    initial_snapshot: createDefaultSnapshot(),
    frames,
  };
}

async function post(
  running: RunningCollectorServer,
  request: ReturnType<typeof validRequest>,
): Promise<Response> {
  return await fetch(`${running.url}/api/simulate`, {
    method: "POST",
    headers: { "content-type": CONTENT_TYPE },
    body: encode(request),
  });
}

async function postRecording(
  running: RunningCollectorServer,
  action: "start" | "status" | "stop" | "finalize",
  request: Record<string, unknown>,
): Promise<Response> {
  return await fetch(`${running.url}/api/recording/${action}`, {
    method: "POST",
    headers: { "content-type": CONTENT_TYPE },
    body: encode(request),
  });
}

async function postGym(
  running: RunningCollectorServer,
  action: "reset" | "step" | "observe" | "close",
  request: Record<string, unknown>,
): Promise<Response> {
  return await fetch(`${running.url}/api/gym/${action}`, {
    method: "POST",
    headers: { "content-type": CONTENT_TYPE },
    body: encode(request),
  });
}

async function decodeResponse(response: Response): Promise<SimulateResponse> {
  return decode(
    new Uint8Array(await response.arrayBuffer()),
  ) as SimulateResponse;
}
