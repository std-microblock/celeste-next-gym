/**
 * Input formats → per-frame simulator inputs.
 *
 *  - TAS action lines (CelesteTAS syntax): `frames,keys...` e.g. `5,R,J`
 *    keys: L R U D, J K (jump), X C (dash), Z V (crouch/demo dash), G H (grab),
 *    A<dirs> dash-only aim, M<dirs> move-only, F,<angle>[,<mag>] analog feather.
 *    Inline text separates lines with newlines or `;`.
 *  - JSON: array of FrameButtons (`{left,right,up,down,jump,dash,crouch_dash,grab}`),
 *    array of SimInput (`{move_x,move_y,jump_pressed,...}`), or a timeline
 *    document (`{version:2, inputs:[FrameButtons...]}`), or a resolved
 *    tas-replay document (`{inputs:[{frames,action}]}`).
 *  - `.tas` files are flattened with tools/tas-replay (Read/Repeat/labels).
 */
import { readFileSync } from "node:fs";
import { basename, dirname, extname, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { TAS_RESOLVER } from "./paths.ts";
import type { FrameButtons, SimInput } from "./types.ts";

// Bit values of StudioCommunication.Actions (tools/tas-replay/src/resolve.mjs).
export const TasActions = {
  Left: 1 << 0,
  Right: 1 << 1,
  Up: 1 << 2,
  Down: 1 << 3,
  Jump: 1 << 4,
  Jump2: 1 << 5,
  Dash: 1 << 6,
  Dash2: 1 << 7,
  Grab: 1 << 8,
  Grab2: 1 << 9,
  Feather: 1 << 12,
  DemoDash: 1 << 15,
  DemoDash2: 1 << 16,
  LeftDashOnly: 1 << 18,
  RightDashOnly: 1 << 19,
  UpDashOnly: 1 << 20,
  DownDashOnly: 1 << 21,
  LeftMoveOnly: 1 << 23,
  RightMoveOnly: 1 << 24,
  UpMoveOnly: 1 << 25,
  DownMoveOnly: 1 << 26,
} as const;

export interface TasFrame {
  actions: number;
  featherAngle?: number | null;
  featherMagnitude?: number | null;
  /** Source of this frame, for diagnostics. */
  source?: { file: string; line: number };
}

interface ActionLine {
  actions: number;
  frameCount: number;
  featherAngle: number | null;
  featherMagnitude: number | null;
}

interface TasResolver {
  parseActionLine(line: string): ActionLine | null;
  resolveTas(
    root: string,
    entry: string,
    options?: { onWarning?: (message: string) => void },
  ): {
    totalFrames: number;
    inputs: { frames: number; action: string; line: number; file: string }[];
    events: { frame: number; type: string; args: string[]; file: string; line: number }[];
  };
}

let resolverPromise: Promise<TasResolver> | null = null;
export function tasResolver(): Promise<TasResolver> {
  resolverPromise ??= import(pathToFileURL(TAS_RESOLVER).href) as Promise<TasResolver>;
  return resolverPromise;
}

/** Monocle VirtualIntegerAxis with OverlapBehaviors.TakeNewer (Celeste MoveX/MoveY). */
class TakeNewerAxis {
  value = 0;
  private turned = false;
  update(negative: boolean, positive: boolean): number {
    if (positive && negative) {
      if (!this.turned) {
        this.value *= -1;
        this.turned = true;
      }
    } else {
      this.turned = false;
      this.value = positive ? 1 : negative ? -1 : 0;
    }
    return this.value;
  }
}

const has = (actions: number, flag: number) => (actions & flag) !== 0;

/**
 * Convert expanded TAS frames to simulator inputs. Button presses are edge
 * detected per physical binding (J then K while J held is a new jump press),
 * like Monocle's VirtualButton nodes.
 */
export function tasFramesToInputs(frames: readonly TasFrame[], warnings: string[] = []): SimInput[] {
  const axisX = new TakeNewerAxis();
  const axisY = new TakeNewerAxis();
  let previous = 0;
  let warnedAnalog = false;
  let warnedSplit = false;
  return frames.map((frame) => {
    const a = frame.actions;
    let left = has(a, TasActions.Left) || has(a, TasActions.LeftMoveOnly);
    let right = has(a, TasActions.Right) || has(a, TasActions.RightMoveOnly);
    let up = has(a, TasActions.Up) || has(a, TasActions.UpMoveOnly);
    let down = has(a, TasActions.Down) || has(a, TasActions.DownMoveOnly);
    const dashOnly =
      TasActions.LeftDashOnly | TasActions.RightDashOnly | TasActions.UpDashOnly | TasActions.DownDashOnly;
    if (a & dashOnly) {
      if (!warnedSplit) {
        warnings.push("dash-only (A) / move-only (M) aim is approximated: the simulator has a single move/aim direction");
        warnedSplit = true;
      }
      left ||= has(a, TasActions.LeftDashOnly);
      right ||= has(a, TasActions.RightDashOnly);
      up ||= has(a, TasActions.UpDashOnly);
      down ||= has(a, TasActions.DownDashOnly);
    }
    if (has(a, TasActions.Feather) && frame.featherAngle != null) {
      if (!warnedAnalog) {
        warnings.push("analog feather input (F) is quantised to 8 directions");
        warnedAnalog = true;
      }
      const angle = (frame.featherAngle * Math.PI) / 180;
      const magnitude = frame.featherMagnitude ?? 1;
      const x = Math.sin(angle) * magnitude;
      const y = Math.cos(angle) * magnitude;
      const dead = 0.3;
      left ||= x < -dead;
      right ||= x > dead;
      up ||= y > dead;
      down ||= y < -dead;
    }
    const pressed = (flag: number) => has(a, flag) && !has(previous, flag);
    const input: SimInput = {
      move_x: axisX.update(left, right) as -1 | 0 | 1,
      move_y: axisY.update(up, down) as -1 | 0 | 1,
      jump_pressed: pressed(TasActions.Jump) || pressed(TasActions.Jump2),
      jump_held: has(a, TasActions.Jump) || has(a, TasActions.Jump2),
      dash_pressed: pressed(TasActions.Dash) || pressed(TasActions.Dash2),
      crouch_dash_pressed: pressed(TasActions.DemoDash) || pressed(TasActions.DemoDash2),
      grab_held: has(a, TasActions.Grab) || has(a, TasActions.Grab2),
      talk_pressed: false,
    };
    previous = a;
    return input;
  });
}

/** Expand TAS action lines (`5,R,J`) into frames. */
export async function parseTasLines(text: string, file = "<inline>"): Promise<TasFrame[]> {
  const resolver = await tasResolver();
  const frames: TasFrame[] = [];
  text
    .split(/\r?\n|;/)
    .forEach((rawLine, index) => {
      const line = rawLine.replace(/#.*$/, "").trim();
      if (!line) return;
      const parsed = resolver.parseActionLine(line);
      if (!parsed) throw new Error(`${file}:${index + 1}: not a TAS action line: "${rawLine.trim()}"`);
      for (let i = 0; i < parsed.frameCount; i += 1) {
        frames.push({
          actions: parsed.actions,
          featherAngle: parsed.featherAngle,
          featherMagnitude: parsed.featherMagnitude,
          source: { file, line: index + 1 },
        });
      }
    });
  return frames;
}

export function buttonsToInputs(buttons: readonly Partial<FrameButtons>[]): SimInput[] {
  let previous: Partial<FrameButtons> = {};
  return buttons.map((current) => {
    const input: SimInput = {
      move_x: !!current.left === !!current.right ? 0 : current.left ? -1 : 1,
      move_y: !!current.up === !!current.down ? 0 : current.up ? -1 : 1,
      jump_pressed: !!current.jump && !previous.jump,
      jump_held: !!current.jump,
      dash_pressed: !!current.dash && !previous.dash,
      crouch_dash_pressed: !!current.crouch_dash && !previous.crouch_dash,
      grab_held: !!current.grab,
      talk_pressed: false,
    };
    previous = current;
    return input;
  });
}

function normaliseSimInput(value: Partial<SimInput>): SimInput {
  return {
    move_x: Math.max(-1, Math.min(1, Math.trunc(value.move_x ?? 0))) as -1 | 0 | 1,
    move_y: Math.max(-1, Math.min(1, Math.trunc(value.move_y ?? 0))) as -1 | 0 | 1,
    jump_pressed: !!value.jump_pressed,
    jump_held: !!value.jump_held,
    dash_pressed: !!value.dash_pressed,
    crouch_dash_pressed: !!value.crouch_dash_pressed,
    grab_held: !!value.grab_held,
    talk_pressed: !!value.talk_pressed,
  };
}

/** Interpret any JSON input document. */
export async function inputsFromJson(document: unknown, warnings: string[] = []): Promise<SimInput[]> {
  if (Array.isArray(document)) {
    if (document.length === 0) return [];
    const first = document[0] as Record<string, unknown>;
    if ("move_x" in first || "jump_pressed" in first) return document.map((value) => normaliseSimInput(value));
    if (typeof first === "string") return tasFramesToInputs(await parseTasLines(document.join("\n")), warnings);
    if ("frames" in first && "action" in first) {
      return tasFramesToInputs(await parseTasLines((document as { action: string; frames: number }[]).map((entry) => entry.action).join("\n")), warnings);
    }
    return buttonsToInputs(document as Partial<FrameButtons>[]);
  }
  if (document && typeof document === "object") {
    const record = document as Record<string, unknown>;
    if (Array.isArray(record.inputs)) return inputsFromJson(record.inputs, warnings);
  }
  throw new Error("unrecognised input JSON: expected an array of FrameButtons/SimInput, or {inputs:[...]}");
}

/** Load inputs from a file: `.json`, `.tas`-style action lines, or a full `.tas` tree. */
export async function inputsFromFile(path: string, warnings: string[] = []): Promise<SimInput[]> {
  const text = readFileSync(path, "utf8");
  if (extname(path).toLowerCase() === ".json") return inputsFromJson(JSON.parse(text), warnings);
  if (extname(path).toLowerCase() === ".tas") return (await resolveTasFile(path, {}, warnings)).inputs;
  return tasFramesToInputs(await parseTasLines(text, path), warnings);
}

export interface TasResolveOptions {
  /** Directory `Read` commands resolve against (default: the file's directory). */
  root?: string;
  /** Skip frames until this label (`#lvl_2` or `lvl_2`). */
  fromLabel?: string;
  /** Stop before this label. */
  toLabel?: string;
  /** Absolute start frame (applied after fromLabel when both are given? no: overrides). */
  startFrame?: number;
  /** Number of frames to keep. */
  frameCount?: number;
}

export interface ResolvedTas {
  file: string;
  totalFrames: number;
  startFrame: number;
  endFrame: number;
  frames: TasFrame[];
  inputs: SimInput[];
  labels: { name: string; frame: number; line: number }[];
  events: { frame: number; type: string; args: string[]; file: string; line: number }[];
  /** First `console load|hard|rmx2 ...` command, if any. */
  consoleLoad?: { mode: "A" | "B" | "C"; area: string; room?: string; position?: { x: number; y: number } };
}

/** Flatten a `.tas` file (Read/Repeat/...) and convert it to simulator inputs. */
export async function resolveTasFile(
  path: string,
  options: TasResolveOptions = {},
  warnings: string[] = [],
): Promise<ResolvedTas> {
  const resolver = await tasResolver();
  const file = resolve(path);
  const root = resolve(options.root ?? dirname(file));
  const entry = file.slice(root.length + 1) || basename(file);
  const resolution = resolver.resolveTas(root, entry, { onWarning: (message) => warnings.push(message) });
  const entryName = resolution.inputs.find((input) => input.file)?.file;

  const frames: TasFrame[] = [];
  const inputStarts: { file: string; line: number; frame: number }[] = [];
  for (const input of resolution.inputs) {
    const parsed = resolver.parseActionLine(input.action);
    inputStarts.push({ file: input.file, line: input.line, frame: frames.length });
    for (let i = 0; i < input.frames; i += 1) {
      frames.push({
        actions: parsed?.actions ?? 0,
        featherAngle: parsed?.featherAngle,
        featherMagnitude: parsed?.featherMagnitude,
        source: { file: input.file, line: input.line },
      });
    }
  }

  // Labels of the entry file: frame = first input/event after the label line.
  const lines = readFileSync(file, "utf8").split(/\r?\n/);
  const entryFile = entryName && inputStarts.some((start) => start.file === entry.replace(/\\/g, "/")) ? entry.replace(/\\/g, "/") : entryName;
  const labels: ResolvedTas["labels"] = [];
  lines.forEach((text, index) => {
    const match = /^\s*#([^\s#].*?)\s*$/.exec(text);
    if (!match || /^\s*#\s/.test(text)) return;
    const lineNumber = index + 1;
    const candidates = [
      ...inputStarts.filter((start) => start.file === entryFile && start.line > lineNumber).map((start) => start.frame),
      ...resolution.events.filter((event) => event.file === entryFile && event.line > lineNumber).map((event) => event.frame),
    ];
    labels.push({ name: match[1], line: lineNumber, frame: candidates.length ? Math.min(...candidates) : frames.length });
  });

  const findLabel = (name: string) => {
    const wanted = name.replace(/^#/, "");
    const label = labels.find((candidate) => candidate.name === wanted);
    if (!label) throw new Error(`label #${wanted} not found; labels: ${labels.map((l) => l.name).join(", ")}`);
    return label.frame;
  };
  let startFrame = options.startFrame ?? (options.fromLabel ? findLabel(options.fromLabel) : 0);
  let endFrame = options.toLabel ? findLabel(options.toLabel) : frames.length;
  if (options.frameCount !== undefined) endFrame = Math.min(endFrame, startFrame + options.frameCount);
  startFrame = Math.max(0, Math.min(startFrame, frames.length));
  endFrame = Math.max(startFrame, Math.min(endFrame, frames.length));

  // Convert the whole stream so press edges at the cut point are correct.
  const allInputs = tasFramesToInputs(frames, warnings);

  let consoleLoad: ResolvedTas["consoleLoad"];
  const load = resolution.events.find((event) => event.type === "console");
  if (load) {
    const [command, area, ...rest] = load.args.flatMap((arg) => arg.split(/\s+/)).filter(Boolean);
    const mode = { load: "A", hard: "B", rmx2: "C" }[command?.toLowerCase() ?? ""] as "A" | "B" | "C" | undefined;
    if (mode && area) {
      const numbers = rest.map(Number);
      consoleLoad = { mode, area };
      if (rest.length >= 2 && numbers.slice(-2).every(Number.isFinite)) {
        consoleLoad.position = { x: numbers[rest.length - 2], y: numbers[rest.length - 1] };
        if (rest.length >= 3) consoleLoad.room = rest[0];
      } else if (rest.length >= 1) {
        consoleLoad.room = rest[0];
      }
    }
  }
  return {
    file,
    totalFrames: frames.length,
    startFrame,
    endFrame,
    frames: frames.slice(startFrame, endFrame),
    inputs: allInputs.slice(startFrame, endFrame),
    labels,
    events: resolution.events,
    consoleLoad,
  };
}

/** Compact one-line description of an input (for tables / HUD). */
export function describeInput(input: SimInput): string {
  const parts: string[] = [];
  if (input.move_x < 0) parts.push("L");
  if (input.move_x > 0) parts.push("R");
  if (input.move_y < 0) parts.push("U");
  if (input.move_y > 0) parts.push("D");
  if (input.jump_pressed) parts.push("J!");
  else if (input.jump_held) parts.push("J");
  if (input.dash_pressed) parts.push("X!");
  if (input.crouch_dash_pressed) parts.push("Z!");
  if (input.grab_held) parts.push("G");
  return parts.join(",") || "-";
}

/** Render inputs back as run-length TAS lines (`5,R,J`). Uses held state, so it is lossy only for press edges. */
export function inputsToTas(inputs: readonly SimInput[]): string {
  const tokens = (input: SimInput) => {
    const keys: string[] = [];
    if (input.move_x < 0) keys.push("L");
    if (input.move_x > 0) keys.push("R");
    if (input.move_y < 0) keys.push("U");
    if (input.move_y > 0) keys.push("D");
    if (input.jump_held || input.jump_pressed) keys.push("J");
    if (input.dash_pressed) keys.push("X");
    if (input.crouch_dash_pressed) keys.push("Z");
    if (input.grab_held) keys.push("G");
    return keys.join(",");
  };
  const lines: string[] = [];
  let current = "";
  let count = 0;
  // A new press of an already-held button needs its own line boundary; we
  // alternate J/K for back-to-back jump presses to keep the edge.
  let previous: SimInput | undefined;
  let useK = false;
  const flush = () => {
    if (count > 0) lines.push(`${String(count).padStart(4)}${current ? "," + current : ""}`);
  };
  for (const input of inputs) {
    let line = tokens(input);
    const repress = previous && input.jump_pressed && (previous.jump_held || previous.jump_pressed);
    if (repress) useK = !useK;
    if (useK) line = line.replace(/\bJ\b/, "K");
    const dashRepress = previous?.dash_pressed && input.dash_pressed;
    if (line === current && !repress && !dashRepress && !(input.dash_pressed && count > 0)) {
      count += 1;
    } else {
      flush();
      current = line;
      count = 1;
    }
    if (!input.jump_held && !input.jump_pressed) useK = false;
    previous = input;
  }
  flush();
  return lines.join("\n");
}
