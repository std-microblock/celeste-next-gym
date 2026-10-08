/**
 * Headless rendering: the web app's canvas renderer (web/src/render) driven by
 * @napi-rs/canvas, plus PNG / contact sheet / GIF / MP4-WebM encoding.
 */
import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, extname, join, resolve } from "node:path";
import { Canvas, GlobalFonts, createCanvas, loadImage, type SKRSContext2D } from "@napi-rs/canvas";
import gifenc from "gifenc";
import { setRenderBackend } from "../../../web/src/render/canvasBackend.ts";
import {
  buildSolidGrid,
  buildSpinnerGroups,
  buildStaticMoverAttachments,
  buildTileLayer,
  entityKindIndices,
  loadAssets,
  loadThemeAtlas,
  mergeGameAssets,
  paintGame,
  type GameAssets,
  type GameViewDrawInput,
} from "../../../web/src/render/gameRenderer.ts";
import {
  cameraBounds,
  clampCameraPosition,
  clampCameraViewport,
  fitCameraViewport,
  stateCameraPosition,
  type CameraBounds,
} from "../../../web/src/camera.ts";
import {
  DEFAULT_VISUAL_THEME_ID,
  VISUAL_THEMES,
  visualThemeById,
  type VisualTheme,
  type VisualThemeId,
} from "../../../web/src/visualThemes.ts";
import { WEB_PUBLIC } from "./paths.ts";
import { describeInput } from "./inputs.ts";
import type { GymMap, SimInput, SimState } from "./types.ts";

const { GIFEncoder, quantize, applyPalette } = gifenc as unknown as {
  GIFEncoder(): {
    writeFrame(index: Uint8Array, width: number, height: number, options: Record<string, unknown>): void;
    finish(): void;
    bytes(): Uint8Array;
  };
  quantize(rgba: Uint8Array | Uint8ClampedArray, colors: number, options?: Record<string, unknown>): number[][];
  applyPalette(rgba: Uint8Array | Uint8ClampedArray, palette: number[][], format?: string): Uint8Array;
};

let backendInstalled = false;
let monoFamily: string | undefined;
/** A real monospace family: napi-rs maps the generic `monospace` to an arbitrary font. */
export function monoFont(size: number): string {
  if (!monoFamily) {
    for (const [file, family] of [
      ["C:/Windows/Fonts/consola.ttf", "Consolas"],
      ["/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", "DejaVu Sans Mono"],
      ["/System/Library/Fonts/Menlo.ttc", "Menlo"],
    ] as const) {
      try {
        if (existsSync(file)) GlobalFonts.registerFromPath(file, family);
      } catch {
        // Font registration is best effort.
      }
    }
    const families = new Set(GlobalFonts.families.map((family) => family.family));
    monoFamily =
      ["Consolas", "Cascadia Mono", "DejaVu Sans Mono", "Menlo", "Liberation Mono", "Noto Sans Mono CJK SC", "Courier New"].find((name) =>
        families.has(name),
      ) ?? "monospace";
  }
  return `${size}px "${monoFamily}"`;
}
function assetPath(url: string): string {
  const clean = decodeURIComponent(url.replace(/^\/+/, "").split("?")[0]);
  const path = resolve(WEB_PUBLIC, clean);
  if (!path.startsWith(WEB_PUBLIC)) throw new Error(`asset outside web/public: ${url}`);
  return path;
}

/** Route the shared renderer's canvas/asset calls to @napi-rs/canvas + web/public. */
export function installNodeRenderBackend(): void {
  if (backendInstalled) return;
  backendInstalled = true;
  setRenderBackend({
    createCanvas: (width, height) => createCanvas(width, height) as unknown as HTMLCanvasElement,
    loadImage: async (url) => (await loadImage(readFileSync(assetPath(url)))) as unknown as HTMLImageElement,
    loadJson: async (url) => JSON.parse(readFileSync(assetPath(url), "utf8")),
    // Deliberately no `freeze` override, so `freezeRenderCanvas` returns the
    // canvas itself. @napi-rs/canvas decodes `Image.src = <Buffer>` on a worker
    // thread, so a "frozen" Image built from `canvas.toBuffer("image/png")` is
    // still blank when the next drawImage runs in the same tick: it silently
    // erased the merged gameplay atlas and every cached tile layer, leaving
    // background + HUD only. `Image.decode()` is async and `freeze` cannot
    // await it. The optimisation is also worthless here — measured on
    // @napi-rs/canvas 0.1.100, drawImage(canvas) matches drawImage(decoded
    // Image) (~2 ms/frame for 400 blits out of a 2048x1200 atlas, with the
    // per-frame getImageData that GIF/video encoding does).
  });
}

