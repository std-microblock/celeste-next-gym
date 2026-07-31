import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
  type WheelEvent as ReactWheelEvent,
} from "react";
import {
  FiArrowDown,
  FiArrowLeft,
  FiArrowRight,
  FiArrowUp,
  FiBox,
  FiCircle,
  FiEye,
  FiEyeOff,
  FiMapPin,
  FiMaximize,
  FiMousePointer,
  FiPlay,
  FiRefreshCcw,
  FiRotateCcw,
  FiRotateCw,
  FiSquare,
  FiStopCircle,
  FiTrash2,
  FiX,
  FiZoomIn,
  FiZoomOut,
} from "react-icons/fi";
import {
  cameraBounds,
  cameraViewBox,
  clampCameraViewport,
  defaultCameraPosition,
  fitCameraViewport,
  pointInCameraViewport,
  zoomCameraViewport,
  type CameraBounds,
} from "../camera";
import type { EntityKind, GymMap, MapEntity, SimState } from "../model";
import type { VisualTheme } from "../visualThemes";
import { GameView } from "./GameView";

const GRID_SIZE = 8;
const PLAYER_HALF_WIDTH = 4;
const PLAYER_STANDING_HEIGHT = 11;
const PLAYER_DUCKING_HEIGHT = 6;
const MAX_TRAJECTORY_COLLIDERS = 480;

type EditorTool = "select" | "solid" | "spawn" | "erase" | `entity:${string}`;
type EditorSelection = { type: "solid" | "entity"; index: number };
type EditorSelectionList = readonly EditorSelection[];

type EditableBounds = { x: number; y: number; width: number; height: number };
type ResizeCorner = "nw" | "ne" | "se" | "sw";

interface EditorTrajectory {
  startFrame: number;
  states: SimState[];
}

function EditorIcon({ children }: { children: ReactNode }) {
  return <span className="editor-icon" aria-hidden="true">{children}</span>;
}

export function playerCollisionBounds(state: SimState): EditableBounds {
  const height = state.ducking
    ? PLAYER_DUCKING_HEIGHT
    : PLAYER_STANDING_HEIGHT;
  return {
    x: state.pos.x - PLAYER_HALF_WIDTH,
    y: state.pos.y - height,
    width: PLAYER_HALF_WIDTH * 2,
    height,
  };
}

function trajectoryPath(states: readonly SimState[], endIndex: number): string {
  return states
    .slice(0, endIndex + 1)
    .map((snapshot, index) =>
      `${index === 0 ? "M" : "L"} ${snapshot.pos.x} ${snapshot.pos.y}`,
    )
    .join(" ");
}

function trajectoryColliderIndices(endIndex: number): number[] {
  if (endIndex < 0) return [];
  const step = Math.max(
    1,
    Math.ceil((endIndex + 1) / MAX_TRAJECTORY_COLLIDERS),
  );
  const result: number[] = [];
  for (let index = 0; index <= endIndex; index += step) result.push(index);
  if (result.at(-1) !== endIndex) result.push(endIndex);
  return result;
}

interface EntityTemplate {
  id: string;
  kind: EntityKind;
  label: string;
  name: string;
  width: number;
  height: number;
  direction?: { x: number; y: number };
  nodes?: Array<{ x: number; y: number }>;
}

interface DragState {
  kind:
    | "create-solid"
    | "paint-entities"
    | "select-region"
    | "move-selection"
    | "resize-selection"
    | "move-node"
    | "pan-camera";
  start: { x: number; y: number };
  originalMap: GymMap;
  originalCamera?: CameraBounds;
  selection?: EditorSelection;
  selections?: EditorSelection[];
  corner?: ResizeCorner;
  nodeIndex?: number;
  templateId?: string;
  lastPaintPoint?: { x: number; y: number };
  paintedEntities?: MapEntity[];
  paintedKeys?: Set<string>;
}

const ENTITY_TEMPLATES: readonly EntityTemplate[] = [
  {
    id: "jump-thru",
    kind: "jump_thru",
    label: "木板",
    name: "jumpThru",
    width: 32,
    height: 8,
  },
  {
    id: "spikes-up",
    kind: "spikes",
    label: "上刺",
    name: "spikesUp",
    width: 32,
    height: 3,
    direction: { x: 0, y: -1 },
  },
  {
    id: "spikes-down",
    kind: "spikes",
    label: "下刺",
    name: "spikesDown",
    width: 32,
    height: 3,
    direction: { x: 0, y: 1 },
  },
  {
    id: "spikes-left",
    kind: "spikes",
    label: "左刺",
    name: "spikesLeft",
    width: 3,
    height: 32,
    direction: { x: -1, y: 0 },
  },
  {
    id: "spikes-right",
    kind: "spikes",
    label: "右刺",
    name: "spikesRight",
    width: 3,
    height: 32,
    direction: { x: 1, y: 0 },
  },
  {
    id: "crystal-spinner",
    kind: "crystal_static_spinner",
    label: "圆刺",
    name: "spinner",
    width: 16,
    height: 12,
  },
  {
    id: "water",
    kind: "water",
    label: "水",
    name: "water",
    width: 32,
    height: 32,
  },
  {
    id: "dream-block",
    kind: "dream_block",
    label: "梦块",
    name: "dreamBlock",
    width: 32,
    height: 32,
  },
  {
    id: "booster",
    kind: "booster",
    label: "绿泡",
    name: "booster",
    width: 16,
    height: 16,
  },
  {
    id: "red-booster",
    kind: "red_booster",
    label: "红泡",
    name: "redBooster",
    width: 16,
    height: 16,
  },
  {
    id: "spring",
    kind: "spring",
    label: "弹簧",
    name: "spring",
    width: 16,
    height: 8,
    direction: { x: 0, y: -1 },
  },
  {
    id: "strawberry",
    kind: "strawberry",
    label: "草莓",
    name: "strawberry",
    width: 16,
    height: 16,
  },
  {
    id: "refill",
    kind: "refill",
    label: "回填钻石",
    name: "refill",
    width: 16,
    height: 16,
  },
  {
    id: "falling-block",
    kind: "falling_block",
    label: "下落块",
    name: "fallingBlock",
    width: 32,
    height: 16,
    direction: { x: 1, y: 0 },
  },
  {
    id: "fly-feather",
    kind: "fly_feather",
    label: "羽毛",
    name: "infiniteStar",
    width: 20,
    height: 20,
  },
  {
    id: "bumper",
    kind: "bumper",
    label: "碰碰球",
    name: "bigSpinner",
    width: 24,
    height: 24,
  },
  {
    id: "theo-crystal",
    kind: "theo_crystal",
    label: "Theo 水晶",
    name: "theoCrystal",
    width: 8,
    height: 10,
  },
  {
    id: "glider",
    kind: "glider",
    label: "水母",
    name: "glider",
    width: 8,
    height: 10,
  },
  {
    id: "zip-mover",
    kind: "zip_mover",
    label: "Zip Mover",
    name: "zipMover",
    width: 32,
    height: 16,
    nodes: [{ x: 64, y: 0 }],
  },
] as const;

const SPIKE_DIRECTIONS = [
  { label: "上", name: "spikesUp", direction: { x: 0, y: -1 } },
  { label: "下", name: "spikesDown", direction: { x: 0, y: 1 } },
  { label: "左", name: "spikesLeft", direction: { x: -1, y: 0 } },
  { label: "右", name: "spikesRight", direction: { x: 1, y: 0 } },
] as const;

const SPINNER_VARIANTS = [
  { id: "theme", label: "主题" },
  { id: "blue", label: "蓝" },
  { id: "red", label: "红" },
  { id: "purple", label: "紫" },
  { id: "rainbow", label: "彩虹" },
] as const;

export interface MapEditorProps {
  map: GymMap;
  state: SimState;
  frame: number;
  states?: readonly SimState[];
  stateFrameOffset?: number;
  theme: VisualTheme;
  experiencing: boolean;
  ready: boolean;
  onChange: (map: GymMap) => void;
  onExperienceChange: (experiencing: boolean) => void;
  onResetExperience: () => void;
}

