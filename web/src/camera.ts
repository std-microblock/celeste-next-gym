import type { GymMap, SimState, Vec2 } from "./model";

export const CELESTE_CAMERA_WIDTH = 320;
export const CELESTE_CAMERA_HEIGHT = 180;

export interface CameraBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface GameViewViewport {
  width: number;
  height: number;
  camera: CameraBounds;
}

function finitePoint(value: unknown): value is Vec2 {
  if (!value || typeof value !== "object") return false;
  const point = value as Partial<Vec2>;
  return Number.isFinite(point.x) && Number.isFinite(point.y);
}

export function clampCameraPosition(map: GymMap, position: Vec2): Vec2 {
  const maxX = Math.max(map.bounds.x, map.bounds.x + map.bounds.width - CELESTE_CAMERA_WIDTH);
  const maxY = Math.max(
    map.bounds.y,
    map.bounds.y + map.bounds.height - CELESTE_CAMERA_HEIGHT,
  );
  return {
    x: Math.max(map.bounds.x, Math.min(maxX, position.x)),
    y: Math.max(map.bounds.y, Math.min(maxY, position.y)),
  };
}

export function defaultCameraPosition(map: GymMap, focus = map.spawn): Vec2 {
  return clampCameraPosition(map, {
    x: focus.x - CELESTE_CAMERA_WIDTH / 2,
    y: focus.y - CELESTE_CAMERA_HEIGHT / 2,
  });
}

export function stateCameraPosition(map: GymMap, state: SimState): Vec2 {
  return state.camera_initialized !== false && finitePoint(state.camera)
    ? { ...state.camera }
    : defaultCameraPosition(map, state.pos);
}

export function cameraBounds(position: Vec2): CameraBounds {
  return {
    x: position.x,
    y: position.y,
    width: CELESTE_CAMERA_WIDTH,
    height: CELESTE_CAMERA_HEIGHT,
  };
}

export function clampCameraViewport(
  map: GymMap,
  viewport: CameraBounds,
): CameraBounds {
  const width = Math.max(1, viewport.width);
  const height = Math.max(1, viewport.height);
  const centerWhenOversized = (
    start: number,
    size: number,
    viewportSize: number,
  ) => start - (viewportSize - size) / 2;
  const clampAxis = (
    position: number,
    start: number,
    size: number,
    viewportSize: number,
  ) =>
    viewportSize >= size
      ? centerWhenOversized(start, size, viewportSize)
      : Math.max(start, Math.min(start + size - viewportSize, position));
  return {
    x: clampAxis(viewport.x, map.bounds.x, map.bounds.width, width),
    y: clampAxis(viewport.y, map.bounds.y, map.bounds.height, height),
    width,
    height,
  };
}

export function fitCameraViewport(
  map: GymMap,
  aspectRatio = CELESTE_CAMERA_WIDTH / CELESTE_CAMERA_HEIGHT,
): CameraBounds {
  const safeAspect =
    Number.isFinite(aspectRatio) && aspectRatio > 0
      ? aspectRatio
      : CELESTE_CAMERA_WIDTH / CELESTE_CAMERA_HEIGHT;
  let width = map.bounds.width;
  let height = width / safeAspect;
  if (height < map.bounds.height) {
    height = map.bounds.height;
    width = height * safeAspect;
  }
  return clampCameraViewport(map, {
    x: map.bounds.x + (map.bounds.width - width) / 2,
    y: map.bounds.y + (map.bounds.height - height) / 2,
    width,
    height,
  });
}

export function zoomCameraViewport(
  map: GymMap,
  viewport: CameraBounds,
  factor: number,
  focus: Vec2 = {
    x: viewport.x + viewport.width / 2,
    y: viewport.y + viewport.height / 2,
  },
): CameraBounds {
  const fit = fitCameraViewport(map, viewport.width / viewport.height);
  const minWidth = 40;
  const maxWidth = Math.max(fit.width * 2, CELESTE_CAMERA_WIDTH);
  const requestedWidth = viewport.width * factor;
  const width = Math.max(minWidth, Math.min(maxWidth, requestedWidth));
  const appliedFactor = width / viewport.width;
  const height = viewport.height * appliedFactor;
  return clampCameraViewport(map, {
    x: focus.x - (focus.x - viewport.x) * appliedFactor,
    y: focus.y - (focus.y - viewport.y) * appliedFactor,
    width,
    height,
  });
}

export function cameraViewBox(bounds: CameraBounds): string {
  return `${bounds.x} ${bounds.y} ${bounds.width} ${bounds.height}`;
}

export function pointInCameraViewport(
  clientX: number,
  clientY: number,
  rect: Pick<DOMRect, "left" | "top" | "width" | "height">,
  camera: CameraBounds,
): Vec2 {
  const scale = Math.min(rect.width / camera.width, rect.height / camera.height);
  const offsetX = (rect.width - camera.width * scale) / 2;
  const offsetY = (rect.height - camera.height * scale) / 2;
  return {
    x: camera.x + (clientX - rect.left - offsetX) / scale,
    y: camera.y + (clientY - rect.top - offsetY) / scale,
  };
}
