import type { ChildProcess } from "node:child_process";

export type JsonPrimitive = string | number | boolean | null;
export type JsonValue =
  | JsonPrimitive
  | JsonValue[]
  | { readonly [key: string]: JsonValue };
export type Vector2 = readonly [number, number];
export type Rect = readonly [number, number, number, number];
export type Axis = -1 | 0 | 1;

export interface InputState {
  readonly move_x: Axis;
  readonly move_y: Axis;
  readonly jump_pressed: boolean;
  readonly jump_held: boolean;
  readonly dash_pressed: boolean;
  readonly crouch_dash_pressed: boolean;
  readonly grab_held: boolean;
  readonly talk_pressed: boolean;
}

export type PlayerStateName =
  | "Normal"
  | "Climb"
  | "Dash"
  | "Swim"
  | "Boost"
  | "RedDash"
  | "HitSquash"
  | "Launch"
  | "Pickup"
  | "DreamDash"
  | "SummitLaunch"
  | "Dummy"
  | "Frozen"
  | "ReflectionFall"
  | "StarFly"
  | "TempleFall";

export interface PlayerSnapshot {
  readonly pos: Vector2;
  readonly speed: Vector2;
  readonly state: PlayerStateName | number;
  readonly facing: boolean | "Left" | "Right";
  readonly dashes: number;
  readonly stamina: number;
  readonly on_ground: boolean;
  readonly ducking: boolean;
  readonly dead?: boolean;
  readonly can_dream_dash?: boolean;
  readonly holding_theo?: boolean;
  readonly holding_glider?: boolean;
  readonly [key: string]: unknown;
}

export interface E2EState extends PlayerSnapshot {
  readonly _frame: number;
  readonly _everest_fields: Readonly<Record<string, unknown>>;
  readonly dead: boolean;
}

export type ScenarioStatus = "active" | "candidate";
export type TargetId = "playground" | "area-1" | "area-2" | "area-4";

interface FixtureEntityBase {
  readonly id: string;
  readonly bounds: Rect;
  readonly name?: string;
}

export type FixtureEntity =
  | (FixtureEntityBase & {
      readonly kind:
        | "jump_thru"
        | "water"
        | "dream_block"
        | "bumper"
        | "puffer"
        | "angry_oshiro"
        | "seeker"
        | "snowball"
        | "cloud"
        | "bounce_block"
        | "theo_crystal"
        | "heart_gem"
        | "sandwich_lava"
        | "glider"
        | "strawberry"
        | "booster"
        | "red_booster"
        | "temple_gate"
        | "training_trigger";
    })
  | (FixtureEntityBase & {
      readonly kind: "rising_lava";
      readonly singleUse?: boolean;
    })
  | (FixtureEntityBase & {
      readonly kind: "fly_feather";
      readonly shielded?: boolean;
      readonly singleUse?: boolean;
    })
  | (FixtureEntityBase & {
      readonly kind: "badeline_boost";
      readonly nodes?: readonly Vector2[];
    })
  | (FixtureEntityBase & {
      readonly kind: "ice_ball";
      readonly nodes?: readonly Vector2[];
      readonly singleUse?: boolean;
    })
  | (FixtureEntityBase & {
      readonly kind:
        | "spikes"
        | "spring"
        | "wind"
        | "move_block"
        | "moving_solid"
        | "cassette_block";
      readonly direction: Vector2;
    })
  | (FixtureEntityBase & { readonly kind: "crystal_static_spinner" })
  | (FixtureEntityBase & {
      readonly kind: "lookout";
      readonly nodes?: readonly Vector2[];
      readonly direction?: Vector2;
    })
  | (FixtureEntityBase & {
      readonly kind: "zip_mover";
      readonly nodes: readonly [Vector2];
    });

export type FixtureEntityKind = FixtureEntity["kind"];

export interface CanonicalFixtureEntity {
  readonly id: string;
  readonly kind: FixtureEntityKind;
  readonly bounds: Rect;
  readonly direction: Vector2;
  readonly shielded: boolean;
  readonly singleUse: boolean;
  readonly nodes: readonly Vector2[];
  readonly name: string | null;
}

export interface RoomContribution {
  readonly name: string;
  readonly bounds?: Rect;
  readonly spawn?: Vector2;
  readonly additionalSpawns?: readonly Vector2[];
  readonly solids?: readonly Rect[];
  readonly entities?: readonly FixtureEntity[];
}

