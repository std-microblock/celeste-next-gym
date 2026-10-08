import {
  memo,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
  type RefObject,
} from "react";
import {
  cameraBounds,
  clampCameraViewport,
  clampCameraPosition,
  stateCameraPosition,
  type CameraBounds,
  type GameViewViewport,
} from "../camera";
import type { GymMap, SimState, Vec2 } from "../model";
import {
  buildSolidGrid,
  buildSpinnerGroups,
  buildStaticMoverAttachments,
  buildTileLayer,
  entityKindIndices,
  gameViewDrawChanged,
  loadAssets,
  loadThemeAtlas,
  mergeGameAssets,
  paintGame,
  type GameAssets,
  type GameViewDrawInput,
} from "../render/gameRenderer";
import type { VisualTheme } from "../visualThemes";

// The drawing code lives in ../render/gameRenderer so headless tooling can
// reuse it; keep the historical exports available from this module.
export {
  activeBoosterCenter,
  centeredAtlasEntryDestination,
  loadAssets,
  loadThemeAtlas,
  runtimeAttachedEntityBounds,
  runtimeEntityBounds,
  strawberryIsPicked,
} from "../render/gameRenderer";

/** Refs the live loop updates every simulated tick without a React commit.
 * GameView's own rAF loop reads these so the canvas can keep running at the
 * display cadence even when React state updates are throttled. */
export interface LiveRenderRefs {
  /** Latest live player state. */
  state: RefObject<SimState>;
  /** Latest live frame number. */
  frame: RefObject<number>;
  /** Latest live history window (mutated in place by the host). */
  history: RefObject<{ startFrame: number; states: SimState[] }>;
  /** Latest live staleness flag (optional; defaults to the prop value). */
  stale?: RefObject<boolean>;
}

