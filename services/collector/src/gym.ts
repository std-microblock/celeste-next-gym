import { decode } from "@msgpack/msgpack";
import {
  PLAYER_STATES,
  ProtocolValidationError,
  type InputState,
  type PlayerSnapshot,
  validateInputState,
  validatePlayerSnapshot,
} from "./protocol.js";

export interface GymResetRequest {
  area_id?: number;
  area_sid?: string;
  room?: string;
  dream_dash?: boolean;
  initial_snapshot: PlayerSnapshot | null;
  skip_transitions: boolean;
  max_episode_frames: number;
  include_entities: boolean;
  include_player_states: boolean;
  fast_mode: boolean;
}

export interface GymStepRequest {
  episode_id: string;
  inputs: InputState[];
}

export interface GymControlRequest {
  episode_id: string;
}

export interface GymRoomGeometry {
  tile_size: 8;
  bounds: [number, number, number, number];
  tile_origin: [number, number];
  width: number;
  height: number;
  solids: string[];
}

export interface GymEntityFrame {
  id: number;
  type: string;
  position: [number, number];
  collider?: [number, number, number, number] | null;
  collider_type?: string | null;
  speed?: [number, number] | null;
  lift_speed?: [number, number] | null;
  active: boolean;
  visible: boolean;
  collidable: boolean;
  depth: number;
  tag: number;
  fields: Record<string, unknown>;
}

export interface GymObservation {
  episode_id: string;
  episode_frame: number;
  area_id: number;
  area_sid: string;
  room: string;
  fast_mode: boolean;
  player: PlayerSnapshot;
  room_geometry?: GymRoomGeometry | null;
  entities: GymEntityFrame[];
  terminated: boolean;
  truncated: boolean;
  success: boolean;
  termination_reason?: string | null;
}

export interface GymResult {
  observation?: GymObservation;
  player_states: PlayerSnapshot[];
  frames_executed: number;
}

export type GymAction = "reset" | "step" | "observe" | "close";

export function decodeGymRequest(
  body: Uint8Array,
  action: GymAction,
  maxFrames: number,
): GymResetRequest | GymStepRequest | GymControlRequest {
  let decoded: unknown;
  try {
    decoded = decode(body);
  } catch {
    throw new ProtocolValidationError("Request body is not valid MessagePack");
  }
  const root = requireRecord(decoded, "request");
  if (action === "reset") return decodeReset(root);

  const episodeId = requireEpisodeId(root.episode_id);
  if (action === "step") {
    if (!Array.isArray(root.inputs)) {
      throw new ProtocolValidationError("inputs must be an array");
    }
    if (root.inputs.length === 0) {
      throw new ProtocolValidationError("inputs must not be empty");
    }
    if (root.inputs.length > Math.min(maxFrames, 4096)) {
      throw new ProtocolValidationError(
        `inputs exceeds the ${Math.min(maxFrames, 4096)} frame gym-step limit`,
      );
    }
    return {
      episode_id: episodeId,
      inputs: root.inputs.map((value, index) =>
        validateInputState(value, `inputs[${index}]`),
      ),
    };
  }
  return { episode_id: episodeId };
}

export function validateGymResult(
  result: GymResult,
  requireObservation: boolean,
): void {
  if (!Number.isSafeInteger(result.frames_executed) || result.frames_executed < 0) {
    throw new Error("Collector backend returned an invalid frames_executed");
  }
  if (!Array.isArray(result.player_states)) {
    throw new Error("Collector backend returned invalid player_states");
  }
  result.player_states.forEach((state, index) =>
    validatePlayerSnapshot(state, `player_states[${index}]`),
  );
  if (requireObservation && result.observation === undefined) {
    throw new Error("Collector backend returned no gym observation");
  }
  if (result.observation !== undefined) validateObservation(result.observation);
}

