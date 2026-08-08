import net from "node:net";
import {
  PLAYER_STATES,
  type PlayerSnapshot,
  type SimulateRequest,
} from "./protocol.js";
import type { BackendHealth, CollectorBackend } from "./backend.js";
import type {
  RecordingAction,
  RecordingControlRequest,
  RecordingStartRequest,
  RecordingStatus,
} from "./recording.js";
import {
  toEverestInitialSnapshot,
  type GymControlRequest,
  type GymObservation,
  type GymResetRequest,
  type GymResult,
  type GymStepRequest,
} from "./gym.js";

interface EverestFrame {
  frame: number;
  pos: [number, number];
  speed: [number, number];
  state: number;
  facing: number;
  dashes: number;
  stamina: number;
  on_ground: boolean;
  ducking: boolean;
  can_dream_dash: boolean;
  holding_theo: boolean;
  holding_glider: boolean;
  dead: boolean;
  freeze_timer: number;
  fields: Record<string, unknown>;
}

interface EverestResponse {
  success: boolean;
  error?: string;
  version?: string;
  states?: EverestFrame[];
  run_nonce?: string;
  process_id?: number;
  recording?: RecordingStatus;
  observation?: Omit<GymObservation, "player"> & { player: EverestFrame };
  player_states?: EverestFrame[];
  frames_executed?: number;
}

export interface EverestTcpBackendOptions {
  host?: string;
  port?: number;
  areaId?: number;
  areaSid?: string;
  runNonce?: string;
  processId?: number;
}

export class EverestTcpBackend implements CollectorBackend {
  readonly name = "everest-tcp";
  private readonly host: string;
  private readonly port: number;
  private readonly areaId: number;
  private readonly areaSid: string | undefined;
  private readonly runNonce: string | undefined;
  private readonly processId: number | undefined;

  constructor(options: EverestTcpBackendOptions = {}) {
    this.host = options.host ?? "127.0.0.1";
    this.port = options.port ?? 32270;
    this.areaId = options.areaId ?? 1;
    this.areaSid = options.areaSid;
    this.runNonce = options.runNonce;
    this.processId = options.processId;
  }

  async collect(
    request: SimulateRequest,
    signal: AbortSignal,
  ): Promise<PlayerSnapshot[]> {
    const initial = request.initial_snapshot;
    const response = await this.send(
      {
        command: "simulate_area",
        area_id: this.areaId,
        ...(this.areaSid === undefined ? {} : { area_sid: this.areaSid }),
        room: request.room,
        dream_dash: request.dream_dash ?? false,
        inputs: request.inputs,
        initial_snapshot:
          initial === null
            ? null
            : {
                pos: initial.pos,
                speed: initial.speed,
                dashes: initial.dashes,
                stamina: initial.stamina,
                state:
                  typeof initial.state === "number"
                    ? initial.state
                    : PLAYER_STATES.indexOf(initial.state),
                facing: initial.facing === true || initial.facing === "Right",
                ducking: initial.ducking,
              },
        skip_transitions: request.skip_transitions ?? false,
        ...(request.capture_token === undefined
          ? {}
          : { capture_token: request.capture_token }),
      },
      signal,
    );
    if (!response.success || !response.states) {
      throw new Error(response.error ?? "Everest collector returned no states");
    }
    return response.states.map(convertFrame);
  }

  async gymReset(
    request: GymResetRequest,
    signal: AbortSignal,
  ): Promise<GymResult> {
    const areaSid =
      request.area_sid ??
      (request.area_id === undefined ? this.areaSid : undefined);
    return await this.sendGym(
      "reset",
      {
        area_id: request.area_id ?? this.areaId,
        ...(areaSid === undefined ? {} : { area_sid: areaSid }),
        room: request.room,
        dream_dash: request.dream_dash,
        initial_snapshot: toEverestInitialSnapshot(request.initial_snapshot),
        skip_transitions: request.skip_transitions,
        max_episode_frames: request.max_episode_frames,
        include_entities: request.include_entities,
      },
      signal,
      true,
    );
  }

  async gymStep(
    request: GymStepRequest,
    signal: AbortSignal,
  ): Promise<GymResult> {
    return await this.sendGym("step", request, signal, true);
  }

  async gymObserve(
    request: GymControlRequest,
    signal: AbortSignal,
  ): Promise<GymResult> {
    return await this.sendGym("observe", request, signal, true);
  }

  async gymClose(
    request: GymControlRequest,
    signal: AbortSignal,
  ): Promise<GymResult> {
    return await this.sendGym("close", request, signal, false);
  }