export function listThemes(): { id: string; label: string; chapter: string; collection: string }[] {
  return VISUAL_THEMES.map((theme) => ({
    id: theme.id,
    label: theme.label,
    chapter: theme.chapter,
    collection: theme.collection,
  }));
}

/** Pick a theme: explicit id, else the one extracted from the same map file, else Forsaken City. */
export function chooseTheme(mapPath: string | undefined, explicit?: string): VisualTheme {
  if (explicit) {
    const id = (explicit.startsWith("room:") ? explicit : `room:${explicit}`) as VisualThemeId;
    const found = VISUAL_THEMES.find((theme) => theme.id === id || theme.id === explicit);
    if (!found) throw new Error(`unknown theme "${explicit}" (see \`celeste-gym themes\`)`);
    return found;
  }
  const normalised = (mapPath ?? "").replace(/\\/g, "/").toLowerCase();
  const byChapter =
    VISUAL_THEMES.find((theme) => theme.chapter.toLowerCase() === normalised) ??
    VISUAL_THEMES.find((theme) => normalised.endsWith(theme.chapter.toLowerCase()));
  return byChapter ?? visualThemeById(DEFAULT_VISUAL_THEME_ID);
}

export interface RenderOptions {
  /** Integer pixel scale of the 320x180 camera (default 2). */
  scale?: number;
  /** `follow` = in-game camera (default), `room` = whole room. */
  camera?: "follow" | "room";
  theme?: string;
  /** Overlay frame/state/position text. */
  hud?: boolean;
  /** Overlay solids, entity bounds and the player hitbox. */
  hitboxes?: boolean;
  /** Draw the player's path so far (room camera only is most useful). */
  path?: boolean;
}

interface SceneRoom {
  map: GymMap;
  tileLayer: HTMLCanvasElement | undefined;
  spinnerGroups: GameViewDrawInput["spinnerGroups"];
  kindIndices: number[];
  staticMoverAttachments: GameViewDrawInput["staticMoverAttachments"];
}

/** Renderer bound to one theme; rooms are prepared lazily and cached. */
export class SceneRenderer {
  readonly theme: VisualTheme;
  readonly options: Required<Omit<RenderOptions, "theme">>;
  private assets!: GameAssets;
  private rooms = new Map<GymMap, SceneRoom>();

  private constructor(theme: VisualTheme, options: RenderOptions) {
    this.theme = theme;
    this.options = {
      scale: Math.max(1, Math.round(options.scale ?? 2)),
      camera: options.camera ?? "follow",
      hud: options.hud ?? false,
      hitboxes: options.hitboxes ?? false,
      path: options.path ?? false,
    };
  }

  static async create(mapPath: string | undefined, options: RenderOptions = {}): Promise<SceneRenderer> {
    installNodeRenderBackend();
    const renderer = new SceneRenderer(chooseTheme(mapPath, options.theme), options);
    const base = await loadAssets();
    let assets = base;
    if (renderer.theme.atlasUrl) {
      try {
        assets = mergeGameAssets(base, await loadThemeAtlas(renderer.theme.atlasUrl));
      } catch {
        // Theme atlases are optional; the base atlas covers fallbacks.
      }
    }
    renderer.assets = assets;
    return renderer;
  }

  private room(map: GymMap): SceneRoom {
    let room = this.rooms.get(map);
    if (!room) {
      room = {
        map,
        tileLayer: buildTileLayer(this.assets, map, buildSolidGrid(map), this.theme),
        spinnerGroups: buildSpinnerGroups(this.assets, map, this.theme),
        kindIndices: entityKindIndices(map),
        staticMoverAttachments: buildStaticMoverAttachments(map),
      };
      this.rooms.set(map, room);
    }
    return room;
  }