function decodeReset(root: Record<string, unknown>): GymResetRequest {
  const initial = root.initial_snapshot;
  if (initial !== undefined && initial !== null) {
    validatePlayerSnapshot(initial, "initial_snapshot");
  }
  const areaId =
    root.area_id === undefined
      ? undefined
      : requireUnsignedInteger(root.area_id, "area_id");
  const areaSid =
    root.area_sid === undefined
      ? undefined
      : requireNonEmptyString(root.area_sid, "area_sid");
  if (areaId === undefined && areaSid === undefined) {
    throw new ProtocolValidationError("area_id or area_sid is required");
  }
  return {
    ...(areaId === undefined ? {} : { area_id: areaId }),
    ...(areaSid === undefined ? {} : { area_sid: areaSid }),
    ...(root.room === undefined
      ? {}
      : { room: requireNonEmptyString(root.room, "room") }),
    dream_dash:
      root.dream_dash === undefined
        ? false
        : requireBoolean(root.dream_dash, "dream_dash"),
    initial_snapshot:
      initial === undefined || initial === null
        ? null
        : (initial as PlayerSnapshot),
    skip_transitions:
      root.skip_transitions === undefined
        ? true
        : requireBoolean(root.skip_transitions, "skip_transitions"),
    max_episode_frames:
      root.max_episode_frames === undefined
        ? 36_000
        : requireBoundedInteger(
            root.max_episode_frames,
            "max_episode_frames",
            1,
            10_000_000,
          ),
    include_entities:
      root.include_entities === undefined
        ? true
        : requireBoolean(root.include_entities, "include_entities"),
    include_player_states:
      root.include_player_states === undefined
        ? true
        : requireBoolean(root.include_player_states, "include_player_states"),
    fast_mode:
      root.fast_mode === undefined
        ? false
        : requireBoolean(root.fast_mode, "fast_mode"),
  };
}

function validateObservation(observation: GymObservation): void {
  requireEpisodeId(observation.episode_id);
  requireUnsignedInteger(observation.episode_frame, "observation.episode_frame");
  requireUnsignedInteger(observation.area_id, "observation.area_id");
  requireNonEmptyString(observation.room, "observation.room");
  if (typeof observation.fast_mode !== "boolean") {
    throw new Error("Collector backend returned invalid observation.fast_mode");
  }
  validatePlayerSnapshot(observation.player, "observation.player");
  if (!Array.isArray(observation.entities)) {
    throw new Error("Collector backend returned invalid observation.entities");
  }
  if (observation.room_geometry != null) {
    const geometry = observation.room_geometry;
    if (geometry.tile_size !== 8) {
      throw new Error("Collector backend returned non-8px room geometry");
    }
    if (
      !Number.isSafeInteger(geometry.width) ||
      !Number.isSafeInteger(geometry.height) ||
      geometry.width < 0 ||
      geometry.height < 0 ||
      !Array.isArray(geometry.solids) ||
      geometry.solids.length !== geometry.height ||
      geometry.solids.some(
        (row) => typeof row !== "string" || row.length !== geometry.width || /[^01]/.test(row),
      )
    ) {
      throw new Error("Collector backend returned malformed room geometry");
    }
  }
}

export function toEverestInitialSnapshot(snapshot: PlayerSnapshot | null): unknown {
  if (snapshot === null) return null;
  return {
    pos: snapshot.pos,
    speed: snapshot.speed,
    dashes: snapshot.dashes,
    stamina: snapshot.stamina,
    state:
      typeof snapshot.state === "number"
        ? snapshot.state
        : PLAYER_STATES.indexOf(snapshot.state),
    facing: snapshot.facing === true || snapshot.facing === "Right",
    ducking: snapshot.ducking,
  };
}

function requireRecord(value: unknown, path: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new ProtocolValidationError(`${path} must be a map`);
  }
  return value as Record<string, unknown>;
}

function requireEpisodeId(value: unknown): string {
  if (typeof value !== "string" || !/^[a-f0-9]{32}$/.test(value)) {
    throw new ProtocolValidationError("episode_id must be a 32-character lowercase hex id");
  }
  return value;
}

function requireNonEmptyString(value: unknown, path: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new ProtocolValidationError(`${path} must be a non-empty string`);
  }
  return value;
}

function requireBoolean(value: unknown, path: string): boolean {
  if (typeof value !== "boolean") {
    throw new ProtocolValidationError(`${path} must be a boolean`);
  }
  return value;
}

function requireUnsignedInteger(value: unknown, path: string): number {
  return requireBoundedInteger(value, path, 0, 0xffff_ffff);
}

function requireBoundedInteger(
  value: unknown,
  path: string,
  minimum: number,
  maximum: number,
): number {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    value < minimum ||
    value > maximum
  ) {
    throw new ProtocolValidationError(
      `${path} must be an integer between ${minimum} and ${maximum}`,
    );
  }
  return value;
}