export function snapToGrid(
  value: number,
  origin = 0,
  grid = GRID_SIZE,
): number {
  return Math.round((value - origin) / grid) * grid + origin;
}

/**
 * Snap an editor coordinate. When {@link useGrid} is true the value snaps to
 * the 8px Celeste tile grid; otherwise it snaps to whole pixels so objects can
 * be placed freely while still keeping integer coordinates.
 */
export function snapCoordinate(
  value: number,
  origin = 0,
  useGrid: boolean,
): number {
  return useGrid ? snapToGrid(value, origin, GRID_SIZE) : Math.round(value);
}

export function selectionKey(selection: EditorSelection): string {
  return `${selection.type}:${selection.index}`;
}

export function selectionInList(
  list: EditorSelectionList,
  selection: EditorSelection,
): boolean {
  return list.some(
    (candidate) => selectionKey(candidate) === selectionKey(selection),
  );
}

export function selectionBoundsUnion(
  map: GymMap,
  selections: EditorSelectionList,
): EditableBounds | null {
  if (!selections.length) return null;
  let left = Infinity;
  let top = Infinity;
  let right = -Infinity;
  let bottom = -Infinity;
  for (const item of selections) {
    const itemBounds =
      item.type === "solid"
        ? map.solids[item.index]
        : map.entities[item.index]?.bounds;
    if (!itemBounds) continue;
    left = Math.min(left, itemBounds.x);
    top = Math.min(top, itemBounds.y);
    right = Math.max(right, itemBounds.x + itemBounds.width);
    bottom = Math.max(bottom, itemBounds.y + itemBounds.height);
  }
  if (!Number.isFinite(left)) return null;
  return { x: left, y: top, width: right - left, height: bottom - top };
}

export function deleteSelections(
  map: GymMap,
  selections: EditorSelectionList,
): GymMap {
  const solidIndices = new Set(
    selections.filter((item) => item.type === "solid").map((item) => item.index),
  );
  const entityIndices = new Set(
    selections
      .filter((item) => item.type === "entity")
      .map((item) => item.index),
  );
  return {
    ...map,
    solids: map.solids.filter((_, index) => !solidIndices.has(index)),
    entities: map.entities.filter((_, index) => !entityIndices.has(index)),
  };
}

function intersects(
  a: EditableBounds,
  b: EditableBounds,
): boolean {
  return (
    a.x < b.x + b.width &&
    a.x + a.width > b.x &&
    a.y < b.y + b.height &&
    a.y + a.height > b.y
  );
}

export function objectsInRegion(
  map: GymMap,
  region: EditableBounds,
): EditorSelection[] {
  const result: EditorSelection[] = [];
  map.solids.forEach((solid, index) => {
    if (intersects(solid, region)) result.push({ type: "solid", index });
  });
  map.entities.forEach((entity, index) => {
    if (intersects(editorEntityHitBounds(entity), region)) {
      result.push({ type: "entity", index });
    }
  });
  return result;
}

export function createEditorEntity(
  templateIdOrKind: string,
  x: number,
  y: number,
): MapEntity | null {
  const template =
    ENTITY_TEMPLATES.find((candidate) => candidate.id === templateIdOrKind) ??
    ENTITY_TEMPLATES.find((candidate) => candidate.kind === templateIdOrKind);
  if (!template) return null;
  const bounds = {
    x:
      x +
      (template.kind === "spikes" && template.direction?.x === -1 ? -3 : 0),
    y:
      y +
      (template.kind === "spikes" && template.direction?.y === -1 ? -3 : 0),
    width: template.width,
    height: template.height,
  };
  return {
    kind: template.kind,
    bounds,
    direction: template.direction ? { ...template.direction } : { x: 0, y: 0 },
    ...(template.nodes
      ? {
          nodes: template.nodes.map((node) => ({
            x: x + node.x,
            y: y + node.y,
          })),
        }
      : {}),
    name: template.name,
  };
}

export function createEditorBrushEntity(
  templateId: string,
  x: number,
  y: number,
  spinnerVariant = "theme",
): MapEntity | null {
  const entity = createEditorEntity(templateId, x, y);
  if (!entity) return null;
  if (entity.kind === "spikes") {
    entity.bounds = {
      ...entity.bounds,
      width: entity.direction.y === 0 ? 3 : GRID_SIZE,
      height: entity.direction.y === 0 ? GRID_SIZE : 3,
    };
  }
  if (entity.kind === "crystal_static_spinner" && spinnerVariant !== "theme") {
    entity.variant = spinnerVariant;
  }
  return entity;
}

export function entityBrushPoints(
  from: { x: number; y: number },
  to: { x: number; y: number },
  map: GymMap,
  useGrid = true,
): Array<{ x: number; y: number }> {
  const step = useGrid ? GRID_SIZE : 1;
  let x0 = Math.round(
    (snapCoordinate(from.x, map.bounds.x, useGrid) - map.bounds.x) / step,
  );
  let y0 = Math.round(
    (snapCoordinate(from.y, map.bounds.y, useGrid) - map.bounds.y) / step,
  );
  const x1 = Math.round(
    (snapCoordinate(to.x, map.bounds.x, useGrid) - map.bounds.x) / step,
  );
  const y1 = Math.round(
    (snapCoordinate(to.y, map.bounds.y, useGrid) - map.bounds.y) / step,
  );
  const points: Array<{ x: number; y: number }> = [];
  const dx = Math.abs(x1 - x0);
  const sx = x0 < x1 ? 1 : -1;
  const dy = -Math.abs(y1 - y0);
  const sy = y0 < y1 ? 1 : -1;
  let error = dx + dy;

  while (true) {
    points.push({
      x: map.bounds.x + x0 * step,
      y: map.bounds.y + y0 * step,
    });
    if (x0 === x1 && y0 === y1) break;
    const twiceError = error * 2;
    if (twiceError >= dy) {
      error += dy;
      x0 += sx;
    }
    if (twiceError <= dx) {
      error += dx;
      y0 += sy;
    }
  }
  return points;
}

function entityPaintKey(entity: MapEntity): string {
  return [
    entity.kind,
    entity.bounds.x,
    entity.bounds.y,
    entity.bounds.width,
    entity.bounds.height,
    entity.direction.x,
    entity.direction.y,
  ].join(":");
}

export function setEditorSpikeDirection(
  entity: MapEntity,
  direction: { x: number; y: number },
): MapEntity {
  const config = SPIKE_DIRECTIONS.find(
    (candidate) =>
      candidate.direction.x === direction.x &&
      candidate.direction.y === direction.y,
  );
  if (entity.kind !== "spikes" || !config) return entity;
  const wasHorizontal = Math.abs(entity.direction.y) > 0;
  const willBeHorizontal = Math.abs(direction.y) > 0;
  const length = wasHorizontal ? entity.bounds.width : entity.bounds.height;
  const anchorX = entity.bounds.x + (entity.direction.x < 0 ? 3 : 0);
  const anchorY = entity.bounds.y + (entity.direction.y < 0 ? 3 : 0);
  return {
    ...entity,
    name: config.name,
    direction: { ...direction },
    bounds: {
      x: anchorX + (direction.x < 0 ? -3 : 0),
      y: anchorY + (direction.y < 0 ? -3 : 0),
      width: willBeHorizontal ? length : 3,
      height: willBeHorizontal ? 3 : length,
    },
  };
}

export function editorEntityHitBounds(entity: MapEntity): MapEntity["bounds"] {
  const box = entity.bounds;
  if (entity.kind !== "spikes") return box;
  if (Math.abs(entity.direction.y) > 0)
    return { x: box.x, y: box.y - 3, width: box.width, height: 9 };
  return { x: box.x - 3, y: box.y, width: 9, height: box.height };
}

