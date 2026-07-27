import { decode, encode } from "@msgpack/msgpack";

export const CONTENT_TYPE = "application/octet-stream";

export const PLAYER_STATES = [
  "Normal",
  "Climb",
  "Dash",
  "Swim",
  "Boost",
  "RedDash",
  "HitSquash",
  "Launch",
  "Pickup",
  "DreamDash",
  "SummitLaunch",
  "Dummy",
  "IntroWalk",
  "IntroJump",
  "IntroRespawn",
  "IntroWakeUp",
  "BirdDashTutorial",
  "Frozen",
  "ReflectionFall",
  "StarFly",
  "TempleFall",
  "CassetteFly",
  "Attract",
  "IntroMoonJump",
  "FlingBird",
  "IntroThinkForABit",
] as const;

export type PlayerState = (typeof PLAYER_STATES)[number] | number;
export type Vector2 = [number, number];

export interface InputState {
  move_x: -1 | 0 | 1;
  move_y: -1 | 0 | 1;
  jump_pressed: boolean;
  jump_held: boolean;
  dash_pressed: boolean;
  crouch_dash_pressed: boolean;
  grab_held: boolean;
}

/**
 * Known portable snapshot fields. Everest integrations may append fields as the
 * reverse-engineered snapshot contract grows, so unknown fields are preserved.
 */
export interface PlayerSnapshot {
  pos: Vector2;
  speed: Vector2;
  state: PlayerState;
  facing: boolean | "Left" | "Right";
  dashes: number;
  stamina: number;
  on_ground: boolean;
  ducking: boolean;
  can_dream_dash?: boolean;
  freeze_timer?: number;
  dash_dir?: Vector2;
  dash_attack_timer?: number;
  dash_cooldown_timer?: number;
  wall_slide_timer?: number;
  wall_slide_dir?: -1 | 0 | 1;
  jump_grace_timer?: number;
  dash_buffer_timer?: number;
  crouch_dash_buffer_timer?: number;
  var_jump_timer?: number;
  var_jump_speed?: number;
  force_move_x_timer?: number;
  wall_speed_retention_timer?: number;
  wall_boost_timer?: number;
  climb_no_move_timer?: number;
  dream_dash_can_end_timer?: number;
  launch_approach_x?: number | null;
  summit_launch_target_x?: number;
  summit_launch_particle_timer?: number;
  star_fly_timer?: number;
  star_fly_transforming?: boolean;
  star_fly_transform_frames?: number;
  star_fly_speed_lerp?: number;
  star_fly_last_dir?: Vector2;
  last_feather_target?: Vector2;
  feather_reuse_timer?: number;
  last_bumper_target?: Vector2;
  bumper_reuse_timer?: number;
  explode_launch_boost_timer?: number;
  explode_launch_boost_speed?: number;
  badeline_boost_active?: boolean;
  badeline_boost_final?: boolean;
  badeline_boost_phase?: number;
  badeline_boost_frame?: number;
  badeline_boost_start?: Vector2;
  badeline_boost_target?: Vector2;
  last_badeline_boost_target?: Vector2;
  badeline_boost_entity_origin?: Vector2;
  badeline_boost_current_position?: Vector2;
  badeline_boost_relocation_from?: Vector2;
  badeline_boost_relocation_to?: Vector2;
  badeline_boost_relocation_elapsed?: number;
  badeline_boost_relocation_duration?: number;
  badeline_boost_stage?: number;
  badeline_boost_relocating?: boolean;
  badeline_boost_collidable?: boolean;
  dummy_moving?: boolean;
  dummy_gravity?: boolean;
  dummy_friction?: boolean;
  dummy_maxspeed?: boolean;
  temple_fall_landed?: boolean;
  temple_fall_wait_frames?: number;
  reflection_fall_phase?: number;
  reflection_fall_frames?: number;
  reflection_fall_wait_timer?: number;
  ignore_jump_thrus?: boolean;
  launched?: boolean;
  [key: string]: unknown;
}

export interface SimulateRequest {
  map: Uint8Array;
  room?: string;
  dream_dash?: boolean;
  inputs: InputState[];
  initial_snapshot: PlayerSnapshot | null;
  frames: number;
  skip_transitions?: boolean;
}

export interface SimulateSuccess {
  success: true;
  states: PlayerSnapshot[];
}

export interface SimulateFailure {
  success: false;
  error: string;
  code: string;
}

export type SimulateResponse = SimulateSuccess | SimulateFailure;

export interface ValidationLimits {
  maxMapBytes: number;
  maxFrames: number;
}

export class ProtocolValidationError extends Error {
  readonly code = "INVALID_REQUEST";

  constructor(message: string) {
    super(message);
    this.name = "ProtocolValidationError";
  }
}

