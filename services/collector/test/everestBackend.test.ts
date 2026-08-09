import assert from "node:assert/strict";
import net from "node:net";
import { afterEach, describe, it } from "node:test";
import { EverestTcpBackend } from "../src/everestBackend.js";
import {
  createDefaultSnapshot,
  type SimulateRequest,
} from "../src/protocol.js";

const servers: net.Server[] = [];

afterEach(async () => {
  await Promise.all(
    servers
      .splice(0)
      .map(
        (server) =>
          new Promise<void>((resolve) => server.close(() => resolve())),
      ),
  );
});

describe("Everest TCP backend", () => {
  it("translates requests and preserves reflected frame fields", async () => {
    const { port } = await startFakeEverest((request) => {
      assert.equal(request.command, "simulate_area");
      assert.equal(request.area_id, 2);
      assert.equal(request.area_sid, "CelesteGymPlayground/Playground");
      assert.equal(request.room, "1");
      assert.equal(request.dream_dash, true);
      assert.equal(request.initial_snapshot.state, 0);
      assert.equal(request.inputs[0].crouch_dash_pressed, false);
      return {
        success: true,
        states: [0, 1].map((frame) => ({
          frame,
          pos: [10 + frame, 20],
          speed: [60, 0],
          state: 0,
          facing: 1,
          dashes: 1,
          stamina: 110,
          on_ground: true,
          ducking: false,
          holding_theo: frame === 1,
          holding_glider: frame === 0,
          dead: false,
          fields: { dashAttackTimer: 0.25 },
        })),
      };
    });
    const backend = new EverestTcpBackend({
      port,
      areaId: 2,
      areaSid: "CelesteGymPlayground/Playground",
    });
    const request = validRequest();
    const states = await backend.collect(request, new AbortController().signal);
    assert.equal(states.length, 2);
    assert.equal(states[1]?.facing, true);
    assert.equal(states[1]?.holding_theo, true);
    assert.equal(states[0]?.holding_glider, true);
    assert.deepEqual(states[1]?._everest_fields, { dashAttackTimer: 0.25 });
  });

  it("reports a live mod through ping", async () => {
    const { port } = await startFakeEverest(() => ({
      success: true,
      version: "test",
    }));
    const health = await new EverestTcpBackend({ port }).health();
    assert.equal(health.ready, true);
    assert.match(health.detail ?? "", /test/);
  });

  it("injects authenticated ownership into constrained recording commands", async () => {
    const token = "capture_token_abcdefghijklmnopqrstuvwxyz";
    const { port } = await startFakeEverest((request) => {
      assert.equal(request.command, "capture_start");
      assert.equal(request.run_nonce, "run-nonce");
      assert.equal(request.process_id, 4242);
      assert.equal(request.capture_token, token);
      assert.equal(request.scenario_id, "scenario-1");
      assert.equal("recording_root" in request, false);
      assert.equal("output_path" in request, false);
      return {
        success: true,
        recording: {
          state: "active",
          scenario_id: "scenario-1",
          start_state_index: 0,
          end_state_index: 1,
          latest_state_index: -1,
          render_frame_count: 0,
          final_state_presented: false,
          repeated_presentation_count: 0,
          unpresented_update_ranges: [],
        },
      };
    });
    const backend = new EverestTcpBackend({
      port,
      runNonce: "run-nonce",
      processId: 4242,
    });
    const status = await backend.recordingStart(
      {
        capture_token: token,
        scenario_id: "scenario-1",
        start_state_index: 0,
        end_state_index: 1,
        timeout_ms: 30_000,
      },
      new AbortController().signal,
    );

    assert.equal(status.state, "active");
  });

  it("refuses recording without an authenticated nonce and child pid", async () => {
    const backend = new EverestTcpBackend({ port: 1 });
    await assert.rejects(
      () =>
        backend.recordingStatus(
          { capture_token: "c".repeat(32) },
          new AbortController().signal,
        ),
      /authenticated runNonce and exact processId/,
    );
  });

  it("keeps a persistent gym episode and converts its observation", async () => {
    const episodeId = "a".repeat(32);
    let calls = 0;
    const { port } = await startFakeEverest((request) => {
      calls++;
      if (calls === 1) {
        assert.equal(request.command, "gym_reset");
        assert.equal(request.run_nonce, "run-nonce");
        assert.equal(request.process_id, 4242);
        assert.equal(request.area_sid, "Example/Map");
        assert.equal(request.room, "start");
        assert.equal(request.seed, -123456789);
        assert.equal(request.skip_transitions, true);
        assert.equal(request.fast_mode, true);
        assert.equal(request.include_player_states, false);
        return gymResponse(episodeId, 0, true);
      }
      assert.equal(request.command, "gym_step");
      assert.equal(request.run_nonce, "run-nonce");
      assert.equal(request.process_id, 4242);
      assert.equal(request.episode_id, episodeId);
      assert.equal(request.inputs.length, 2);
      return gymResponse(episodeId, 2, false);
    });
    const backend = new EverestTcpBackend({
      port,
      runNonce: "run-nonce",
      processId: 4242,
    });
    const reset = await backend.gymReset(
      {
        area_sid: "Example/Map",
        room: "start",
        seed: -123456789,
        dream_dash: false,
        initial_snapshot: null,
        skip_transitions: true,
        max_episode_frames: 10_000,
        include_entities: true,
        include_player_states: false,
        fast_mode: true,
      },
      new AbortController().signal,
    );
    assert.equal(reset.observation?.room_geometry?.tile_size, 8);
    assert.equal(reset.observation?.entities[0]?.type, "Example.CustomEntity");
    assert.equal(reset.observation?.player.dead, false);

    const step = await backend.gymStep(
      { episode_id: episodeId, inputs: [validInput(), validInput()] },
      new AbortController().signal,
    );
    assert.equal(step.frames_executed, 2);
    assert.equal(step.observation?.episode_frame, 2);
    assert.equal(step.player_states.length, 1);
  });
});