export function resizeEditorBounds(
  bounds: EditableBounds,
  corner: ResizeCorner,
  point: { x: number; y: number },
  map: GymMap,
  useGrid = true,
): EditableBounds {
  const left = bounds.x;
  const top = bounds.y;
  const right = bounds.x + bounds.width;
  const bottom = bounds.y + bounds.height;
  const snappedX = snapCoordinate(point.x, map.bounds.x, useGrid);
  const snappedY = snapCoordinate(point.y, map.bounds.y, useGrid);
  const minimum = useGrid ? GRID_SIZE : 1;
  const nextLeft =
    corner === "nw" || corner === "sw"
      ? Math.min(snappedX, right - minimum)
      : left;
  const nextRight =
    corner === "ne" || corner === "se"
      ? Math.max(snappedX, left + minimum)
      : right;
  const nextTop =
    corner === "nw" || corner === "ne"
      ? Math.min(snappedY, bottom - minimum)
      : top;
  const nextBottom =
    corner === "sw" || corner === "se"
      ? Math.max(snappedY, top + minimum)
      : bottom;
  return {
    x: Math.max(map.bounds.x, nextLeft),
    y: Math.max(map.bounds.y, nextTop),
    width:
      Math.min(map.bounds.x + map.bounds.width, nextRight) -
      Math.max(map.bounds.x, nextLeft),
    height:
      Math.min(map.bounds.y + map.bounds.height, nextBottom) -
      Math.max(map.bounds.y, nextTop),
  };
}

function selectionBounds(
  map: GymMap,
  selection: EditorSelection | null,
): EditableBounds | null {
  if (!selection) return null;
  return selection.type === "solid"
    ? (map.solids[selection.index] ?? null)
    : (map.entities[selection.index]?.bounds ?? null);
}

function replaceSelectionBounds(
  map: GymMap,
  selection: EditorSelection,
  bounds: EditableBounds,
): GymMap {
  if (selection.type === "solid") {
    const solids = map.solids.map((solid, index) =>
      index === selection.index ? bounds : solid,
    );
    return { ...map, solids };
  }
  const entities = map.entities.map((entity, index) =>
    index === selection.index ? { ...entity, bounds } : entity,
  );
  return { ...map, entities };
}

function deleteSelection(map: GymMap, selection: EditorSelection): GymMap {
  return selection.type === "solid"
    ? {
        ...map,
        solids: map.solids.filter((_, index) => index !== selection.index),
      }
    : {
        ...map,
        entities: map.entities.filter((_, index) => index !== selection.index),
      };
}

function pointInMap(
  clientX: number,
  clientY: number,
  svg: SVGSVGElement,
  map: GymMap,
  camera: CameraBounds,
): { x: number; y: number } {
  const rect = svg.getBoundingClientRect();
  const point = pointInCameraViewport(
    clientX,
    clientY,
    rect,
    clampCameraViewport(map, camera),
  );
  return {
    x: Math.max(
      map.bounds.x,
      Math.min(map.bounds.x + map.bounds.width, point.x),
    ),
    y: Math.max(
      map.bounds.y,
      Math.min(map.bounds.y + map.bounds.height, point.y),
    ),
  };
}

function normalizedRect(
  from: { x: number; y: number },
  to: { x: number; y: number },
  map: GymMap,
  useGrid = true,
): EditableBounds {
  const x1 = snapCoordinate(from.x, map.bounds.x, useGrid);
  const y1 = snapCoordinate(from.y, map.bounds.y, useGrid);
  const x2 = snapCoordinate(to.x, map.bounds.x, useGrid);
  const y2 = snapCoordinate(to.y, map.bounds.y, useGrid);
  return {
    x: Math.min(x1, x2),
    y: Math.min(y1, y2),
    width: Math.max(useGrid ? GRID_SIZE : 1, Math.abs(x2 - x1)),
    height: Math.max(useGrid ? GRID_SIZE : 1, Math.abs(y2 - y1)),
  };
}

function normalizedRegion(
  from: { x: number; y: number },
  to: { x: number; y: number },
): EditableBounds {
  return {
    x: Math.min(from.x, to.x),
    y: Math.min(from.y, to.y),
    width: Math.abs(to.x - from.x),
    height: Math.abs(to.y - from.y),
  };
}

function clampToMap(
  bounds: EditableBounds,
  map: GymMap,
): EditableBounds {
  return {
    ...bounds,
    x: Math.max(
      map.bounds.x,
      Math.min(
        map.bounds.x + map.bounds.width - bounds.width,
        bounds.x,
      ),
    ),
    y: Math.max(
      map.bounds.y,
      Math.min(
        map.bounds.y + map.bounds.height - bounds.height,
        bounds.y,
      ),
    ),
  };
}

function moveSelections(
  map: GymMap,
  selections: EditorSelectionList,
  dx: number,
  dy: number,
  entityGrid: boolean,
): GymMap {
  const keys = new Set(selections.map(selectionKey));
  return {
    ...map,
    solids: map.solids.map((solid, index) =>
      keys.has(selectionKey({ type: "solid", index }))
        ? clampToMap(
            {
              ...solid,
              x: snapToGrid(solid.x + dx, map.bounds.x),
              y: snapToGrid(solid.y + dy, map.bounds.y),
            },
            map,
          )
        : solid,
    ),
    entities: map.entities.map((entity, index) =>
      keys.has(selectionKey({ type: "entity", index }))
        ? {
            ...entity,
            bounds: clampToMap(
              {
                ...entity.bounds,
                x: snapCoordinate(
                  entity.bounds.x + dx,
                  map.bounds.x,
                  entityGrid,
                ),
                y: snapCoordinate(
                  entity.bounds.y + dy,
                  map.bounds.y,
                  entityGrid,
                ),
              },
              map,
            ),
          }
        : entity,
    ),
  };
}

function toolLabel(tool: EditorTool): string {
  if (tool === "select") return "选择";
  if (tool === "solid") return "实心块";
  if (tool === "spawn") return "出生点";
  if (tool === "erase") return "删除";
  const templateId = tool.slice("entity:".length);
  return (
    ENTITY_TEMPLATES.find((template) => template.id === templateId)?.label ??
    templateId
  );
}