  async health(): Promise<BackendHealth> {
    try {
      const response = await this.send(
        { command: "ping" },
        AbortSignal.timeout(2_000),
      );
      if (this.runNonce !== undefined && response.run_nonce !== this.runNonce) {
        return { ready: false, detail: "Everest collector run nonce mismatch" };
      }
      if (
        this.processId !== undefined &&
        response.process_id !== this.processId
      ) {
        return {
          ready: false,
          detail: "Everest collector process id mismatch",
        };
      }
      return {
        ready: response.success,
        detail: response.success
          ? `CelesteGymCollector ${response.version ?? "unknown"} at ${this.host}:${this.port}`
          : (response.error ?? "Everest collector reported unhealthy"),
      };
    } catch (error) {
      return {
        ready: false,
        detail: error instanceof Error ? error.message : String(error),
      };
    }
  }

  async recordingStart(
    request: RecordingStartRequest,
    signal: AbortSignal,
  ): Promise<RecordingStatus> {
    return await this.sendRecording("start", request, signal);
  }

  async recordingStatus(
    request: RecordingControlRequest,
    signal: AbortSignal,
  ): Promise<RecordingStatus> {
    return await this.sendRecording("status", request, signal);
  }

  async recordingStop(
    request: RecordingControlRequest,
    signal: AbortSignal,
  ): Promise<RecordingStatus> {
    return await this.sendRecording("stop", request, signal);
  }

  async recordingFinalize(
    request: RecordingControlRequest,
    signal: AbortSignal,
  ): Promise<RecordingStatus> {
    return await this.sendRecording("finalize", request, signal);
  }

  private async sendRecording(
    action: RecordingAction,
    request: RecordingStartRequest | RecordingControlRequest,
    signal: AbortSignal,
  ): Promise<RecordingStatus> {
    if (!this.runNonce || this.processId === undefined) {
      throw new Error(
        "Everest recording requires an authenticated runNonce and exact processId",
      );
    }
    const response = await this.send(
      {
        command: `capture_${action}`,
        run_nonce: this.runNonce,
        process_id: this.processId,
        ...request,
      },
      signal,
    );
    if (!response.success || !response.recording) {
      throw new Error(
        response.error ?? "Everest collector returned no recording status",
      );
    }
    return response.recording;
  }

  private async sendGym(
    action: "reset" | "step" | "observe" | "close",
    request: object,
    signal: AbortSignal,
    requireObservation: boolean,
  ): Promise<GymResult> {
    const response = await this.send(
      { command: `gym_${action}`, ...request },
      signal,
    );
    if (!response.success) {
      throw new Error(response.error ?? `Everest gym ${action} failed`);
    }
    if (requireObservation && response.observation === undefined) {
      throw new Error(`Everest gym ${action} returned no observation`);
    }
    return {
      ...(response.observation === undefined
        ? {}
        : {
            observation: {
              ...response.observation,
              player: convertFrame(response.observation.player),
            },
          }),
      player_states: (response.player_states ?? []).map(convertFrame),
      frames_executed: response.frames_executed ?? 0,
    };
  }

  private async send(
    payload: unknown,
    signal: AbortSignal,
  ): Promise<EverestResponse> {
    if (signal.aborted) throw signal.reason;
    return await new Promise<EverestResponse>((resolve, reject) => {
      const socket = net.createConnection({ host: this.host, port: this.port });
      let data = "";
      let settled = false;
      const finish = (callback: () => void) => {
        if (settled) return;
        settled = true;
        signal.removeEventListener("abort", abort);
        socket.destroy();
        callback();
      };
      const abort = () => finish(() => reject(signal.reason));
      signal.addEventListener("abort", abort, { once: true });
      socket.setEncoding("utf8");
      socket.setNoDelay(true);
      socket.once("connect", () =>
        socket.write(`${JSON.stringify(payload)}\n`),
      );
      socket.on("data", (chunk: string) => {
        data += chunk;
        const newline = data.indexOf("\n");
        if (newline < 0) return;
        const line = data.slice(0, newline);
        finish(() => {
          try {
            resolve(JSON.parse(line) as EverestResponse);
          } catch (error) {
            reject(
              new Error(
                `Invalid JSON from Everest collector: ${String(error)}`,
              ),
            );
          }
        });
      });
      socket.once("error", (error) => finish(() => reject(error)));
      socket.once("end", () =>
        finish(() =>
          reject(new Error("Everest collector closed without a response")),
        ),
      );
    });
  }
}

function convertFrame(frame: EverestFrame): PlayerSnapshot {
  return {
    pos: frame.pos,
    speed: frame.speed,
    state: frame.state,
    facing: frame.facing > 0,
    dashes: frame.dashes,
    stamina: frame.stamina,
    on_ground: frame.on_ground,
    ducking: frame.ducking,
    can_dream_dash: frame.can_dream_dash,
    holding_theo: frame.holding_theo,
    holding_glider: frame.holding_glider,
    dead: frame.dead,
    freeze_timer: frame.freeze_timer,
    _frame: frame.frame,
    _everest_fields: frame.fields,
  };
}