  /** Output size in pixels for a map (room camera keeps the room aspect). */
  size(map: GymMap): { width: number; height: number; camera?: CameraBounds } {
    const scale = this.options.scale;
    if (this.options.camera === "room") {
      const width = Math.ceil(map.bounds.width * scale / 2) * 2;
      const height = Math.ceil(map.bounds.height * scale / 2) * 2;
      return { width, height, camera: fitCameraViewport(map, map.bounds.width / map.bounds.height) };
    }
    return { width: 320 * scale, height: 180 * scale };
  }

  /**
   * Draw `states[index]` of a trace into a canvas. `states` is the whole trace
   * (used for hair/trail/particles history); `inputs[index - 1]` is the input
   * that produced the state.
   */
  drawFrame(
    map: GymMap,
    states: readonly SimState[],
    index: number,
    inputs?: readonly SimInput[],
    target?: Canvas,
    viewSize?: { width: number; height: number },
  ): Canvas {
    const room = this.room(map);
    const size = viewSize ?? this.size(map);
    const canvas = target ?? createCanvas(size.width, size.height);
    const context = canvas.getContext("2d");
    const state = states[index];
    const camera =
      this.options.camera === "room"
        ? clampCameraViewport(map, fitCameraViewport(map, size.width / size.height))
        : cameraBounds(clampCameraPosition(map, stateCameraPosition(map, state)));
    const input: GameViewDrawInput = {
      assets: this.assets,
      map,
      theme: this.theme,
      frame: index,
      state,
      states,
      stateFrameOffset: 0,
      stale: false,
      camera,
      tileLayer: room.tileLayer,
      spinnerGroups: room.spinnerGroups,
      entityKindIndices: room.kindIndices,
      staticMoverAttachments: room.staticMoverAttachments,
      viewportRevision: 0,
    };
    context.setTransform(1, 0, 0, 1, 0, 0);
    paintGame(
      canvas as unknown as HTMLCanvasElement,
      context as unknown as CanvasRenderingContext2D,
      1,
      input,
      size,
    );
    const scale = Math.min(size.width / camera.width, size.height / camera.height);
    const offsetX = (size.width - camera.width * scale) / 2;
    const offsetY = (size.height - camera.height * scale) / 2;
    const toScreen = (x: number, y: number) => ({
      x: offsetX + (x - camera.x) * scale,
      y: offsetY + (y - camera.y) * scale,
    });
    context.save();
    if (this.options.path) drawPath(context, states, index, toScreen);
    if (this.options.hitboxes) drawHitboxes(context, map, state, scale, toScreen);
    context.restore();
    if (this.options.hud) drawHud(context, size, state, index, inputs?.[index - 1]);
    return canvas;
  }
}

function drawPath(
  context: SKRSContext2D,
  states: readonly SimState[],
  index: number,
  toScreen: (x: number, y: number) => { x: number; y: number },
): void {
  context.lineWidth = 1.5;
  context.strokeStyle = "rgba(255, 230, 0, 0.85)";
  context.beginPath();
  for (let i = 0; i <= index; i += 1) {
    const p = toScreen(states[i].pos.x, states[i].pos.y - 5);
    if (i === 0) context.moveTo(p.x, p.y);
    else context.lineTo(p.x, p.y);
  }
  context.stroke();
}

function drawHitboxes(
  context: SKRSContext2D,
  map: GymMap,
  state: SimState,
  scale: number,
  toScreen: (x: number, y: number) => { x: number; y: number },
): void {
  const rect = (r: { x: number; y: number; width: number; height: number }, color: string) => {
    const p = toScreen(r.x, r.y);
    context.strokeStyle = color;
    context.strokeRect(p.x + 0.5, p.y + 0.5, r.width * scale - 1, r.height * scale - 1);
  };
  context.lineWidth = 1;
  for (const solid of map.solids) rect(solid, "rgba(255,0,0,0.6)");
  for (const entity of map.entities) {
    const color =
      entity.kind === "spikes" || entity.kind === "crystal_static_spinner"
        ? "rgba(255,80,255,0.8)"
        : entity.kind === "unknown"
          ? "rgba(160,160,160,0.6)"
          : "rgba(0,200,255,0.8)";
    rect(entity.bounds, color);
  }
  // Player.normalHitbox = Hitbox(8, 11, -4, -11); duckHitbox = Hitbox(8, 6, -4, -6).
  const height = state.ducking ? 6 : 11;
  rect({ x: state.pos.x - 4, y: state.pos.y - height, width: 8, height }, "rgba(0,255,0,1)");
}