export const GameView = memo(function GameView({
  map,
  state,
  states,
  stateFrameOffset = 0,
  frame,
  stale,
  theme,
  cameraPosition,
  cameraViewport,
  children,
  liveRefs,
}: {
  map: GymMap;
  state: SimState;
  states: readonly (SimState | undefined)[];
  stateFrameOffset?: number;
  frame: number;
  stale: boolean;
  theme: VisualTheme;
  /** Manual top-left camera position. Omit to render SimState.camera. */
  cameraPosition?: Vec2;
  /** Editor-only viewport override that may zoom beyond Celeste's 320x180 camera. */
  cameraViewport?: CameraBounds;
  children?:
    | ReactNode
    | ((viewport: GameViewViewport) => ReactNode);
  /** Live-render refs: when present, the rAF loop draws from these refs
   * (updated by the host without React commits) instead of the props. */
  liveRefs?: LiveRenderRefs;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [assets, setAssets] = useState<GameAssets | null>(null);
  const [themeAssets, setThemeAssets] = useState<GameAssets | null>(null);
  const [viewportRevision, setViewportRevision] = useState(0);
  const solidGrid = useMemo(() => buildSolidGrid(map), [map]);
  const mergedAssets = useMemo(
    () => (assets ? (themeAssets ? mergeGameAssets(assets, themeAssets) : assets) : null),
    [assets, themeAssets],
  );
  const tileLayer = useMemo(
    () =>
      mergedAssets ? buildTileLayer(mergedAssets, map, solidGrid, theme) : undefined,
    [mergedAssets, map, solidGrid, theme],
  );
  const spinnerGroups = useMemo(
    () => (mergedAssets ? buildSpinnerGroups(mergedAssets, map, theme) : null),
    [mergedAssets, map, theme],
  );
  const kindIndices = useMemo(() => entityKindIndices(map), [map]);
  const staticMoverAttachments = useMemo(
    () => buildStaticMoverAttachments(map),
    [map],
  );
  const camera = cameraViewport
    ? clampCameraViewport(map, cameraViewport)
    : cameraBounds(
        cameraPosition
          ? clampCameraPosition(map, cameraPosition)
          : stateCameraPosition(map, state),
      );

  useEffect(() => {
    void loadAssets().then(setAssets);
  }, []);
  useEffect(() => {
    let cancelled = false;
    setThemeAssets(null);
    if (!theme.atlasUrl) return;
    void loadThemeAtlas(theme.atlasUrl)
      .then((atlas) => {
        if (!cancelled) setThemeAssets(atlas);
      })
      .catch(() => {
        // The theme atlas is optional; the base atlas still covers fallbacks.
      });
    return () => {
      cancelled = true;
    };
  }, [theme]);
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const observer = new ResizeObserver(() =>
      setViewportRevision((value) => value + 1),
    );
    observer.observe(canvas);
    return () => observer.disconnect();
  }, []);

  // Latest draw inputs, refreshed on every render. The rAF loop below reads
  // this instead of re-running a full redraw inside a props-driven effect, so
  // the canvas work is decoupled from React commit timing and frequency.
  const drawInputRef = useRef<GameViewDrawInput | null>(null);
  drawInputRef.current = mergedAssets
    ? {
        assets: mergedAssets,
        map,
        theme,
        frame,
        state,
        states,
        stateFrameOffset,
        stale,
        camera,
        cameraPosition,
        cameraViewport,
        tileLayer,
        spinnerGroups,
        entityKindIndices: kindIndices,
        staticMoverAttachments,
        viewportRevision,
      }
    : null;

  const drawSnapshotForFrame = (
    last: GameViewDrawInput | null,
  ): GameViewDrawInput | null => {
    const base = drawInputRef.current;
    if (!base) return null;
    if (!liveRefs) return base;
    const liveState = liveRefs.state.current;
    const history = liveRefs.history.current;
    const liveFrame = liveRefs.frame.current;
    const liveStale = liveRefs.stale?.current ?? base.stale;
    if (
      last &&
      base.assets === last.assets &&
      base.map === last.map &&
      base.theme === last.theme &&
      base.tileLayer === last.tileLayer &&
      base.spinnerGroups === last.spinnerGroups &&
      base.entityKindIndices === last.entityKindIndices &&
      base.staticMoverAttachments === last.staticMoverAttachments &&
      base.viewportRevision === last.viewportRevision &&
      base.cameraPosition === last.cameraPosition &&
      base.cameraViewport === last.cameraViewport &&
      liveState === last.state &&
      liveFrame === last.frame &&
      history.states === last.states &&
      history.startFrame === last.stateFrameOffset &&
      liveStale === last.stale
    )
      return last;
    return {
      ...base,
      state: liveState,
      frame: liveFrame,
      states: history.states,
      stateFrameOffset: history.startFrame,
      stale: liveStale,
      camera: base.cameraViewport
        ? clampCameraViewport(base.map, base.cameraViewport)
        : cameraBounds(
            base.cameraPosition
              ? clampCameraPosition(base.map, base.cameraPosition)
              : stateCameraPosition(base.map, liveState),
          ),
    };
  };

  useEffect(() => {
    if (!mergedAssets) return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const context = canvas.getContext("2d");
    if (!context) return;
    const dpr = window.devicePixelRatio || 1;
    let animation = 0;
    let last: GameViewDrawInput | null = null;
    const tick = () => {
      animation = requestAnimationFrame(tick);
      const snapshot = drawSnapshotForFrame(last);
      if (!snapshot) return;
      if (snapshot === last) return;
      if (!gameViewDrawChanged(snapshot, last)) return;
      paintGame(canvas, context, dpr, snapshot);
      last = snapshot;
    };
    animation = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(animation);
  }, [mergedAssets, liveRefs]);

  return (
    <div className="game-screen">
      <canvas
        ref={canvasRef}
        aria-label={`CelesteGymPlayground 原版资源渲染画面 · ${theme.label}`}
      />
      {typeof children === "function"
        ? children({
            width: canvasRef.current?.clientWidth ?? 0,
            height: canvasRef.current?.clientHeight ?? 0,
            camera,
          })
        : children}
      <div className="screen-vignette" />
      <div className="screen-noise" />
      {!assets && (
        <div className="recompute-flag">
          <span />
          加载 Gameplay atlas
        </div>
      )}
      {assets && stale && (
        <div className="recompute-flag">
          <span />
          等待重算
        </div>
      )}
    </div>
  );
});