export function decodeSimulateRequest(
  body: Uint8Array,
  limits: ValidationLimits,
): SimulateRequest {
  let decoded: unknown;
  try {
    decoded = decode(body);
  } catch {
    throw new ProtocolValidationError("Request body is not valid MessagePack");
  }

  const root = requireRecord(decoded, "request");
  const map = root.map;
  if (!(map instanceof Uint8Array)) {
    throw new ProtocolValidationError("map must be MessagePack binary data");
  }
  if (map.byteLength === 0) {
    throw new ProtocolValidationError("map must not be empty");
  }
  if (map.byteLength > limits.maxMapBytes) {
    throw new ProtocolValidationError(
      `map exceeds the ${limits.maxMapBytes} byte limit`,
    );
  }

  if (!Array.isArray(root.inputs)) {
    throw new ProtocolValidationError("inputs must be an array");
  }
  const inputs = root.inputs.map((input, index) => validateInput(input, index));

  const frames = requireUnsignedInteger(root.frames, "frames");
  if (frames > limits.maxFrames) {
    throw new ProtocolValidationError(
      `frames exceeds the ${limits.maxFrames} frame limit`,
    );
  }
  if (inputs.length !== frames) {
    throw new ProtocolValidationError("inputs length must equal frames");
  }

  const initial = root.initial_snapshot;
  if (initial !== null && initial !== undefined) {
    validateSnapshot(initial, "initial_snapshot");
  }

  const room = root.room === undefined ? undefined : requireNonEmptyString(root.room, "room");
  const dreamDash = root.dream_dash === undefined
    ? undefined
    : requireBoolean(root.dream_dash, "dream_dash");
  return {
    map,
    ...(room === undefined ? {} : { room }),
    ...(dreamDash === undefined ? {} : { dream_dash: dreamDash }),
    inputs,
    initial_snapshot: initial == null ? null : (initial as PlayerSnapshot),
    frames,
    skip_transitions: root.skip_transitions === undefined
      ? false
      : requireBoolean(root.skip_transitions, "skip_transitions"),
  };
}

export function encodeResponse(response: SimulateResponse): Uint8Array {
  return encode(response);
}

export function validateBackendStates(
  states: unknown,
  frames: number,
): asserts states is PlayerSnapshot[] {
  if (!Array.isArray(states)) {
    throw new Error("Collector backend returned a non-array states value");
  }
  if (states.length !== frames + 1) {
    throw new Error(
      `Collector backend returned ${states.length} states; expected ${frames + 1}`,
    );
  }
  states.forEach((state, index) => validateSnapshot(state, `states[${index}]`));
}

export function createDefaultSnapshot(): PlayerSnapshot {
  return {
    pos: [0, 0],
    speed: [0, 0],
    state: "Normal",
    facing: "Right",
    dashes: 1,
    stamina: 110,
    on_ground: false,
    ducking: false,
  };
}

function validateInput(value: unknown, index: number): InputState {
  const input = requireRecord(value, `inputs[${index}]`);
  return {
    move_x: requireAxis(input.move_x, `inputs[${index}].move_x`),
    move_y: requireAxis(input.move_y, `inputs[${index}].move_y`),
    jump_pressed: requireBoolean(
      input.jump_pressed,
      `inputs[${index}].jump_pressed`,
    ),
    jump_held: requireBoolean(input.jump_held, `inputs[${index}].jump_held`),
    dash_pressed: requireBoolean(
      input.dash_pressed,
      `inputs[${index}].dash_pressed`,
    ),
    crouch_dash_pressed: input.crouch_dash_pressed === undefined
      ? false
      : requireBoolean(
        input.crouch_dash_pressed,
        `inputs[${index}].crouch_dash_pressed`,
      ),
    grab_held: requireBoolean(input.grab_held, `inputs[${index}].grab_held`),
  };
}

function validateSnapshot(value: unknown, path: string): asserts value is PlayerSnapshot {
  const snapshot = requireRecord(value, path);
  requireVector(snapshot.pos, `${path}.pos`);
  requireVector(snapshot.speed, `${path}.speed`);

  if (
    !(
      (typeof snapshot.state === "number" &&
        Number.isInteger(snapshot.state) &&
        snapshot.state >= 0 &&
        snapshot.state < PLAYER_STATES.length) ||
      (typeof snapshot.state === "string" &&
        (PLAYER_STATES as readonly string[]).includes(snapshot.state))
    )
  ) {
    throw new ProtocolValidationError(`${path}.state is not a valid player state`);
  }

  if (
    typeof snapshot.facing !== "boolean" &&
    snapshot.facing !== "Left" &&
    snapshot.facing !== "Right"
  ) {
    throw new ProtocolValidationError(
      `${path}.facing must be a boolean, Left, or Right`,
    );
  }
  requireUnsignedInteger(snapshot.dashes, `${path}.dashes`);
  requireFiniteNumber(snapshot.stamina, `${path}.stamina`);
  requireBoolean(snapshot.on_ground, `${path}.on_ground`);
  requireBoolean(snapshot.ducking, `${path}.ducking`);
}

function requireRecord(value: unknown, path: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new ProtocolValidationError(`${path} must be a map`);
  }
  return value as Record<string, unknown>;
}

function requireVector(value: unknown, path: string): Vector2 {
  if (!Array.isArray(value) || value.length !== 2) {
    throw new ProtocolValidationError(`${path} must be a two-element array`);
  }
  return [
    requireFiniteNumber(value[0], `${path}[0]`),
    requireFiniteNumber(value[1], `${path}[1]`),
  ];
}

function requireFiniteNumber(value: unknown, path: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new ProtocolValidationError(`${path} must be a finite number`);
  }
  return value;
}

function requireUnsignedInteger(value: unknown, path: string): number {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    value < 0 ||
    value > 0xffff_ffff
  ) {
    throw new ProtocolValidationError(`${path} must be a u32 integer`);
  }
  return value;
}

function requireAxis(value: unknown, path: string): -1 | 0 | 1 {
  if (value !== -1 && value !== 0 && value !== 1) {
    throw new ProtocolValidationError(`${path} must be -1, 0, or 1`);
  }
  return value;
}

function requireBoolean(value: unknown, path: string): boolean {
  if (typeof value !== "boolean") {
    throw new ProtocolValidationError(`${path} must be a boolean`);
  }
  return value;
}

function requireNonEmptyString(value: unknown, path: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new ProtocolValidationError(`${path} must be a non-empty string`);
  }
  return value;
}