function drawHud(
  context: SKRSContext2D,
  size: { width: number; height: number },
  state: SimState,
  index: number,
  input: SimInput | undefined,
): void {
  const fontSize = Math.max(10, Math.round(size.height / 30));
  context.font = monoFont(fontSize);
  const fmt = (value: number) => (Number.isFinite(value) ? value.toFixed(2) : String(value));
  const lines = [
    `f ${index}  ${state.state}${state.dead ? " DEAD" : ""}  in ${input ? describeInput(input) : "-"}`,
    `pos ${fmt(state.pos.x)}, ${fmt(state.pos.y)}  spd ${fmt(state.speed.x)}, ${fmt(state.speed.y)}`,
    `dash ${state.dashes}  st ${fmt(state.stamina)}${state.on_ground ? "  ground" : ""}${state.ducking ? "  duck" : ""}`,
  ];
  const lineHeight = Math.round(fontSize * 1.25);
  const width = Math.max(...lines.map((line) => context.measureText(line).width)) + 8;
  context.fillStyle = "rgba(0,0,0,0.6)";
  context.fillRect(0, 0, width, lineHeight * lines.length + 6);
  context.fillStyle = "#ffffff";
  context.textBaseline = "top";
  lines.forEach((line, row) => context.fillText(line, 4, 3 + row * lineHeight));
}

// ---------------------------------------------------------------------------
// Output encoders
// ---------------------------------------------------------------------------

export type OutputKind = "png" | "gif" | "video" | "frames";

export function outputKind(path: string): OutputKind {
  const extension = extname(path).toLowerCase();
  if (extension === ".png") return "png";
  if (extension === ".gif") return "gif";
  if ([".mp4", ".webm", ".mkv", ".mov"].includes(extension)) return "video";
  if (extension === "") return "frames";
  throw new Error(`unsupported render output ${path} (use .png, .gif, .mp4, .webm, .mkv, .mov, or a directory)`);
}

export function findFfmpeg(explicit?: string): string {
  const candidates = [explicit, process.env.FFMPEG, process.env.FFMPEG_PATH, "ffmpeg"].filter(Boolean) as string[];
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ["-version"], { stdio: "ignore" });
    if (probe.status === 0) return candidate;
  }
  throw new Error("ffmpeg not found: install it on PATH or pass --ffmpeg / set FFMPEG (GIF and PNG need no ffmpeg)");
}

export interface FrameSequence {
  width: number;
  height: number;
  /** Number of frames that will be produced. */
  count: number;
  /** Produce frame `i` (0-based, in output order). */
  frame(i: number): Canvas;
}

function ensureParent(path: string): void {
  mkdirSync(dirname(resolve(path)), { recursive: true });
}

export function writePng(canvas: Canvas, path: string): void {
  ensureParent(path);
  writeFileSync(path, canvas.toBuffer("image/png"));
}

export function writeFrames(sequence: FrameSequence, directory: string): string[] {
  mkdirSync(directory, { recursive: true });
  const files: string[] = [];
  for (let i = 0; i < sequence.count; i += 1) {
    const file = join(directory, `frame-${String(i).padStart(5, "0")}.png`);
    writeFileSync(file, sequence.frame(i).toBuffer("image/png"));
    files.push(file);
  }
  return files;
}

/** GIF via gifenc (no external tools). `fps` is the output frame rate (max 50). */
export function writeGif(sequence: FrameSequence, path: string, fps: number): void {
  const encoder = GIFEncoder();
  const delay = Math.max(20, Math.round(1000 / Math.min(50, fps)));
  for (let i = 0; i < sequence.count; i += 1) {
    const canvas = sequence.frame(i);
    const rgba = canvas.getContext("2d").getImageData(0, 0, sequence.width, sequence.height).data;
    const palette = quantize(rgba, 256, { format: "rgb565" });
    const index = applyPalette(rgba, palette, "rgb565");
    encoder.writeFrame(index, sequence.width, sequence.height, { palette, delay, repeat: 0 });
  }
  encoder.finish();
  ensureParent(path);
  writeFileSync(path, encoder.bytes());
}