export interface MapPart {
  readonly id: string;
  readonly dependencies: readonly string[];
  readonly package: string;
  readonly sid: string;
  readonly rooms: readonly RoomContribution[];
}

export interface FixtureRoom {
  readonly name: string;
  readonly bounds: Rect;
  readonly spawn: Vector2;
  readonly additionalSpawns?: readonly Vector2[];
  readonly solids: readonly Rect[];
  readonly entities: readonly CanonicalFixtureEntity[];
}

export interface FixturePackage {
  readonly formatVersion: 1;
  readonly package: string;
  readonly sid: string;
  readonly rooms: readonly FixtureRoom[];
}

export interface ExternalMapTarget {
  readonly id: TargetId;
  readonly kind: "external";
  readonly areaId: number;
  readonly defaultMapFile: string;
}

export interface PlaygroundTarget {
  readonly id: "playground";
  readonly kind: "playground";
  readonly areaId: number;
  readonly areaSid: "CelesteGymPlayground/Playground";
}

export type ScenarioTarget = ExternalMapTarget | PlaygroundTarget;

export interface CoreSnapshot {
  readonly frame: number;
  readonly pos: Vector2;
  readonly speed: Vector2;
  readonly state: PlayerStateName | number;
  readonly facing: boolean | "Left" | "Right";
  readonly dashes: number;
  readonly stamina: number;
  readonly on_ground: boolean;
  readonly ducking: boolean;
  readonly dead: boolean;
}

export interface VerifyContext {
  readonly scenarioName: string;
  readonly target: ScenarioTarget;
  readonly inputs: readonly InputState[];
  readonly initialSnapshot: PlayerSnapshot;
  readonly room: string | undefined;
  readonly mapPath: string;
  readonly tracePath: string;
  readonly tolerance: number;
  near(
    actual: number | undefined,
    expected: number,
    tolerance?: number,
  ): boolean;
  field<T = unknown>(state: E2EState | undefined, name: string): T | undefined;
  core(state: E2EState | undefined): CoreSnapshot | null;
  assert(
    condition: unknown,
    message: string,
    details?: unknown,
  ): asserts condition;
}

export type ScenarioVerifier = (
  states: readonly E2EState[],
  context: VerifyContext,
) => void | Promise<void>;

export type RecordingWindow =
  | {
      readonly startFrame: number;
      readonly endFrame: number;
      readonly preRollFrames?: never;
      readonly postRollFrames?: never;
    }
  | {
      readonly startFrame?: never;
      readonly endFrame?: never;
      readonly preRollFrames: number;
      readonly postRollFrames: number;
    };

export type ScenarioRecording = RecordingWindow & {
  readonly primaryFor: readonly string[];
  readonly posterFrame?: number;
};

export interface ScenarioDefinition {
  readonly name: string;
  readonly target: ScenarioTarget;
  readonly room?: string;
  readonly status: ScenarioStatus;
  readonly tags: readonly string[];
  readonly techniqueIds: readonly string[];
  readonly mapParts: readonly MapPart[];
  readonly recording?: ScenarioRecording;
  readonly initial?: Readonly<Partial<PlayerSnapshot>>;
  readonly inputs: readonly InputState[];
  readonly verify?: ScenarioVerifier;
}

export interface SimulateRequest {
  readonly map: Uint8Array;
  readonly room?: string;
  readonly dream_dash: boolean;
  readonly inputs: readonly InputState[];
  readonly initial_snapshot: PlayerSnapshot;
  readonly frames: number;
  readonly skip_transitions: boolean;
  readonly capture_token?: string;
}

export interface ProcessIdentity {
  readonly processId: number;
  readonly executablePath: string;
  readonly creationTimeUtc: string;
}

export interface GameInstall {
  readonly gameRoot: string;
  readonly executable: string;
}

export interface PortReservation {
  readonly port: number;
  release(): Promise<void>;
}

export interface OwnedChild {
  readonly child: ChildProcess;
  readonly identity: ProcessIdentity;
}

export interface GitIdentity {
  readonly branch: string;
  readonly head: string;
}

export interface RunContext {
  readonly runId: string;
  readonly runNonce: string;
  readonly runRoot: string;
  readonly saveRoot: string;
  readonly tempRoot: string;
  readonly manifestPath: string;
  manifest: Record<string, unknown>;
}
