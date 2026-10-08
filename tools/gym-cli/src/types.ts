export type {
  FrameButtons,
  GymMap,
  MapEntity,
  SimInput,
  SimState,
  Vec2,
} from "../../../web/src/model.ts";

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * The simulator's own map document (Rust `celeste_physics::Map`) as decoded
 * by WASM. It is what gets sent back to the simulator, so it keeps every
 * field (transition rooms, runtime of other rooms, visuals).
 */
export interface RawMap {
  bounds: Rect;
  transition_rooms?: Rect[];
  transition_runtime?: { bounds: Rect; spawns: { x: number; y: number }[] }[];
  room_spawns?: { x: number; y: number }[];
  spawn: { x: number; y: number };
  solids: Rect[];
  entities: {
    kind: string;
    bounds: Rect;
    direction: { x: number; y: number };
    name: string;
    [key: string]: unknown;
  }[];
  source_package: string | null;
  tile_grid?: string[];
  entity_visuals?: Record<string, unknown>[];
  [key: string]: unknown;
}

/** Raw per-room contents from `audit_celeste_map` (every original name). */
export interface RoomAudit {
  name: string;
  bounds: Rect;
  spawns: { x: number; y: number }[];
  entity_names: Record<string, number>;
  trigger_names: Record<string, number>;
}