/** MP4/WebM/MKV/MOV by streaming raw RGBA frames into ffmpeg. */
export async function writeVideo(
  sequence: FrameSequence,
  path: string,
  fps: number,
  ffmpegPath?: string,
): Promise<void> {
  const ffmpeg = findFfmpeg(ffmpegPath);
  const extension = extname(path).toLowerCase();
  const codec =
    extension === ".webm"
      ? ["-c:v", "libvpx-vp9", "-b:v", "0", "-crf", "30", "-pix_fmt", "yuv420p"]
      : ["-c:v", "libx264", "-preset", "veryfast", "-crf", "18", "-pix_fmt", "yuv420p", "-movflags", "+faststart"];
  ensureParent(path);
  const child = spawn(
    ffmpeg,
    [
      "-y", "-loglevel", "error",
      "-f", "rawvideo", "-pix_fmt", "rgba",
      "-s", `${sequence.width}x${sequence.height}`,
      "-r", String(fps),
      "-i", "-",
      ...codec,
      resolve(path),
    ],
    { stdio: ["pipe", "ignore", "pipe"] },
  );
  let stderr = "";
  child.stderr.on("data", (chunk) => (stderr += chunk));
  const done = new Promise<void>((resolvePromise, reject) => {
    child.on("error", reject);
    child.on("close", (code) =>
      code === 0 ? resolvePromise() : reject(new Error(`ffmpeg exited with ${code}: ${stderr.trim()}`)),
    );
  });
  for (let i = 0; i < sequence.count; i += 1) {
    const data = sequence.frame(i).getContext("2d").getImageData(0, 0, sequence.width, sequence.height).data;
    if (!child.stdin.write(Buffer.from(data.buffer, data.byteOffset, data.byteLength))) {
      await new Promise((resolveDrain) => child.stdin.once("drain", resolveDrain));
    }
  }
  child.stdin.end();
  await done;
}

/** Grid of labelled thumbnails: the quickest way for an agent to "see" a run. */
export function contactSheet(
  frames: { canvas: Canvas; label: string }[],
  columns: number,
): Canvas {
  if (frames.length === 0) throw new Error("contact sheet needs at least one frame");
  const cellWidth = frames[0].canvas.width;
  const cellHeight = frames[0].canvas.height;
  const labelHeight = Math.max(14, Math.round(cellHeight / 12));
  const cols = Math.max(1, Math.min(columns, frames.length));
  const rows = Math.ceil(frames.length / cols);
  const sheet = createCanvas(cols * cellWidth, rows * (cellHeight + labelHeight));
  const context = sheet.getContext("2d");
  context.fillStyle = "#111";
  context.fillRect(0, 0, sheet.width, sheet.height);
  context.font = monoFont(Math.round(labelHeight * 0.8));
  context.textBaseline = "top";
  frames.forEach(({ canvas, label }, i) => {
    const x = (i % cols) * cellWidth;
    const y = Math.floor(i / cols) * (cellHeight + labelHeight);
    context.fillStyle = "#ddd";
    context.fillText(label, x + 3, y + 1);
    context.drawImage(canvas, x, y + labelHeight);
  });
  return sheet;
}

export async function writeSequence(
  sequence: FrameSequence,
  path: string,
  options: { fps: number; ffmpeg?: string },
): Promise<{ kind: OutputKind; path: string; frames: number }> {
  const kind = outputKind(path);
  if (kind === "png") writePng(sequence.frame(sequence.count - 1), path);
  else if (kind === "gif") writeGif(sequence, path, options.fps);
  else if (kind === "video") await writeVideo(sequence, path, options.fps, options.ffmpeg);
  else writeFrames(sequence, path);
  return { kind, path: resolve(path), frames: kind === "png" ? 1 : sequence.count };
}