function validInput() {
  return validRequest().inputs[0]!;
}

function gymResponse(episodeId: string, frame: number, geometry: boolean) {
  const player = {
    frame,
    pos: [16, 24],
    speed: [0, 0],
    state: 0,
    facing: 1,
    dashes: 1,
    stamina: 110,
    on_ground: true,
    ducking: false,
    can_dream_dash: false,
    holding_theo: false,
    holding_glider: false,
    dead: false,
    freeze_timer: 0,
    fields: {},
  };
  return {
    success: true,
    observation: {
      episode_id: episodeId,
      episode_frame: frame,
      area_id: 1,
      area_sid: "Example/Map",
    room: "start",
    fast_mode: frame > 0,
      player,
      ...(geometry
        ? {
            room_geometry: {
              tile_size: 8,
              bounds: [0, 0, 16, 8],
              tile_origin: [0, 0],
              width: 2,
              height: 1,
              solids: ["10"],
            },
          }
        : {}),
      entities: [
        {
          id: 1,
          type: "Example.CustomEntity",
          position: [4.5, 7.25],
          active: true,
          visible: true,
          collidable: true,
          depth: 0,
          tag: 0,
          fields: { phase: 0.5 },
        },
      ],
      terminated: false,
      truncated: false,
      success: false,
    },
    player_states: frame === 0 ? [player] : [player],
    frames_executed: frame,
  };
}

function validRequest(): SimulateRequest {
  return {
    map: Uint8Array.of(1),
    room: "1",
    dream_dash: true,
    inputs: [
      {
        move_x: 1,
        move_y: 0,
        jump_pressed: false,
        jump_held: false,
        dash_pressed: false,
        crouch_dash_pressed: false,
        grab_held: false,
        talk_pressed: false,
      },
    ],
    initial_snapshot: createDefaultSnapshot(),
    frames: 1,
  };
}

async function startFakeEverest(
  respond: (request: any) => unknown,
): Promise<{ port: number }> {
  const server = net.createServer((socket) => {
    socket.setEncoding("utf8");
    let data = "";
    socket.on("data", (chunk: string) => {
      data += chunk;
      const newline = data.indexOf("\n");
      if (newline < 0) return;
      socket.end(
        `${JSON.stringify(respond(JSON.parse(data.slice(0, newline))))}\n`,
      );
    });
  });
  servers.push(server);
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (address === null || typeof address === "string")
    throw new Error("fake server did not bind TCP");
  return { port: address.port };
}