export function MapEditor({
  map,
  state,
  frame,
  states = [],
  stateFrameOffset = 0,
  theme,
  experiencing,
  ready,
  onChange,
  onExperienceChange,
  onResetExperience,
}: MapEditorProps) {
  const [tool, setTool] = useState<EditorTool>("select");
  const [selected, setSelected] = useState<EditorSelection[]>([]);
  const [marquee, setMarquee] = useState<EditableBounds | null>(null);
  const [draft, setDraft] = useState<EditableBounds | null>(null);
  const [cameraViewport, setCameraViewport] = useState<CameraBounds>(() =>
    cameraBounds(defaultCameraPosition(map)),
  );
  const [historyRevision, setHistoryRevision] = useState(0);
  const [trajectory, setTrajectory] = useState<EditorTrajectory | null>(null);
  const [trajectoryRecording, setTrajectoryRecording] = useState(false);
  const [trajectoryFrame, setTrajectoryFrame] = useState(0);
  const [showAllTrajectory, setShowAllTrajectory] = useState(false);
  const [spinnerBrushVariant, setSpinnerBrushVariant] = useState("theme");
  const capturedThroughFrame = useRef<number | null>(null);
  const drag = useRef<DragState | null>(null);
  const undoStack = useRef<GymMap[]>([]);
  const redoStack = useRef<GymMap[]>([]);
  const selection = selected.at(-1) ?? null;
  const selectionCount = selected.length;
  const bounds = selectionBounds(map, selection);
  const selectionUnion = selectionBoundsUnion(map, selected);
  const selectedEntity =
    selection?.type === "entity" ? map.entities[selection.index] : null;
  const stats = useMemo(
    () => ({ solids: map.solids.length, entities: map.entities.length }),
    [map.entities.length, map.solids.length],
  );
  const reviewedTrajectoryState = trajectory?.states[
    Math.min(trajectoryFrame, Math.max(0, trajectory.states.length - 1))
  ];
  const renderedState =
    !experiencing && reviewedTrajectoryState
      ? reviewedTrajectoryState
      : state;
  const renderedFrame =
    !experiencing && trajectory
      ? trajectory.startFrame + trajectoryFrame
      : frame;
  const renderedStates =
    !experiencing && trajectory ? trajectory.states : states;
  const renderedStateFrameOffset =
    !experiencing && trajectory ? trajectory.startFrame : stateFrameOffset;
  const trajectoryEndIndex = trajectory
    ? showAllTrajectory
      ? trajectory.states.length - 1
      : Math.min(trajectoryFrame, trajectory.states.length - 1)
    : -1;
  const trajectoryLine = useMemo(
    () => trajectoryPath(trajectory?.states ?? [], trajectoryEndIndex),
    [trajectory, trajectoryEndIndex],
  );
  const colliderIndices = useMemo(
    () =>
      showAllTrajectory
        ? trajectoryColliderIndices(trajectoryEndIndex)
        : trajectoryEndIndex >= 0
          ? [trajectoryEndIndex]
          : [],
    [showAllTrajectory, trajectoryEndIndex],
  );

  useEffect(() => {
    setCameraViewport((viewport) => clampCameraViewport(map, viewport));
  }, [map.bounds.height, map.bounds.width, map.bounds.x, map.bounds.y, map.room]);

  useEffect(() => {
    setTrajectory(null);
    setTrajectoryRecording(false);
    setTrajectoryFrame(0);
    setShowAllTrajectory(false);
    capturedThroughFrame.current = null;
  }, [map.room]);

  useEffect(() => {
    if (!trajectoryRecording) return;
    const capturedThrough = capturedThroughFrame.current;
    if (capturedThrough === null || frame < capturedThrough) return;
    const appended: SimState[] = [];
    for (let target = capturedThrough + 1; target <= frame; target += 1) {
      const snapshot =
        states[target - stateFrameOffset] ?? (target === frame ? state : null);
      if (snapshot) appended.push(structuredClone(snapshot));
    }
    if (!appended.length) return;
    capturedThroughFrame.current = frame;
    setTrajectory((current) =>
      current
        ? { ...current, states: [...current.states, ...appended] }
        : current,
    );
    setTrajectoryFrame((current) => current + appended.length);
  }, [frame, state, stateFrameOffset, states, trajectoryRecording]);

  const startTrajectoryRecording = () => {
    const initial = structuredClone(
      states[frame - stateFrameOffset] ?? state,
    );
    setTrajectory({ startFrame: frame, states: [initial] });
    setTrajectoryFrame(0);
    setShowAllTrajectory(false);
    capturedThroughFrame.current = frame;
    setTrajectoryRecording(true);
  };

  const stopTrajectoryRecording = () => {
    setTrajectoryRecording(false);
    capturedThroughFrame.current = null;
    setTrajectoryFrame(Math.max(0, (trajectory?.states.length ?? 1) - 1));
  };

  const toggleExperience = () => {
    if (experiencing && trajectoryRecording) stopTrajectoryRecording();
    onExperienceChange(!experiencing);
  };

  const resetExperience = () => {
    if (trajectoryRecording) stopTrajectoryRecording();
    onResetExperience();
  };

  const moveCamera = (x: number, y: number) =>
    setCameraViewport((viewport) =>
      clampCameraViewport(map, {
        ...viewport,
        x: viewport.x + x,
        y: viewport.y + y,
      }),
    );

  const zoomCamera = (factor: number, focus?: { x: number; y: number }) =>
    setCameraViewport((viewport) =>
      zoomCameraViewport(map, viewport, factor, focus),
    );

  const resetCamera = () =>
    setCameraViewport(cameraBounds(defaultCameraPosition(map)));

  const fitCamera = () =>
    setCameraViewport(
      fitCameraViewport(map, cameraViewport.width / cameraViewport.height),
    );

  const rememberAndChange = (next: GymMap) => {
    undoStack.current.push(structuredClone(map));
    redoStack.current = [];
    setHistoryRevision((value) => value + 1);
    onChange(next);
  };

  const finishContinuousChange = (originalMap: GymMap) => {
    undoStack.current.push(structuredClone(originalMap));
    redoStack.current = [];
    setHistoryRevision((value) => value + 1);
  };

  const undo = () => {
    const previous = undoStack.current.pop();
    if (!previous) return;
    redoStack.current.push(structuredClone(map));
    setSelected([]);
    setHistoryRevision((value) => value + 1);
    onChange(previous);
  };

  const redo = () => {
    const next = redoStack.current.pop();
    if (!next) return;
    undoStack.current.push(structuredClone(map));
    setSelected([]);
    setHistoryRevision((value) => value + 1);
    onChange(next);
  };

  const chooseTool = (next: EditorTool) => {
    setTool(next);
    if (next !== "select") setSelected([]);
  };

  const beginSelectionDrag = (
    event: ReactPointerEvent<SVGRectElement>,
    nextSelection: EditorSelection,
  ) => {
    if (tool === "erase") {
      event.stopPropagation();
      rememberAndChange(deleteSelection(map, nextSelection));
      setSelected([]);
      return;
    }
    if (tool !== "select") return;
    event.stopPropagation();
    const svg = event.currentTarget.ownerSVGElement;
    if (!svg) return;
    const dragStart = pointInMap(
      event.clientX,
      event.clientY,
      svg,
      map,
      cameraViewport,
    );
    let nextSelected: EditorSelection[];
    if (event.ctrlKey) {
      if (selectionInList(selected, nextSelection)) {
        setSelected(
          selected.filter(
            (candidate) =>
              selectionKey(candidate) !== selectionKey(nextSelection),
          ),
        );
        return;
      }
      nextSelected = [...selected, nextSelection];
    } else if (selectionInList(selected, nextSelection)) {
      nextSelected = [...selected];
    } else {
      nextSelected = [nextSelection];
    }
    drag.current = {
      kind: "move-selection",
      start: dragStart,
      originalMap: structuredClone(map),
      selections: nextSelected,
    };
    setSelected(nextSelected);
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const beginResize = (
    event: ReactPointerEvent<SVGRectElement>,
    corner: ResizeCorner,
  ) => {
    event.stopPropagation();
    if (!selection || tool !== "select") return;
    const svg = event.currentTarget.ownerSVGElement;
    if (!svg) return;
    drag.current = {
      kind: "resize-selection",
      corner,
      start: pointInMap(
        event.clientX,
        event.clientY,
        svg,
        map,
        cameraViewport,
      ),
      originalMap: structuredClone(map),
      selection,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const beginNodeDrag = (
    event: ReactPointerEvent<SVGCircleElement>,
    nodeIndex: number,
  ) => {
    event.stopPropagation();
    if (!selection || selection.type !== "entity" || tool !== "select") return;
    const svg = event.currentTarget.ownerSVGElement;
    if (!svg) return;
    drag.current = {
      kind: "move-node",
      nodeIndex,
      start: pointInMap(
        event.clientX,
        event.clientY,
        svg,
        map,
        cameraViewport,
      ),
      originalMap: structuredClone(map),
      selection,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const pointerDown = (event: ReactPointerEvent<SVGSVGElement>) => {
    if (
      !tool.startsWith("entity:") &&
      event.target !== event.currentTarget &&
      !(event.target as SVGElement).matches?.(
        '[data-editor-background="true"]',
      )
    )
      return;
    const point = pointInMap(
      event.clientX,
      event.clientY,
      event.currentTarget,
      map,
      cameraViewport,
    );
    if (tool === "select") {
      if (event.ctrlKey) {
        drag.current = {
          kind: "select-region",
          start: point,
          originalMap: structuredClone(map),
        };
        setMarquee({ x: point.x, y: point.y, width: 0, height: 0 });
      } else {
        setSelected([]);
        drag.current = {
          kind: "pan-camera",
          start: { x: event.clientX, y: event.clientY },
          originalMap: structuredClone(map),
          originalCamera: cameraViewport,
        };
      }
      event.currentTarget.setPointerCapture(event.pointerId);
    } else if (tool === "solid") {
      drag.current = {
        kind: "create-solid",
        start: point,
        originalMap: structuredClone(map),
      };
      setDraft(normalizedRect(point, point, map, true));
      event.currentTarget.setPointerCapture(event.pointerId);
    } else if (tool === "spawn") {
      rememberAndChange({
        ...map,
        spawn: {
          x: snapCoordinate(point.x, map.bounds.x, event.ctrlKey),
          y: snapCoordinate(point.y, map.bounds.y, event.ctrlKey),
        },
      });
    } else if (tool.startsWith("entity:")) {
      const templateId = tool.slice("entity:".length);
      drag.current = {
        kind: "paint-entities",
        start: point,
        originalMap: structuredClone(map),
        templateId,
        paintedEntities: [],
        paintedKeys: new Set(map.entities.map(entityPaintKey)),
      };
      const entity = createEditorBrushEntity(
        templateId,
        snapCoordinate(point.x, map.bounds.x, event.ctrlKey),
        snapCoordinate(point.y, map.bounds.y, event.ctrlKey),
        spinnerBrushVariant,
      );
      if (!entity) {
        drag.current = null;
        return;
      }
      drag.current.lastPaintPoint = point;
      const key = entityPaintKey(entity);
      if (!drag.current.paintedKeys?.has(key)) {
        drag.current.paintedEntities = [entity];
        drag.current.paintedKeys?.add(key);
        onChange({ ...map, entities: [...map.entities, entity] });
      }
      event.currentTarget.setPointerCapture(event.pointerId);
    }
  };

  const pointerMove = (event: ReactPointerEvent<SVGSVGElement>) => {
    const currentDrag = drag.current;
    if (!currentDrag) return;
    if (currentDrag.kind === "pan-camera" && currentDrag.originalCamera) {
      const rect = event.currentTarget.getBoundingClientRect();
      const scale = Math.min(
        rect.width / currentDrag.originalCamera.width,
        rect.height / currentDrag.originalCamera.height,
      );
      if (scale > 0) {
        setCameraViewport(
          clampCameraViewport(map, {
            ...currentDrag.originalCamera,
            x:
              currentDrag.originalCamera.x -
              (event.clientX - currentDrag.start.x) / scale,
            y:
              currentDrag.originalCamera.y -
              (event.clientY - currentDrag.start.y) / scale,
          }),
        );
      }
      return;
    }
    const point = pointInMap(
      event.clientX,
      event.clientY,
      event.currentTarget,
      map,
      cameraViewport,
    );
    if (currentDrag.kind === "select-region") {
      setMarquee(normalizedRegion(currentDrag.start, point));
      return;
    }
    if (currentDrag.kind === "create-solid") {
      setDraft(normalizedRect(currentDrag.start, point, map, true));
      return;
    }
    if (
      currentDrag.kind === "paint-entities" &&
      currentDrag.templateId &&
      currentDrag.lastPaintPoint &&
      currentDrag.paintedEntities &&
      currentDrag.paintedKeys
    ) {
      const painted = [...currentDrag.paintedEntities];
      let changed = false;
      for (const anchor of entityBrushPoints(
        currentDrag.lastPaintPoint,
        point,
        map,
        event.ctrlKey,
      )) {
        const entity = createEditorBrushEntity(
          currentDrag.templateId,
          anchor.x,
          anchor.y,
          spinnerBrushVariant,
        );
        if (!entity) continue;
        const key = entityPaintKey(entity);
        if (currentDrag.paintedKeys.has(key)) continue;
        currentDrag.paintedKeys.add(key);
        painted.push(entity);
        changed = true;
      }
      currentDrag.lastPaintPoint = point;
      if (changed) {
        currentDrag.paintedEntities = painted;
        onChange({
          ...currentDrag.originalMap,
          entities: [...currentDrag.originalMap.entities, ...painted],
        });
      }
      return;
    }
    if (currentDrag.kind === "move-selection") {
      onChange(
        moveSelections(
          currentDrag.originalMap,
          currentDrag.selections ?? [],
          point.x - currentDrag.start.x,
          point.y - currentDrag.start.y,
          event.ctrlKey,
        ),
      );
      return;
    }
    const originalBounds = selectionBounds(
      currentDrag.originalMap,
      currentDrag.selection ?? null,
    );
    if (!originalBounds || !currentDrag.selection) return;
    if (currentDrag.kind === "resize-selection" && currentDrag.corner) {
      onChange(
        replaceSelectionBounds(
          currentDrag.originalMap,
          currentDrag.selection,
          resizeEditorBounds(
            originalBounds,
            currentDrag.corner,
            point,
            map,
            currentDrag.selection.type === "solid" ? true : event.ctrlKey,
          ),
        ),
      );
      return;
    }
    if (
      currentDrag.kind === "move-node" &&
      currentDrag.selection.type === "entity" &&
      currentDrag.nodeIndex !== undefined
    ) {
      const entities = currentDrag.originalMap.entities.map((entity, index) => {
        if (index !== currentDrag.selection?.index) return entity;
        const nodes = [...(entity.nodes ?? [])];
        nodes[currentDrag.nodeIndex!] = {
          x: snapCoordinate(
            point.x - entity.bounds.width / 2,
            map.bounds.x,
            event.ctrlKey,
          ),
          y: snapCoordinate(
            point.y - entity.bounds.height / 2,
            map.bounds.y,
            event.ctrlKey,
          ),
        };
        return { ...entity, nodes };
      });
      onChange({ ...currentDrag.originalMap, entities });
      return;
    }
  };

  const pointerUp = () => {
    const currentDrag = drag.current;
    if (!currentDrag) return;
    if (currentDrag.kind === "create-solid" && draft) {
      rememberAndChange({ ...map, solids: [...map.solids, draft] });
      setSelected([{ type: "solid", index: map.solids.length }]);
      setTool("select");
    } else if (currentDrag.kind === "select-region") {
      if (marquee) setSelected(objectsInRegion(map, marquee));
      setMarquee(null);
    } else if (currentDrag.kind === "paint-entities") {
      finishContinuousChange(currentDrag.originalMap);
    } else if (
      currentDrag.kind === "move-selection" ||
      currentDrag.kind === "resize-selection" ||
      currentDrag.kind === "move-node"
    ) {
      finishContinuousChange(currentDrag.originalMap);
    }
    drag.current = null;
    setDraft(null);
  };

  const zoomAtPointer = (event: ReactWheelEvent<SVGSVGElement>) => {
    event.preventDefault();
    const rect = event.currentTarget.getBoundingClientRect();
    const focus = pointInCameraViewport(
      event.clientX,
      event.clientY,
      rect,
      cameraViewport,
    );
    zoomCamera(event.deltaY < 0 ? 0.8 : 1.25, focus);
  };

  const updateBounds = (field: keyof EditableBounds, value: number) => {
    if (!selection || !bounds || !Number.isFinite(value)) return;
    const next = {
      ...bounds,
      [field]:
        field === "width" || field === "height" ? Math.max(1, value) : value,
    };
    rememberAndChange(replaceSelectionBounds(map, selection, next));
  };

  const removeSelected = () => {
    if (!selected.length) return;
    rememberAndChange(deleteSelections(map, selected));
    setSelected([]);
  };

  const updateSelectedEntity = (mutator: (entity: MapEntity) => MapEntity) => {
    if (!selection || selection.type !== "entity") return;
    rememberAndChange({
      ...map,
      entities: map.entities.map((entity, index) =>
        index === selection.index ? mutator(entity) : entity,
      ),
    });
  };

  const updateMapBounds = (field: keyof GymMap["bounds"], value: number) => {
    if (!Number.isFinite(value)) return;
    rememberAndChange({
      ...map,
      bounds: {
        ...map.bounds,
        [field]:
          field === "width" || field === "height" ? Math.max(8, value) : value,
      },
    });
  };

  useEffect(() => {
    const keydown = (event: KeyboardEvent) => {
      if (
        event.target instanceof HTMLElement &&
        event.target.matches("input, textarea, select, button")
      )
        return;
      if (experiencing && event.code === "KeyR") {
        event.preventDefault();
        onResetExperience();
        return;
      }
      if (event.key === "Escape") {
        setSelected([]);
        return;
      }
      if (
        !selected.length ||
        (event.key !== "Delete" && event.key !== "Backspace")
      )
        return;
      event.preventDefault();
      rememberAndChange(deleteSelections(map, selected));
      setSelected([]);
    };
    window.addEventListener("keydown", keydown);
    return () => window.removeEventListener("keydown", keydown);
  }, [experiencing, map, onResetExperience, selected]);

  return (
    <main className={`map-editor ${experiencing ? "experiencing" : ""}`}>
      <aside className="editor-palette">
        <div className="editor-panel-heading">
          <small>MAP TOOLS</small>
          <h1>地图编辑器</h1>
        </div>
        <div className="editor-tool-group editor-primary-tools">
          {(["select", "solid", "spawn", "erase"] as const).map((candidate) => (
            <button
              key={candidate}
              className={tool === candidate ? "active" : ""}
              onClick={() => chooseTool(candidate)}
              aria-pressed={tool === candidate}
            >
              <EditorIcon>
                {candidate === "select" ? (
                  <FiMousePointer />
                ) : candidate === "solid" ? (
                  <FiBox />
                ) : candidate === "spawn" ? (
                  <FiMapPin />
                ) : (
                  <FiTrash2 />
                )}
              </EditorIcon>
              {toolLabel(candidate)}
            </button>
          ))}
        </div>
        <div className="editor-tool-section">
          <small>实体</small>
          <div className="editor-entity-tools">
            {ENTITY_TEMPLATES.map((template) => {
              const candidate = `entity:${template.id}` as EditorTool;
              return (
                <button
                  key={template.id}
                  className={tool === candidate ? "active" : ""}
                  onClick={() => chooseTool(candidate)}
                  aria-pressed={tool === candidate}
                >
                  {template.label}
                </button>
              );
            })}
          </div>
          {tool === "entity:crystal-spinner" && (
            <div className="editor-option-group">
              <small>圆刺画笔外观</small>
              <div className="editor-option-buttons spinner-variants">
                {SPINNER_VARIANTS.map((option) => (
                  <button
                    key={option.id}
                    className={spinnerBrushVariant === option.id ? "active" : ""}
                    onClick={() => setSpinnerBrushVariant(option.id)}
                  >
                    {option.label}
                  </button>
                ))}
              </div>
            </div>
          )}
          {tool.startsWith("entity:") && (
            <small className="editor-tool-hint">
              按住左键拖动可连续铺设，松开后仍保留当前画笔。
            </small>
          )}
        </div>
        <div className="editor-history" data-revision={historyRevision}>
          <button onClick={undo} disabled={undoStack.current.length === 0}>
            <EditorIcon><FiRotateCcw /></EditorIcon>
            撤销
          </button>
          <button onClick={redo} disabled={redoStack.current.length === 0}>
            <EditorIcon><FiRotateCw /></EditorIcon>
            重做
          </button>
        </div>
        <p className="editor-hint">
          实心块始终对齐 8 px 网格 · 实体默认 1 px 精细，按住 Ctrl 对齐 8 px
          <br />
          Ctrl+拖拽框选区域 · Ctrl+点击多选 · 空白处拖拽平移 · 滚轮缩放
        </p>
      </aside>

      <section className={`editor-stage ${trajectory ? "has-trajectory" : ""}`}>
        <div className="editor-stage-bar">
          <div>
            <small>{experiencing ? "LIVE EXPERIENCE" : "EDITING"}</small>
            <strong>{map.name}</strong>
            <span>
              {stats.solids} solids · {stats.entities} entities
            </span>
          </div>
          <div className="editor-stage-actions">
            {!experiencing && (
              <div className="editor-camera-controls" aria-label="编辑相机">
                <button
                  aria-label="相机向左"
                  onClick={() => moveCamera(-cameraViewport.width / 2, 0)}
                >
                  <EditorIcon><FiArrowLeft /></EditorIcon>
                </button>
                <button
                  aria-label="相机向上"
                  onClick={() => moveCamera(0, -cameraViewport.height / 2)}
                >
                  <EditorIcon><FiArrowUp /></EditorIcon>
                </button>
                <button
                  aria-label="相机回到出生点"
                  onClick={resetCamera}
                >
                  <EditorIcon><FiMapPin /></EditorIcon>
                  出生点
                </button>
                <button
                  aria-label="相机向下"
                  onClick={() => moveCamera(0, cameraViewport.height / 2)}
                >
                  <EditorIcon><FiArrowDown /></EditorIcon>
                </button>
                <button
                  aria-label="相机向右"
                  onClick={() => moveCamera(cameraViewport.width / 2, 0)}
                >
                  <EditorIcon><FiArrowRight /></EditorIcon>
                </button>
                <button aria-label="缩小地图" onClick={() => zoomCamera(1.25)}>
                  <EditorIcon><FiZoomOut /></EditorIcon>
                </button>
                <span>
                  {Math.round((320 / cameraViewport.width) * 100)}%
                </span>
                <button aria-label="放大地图" onClick={() => zoomCamera(0.8)}>
                  <EditorIcon><FiZoomIn /></EditorIcon>
                </button>
                <button aria-label="地图适配屏幕" onClick={fitCamera}>
                  <EditorIcon><FiMaximize /></EditorIcon>
                  适配
                </button>
              </div>
            )}
            {experiencing && (
              <>
                <button
                  className={trajectoryRecording ? "recording" : ""}
                  onClick={
                    trajectoryRecording
                      ? stopTrajectoryRecording
                      : startTrajectoryRecording
                  }
                >
                  <EditorIcon>
                    {trajectoryRecording ? <FiStopCircle /> : <FiCircle />}
                  </EditorIcon>
                  {trajectoryRecording ? "结束录制" : "录制轨迹"}
                </button>
                <button onClick={resetExperience}>
                  <EditorIcon><FiRefreshCcw /></EditorIcon>
                  重生
                </button>
              </>
            )}
            <button
              className={experiencing ? "stop" : "experience"}
              disabled={!ready}
              onClick={toggleExperience}
            >
              <EditorIcon>
                {experiencing ? <FiSquare /> : <FiPlay />}
              </EditorIcon>
              {experiencing ? "返回编辑" : "实时体验"}
            </button>
          </div>
        </div>
        <GameView
          map={map}
          state={renderedState}
          states={renderedStates}
          stateFrameOffset={renderedStateFrameOffset}
          frame={renderedFrame}
          stale={false}
          theme={theme}
          cameraViewport={experiencing ? undefined : cameraViewport}
        >
          {(viewport) => (
            <>
            {!experiencing && (
            <svg
              className={`map-editor-overlay tool-${tool.replace(":", "-")}`}
              viewBox={cameraViewBox(viewport.camera)}
              preserveAspectRatio="xMidYMid meet"
              aria-label="地图编辑画布"
              onPointerDown={pointerDown}
              onPointerMove={pointerMove}
              onPointerUp={pointerUp}
              onPointerCancel={pointerUp}
              onWheel={zoomAtPointer}
            >
              <defs>
                <pattern
                  id="editor-grid"
                  width={GRID_SIZE}
                  height={GRID_SIZE}
                  patternUnits="userSpaceOnUse"
                >
                  <path d={`M ${GRID_SIZE} 0 L 0 0 0 ${GRID_SIZE}`} />
                </pattern>
              </defs>
              <rect
                data-editor-background="true"
                className="editor-map-hitarea"
                x={map.bounds.x}
                y={map.bounds.y}
                width={map.bounds.width}
                height={map.bounds.height}
              />
              <rect
                className="editor-grid"
                x={map.bounds.x}
                y={map.bounds.y}
                width={map.bounds.width}
                height={map.bounds.height}
              />
              {map.solids.map((solid, index) => (
                <rect
                  key={`solid-${index}`}
                  className={`editor-object solid ${selectionInList(selected, { type: "solid", index }) ? "selected" : ""}`}
                  {...solid}
                  onPointerDown={(event) =>
                    beginSelectionDrag(event, { type: "solid", index })
                  }
                />
              ))}
              {map.entities.map((entity, index) => (
                <rect
                  key={`entity-${index}`}
                  data-kind={entity.kind}
                  className={`editor-object entity ${selectionInList(selected, { type: "entity", index }) ? "selected" : ""}`}
                  {...editorEntityHitBounds(entity)}
                  onPointerDown={(event) =>
                    beginSelectionDrag(event, { type: "entity", index })
                  }
                />
              ))}
              {selected.length === 1 &&
                selectedEntity?.kind === "zip_mover" &&
                selectedEntity.nodes?.map((node, nodeIndex) => (
                  <g className="editor-zip-node" key={nodeIndex}>
                    <line
                      x1={
                        selectedEntity.bounds.x +
                        selectedEntity.bounds.width / 2
                      }
                      y1={
                        selectedEntity.bounds.y +
                        selectedEntity.bounds.height / 2
                      }
                      x2={node.x + selectedEntity.bounds.width / 2}
                      y2={node.y + selectedEntity.bounds.height / 2}
                    />
                    <rect
                      x={node.x}
                      y={node.y}
                      width={selectedEntity.bounds.width}
                      height={selectedEntity.bounds.height}
                    />
                    <circle
                      cx={node.x + selectedEntity.bounds.width / 2}
                      cy={node.y + selectedEntity.bounds.height / 2}
                      r="5"
                      onPointerDown={(event) => beginNodeDrag(event, nodeIndex)}
                    />
                  </g>
                ))}
              {selected.length === 1 && selection && bounds && (
                <g className="editor-resize-handles">
                  {(
                    [
                      ["nw", bounds.x, bounds.y],
                      ["ne", bounds.x + bounds.width, bounds.y],
                      ["se", bounds.x + bounds.width, bounds.y + bounds.height],
                      ["sw", bounds.x, bounds.y + bounds.height],
                    ] as const
                  ).map(([corner, x, y]) => (
                    <rect
                      key={corner}
                      data-corner={corner}
                      x={x - 2}
                      y={y - 2}
                      width="4"
                      height="4"
                      onPointerDown={(event) => beginResize(event, corner)}
                    />
                  ))}
                </g>
              )}
              {draft && <rect className="editor-draft" {...draft} />}
              {marquee && <rect className="editor-marquee" {...marquee} />}
              {selected.length > 1 && selectionUnion && (
                <rect
                  className="editor-multi-union"
                  {...selectionUnion}
                  pointerEvents="none"
                />
              )}
              <g
                className="editor-spawn"
                transform={`translate(${map.spawn.x} ${map.spawn.y})`}
              >
                <circle r="7" />
                <path d="M -4 0 H 4 M 0 -4 V 4" />
              </g>
            </svg>
            )}
            {trajectory && trajectoryEndIndex >= 0 && (
              <svg
                className="editor-trajectory-overlay"
                viewBox={cameraViewBox(viewport.camera)}
                preserveAspectRatio="xMidYMid meet"
                aria-label="已录制轨迹"
              >
                <path className="editor-trajectory-line-glow" d={trajectoryLine} />
                <path className="editor-trajectory-line" d={trajectoryLine} />
                <g className={showAllTrajectory ? "all" : "selected"}>
                  {colliderIndices.map((index) => {
                    const snapshot = trajectory.states[index];
                    if (!snapshot) return null;
                    return (
                      <rect
                        key={index}
                        data-frame={trajectory.startFrame + index}
                        className={index === trajectoryFrame ? "current" : ""}
                        {...playerCollisionBounds(snapshot)}
                      />
                    );
                  })}
                </g>
                {reviewedTrajectoryState && (
                  <circle
                    className="editor-trajectory-point"
                    cx={reviewedTrajectoryState.pos.x}
                    cy={reviewedTrajectoryState.pos.y}
                    r="2.5"
                  />
                )}
              </svg>
            )}
            </>
          )}
        </GameView>
        {experiencing && (
          <div className="editor-live-note">
            <i />
            {trajectoryRecording
              ? `正在录制轨迹 · ${trajectory?.states.length ?? 0} 帧`
              : "WASM 60 FPS · 与游玩模式相同"}
          </div>
        )}
        {trajectory && (
          <div className="editor-trajectory-timeline">
            <div className="editor-trajectory-meta">
              <strong>
                {trajectoryRecording ? "正在录制" : "轨迹回放"}
              </strong>
              <span>
                F{trajectory.startFrame + trajectoryFrame} · {trajectory.states.length} 帧 · {reviewedTrajectoryState?.state ?? "-"}
              </span>
            </div>
            <input
              type="range"
              aria-label="轨迹进度"
              min={0}
              max={Math.max(0, trajectory.states.length - 1)}
              value={Math.min(
                trajectoryFrame,
                Math.max(0, trajectory.states.length - 1),
              )}
              disabled={trajectoryRecording}
              onChange={(event) => {
                setTrajectoryFrame(Number(event.target.value));
                setShowAllTrajectory(false);
              }}
            />
            <button
              type="button"
              aria-pressed={showAllTrajectory}
              onClick={() => setShowAllTrajectory((value) => !value)}
            >
              <EditorIcon>
                {showAllTrajectory ? <FiEyeOff /> : <FiEye />}
              </EditorIcon>
              {showAllTrajectory ? "聚焦当前帧" : "显示全部轨迹"}
            </button>
            <button
              type="button"
              aria-label="清除轨迹"
              disabled={trajectoryRecording}
              onClick={() => {
                setTrajectory(null);
                setTrajectoryFrame(0);
                setShowAllTrajectory(false);
              }}
            >
              <EditorIcon><FiX /></EditorIcon>
            </button>
          </div>
        )}
      </section>

      <aside className="editor-inspector">
        <div className="editor-panel-heading">
          <small>INSPECTOR</small>
          <h2>
            {selectionCount > 1
              ? `已选 ${selectionCount} 个对象`
              : selection
                ? (selectedEntity?.name ?? "实心块")
                : "房间"}
          </h2>
        </div>
        {selectionCount > 1 ? (
          <div className="editor-multi-fields">
            <div className="editor-kind">
              <span>{selectionCount} 个对象已选中</span>
              <button onClick={removeSelected}>删除对象</button>
            </div>
            <p className="editor-multi-hint">
              拖拽任一选中对象可整体移动；Delete / Backspace 删除全部选中对象；Ctrl+点击可取消单个对象。
            </p>
          </div>
        ) : selection && bounds ? (
          <>
            <div className="editor-kind">
              <span>
                {selection.type === "solid" ? "SOLID" : selectedEntity?.kind}
              </span>
              <button onClick={removeSelected}>删除对象</button>
            </div>
            <div className="editor-field-grid">
              {(["x", "y", "width", "height"] as const).map((field) => (
                <label key={field}>
                  <small>{field.toUpperCase()}</small>
                  <input
                    type="number"
                    value={bounds[field]}
                    onChange={(event) =>
                      updateBounds(field, Number(event.target.value))
                    }
                  />
                </label>
              ))}
            </div>
            {selectedEntity?.kind === "spikes" && (
              <div className="editor-option-group">
                <small>尖刺方向</small>
                <div className="editor-option-buttons">
                  {SPIKE_DIRECTIONS.map((option) => (
                    <button
                      key={option.name}
                      className={
                        selectedEntity.direction.x === option.direction.x &&
                        selectedEntity.direction.y === option.direction.y
                          ? "active"
                          : ""
                      }
                      onClick={() =>
                        updateSelectedEntity((entity) =>
                          setEditorSpikeDirection(entity, option.direction),
                        )
                      }
                    >
                      {option.label}
                    </button>
                  ))}
                </div>
              </div>
            )}
            {selectedEntity?.kind === "crystal_static_spinner" && (
              <div className="editor-option-group">
                <small>圆刺外观</small>
                <div className="editor-option-buttons spinner-variants">
                  {SPINNER_VARIANTS.map((option) => (
                    <button
                      key={option.id}
                      className={
                        (selectedEntity.variant ?? "theme") === option.id
                          ? "active"
                          : ""
                      }
                      onClick={() => {
                        setSpinnerBrushVariant(option.id);
                        updateSelectedEntity((entity) => ({
                          ...entity,
                          variant:
                            option.id === "theme" ? undefined : option.id,
                        }));
                      }}
                    >
                      {option.label}
                    </button>
                  ))}
                </div>
              </div>
            )}
            {selectedEntity && (
              <div className="editor-object-fields">
                <label>
                  <small>NAME</small>
                  <input
                    value={selectedEntity.name}
                    onChange={(event) =>
                      updateSelectedEntity((entity) => ({
                        ...entity,
                        name: event.target.value,
                      }))
                    }
                  />
                </label>
                <label>
                  <small>KIND</small>
                  <select
                    value={selectedEntity.kind}
                    onChange={(event) =>
                      updateSelectedEntity((entity) => {
                        const kind = event.target.value as EntityKind;
                        const template = ENTITY_TEMPLATES.find(
                          (candidate) => candidate.kind === kind,
                        );
                        return {
                          ...entity,
                          kind,
                          ...(kind === "zip_mover" && !entity.nodes?.length
                            ? {
                                nodes: [
                                  {
                                    x: entity.bounds.x + 64,
                                    y: entity.bounds.y,
                                  },
                                ],
                              }
                            : {}),
                          ...(template && entity.name === selectedEntity.name
                            ? { name: template.name }
                            : {}),
                        };
                      })
                    }
                  >
                    {[
                      ...new Set([
                        selectedEntity.kind,
                        ...ENTITY_TEMPLATES.map((template) => template.kind),
                      ]),
                    ].map((kind) => (
                      <option value={kind} key={kind}>
                        {kind}
                      </option>
                    ))}
                  </select>
                </label>
                <div className="editor-field-grid compact">
                  <label>
                    <small>DIRECTION X</small>
                    <input
                      type="number"
                      step="0.1"
                      value={selectedEntity.direction.x}
                      onChange={(event) =>
                        updateSelectedEntity((entity) => ({
                          ...entity,
                          direction: {
                            ...entity.direction,
                            x: Number(event.target.value),
                          },
                        }))
                      }
                    />
                  </label>
                  <label>
                    <small>DIRECTION Y</small>
                    <input
                      type="number"
                      step="0.1"
                      value={selectedEntity.direction.y}
                      onChange={(event) =>
                        updateSelectedEntity((entity) => ({
                          ...entity,
                          direction: {
                            ...entity.direction,
                            y: Number(event.target.value),
                          },
                        }))
                      }
                    />
                  </label>
                </div>
                <div className="editor-boolean-fields">
                  <label>
                    <input
                      type="checkbox"
                      checked={Boolean(selectedEntity.shielded)}
                      onChange={(event) =>
                        updateSelectedEntity((entity) => ({
                          ...entity,
                          shielded: event.target.checked,
                        }))
                      }
                    />
                    shielded
                  </label>
                  <label>
                    <input
                      type="checkbox"
                      checked={Boolean(selectedEntity.single_use)}
                      onChange={(event) =>
                        updateSelectedEntity((entity) => ({
                          ...entity,
                          single_use: event.target.checked,
                        }))
                      }
                    />
                    single_use
                  </label>
                </div>
                {selectedEntity.kind === "zip_mover" && (
                  <fieldset className="editor-node-fields">
                    <legend>ZIP MOVER 终点</legend>
                    <div className="editor-field-grid compact">
                      <label>
                        <small>NODE X</small>
                        <input
                          type="number"
                          value={
                            selectedEntity.nodes?.[0]?.x ??
                            selectedEntity.bounds.x + 64
                          }
                          onChange={(event) =>
                            updateSelectedEntity((entity) => ({
                              ...entity,
                              nodes: [
                                {
                                  x: Number(event.target.value),
                                  y: entity.nodes?.[0]?.y ?? entity.bounds.y,
                                },
                              ],
                            }))
                          }
                        />
                      </label>
                      <label>
                        <small>NODE Y</small>
                        <input
                          type="number"
                          value={
                            selectedEntity.nodes?.[0]?.y ??
                            selectedEntity.bounds.y
                          }
                          onChange={(event) =>
                            updateSelectedEntity((entity) => ({
                              ...entity,
                              nodes: [
                                {
                                  x:
                                    entity.nodes?.[0]?.x ??
                                    entity.bounds.x + 64,
                                  y: Number(event.target.value),
                                },
                              ],
                            }))
                          }
                        />
                      </label>
                    </div>
                    <small>画布上的圆形手柄也可直接拖动终点。</small>
                  </fieldset>
                )}
              </div>
            )}
          </>
        ) : (
          <>
            <div className="editor-map-fields">
              <label>
                <small>ROOM NAME</small>
                <input
                  value={map.name}
                  onChange={(event) =>
                    rememberAndChange({ ...map, name: event.target.value })
                  }
                />
              </label>
              <label>
                <small>ROOM ID</small>
                <input
                  value={map.room ?? ""}
                  onChange={(event) =>
                    rememberAndChange({ ...map, room: event.target.value })
                  }
                />
              </label>
              <div className="editor-field-grid compact">
                {(["x", "y", "width", "height"] as const).map((field) => (
                  <label key={field}>
                    <small>BOUND {field.toUpperCase()}</small>
                    <input
                      type="number"
                      value={map.bounds[field]}
                      onChange={(event) =>
                        updateMapBounds(field, Number(event.target.value))
                      }
                    />
                  </label>
                ))}
              </div>
              <div className="editor-field-grid compact">
                <label>
                  <small>SPAWN X</small>
                  <input
                    type="number"
                    value={map.spawn.x}
                    onChange={(event) =>
                      rememberAndChange({
                        ...map,
                        spawn: { ...map.spawn, x: Number(event.target.value) },
                      })
                    }
                  />
                </label>
                <label>
                  <small>SPAWN Y</small>
                  <input
                    type="number"
                    value={map.spawn.y}
                    onChange={(event) =>
                      rememberAndChange({
                        ...map,
                        spawn: { ...map.spawn, y: Number(event.target.value) },
                      })
                    }
                  />
                </label>
              </div>
              <label>
                <small>SOURCE PACKAGE</small>
                <input
                  placeholder="custom"
                  value={map.source_package ?? ""}
                  onChange={(event) =>
                    rememberAndChange({
                      ...map,
                      source_package: event.target.value || null,
                    })
                  }
                />
              </label>
            </div>
          </>
        )}
        <div className="editor-inspector-tip">
          <strong>{experiencing ? "正在实时体验" : toolLabel(tool)}</strong>
          <span>
            {experiencing
              ? "键盘和手柄输入直接送入 WASM；返回编辑时地图保持不变。"
              : tool === "select"
                ? "拖动对象移动，拖动四角缩放；Ctrl+点击多选，Ctrl+拖拽框选；Delete / Backspace 删除。"
                : tool === "solid"
                  ? "在画布空白处拖动，创建新的碰撞块。"
                  : "在画布中点击即可应用当前工具。"}
          </span>
        </div>
      </aside>
    </main>
  );
}
