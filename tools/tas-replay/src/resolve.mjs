// resolve.mjs
//
// Exact, self-validating resolver that flattens a Celeste `.tas` file (including
// its `Read` / `Repeat` / `AutoInput` / `StunPause` tree) into a canonical flat
// per-frame input stream.
//
// The implementation is a line-by-line port of the CelesteTAS parse pipeline:
//
//   EverestAPI/CelesteTAS-EverestInterop
//     Source/TAS/Input/InputController.cs          ReadLine / ReadLines / ReadFile / AddFrames
//     Source/TAS/Input/InputFrame.cs               InputFrame.TryParse
//     Source/TAS/Input/Command.cs                  Command.TryParse / Setup
//     Source/TAS/Input/Commands/*.cs               every directive
//     StudioCommunication/CommandLine.cs           command token splitter
//     StudioCommunication/ActionLine.cs            input-line grammar
//     StudioCommunication/Actions.cs               action character table
//     StudioCommunication/CommentLine.cs           label detection
//     StudioCommunication/Parsing.cs               Read file/line resolution
//     StudioCommunication/FastForwardLine.cs       breakpoint lines
//
// Every behavioural claim is annotated with the reference `file:line` it was
// ported from. See docs/tas-format.md.

import fs from 'node:fs';
import path from 'node:path';

export const RESOLVE_VERSION = 1;

const INT32_MAX = 2147483647;

// ---------------------------------------------------------------------------
// StudioCommunication/Actions.cs:9-39  -- the [Flags] Actions enum
// ---------------------------------------------------------------------------
export const Actions = {
  None: 0,
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
  Start: 1 << 10,
  Restart: 1 << 11,
  Feather: 1 << 12,
  Journal: 1 << 13,
  Confirm: 1 << 14,
  DemoDash: 1 << 15,
  DemoDash2: 1 << 16,
  DashOnly: 1 << 17,
  LeftDashOnly: 1 << 18,
  RightDashOnly: 1 << 19,
  UpDashOnly: 1 << 20,
  DownDashOnly: 1 << 21,
  MoveOnly: 1 << 22,
  LeftMoveOnly: 1 << 23,
  RightMoveOnly: 1 << 24,
  UpMoveOnly: 1 << 25,
  DownMoveOnly: 1 << 26,
  PressedKey: 1 << 27,
};

export const ACTIONS_BY_NAME = Actions;

// ---------------------------------------------------------------------------
// StudioCommunication/Actions.cs:42-64  -- Chars table
// StudioCommunication/Actions.cs:66-80  -- DashOnlyChars / MoveOnlyChars
// StudioCommunication/Actions.cs:86-109 -- ActionForChar
// ---------------------------------------------------------------------------
export const ACTION_CHARS = Object.freeze({
  L: 'Left',
  R: 'Right',
  U: 'Up',
  D: 'Down',
  J: 'Jump',
  K: 'Jump2',
  Z: 'DemoDash',
  V: 'DemoDash2',
  X: 'Dash',
  C: 'Dash2',
  G: 'Grab',
  H: 'Grab2',
  S: 'Start',
  Q: 'Restart',
  N: 'Journal',
  O: 'Confirm',
  A: 'DashOnly',
  M: 'MoveOnly',
  P: 'PressedKey',
  F: 'Feather',
});

export const DASH_ONLY_CHARS = Object.freeze({
  L: 'LeftDashOnly',
  R: 'RightDashOnly',
  U: 'UpDashOnly',
  D: 'DownDashOnly',
});

export const MOVE_ONLY_CHARS = Object.freeze({
  L: 'LeftMoveOnly',
  R: 'RightMoveOnly',
  U: 'UpMoveOnly',
  D: 'DownMoveOnly',
});

export function actionForChar(c) {
  if (typeof c !== 'string' || c.length === 0) return Actions.None;
  const up = c.toUpperCase();
  const name = ACTION_CHARS[up];
  return name === undefined ? Actions.None : Actions[name];
}

// Actions.cs:197-205 ToDashOnlyActions
export function toDashOnlyAction(a) {
  switch (a) {
    case Actions.Left: return Actions.LeftDashOnly;
    case Actions.Right: return Actions.RightDashOnly;
    case Actions.Up: return Actions.UpDashOnly;
    case Actions.Down: return Actions.DownDashOnly;
    default: return a;
  }
}

// Actions.cs:207-215 ToMoveOnlyActions
export function toMoveOnlyAction(a) {
  switch (a) {
    case Actions.Left: return Actions.LeftMoveOnly;
    case Actions.Right: return Actions.RightMoveOnly;
    case Actions.Up: return Actions.UpMoveOnly;
    case Actions.Down: return Actions.DownMoveOnly;
    default: return a;
  }
}

// ---------------------------------------------------------------------------
// Small C#-compatible primitive helpers
// ---------------------------------------------------------------------------

const INT_RE = /^[+-]?[0-9]+$/;
/** C# `int.TryParse(string, out int)` with default (Integer) NumberStyles. */
export function intTryParse(s) {
  if (typeof s !== 'string') return null;
  const t = s.trim();
  if (!INT_RE.test(t)) return null;
  const v = Number(t);
  if (!Number.isInteger(v) || v < -2147483648 || v > 2147483647) return null;
  return v;
}

// C# float.TryParse(s, NumberStyles.Float, InvariantCulture, out _)
const FLOAT_RE = /^[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$/;
export function floatTryParse(s) {
  if (typeof s !== 'string') return false;
  const t = s.trim();
  if (!FLOAT_RE.test(t)) return false;
  return Number.isFinite(Number(t));
}

function isWhiteSpace(ch) {
  return /\s/.test(ch);
}

// C# char.IsLetter -- Unicode aware.
function isLetter(ch) {
  return /\p{L}/u.test(ch);
}

// C# char.IsDigit -- the data is ASCII; non-ASCII digits would fail int.TryParse
// in CelesteTAS anyway, producing the same "not an input line" outcome.
function isAsciiDigit(ch) {
  return ch >= '0' && ch <= '9';
}

/**
 * Mimics `System.IO.File.ReadAllLines` / `StreamReader.ReadLine` line splitting:
 * `\r\n`, `\n` and `\r` all terminate a line, and a trailing terminator does not
 * produce a phantom final empty line.
 */
export function splitDotNetLines(text) {
  const out = [];
  let cur = '';
  let started = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '\n') {
      out.push(cur);
      cur = '';
      started = false;
    } else if (c === '\r') {
      if (text[i + 1] === '\n') i++;
      out.push(cur);
      cur = '';
      started = false;
    } else {
      cur += c;
      started = true;
    }
  }
  if (started || cur.length > 0) out.push(cur);
  return out;
}

// ---------------------------------------------------------------------------
// StudioCommunication/CommentLine.cs:14-18
// ---------------------------------------------------------------------------
export function isLabel(line) {
  return line.length >= 2 && line[0] === '#' && isLetter(line[1]);
}

// ---------------------------------------------------------------------------
// StudioCommunication/CommandLine.cs:24-150  CommandLine.TryParse
// ---------------------------------------------------------------------------
const SEPARATOR_RE = /(?:\s+)|(?:\s*,\s*)/;

export function parseCommandLine(line) {
  const lineTrimmed = line.replace(/^\s+/, '');
  if (lineTrimmed.length === 0 || !isLetter(lineTrimmed[0])) return null;

  const leadingWhitespace = line.length - lineTrimmed.length;
  const m = SEPARATOR_RE.exec(lineTrimmed);
  if (!m || m[0].length === 0) {
    return { command: lineTrimmed, args: [], originalText: line, argumentSeparator: '' };
  }

  const separator = m[0];
  const args = [];
  const groupStack = [];
  let currentArg = '';
  // CommandLine.cs:61
  let i = m.index + separator.length + leadingWhitespace;

  for (; i < line.length; i++) {
    // CommandLine.cs:66
    if (groupStack.length === 0 && line.startsWith(separator, i)) {
      args.push(currentArg);
      currentArg = '';
      i += separator.length - 1;
      continue;
    }

    const c = line[i];
    // CommandLine.cs:77-133
    if (c === '"') {
      if (groupStack.length === 0 || groupStack[groupStack.length - 1] === '"') {
        if (groupStack.length > 0 && groupStack[groupStack.length - 1] === '"') groupStack.pop();
        else groupStack.push('"');
      } else {
        currentArg += c;
      }
    } else if (c === '[' || c === '{') {
      groupStack.push(c);
      currentArg += c;
    } else if (c === ']') {
      if (groupStack.length > 0 && groupStack[groupStack.length - 1] === '[') groupStack.pop();
      else return null; // unopened bracket
      currentArg += c;
    } else if (c === '}') {
      if (groupStack.length > 0 && groupStack[groupStack.length - 1] === '{') groupStack.pop();
      else return null; // unopened brace
      currentArg += c;
    } else if (c === '\\') {
      if (i === line.length - 1) return null; // invalid escape sequence
      const next = line[++i];
      currentArg += next === 'n' ? '\n' : next;
    } else {
      currentArg += c;
    }
  }

  args.push(currentArg); // CommandLine.cs:137
  return {
    command: lineTrimmed.slice(0, m.index),
    args,
    originalText: line,
    argumentSeparator: separator,
  };
}

// ---------------------------------------------------------------------------
// StudioCommunication/ActionLine.cs:8-317
// ---------------------------------------------------------------------------

/** The subset of `ActionLine` this resolver needs: parse success + frame count. */
function makeActionLine() {
  return {
    actions: Actions.None,
    frames: '',
    frameCount: 0,
    featherAngle: null,
    featherMagnitude: null,
    customBindings: new Set(),
  };
}

function setFrames(al, value) {
  al.frames = value.trim(); // ActionLine.cs:20-23
  const parsed = intTryParse(al.frames);
  al.frameCount = parsed === null ? 0 : parsed;
}

// ActionLine.cs:42-142  TryParseStrict
function tryParseStrict(line, al, ignoreInvalidFloats = true) {
  al.actions = Actions.None;
  al.frames = '';
  al.frameCount = 0;
  al.featherAngle = null;
  al.featherMagnitude = null;
  al.customBindings = new Set();

  // ActionLine.cs:47  line.Trim().Split(',', StringSplitOptions.TrimEntries)
  const tokens = line.trim().split(',').map((t) => t.trim());
  if (tokens.length === 0) return false;

  // ActionLine.cs:53-57
  if (tokens[0].trim().length === 0 || intTryParse(tokens[0]) !== null) {
    setFrames(al, tokens[0]);
  } else {
    return false;
  }

  for (let i = 1; i < tokens.length; i++) {
    if (tokens[i].trim().length === 0) continue; // ActionLine.cs:60

    const action = actionForChar(tokens[i][0]); // ActionLine.cs:62
    al.actions |= action;

    if (action === Actions.DashOnly) { // ActionLine.cs:66-71
      for (let j = 1; j < tokens[i].length; j++) {
        al.actions |= toDashOnlyAction(actionForChar(tokens[i][j]));
      }
      continue;
    }
    if (action === Actions.MoveOnly) { // ActionLine.cs:72-77
      for (let j = 1; j < tokens[i].length; j++) {
        al.actions |= toMoveOnlyAction(actionForChar(tokens[i][j]));
      }
      continue;
    }
    if (action === Actions.PressedKey) { // ActionLine.cs:78-81
      al.customBindings = new Set(
        tokens[i].slice(1).toUpperCase().split('').map((c) => c),
      );
      continue;
    }
    if (tokens[i].length !== 1) return false; // ActionLine.cs:82-85

    // ActionLine.cs:87-128  feather angle / magnitude
    let validAngle = true;
    if (action === Actions.Feather && i + 1 < tokens.length
        && (validAngle = floatTryParse(tokens[i + 1]))) {
      const angle = Number(tokens[i + 1].trim());
      if (angle > 360) al.featherAngle = '360';
      else if (angle < 0) al.featherAngle = '0';
      else al.featherAngle = tokens[i + 1];
      i++;

      let validMagnitude = true;
      if (i + 1 < tokens.length
          && (tokens[i + 1].trim().length === 0
              || (validMagnitude = floatTryParse(tokens[i + 1])))) {
        if (floatTryParse(tokens[i + 1])) {
          const mag = Number(tokens[i + 1].trim());
          if (mag > 1) al.featherMagnitude = '1';
          else if (mag < 0) al.featherMagnitude = '0';
          else al.featherMagnitude = tokens[i + 1];
        } else {
          al.featherMagnitude = tokens[i + 1];
        }
        i++;
      } else if (!validMagnitude && !ignoreInvalidFloats) {
        return false;
      }
    } else if (!validAngle && i + 2 < tokens.length && tokens[i + 1].length === 0
               && (validAngle = floatTryParse(tokens[i + 2]))) {
      const angle = Number(tokens[i + 2].trim());
      if (angle > 360) al.featherAngle = '360';
      else if (angle < 0) al.featherAngle = '0';
      else al.featherAngle = tokens[i + 1];
      i += 2;
    } else if (!validAngle && !ignoreInvalidFloats) {
      return false;
    }
  }

  // ActionLine.cs:131-139
  if (al.frames.length === 0
      && al.actions === Actions.None
      && al.customBindings.size === 0
      && al.featherAngle === null
      && al.featherMagnitude === null) {
    return false;
  }
  return true;
}

const PS = {
  Frame: 'Frame',
  Action: 'Action',
  DashOnly: 'DashOnly',
  MoveOnly: 'MoveOnly',
  PressedKey: 'PressedKey',
  FeatherAngle: 'FeatherAngle',
  FeatherMagnitude: 'FeatherMagnitude',
};

// ActionLine.cs:146-299  TryParseLoose
function tryParseLoose(line, al, ignoreInvalidFloats = true) {
  al.actions = Actions.None;
  al.frames = '';
  al.frameCount = 0;
  al.featherAngle = null;
  al.featherMagnitude = null;
  al.customBindings = new Set();

  let state = PS.Frame;
  let currValue = '';

  const handleAction = (c) => {
    const action = actionForChar(c);
    al.actions |= action;
    switch (action) {
      case Actions.DashOnly: state = PS.DashOnly; break;
      case Actions.MoveOnly: state = PS.MoveOnly; break;
      case Actions.PressedKey: state = PS.PressedKey; break;
      case Actions.Feather: state = PS.FeatherAngle; break;
      default: state = PS.Action; break;
    }
  };

  for (const c of line) {
    if (isWhiteSpace(c)) continue; // ActionLine.cs:154-156

    if (state === PS.Frame) {
      if (c === ',') { // ActionLine.cs:161-172
        if (currValue.trim().length > 0 && intTryParse(currValue) === null) return false;
        setFrames(al, currValue);
        currValue = '';
        state = PS.Action;
        continue;
      }
      if (isAsciiDigit(c)) { // ActionLine.cs:174-175
        currValue += c;
      } else {
        if (intTryParse(currValue) === null) return false; // ActionLine.cs:177-180
        setFrames(al, currValue);
        currValue = '';
        handleAction(c);
      }
      continue;
    }

    if (state === PS.Action) { // ActionLine.cs:189-205
      if (c === ',') continue;
      handleAction(c);
      continue;
    }

    if (state === PS.DashOnly) { // ActionLine.cs:207-220
      if (c === ',') { state = PS.Action; continue; }
      const action = actionForChar(c);
      if (action !== Actions.Left && action !== Actions.Right
          && action !== Actions.Up && action !== Actions.Down) {
        handleAction(c);
      } else {
        al.actions |= toDashOnlyAction(action);
      }
      continue;
    }

    if (state === PS.MoveOnly) { // ActionLine.cs:222-235
      if (c === ',') { state = PS.Action; continue; }
      const action = actionForChar(c);
      if (action !== Actions.Left && action !== Actions.Right
          && action !== Actions.Up && action !== Actions.Down) {
        handleAction(c);
      } else {
        al.actions |= toMoveOnlyAction(action);
      }
      continue;
    }

    if (state === PS.PressedKey) { // ActionLine.cs:237-246
      if (c === ',') { state = PS.Action; continue; }
      al.customBindings.add(c.toUpperCase());
      continue;
    }

    if (state === PS.FeatherAngle) { // ActionLine.cs:248-262
      if (c === ',') { state = PS.FeatherMagnitude; continue; }
      if (isAsciiDigit(c) || c === '.') {
        al.featherAngle = (al.featherAngle ?? '') + c;
      } else {
        handleAction(c);
      }
      continue;
    }

    if (state === PS.FeatherMagnitude) { // ActionLine.cs:264-278
      if (c === ',') { state = PS.Action; continue; }
      if (isAsciiDigit(c) || c === '.') {
        al.featherMagnitude = (al.featherMagnitude ?? '') + c;
      } else {
        handleAction(c);
      }
      continue;
    }
  }

  // ActionLine.cs:283-296  clamp angle / magnitude
  if (al.featherAngle !== null) {
    if (floatTryParse(al.featherAngle)) {
      const a = Number(al.featherAngle);
      al.featherAngle = String(Math.min(360, Math.max(0, a)));
    } else if (!ignoreInvalidFloats) {
      return false;
    }
  }
  if (al.featherMagnitude !== null) {
    if (floatTryParse(al.featherMagnitude)) {
      const mg = Number(al.featherMagnitude);
      al.featherMagnitude = String(Math.min(1, Math.max(0, mg)));
    } else if (!ignoreInvalidFloats) {
      return false;
    }
  }

  return state !== PS.Frame; // ActionLine.cs:298
}

/**
 * ActionLine.TryParse -- ActionLine.cs:39.
 * Returns null when the line is not an action line at all.
 */
export function parseActionLine(line, ignoreInvalidFloats = true) {
  const al = makeActionLine();
  if (tryParseStrict(line, al, ignoreInvalidFloats)) return al;
  const al2 = makeActionLine();
  if (tryParseLoose(line, al2, ignoreInvalidFloats)) return al2;
  return null;
}

// ---------------------------------------------------------------------------
// StudioCommunication/FastForwardLine.cs:10-36
// ---------------------------------------------------------------------------
export function isFastForwardLine(line) {
  return line.trimStart().startsWith('***');
}

// ---------------------------------------------------------------------------
// StudioCommunication/Parsing.cs:11-80  FindReadTargetFile
// ---------------------------------------------------------------------------
function listDir(dir) {
  try {
    return fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return [];
  }
}

export function findReadTargetFile(fileDirectory, filePath) {
  let candidate = path.join(fileDirectory, filePath);
  if (!candidate.endsWith('.tas')) candidate += '.tas';
  if (fs.existsSync(candidate) && fs.statSync(candidate).isFile()) return candidate;

  // Windows allows case-insensitive names, but Linux/macOS don't...
  const components = filePath.split(/[/\\]/).filter((c) => c.length > 0);
  if (components.length === 0) return null;

  let realDirectory = fileDirectory;
  for (let i = 0; i < components.length - 1; i++) {
    const directory = components[i];
    if (directory === '..') {
      const parent = path.dirname(realDirectory);
      if (!parent || parent === realDirectory) return null;
      realDirectory = parent;
      continue;
    }
    const dirs = listDir(realDirectory)
      .filter((e) => e.isDirectory() && e.name.toLowerCase() === directory.toLowerCase());
    if (dirs.length > 1) return null; // ambiguous
    if (dirs.length === 0) return null;
    realDirectory = path.join(realDirectory, dirs[0].name);
  }

  const file = stripExtension(components[components.length - 1]);
  const files = listDir(realDirectory)
    .filter((e) => e.isFile()
      && stripExtension(e.name).toLowerCase().startsWith(file.toLowerCase()));
  if (files.length > 1) return null; // ambiguous
  if (files.length === 1) {
    const p = path.join(realDirectory, files[0].name);
    if (fs.existsSync(p)) return p;
  }
  return null;
}

function stripExtension(name) {
  const base = path.basename(name);
  const ext = path.extname(base);
  return ext.length === 0 ? base : base.slice(0, base.length - ext.length);
}

// ---------------------------------------------------------------------------
// StudioCommunication/Parsing.cs:83-103  TryGetLineTarget
// ---------------------------------------------------------------------------
export function tryGetLineTarget(labelOrLineNumber, lines) {
  const asNumber = intTryParse(labelOrLineNumber);
  if (asNumber !== null) return { lineNumber: asNumber, isLabel: false };

  const escaped = labelOrLineNumber.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const re = new RegExp(`^#\\s*${escaped}$`);
  for (let lineNumber = 1; lineNumber <= lines.length; lineNumber++) {
    if (re.test(lines[lineNumber - 1].trim())) {
      return { lineNumber, isLabel: true };
    }
  }
  return null;
}

// ---------------------------------------------------------------------------
// Registered TasCommand names and aliases.
// Collected from every `[TasCommand(...)]` attribute in the reference checkout.
// `ExecuteTiming` = Parse means CelesteTAS runs the command *while parsing*.
// `runtime` is the default when ExecuteTiming is omitted (Command.cs:38).
// ---------------------------------------------------------------------------
export const COMMANDS = [
  // Command.cs:104 Command.IsName() is case-insensitive.
  { name: 'Read', aliases: [], timing: 'parse' }, // ReadCommand.cs:119
  { name: 'Play', aliases: [], timing: 'parse' }, // PlayCommand.cs:40
  { name: 'Repeat', aliases: [], timing: 'parse' }, // RepeatCommand.cs:42
  { name: 'EndRepeat', aliases: [], timing: 'parse' }, // RepeatCommand.cs:70
  { name: 'Add', aliases: [], timing: 'parse' }, // LibTasHelper.cs:258
  { name: 'ExportLibTAS', aliases: ['StartExportLibTAS'], timing: 'parse' }, // LibTasHelper.cs:237
  { name: 'EndExportLibTAS', aliases: ['FinishExportLibTAS'], timing: 'parse' }, // LibTasHelper.cs:252
  { name: 'Skip', aliases: [], timing: 'parse' }, // LibTasHelper.cs:267
  { name: 'Marker', aliases: [], timing: 'parse' }, // LibTasHelper.cs:275
  { name: 'AnalogMode', aliases: ['AnalogueMode'], timing: 'parse' }, // AnalogHelper.cs:199
  { name: 'AutoInput', aliases: [], timing: 'parse' }, // AutoInputCommand.cs:65
  { name: 'StartAutoInput', aliases: [], timing: 'parse' }, // AutoInputCommand.cs:84
  { name: 'EndAutoInput', aliases: [], timing: 'parse' }, // AutoInputCommand.cs:101
  { name: 'SkipInput', aliases: ['SkipAutoInput'], timing: 'both' }, // AutoInputCommand.cs:122
  { name: 'Assert', aliases: [], timing: 'both' }, // AssertCommand.cs:41
  { name: 'console', aliases: [], timing: 'runtime' }, // ConsoleCommand.cs:197
  { name: 'Gun', aliases: [], timing: 'runtime' }, // GunCommand.cs:24
  { name: 'EnforceLegal', aliases: ['EnforceMainGame'], timing: 'both' }, // EnforceLegalCommand.cs:9
  { name: 'ExitGame', aliases: [], timing: 'runtime' }, // ExitGameCommand.cs:10
  { name: 'Invoke', aliases: [], timing: 'runtime' }, // InvokeCommand.cs:87
  { name: 'EvalLua', aliases: [], timing: 'runtime' }, // EvalLuaCommand.cs:48
  { name: 'Set', aliases: [], timing: 'runtime' }, // SetCommand.cs:84
  { name: 'RecordCount', aliases: ['RecordCount:', 'RecordCount\uff1a'], timing: 'runtime' }, // MetadataCommands.cs:137
  { name: 'FileTime', aliases: ['FileTime:', 'FileTime\uff1a'], timing: 'runtime' }, // MetadataCommands.cs:142
  { name: 'ChapterTime', aliases: ['ChapterTime:', 'ChapterTime\uff1a'], timing: 'runtime' }, // MetadataCommands.cs:147
  { name: 'RealTime', aliases: ['RealTime:', 'RealTime\uff1a'], timing: 'runtime' }, // MetadataCommands.cs:152
  { name: 'MidwayFileTime', aliases: ['MidwayFileTime:', 'MidwayFileTime\uff1a'], timing: 'runtime' }, // MetadataCommands.cs:157
  { name: 'MidwayChapterTime', aliases: ['MidwayChapterTime:', 'MidwayChapterTime\uff1a'], timing: 'runtime' }, // MetadataCommands.cs:168
  { name: 'MidwayRealTime', aliases: ['MidwayRealTime:', 'MidwayRealTime\uff1a'], timing: 'runtime' }, // MetadataCommands.cs:179
  { name: 'ActivatedLobbyWarps', aliases: ['ActivatedLobbyWarps:', 'ActivatedLobbyWarps\uff1a'], timing: 'runtime' }, // MetadataCommands.cs:190
  { name: 'Mouse', aliases: [], timing: 'runtime' }, // MouseCommand.cs:41
  { name: 'Press', aliases: [], timing: 'runtime' }, // PressCommand.cs:30
  { name: 'StartRecording', aliases: [], timing: 'both' }, // RecordingCommand.cs:23
  { name: 'StopRecording', aliases: [], timing: 'both' }, // RecordingCommand.cs:72
  { name: 'Safe', aliases: [], timing: 'both' }, // SafeCommand.cs:11
  { name: 'Unsafe', aliases: [], timing: 'both' }, // SafeCommand.cs:20
  { name: 'SaveAndQuitReenter', aliases: [], timing: 'both' }, // SaveAndQuitReenterCommand.cs:53
  { name: 'SelectCampaign', aliases: [], timing: 'both' }, // SelectCampaignCommand.cs:96
  { name: 'StunPause', aliases: [], timing: 'both' }, // StunPauseCommand.cs:133
  { name: 'EndStunPause', aliases: [], timing: 'both' }, // StunPauseCommand.cs:196
  { name: 'StunPauseMode', aliases: [], timing: 'both' }, // StunPauseCommand.cs:205
  { name: 'RequireDependency', aliases: [], timing: 'parse' }, // RequireDependencyCommand.cs:131
  { name: 'SeedRandom', aliases: [], timing: 'both' }, // SeededRandomness.cs:160
  { name: 'ExportRoomInfo', aliases: ['StartExportRoomInfo'], timing: 'runtime' }, // ExportRoomInfo.cs:82
  { name: 'EndExportRoomInfo', aliases: ['FinishExportRoomInfo'], timing: 'runtime' }, // ExportRoomInfo.cs:89
  { name: 'ExportGameInfo', aliases: ['StartExportGameInfo'], timing: 'runtime' }, // ExportGameInfo.cs:30
  { name: 'EndExportGameInfo', aliases: ['FinishExportGameInfo'], timing: 'runtime' }, // ExportGameInfo.cs:44
  { name: 'CompleteInfo', aliases: [], timing: 'runtime' }, // AreaCompleteInfo.cs:145
  { name: 'Author:', aliases: [], timing: 'runtime' }, // DummyCommands.cs:6
  { name: 'FrameCount:', aliases: [], timing: 'runtime' }, // DummyCommands.cs:9
  { name: 'TotalRecordCount:', aliases: [], timing: 'runtime' }, // DummyCommands.cs:12
];

const COMMAND_LOOKUP = new Map();
for (const cmd of COMMANDS) {
  COMMAND_LOOKUP.set(cmd.name.toLowerCase(), cmd);
  for (const a of cmd.aliases) COMMAND_LOOKUP.set(a.toLowerCase(), cmd);
}

/** Command.cs:93-116  Command.TryParse */
export function tryParseCommand(lineText) {
  if (lineText.trim().length === 0) return null;
  if (!isLetter(lineText[0])) return null;
  const cl = parseCommandLine(lineText);
  if (!cl) return null;
  const info = COMMAND_LOOKUP.get(cl.command.toLowerCase());
  if (!info) return null;
  return { commandLine: cl, attribute: info };
}

function isCommand(command, name) {
  const attr = command.attribute;
  return attr.name.toLowerCase() === name.toLowerCase()
    || attr.aliases.some((a) => a.toLowerCase() === name.toLowerCase());
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------
export class TasAbortError extends Error {
  constructor(message, where) {
    super(where ? `${message} (${where.file}:${where.line})` : message);
    this.name = 'TasAbortError';
    this.where = where;
  }
}

// ---------------------------------------------------------------------------
// Default options
// ---------------------------------------------------------------------------

/**
 * Parts of the CelesteTAS parse pass depend on live *game* state that a TAS file
 * cannot express. They are modelled here as explicit options so callers can pin
 * them; see docs/tas-format.md ("game-state dependent constructs").
 */
export const DEFAULT_OPTIONS = Object.freeze({
  /**
   * `SaveAndQuitReenterCommand.ActiveFileSlot`
   * (SaveAndQuitReenterCommand.cs:14-26). `-1` means "no save file loaded".
   *
   * The default `0` is not arbitrary: the recorded in-game frame trace of
   * `0 - 100%.tas` in this workspace shows the `slot >= 0` branch with no
   * `1,D` / `1,F,180` slot-selection frames, i.e. `ActiveFileSlot == 0`
   * (see docs/tas-format.md, "Validating against a real game trace").
   */
  saveAndQuitReenterSlot: 0,
  /** `ModUtils.IsInstalled("Randomizer")` (SaveAndQuitReenterCommand.cs:78). */
  randomizerInstalled: false,
  /** `Celeste.Celeste.PlayMode` (SaveAndQuitReenterCommand.cs:67). */
  playMode: 'Normal',
  /**
   * `SelectCampaignCommand.EmptyFileSlot` (SelectCampaignCommand.cs:68-93).
   * `-1` = "no save slots exist at all".
   */
  selectCampaignEmptyFileSlot: -1,
  /** `SelectCampaignCommand.MaxSaveFileSlots` (SelectCampaignCommand.cs:39-65). */
  selectCampaignMaxSaveFileSlots: 3,
  /** `Settings.Instance.VariantsUnlocked` (SelectCampaignCommand.cs:417). */
  variantsUnlocked: false,
  /**
   * `Dialog.Clean("name_letters")` split into rows, as used by
   * SelectCampaignCommand.cs:192-193. Space is a separate, widest button.
   */
  nameLetters: ['ABCDEFGHIJKLM', 'NOPQRSTUVWXYZ', '0123456789', '.,-!?\'"<>:;()'],
  /** Language of the game settings (SelectCampaignCommand.cs:175-178). */
  language: 'english',
  /**
   * Called with a message for every construct whose frame count depends on live
   * game state and therefore cannot be resolved from the `.tas` text alone.
   */
  onWarning: null,
  /** Run `selfCheck` on the result and throw on an invariant violation. */
  selfCheck: true,
  /**
   * Per-character advance widths used by `ActiveFont.Measure`
   * (SelectCampaignCommand.cs:202-215). Celeste's font is proportional, so the
   * grid geometry -- and therefore the number of frames the BFS emits -- is
   * font/platform dependent. `null` means "no measurements available", in which
   * case SelectCampaign's name-entry frames cannot be reproduced exactly.
   */
  fontWidths: null,
  /** `CoreModule.Settings.DefaultStartingLevelSet` (SelectCampaignCommand.cs:361). */
  defaultStartingLevelSet: 'Celeste',
  /** Area level sets in `AreaData.Areas` order, used by ChangeSelectedCampaign. */
  areaLevelSets: ['Celeste'],
  /** Ignore Read/commands that fail (AbortTas) instead of throwing. */
  ignoreAborts: false,
});

// ---------------------------------------------------------------------------
// Resolver
// ---------------------------------------------------------------------------
class Resolver {
  constructor(rootDir, options) {
    this.rootDir = path.resolve(rootDir);
    this.options = { ...DEFAULT_OPTIONS, ...options };

    this.inputs = []; // flat per-AddFrames entries, in play order
    this.totalFrames = 0;
    this.events = [];

    this.repeatStack = [];
    this.autoInputArgs = new Map(); // keyed by absolute file path
    this.readCommandStack = [];
    this.disallowUnsafeInputParsing = true; // SafeCommand.cs:9
    this.enforceLegalEnabledWhenParsing = false; // EnforceLegalCommand.cs:7
    this.stunPauseGlobalModeParsing = null; // StunPauseCommand.cs:65
    this.enableBreakpointParsing = true; // InputController.cs:83
    this.usedFiles = new Set();
    this.mainFilePath = null;

    this.fileCache = new Map();
  }

  // -- file helpers -------------------------------------------------------
  readLinesOf(absPath) {
    let cached = this.fileCache.get(absPath);
    if (!cached) {
      let text = fs.readFileSync(absPath, 'utf8');
      // .NET's StreamReader detects and removes a UTF-8 BOM, so `File.ReadLines`
      // never shows it to the parser.
      if (text.charCodeAt(0) === 0xfeff) text = text.slice(1);
      cached = splitDotNetLines(text);
      this.fileCache.set(absPath, cached);
    }
    return cached;
  }

  warn(message) {
    if (typeof this.options.onWarning === 'function') this.options.onWarning(message);
  }

  relPath(absPath) {
    const rel = path.relative(this.rootDir, absPath);
    if (rel.length === 0) return absPath;
    return rel.split(path.sep).join('/');
  }

  // -- entry point --------------------------------------------------------
  resolve(entryName) {
    const entryAbs = path.resolve(this.rootDir, entryName);
    if (!fs.existsSync(entryAbs)) {
      throw new Error(`TAS entry file not found: ${entryAbs}`);
    }
    this.mainFilePath = entryAbs;

    // InputController.cs:233-247  ReadFile(path)
    this.readFile(entryAbs, 0, INT32_MAX, 0, 0, 0);

    // InputController.cs:141  AttributeUtils.Invoke<ParseFileEndAttribute>()
    this.parseFileEnd();

    // InputController.cs:74  Checksum forces the full input list
    const result = {
      version: RESOLVE_VERSION,
      entry: entryName,
      totalFrames: this.totalFrames,
      inputs: this.inputs,
      events: this.events,
    };

    if (this.options.selfCheck !== false) {
      const problems = selfCheck(result);
      if (problems.length > 0) {
        throw new Error(`resolveTas self-check failed:\n  - ${problems.join('\n  - ')}`);
      }
    }
    return result;
  }

  // InputController.cs:233-247
  readFile(absPath, startLine = 0, endLine = INT32_MAX, studioLine = 0, repeatIndex = 0, repeatCount = 0) {
    if (!fs.existsSync(absPath)) return false;
    this.usedFiles.add(absPath);
    // InputController.cs:240  File.ReadLines(path).Take(endLine)
    const lines = this.readLinesOf(absPath);
    const taken = endLine === INT32_MAX ? lines : lines.slice(0, endLine);
    this.readLines(taken, absPath, startLine, studioLine, repeatIndex, repeatCount);
    return true;
  }

  // InputController.cs:250-271
  readLines(lines, absPath, startLine, studioLine, repeatIndex, repeatCount, lockStudioLine = false) {
    let fileLine = 0;
    let sl = studioLine;
    for (const readLine of lines) {
      fileLine++;
      if (fileLine < startLine) continue; // InputController.cs:254-256

      if (!this.readLine(readLine, absPath, fileLine, sl, repeatIndex, repeatCount)) {
        return;
      }

      if (absPath === this.mainFilePath && !lockStudioLine) {
        sl++;
      }
    }
    // InputController.cs:268-270  hidden label at the end of the text block --
    // only affects fast-forward bookkeeping, no frames.
  }

  // InputController.cs:274-316
  readLine(line, absPath, fileLine, studioLine, repeatIndex = 0, repeatCount = 0) {
    const lineText = line.trim();
    const commandParsingFrame = this.totalFrames;

    const parsed = tryParseCommand(lineText);
    if (parsed) {
      this.recordEvent(commandParsingFrame, parsed.attribute, parsed.commandLine, absPath, fileLine);
      this.setupCommand(parsed, commandParsingFrame, studioLine, absPath, fileLine);

      if (isCommand(parsed, 'Play')) {
        // InputController.cs:286-290
        return false;
      }
    } else if (isFastForwardLine(lineText)) {
      // FastForwardLine.cs:10-36 + InputController.cs:291-301 -- breakpoints do
      // not produce inputs and do not exist in the bundled corpus.
    } else if (lineText.startsWith('#')) {
      // InputController.cs:302-310
      if (isLabel(lineText)) {
        this.events.push({
          frame: commandParsingFrame,
          type: 'label',
          args: [lineText.slice(1)],
          file: this.relPath(absPath),
          line: fileLine,
        });
      }
    } else if (!this.autoInputTryInsert(absPath, fileLine, lineText, studioLine, repeatIndex, repeatCount)) {
      // InputController.cs:311-312
      this.addFramesFromText(lineText, absPath, fileLine, studioLine, repeatIndex, repeatCount, 0, null);
    }
    return true;
  }

  // Command.cs:110 / InputController.cs:279-284
  recordEvent(frame, attribute, commandLine, absPath, fileLine) {
    this.events.push({
      frame,
      type: attribute.name.toLowerCase(),
      args: [...commandLine.args],
      file: this.relPath(absPath),
      line: fileLine,
    });
  }

  // Command.cs:119-125  Setup() -- only Parse-timing commands run while parsing.
  setupCommand(parsed, commandParsingFrame, studioLine, absPath, fileLine) {
    const { commandLine, attribute } = parsed;
    if (attribute.timing !== 'parse' && attribute.timing !== 'both') return;
    this.runParseCommand(attribute, commandLine, studioLine, absPath, fileLine, commandParsingFrame);
  }

  // InputController.cs:319-323 / 326-332
  addFramesFromText(line, absPath, fileLine, studioLine, repeatIndex, repeatCount, frameOffset = 0, parentCommand = null) {
    // InputFrame.TryParse -> ActionLine.TryParse (InputFrame.cs:100-114)
    const al = parseActionLine(line);
    if (!al) return;
    this.addFrames(al, absPath, fileLine, studioLine, line, al.frameCount);
  }

  // InputController.cs:326-332
  addFrames(al, absPath, fileLine, studioLine, rawText, frames) {
    if (!Number.isFinite(frames) || frames <= 0) return; // `for (i=0; i<Frames; i++)`
    this.inputs.push({
      frames,
      action: rawText,
      line: fileLine,
      file: this.relPath(absPath),
    });
    this.totalFrames += frames;
  }

  // -----------------------------------------------------------------------
  // Parse-timing directives
  // -----------------------------------------------------------------------

  // Command.cs:119-125 + one branch per file under Source/TAS/Input/Commands/
  runParseCommand(attribute, commandLine, studioLine, absPath, fileLine, commandParsingFrame) {
    const name = attribute.name;
    switch (name) {
      case 'Read': return this.cmdRead(commandLine, absPath, fileLine, studioLine);
      case 'Play': return this.cmdPlay(commandLine, absPath, fileLine, studioLine);
      case 'Repeat': return this.cmdRepeat(commandLine, absPath, fileLine, studioLine, commandParsingFrame);
      case 'EndRepeat': return this.cmdEndRepeat(commandLine, absPath, fileLine, studioLine);
      case 'Add': return this.cmdAdd(commandLine, absPath, fileLine, studioLine);
      case 'AnalogMode': return; // AnalogHelper.cs:199 -- only sets the analog mode
      case 'AutoInput': return this.cmdAutoInput(commandLine, absPath, fileLine);
      case 'StartAutoInput': return this.cmdStartAutoInput(absPath, fileLine);
      case 'EndAutoInput': return this.cmdEndAutoInput(absPath, fileLine);
      case 'SkipInput': return this.cmdSkipInput(commandLine, absPath, fileLine);
      case 'StunPause': return this.cmdStunPause(commandLine, studioLine, absPath, fileLine);
      case 'EndStunPause': return this.cmdEndStunPause(absPath, fileLine);
      case 'StunPauseMode': return this.cmdStunPauseMode(commandLine);
      case 'SaveAndQuitReenter': return this.cmdSaveAndQuitReenter(studioLine, absPath, fileLine);
      case 'SelectCampaign': return this.cmdSelectCampaign(commandLine, studioLine, absPath, fileLine);
      case 'EnforceLegal': this.enforceLegalEnabledWhenParsing = true; return; // EnforceLegalCommand.cs:11-12
      case 'Safe': this.disallowUnsafeInputParsing = true; return; // SafeCommand.cs:13-15
      case 'Unsafe': this.disallowUnsafeInputParsing = false; return; // SafeCommand.cs:22-24
      case 'ExportLibTAS': return; // LibTasHelper.cs:34-42 -- libTAS export only
      case 'EndExportLibTAS': return; // LibTasHelper.cs:51-65
      case 'Skip': return; // LibTasHelper.cs:267-272
      case 'Marker': return; // LibTasHelper.cs:275-283
      case 'RequireDependency': return; // RequireDependencyCommand.cs
      case 'SeedRandom': return; // SeededRandomness.cs:160
      default: return;
    }
  }

  abort(message, where) {
    if (this.options.ignoreAborts) return;
    throw new TasAbortError(message, where);
  }

  // -----------------------------------------------------------------------
  // Read -- ReadCommand.cs:119-204
  // -----------------------------------------------------------------------
  cmdRead(commandLine, absPath, fileLine, studioLine) {
    const args = commandLine.args;
    if (args.length === 0) return; // ReadCommand.cs:122-124

    let fileDirectory = path.dirname(absPath);
    if (!fileDirectory) fileDirectory = process.cwd();

    const where = { file: this.relPath(absPath), line: fileLine };
    const target = findReadTargetFile(fileDirectory, args[0]);
    if (!target) {
      this.abort(`Read failed: couldn't find file '${args[0]}'`, where); // ReadCommand.cs:137-143
      return;
    }
    if (path.resolve(target) === path.resolve(absPath)) {
      this.abort('Read failed: Do not allow reading the file itself', where); // ReadCommand.cs:145-148
      return;
    }

    const lines = this.readLinesOf(target); // ReadCommand.cs:150
    let startLine = 0;
    let endLine = INT32_MAX;
    if (args.length > 1) {
      const t = tryGetLineTarget(args[1], lines); // ReadCommand.cs:156
      if (!t) {
        this.abort(`Read failed: ${args[1]} is invalid`, where);
        return;
      }
      startLine = t.lineNumber;
      if (args.length > 2) {
        const t2 = tryGetLineTarget(args[2], lines); // ReadCommand.cs:162
        if (!t2) {
          this.abort(`Read failed: ${args[2]} is invalid`, where);
          return;
        }
        endLine = t2.lineNumber;
      }
    }

    // ReadCommand.cs:170-174  dead-loop detection
    const detail = `Read, ${args.join(', ')}: line ${fileLine} of the file "${absPath}"`;
    if (this.readCommandStack.includes(detail)) {
      this.abort(`Multiple read commands lead to dead loops:\n${this.readCommandStack.join('\n')}`, where);
      return;
    }

    // ReadCommand.cs:179-190  the '#Start' assertion inserts a 0-frame Assert line
    // ReadCommand.cs:196-198  breakpoint parsing is disabled inside Read
    this.readCommandStack.push(detail);
    const prevBreakpointParsing = this.enableBreakpointParsing;
    this.enableBreakpointParsing = false;
    this.readFile(target, startLine, endLine, studioLine); // ReadCommand.cs:197
    this.enableBreakpointParsing = prevBreakpointParsing;
    this.readCommandStack.pop();
    // ReadCommand.cs:203  AnalogMode restore -- no frames
  }

  // -----------------------------------------------------------------------
  // Play -- PlayCommand.cs:41-59
  // -----------------------------------------------------------------------
  cmdPlay(commandLine, absPath, fileLine, studioLine) {
    const args = commandLine.args;
    const where = { file: this.relPath(absPath), line: fileLine };
    const lines = this.readLinesOf(absPath);
    const t = tryGetLineTarget(args[0], lines); // PlayCommand.cs:44
    if (!t) {
      this.abort(`"Play, ${args.join(', ')}" failed: ${args[0]} is invalid`, where);
      return;
    }
    const startLine = t.lineNumber;

    if (args.length > 1 && intTryParse(args[1]) !== null) {
      this.cmdAddFramesText(args[1], absPath, fileLine, studioLine); // PlayCommand.cs:49-51
    }

    if (startLine <= studioLine + 1) { // PlayCommand.cs:53-56
      return;
    }

    const playStudioLine = absPath === this.mainFilePath ? startLine - 1 : studioLine;
    this.readFile(absPath, startLine, INT32_MAX, playStudioLine); // PlayCommand.cs:58
  }

  /** `InputController.AddFrames(string, ...)` -- InputController.cs:319-323 */
  cmdAddFramesText(text, absPath, fileLine, studioLine, repeatIndex = 0, repeatCount = 0) {
    this.addFramesFromText(text, absPath, fileLine, studioLine, repeatIndex, repeatCount);
  }

  // -----------------------------------------------------------------------
  // Repeat / EndRepeat -- RepeatCommand.cs:42-119
  // -----------------------------------------------------------------------
  cmdRepeat(commandLine, absPath, fileLine, studioLine, commandParsingFrame) {
    const args = commandLine.args;
    const where = { file: this.relPath(absPath), line: fileLine };
    if (args.length === 0) {
      this.abort('Repeat command has no count specified', where); // RepeatCommand.cs:47-50
      return;
    }
    const count = intTryParse(args[0]);
    if (count === null) {
      this.abort("Repeat command's count is not an integer", where); // RepeatCommand.cs:51-54
      return;
    }
    if (count < 1) {
      this.abort("Repeat command's count must not be less than 1", where);
      return;
    }
    if (count > 10000000) {
      this.abort("Repeat command's count must not be greater than 10 million", where);
      return;
    }
    // RepeatCommand.cs:66
    this.repeatStack.push({
      startFrame: commandParsingFrame,
      count,
      startFilePath: absPath,
      startFileLine: fileLine,
    });
  }

  cmdEndRepeat(commandLine, absPath, fileLine, studioLine) {
    const where = { file: this.relPath(absPath), line: fileLine };
    const args = this.repeatStack.pop();
    if (!args) {
      this.abort('EndRepeat command does not have a paired Repeat command', where); // RepeatCommand.cs:73-76
      return;
    }

    // RepeatCommand.cs:79-82
    const startLine = args.startFileLine + 1;
    const endLine = fileLine - 1;
    const count = args.count;
    const startFrame = args.startFrame;

    if (endLine < startLine || !fs.existsSync(absPath) || count <= 1) {
      return; // RepeatCommand.cs:91-93
    }

    const mainFile = absPath === this.mainFilePath; // RepeatCommand.cs:96

    // RepeatCommand.cs:100-105  first-loop RepeatIndex/RepeatCount bookkeeping --
    // metadata only, no frames.

    // RepeatCommand.cs:107
    const lines = this.readLinesOf(absPath).slice(0, endLine);
    for (let i = 2; i <= count; i++) {
      // RepeatCommand.cs:109-117
      this.readLines(
        lines,
        absPath,
        startLine,
        mainFile ? startLine - 1 : studioLine,
        mainFile ? i : 0,
        mainFile ? count : 0,
      );
    }
  }

  // -----------------------------------------------------------------------
  // Add -- LibTasHelper.cs:258-264
  // -----------------------------------------------------------------------
  cmdAdd(commandLine, absPath, fileLine, studioLine) {
    // LibTasHelper.cs:261-263 -> AddInputFrame(string.Join(",", args))
    // LibTasHelper.cs:85-96 -> returns immediately unless `Exporting`, i.e. the
    // line has NO effect on InputController.Inputs. Recorded as an event only.
  }

  // -----------------------------------------------------------------------
  // AutoInput -- AutoInputCommand.cs
  // -----------------------------------------------------------------------
  cmdAutoInput(commandLine, absPath, fileLine) {
    const args = commandLine.args;
    const where = { file: this.relPath(absPath), line: fileLine };
    if (args.length === 0) {
      this.abort('AutoInput command no cycle length given', where);
      return;
    }
    const cycleLength = intTryParse(args[0]);
    if (cycleLength === null) {
      this.abort("AutoInput command's cycle length is not an integer", where);
      return;
    }
    if (this.autoInputArgs.has(absPath)) {
      this.abort('Nesting AutoInput commands are not supported', where);
      return;
    }
    if (cycleLength <= 0) {
      this.abort("AutoInput command's cycle length must be greater than 0", where);
      return;
    }
    this.autoInputArgs.set(absPath, this.makeAutoInputArgs(fileLine + 1, cycleLength)); // AutoInputCommand.cs:80
  }

  makeAutoInputArgs(startLine, cycleLength) {
    // AutoInputCommand.cs:34-37  CycleOffset = CycleLength = cycleLength
    return {
      startLine,
      cycleLength,
      cycleOffset: cycleLength,
      inputs: null,
      inserting: false,
      skipNextInput: false,
      skipFrames: 0,
      skipWaitingFrames: 0,
      lockStudioLine: false,
      stunPause: false,
    };
  }

  cmdStartAutoInput(absPath, fileLine) {
    const where = { file: this.relPath(absPath), line: fileLine };
    const args = this.autoInputArgs.get(absPath);
    if (!args) {
      this.abort('StartAutoInput command does not have a paired AutoInput command', where);
      return;
    }
    if (args.inputs !== null) {
      this.autoInputArgs.delete(absPath);
      this.abort('StartAutoInput command already exists', where);
      return;
    }
    // AutoInputCommand.cs:98  File.ReadLines(filePath).Take(fileLine - 1)
    args.inputs = this.readLinesOf(absPath).slice(0, fileLine - 1);
  }

  cmdEndAutoInput(absPath, fileLine) {
    this.endAutoInputImpl(absPath, fileLine, 'EndAutoInput', 'AutoInput');
  }

  // AutoInputCommand.cs:106-120
  endAutoInputImpl(absPath, fileLine, name, pairedName) {
    const where = { file: this.relPath(absPath), line: fileLine };
    const args = this.autoInputArgs.get(absPath);
    if (!args) {
      this.abort(`${name} command does not have a paired ${pairedName} command`, where);
      return;
    }
    if (args.inputs === null) {
      this.autoInputArgs.delete(absPath);
      this.abort(`EndAutoInput command does not have a paired StartAutoInput command`, where);
      return;
    }
    this.autoInputArgs.delete(absPath);
  }

  // AutoInputCommand.cs:122-160
  cmdSkipInput(commandLine, absPath, fileLine) {
    const args = commandLine.args;
    const where = { file: this.relPath(absPath), line: fileLine };
    const a = this.autoInputArgs.get(absPath);
    if (!a) return;
    if (args.length === 0) {
      a.skipNextInput = true;
      a.skipFrames = 0;
      a.skipWaitingFrames = 0;
      return;
    }
    const frames = intTryParse(args[0]);
    if (frames === null) {
      this.abort("SkipInput command's first parameter is not an integer", where);
      return;
    }
    if (frames <= 0) {
      this.abort("SkipInput command's first parameter must be greater than 0", where);
      return;
    }
    a.skipNextInput = false;
    a.skipFrames = frames;
    if (args.length >= 2) {
      const waitFrames = intTryParse(args[1]);
      if (waitFrames === null) {
        this.abort("SkipInput command's second parameter is not an integer", where);
      } else if (waitFrames < 0) {
        this.abort("SkipInput command's second parameter must be greater than or equal 0", where);
      } else {
        a.skipWaitingFrames = waitFrames;
      }
    } else {
      a.skipWaitingFrames = 0;
    }
  }

  // AutoInputCommand.cs:162-217  TryInsert
  autoInputTryInsert(absPath, fileLine, lineText, studioLine, repeatIndex, repeatCount) {
    // InputFrame.TryParse(lineText, filePath, fileLine, studioLine, null, out ...)
    const al = parseActionLine(lineText);
    if (!al) return false;

    const a = this.autoInputArgs.get(absPath);
    if (!a || a.inputs === null || a.inputs.length === 0) return false;
    if (a.inserting) return false;
    if (a.skipNextInput) {
      a.skipNextInput = false;
      return false;
    }

    const mainFile = absPath === this.mainFilePath;
    const inputFrames = al.frameCount;

    let frames = 0;
    let parsedFrames = 0;
    for (let i = 0; i < inputFrames; i++) {
      if (a.skipFrames > 0) {
        a.skipWaitingFrames--;
        if (a.skipWaitingFrames === -1) {
          a.cycleOffset += a.skipFrames;
          a.skipFrames = 0;
          a.skipWaitingFrames = 0;
        }
      }

      if (a.cycleOffset === 0) {
        this.autoInputParseInsertedLines(a, absPath, studioLine, repeatIndex, repeatCount);
        a.cycleOffset = a.cycleLength;
      }

      frames++;
      a.cycleOffset--;

      if (a.cycleOffset === 0 || i === inputFrames - 1) {
        this.addFrames(al, absPath, fileLine, studioLine, lineText, frames);
        parsedFrames += frames;
        frames = 0;
      }
    }
    return true;
  }

  // AutoInputCommand.cs:219-235  ParseInsertedLines
  autoInputParseInsertedLines(a, absPath, studioLine, repeatIndex, repeatCount) {
    if (a.stunPause) this.stunPauseUpdatePauseInputs(a); // AutoInputCommand.cs:220-222
    a.inserting = true;
    this.readLines(
      a.inputs,
      absPath,
      a.startLine,
      absPath === this.mainFilePath ? a.startLine - 1 : studioLine,
      repeatIndex,
      repeatCount,
      a.lockStudioLine,
    );
    a.inserting = false;
  }

  // -----------------------------------------------------------------------
  // StunPause -- StunPauseCommand.cs
  // -----------------------------------------------------------------------
  stunPauseMode(localMode) {
    // StunPauseCommand.cs:68-77
    if (this.enforceLegalEnabledWhenParsing) return 'Input';
    return localMode ?? this.stunPauseGlobalModeParsing ?? 'Input';
  }

  cmdStunPause(commandLine, studioLine, absPath, fileLine) {
    const args = commandLine.args;
    const where = { file: this.relPath(absPath), line: fileLine };
    let localMode = null; // StunPauseCommand.cs:136

    if (args.length > 0) {
      const lc = args[0].toLowerCase();
      if (lc === 'input') localMode = 'Input';
      else if (lc === 'simulate') localMode = 'Simulate';
      else {
        this.abort('StunPause command failed.\nMode must be Input or Simulate', where);
        return;
      }
    }

    if (this.stunPauseMode(localMode) === 'Input') {
      this.stunPauseAutoInputMode(studioLine, absPath, fileLine); // StunPauseCommand.cs:147-148
    }
  }

  // StunPauseCommand.cs:157-175
  stunPauseAutoInputMode(studioLine, absPath, fileLine) {
    const where = { file: this.relPath(absPath), line: fileLine };
    if (this.autoInputArgs.has(absPath)) {
      this.abort('Nesting StunPause command is not allowed at AutoInput mode', where);
      return;
    }
    const a = this.makeAutoInputArgs(fileLine, 2); // StunPauseCommand.cs:162
    this.autoInputArgs.set(absPath, a);

    // StunPauseCommand.cs:165-168
    const inputs = this.readLinesOf(absPath).slice(0, fileLine - 1);
    inputs.push('1,S,N');
    inputs.push('');
    a.inputs = inputs;
    this.stunPauseUpdatePauseInputs(a); // StunPauseCommand.cs:169

    a.lockStudioLine = true; // StunPauseCommand.cs:171
    a.stunPause = true; // StunPauseCommand.cs:172
    // StunPauseCommand.cs:173
    this.autoInputParseInsertedLines(a, absPath, studioLine, 0, 0);
  }

  // StunPauseCommand.cs:177-194
  stunPauseUpdatePauseInputs(a) {
    const inputs = a.inputs;
    inputs.pop(); // remove the trailing "" placeholder
    const last = this.inputs.length > 0 ? this.inputs[this.inputs.length - 1] : null;
    const acts = last ? actionsOfRawLine(last.action) : Actions.None;
    if ((acts & Actions.Jump) !== 0 && (acts & Actions.Jump2) !== 0) inputs.push('10,J,K');
    else if ((acts & Actions.Jump) !== 0) inputs.push('10,J');
    else if ((acts & Actions.Jump2) !== 0) inputs.push('10,K,O');
    else inputs.push('10,O');
  }

  cmdEndStunPause(absPath, fileLine) {
    // StunPauseCommand.cs:196-199
    if (this.stunPauseMode(null) === 'Input') {
      this.endAutoInputImpl(absPath, fileLine, 'EndStunPause', 'StunPause');
    }
  }

  cmdStunPauseMode(commandLine) {
    const args = commandLine.args;
    if (args.length === 0) return;
    const lc = args[0].toLowerCase();
    if (lc === 'input') this.stunPauseGlobalModeParsing = 'Input';
    else if (lc === 'simulate') this.stunPauseGlobalModeParsing = 'Simulate';
    // StunPauseCommand.cs:205-217
  }

  // -----------------------------------------------------------------------
  // SaveAndQuitReenter -- SaveAndQuitReenterCommand.cs:53-129
  // -----------------------------------------------------------------------
  cmdSaveAndQuitReenter(studioLine, absPath, fileLine) {
    const o = this.options;
    const slot = o.saveAndQuitReenterSlot;
    const safe = this.disallowUnsafeInputParsing; // SaveAndQuitReenterCommand.cs:61
    if (safe) {
      this.readLine('Unsafe', absPath, fileLine, studioLine); // :63
    }
    if (slot === -1 && o.playMode !== 'Debug') {
      this.readLine('Set,Celeste.PlayMode,Debug', absPath, fileLine, studioLine); // :68
    }

    // LibTasHelper.AddInputFrame("58") -- :71, libTAS-export only, no Inputs entry.

    this.cmdAddFramesText('31', absPath, fileLine, studioLine); // :72
    this.cmdAddFramesText('14', absPath, fileLine, studioLine); // :73
    if (slot === -1) {
      this.cmdAddFramesText('1,D', absPath, fileLine, studioLine); // :76
      if (o.randomizerInstalled) {
        this.cmdAddFramesText('1,F,180', absPath, fileLine, studioLine); // :79
        this.cmdAddFramesText('1', absPath, fileLine, studioLine); // :80
      }
      this.cmdAddFramesText('1,O', absPath, fileLine, studioLine); // :82
      this.cmdAddFramesText('33', absPath, fileLine, studioLine); // :83
    } else {
      this.cmdAddFramesText('1,O', absPath, fileLine, studioLine); // :86
      this.cmdAddFramesText('56', absPath, fileLine, studioLine); // :87
      for (let i = 0; i < slot; i++) {
        this.cmdAddFramesText(i % 2 === 0 ? '1,D' : '1,F,180', absPath, fileLine, studioLine); // :90
      }
      this.cmdAddFramesText('1,O', absPath, fileLine, studioLine); // :94
      this.cmdAddFramesText('14', absPath, fileLine, studioLine); // :95
      this.cmdAddFramesText('1,O', absPath, fileLine, studioLine); // :96
      this.cmdAddFramesText('1', absPath, fileLine, studioLine); // :97
    }

    if (slot === -1 && o.playMode !== 'Debug') {
      this.readLine(`Set,Celeste.PlayMode,${o.playMode}`, absPath, fileLine, studioLine); // :103
      this.readLine('Set,Engine.Commands.Enabled,false', absPath, fileLine, studioLine); // :104
    }
    if (safe) {
      this.readLine('Safe', absPath, fileLine, studioLine); // :107
    }
  }

  // -----------------------------------------------------------------------
  // SelectCampaign -- SelectCampaignCommand.cs:96-172
  // -----------------------------------------------------------------------
  cmdSelectCampaign(commandLine, studioLine, absPath, fileLine) {
    const o = this.options;
    const where = { file: this.relPath(absPath), line: fileLine };
    const args = commandLine.args;

    if (args.length === 0) {
      this.abort('No campaign specified', where); // :115-118
      return;
    }
    const campaignName = args[0];
    const saveFileName = args.length >= 2 ? args[1] : 'TAS'; // :121

    if (saveFileName.length < 1 || saveFileName.length > 16) {
      this.abort(`Save-File name must be between 1 and 16 characters long`, where); // :131-134
      return;
    }
    if (saveFileName[0] === ' ') {
      this.abort('Save-File name cannot start with a space', where); // :135-138
      return;
    }
    if (!o.areaLevelSets.includes(campaignName)) {
      this.abort(`Unknown campaign '${campaignName}'`, where); // :123-126
      return;
    }

    this.readLine('Unsafe', absPath, fileLine, studioLine); // :140
    this.readLine('console titlescreen', absPath, fileLine, studioLine); // :141

    this.cmdAddFramesText('2', absPath, fileLine, studioLine); // :143
    // LibTasHelper.AddInputFrame("1,O") / ("89") -- :144-145, export only.
    this.cmdAddFramesText('1,O', absPath, fileLine, studioLine); // :146
    this.cmdAddFramesText('94', absPath, fileLine, studioLine); // :147
    this.cmdAddFramesText('1,O', absPath, fileLine, studioLine); // :148

    const slot = o.selectCampaignEmptyFileSlot; // :150
    if (slot === -1) {
      this.cmdAddFramesText('62', absPath, fileLine, studioLine); // :152
    } else {
      this.cmdAddFramesText('56', absPath, fileLine, studioLine); // :154
      for (let i = 0; i < slot; i++) {
        this.cmdAddFramesText(i % 2 === 0 ? '1,D' : '1,F,180', absPath, fileLine, studioLine); // :156
      }
      this.cmdAddFramesText('1,O', absPath, fileLine, studioLine); // :158
      this.cmdAddFramesText('15', absPath, fileLine, studioLine); // :159
    }

    this.cmdAddFramesText('1,D', absPath, fileLine, studioLine); // :165
    this.selectCampaignInputName(slot, saveFileName, studioLine, absPath, fileLine); // :166
    // SelectCampaignCommand.cs:168 runs unconditionally, so it is resolved even
    // when the name-entry BFS could not be (see selectCampaignWriteName).
    this.selectCampaignChangeSelectedCampaign(campaignName, studioLine, absPath, fileLine); // :168
  }

  // SelectCampaignCommand.cs:174-357  InputName
  selectCampaignInputName(slot, saveFileName, studioLine, absPath, fileLine) {
    const o = this.options;
    if (o.language === 'japanese') {
      this.abort('Japanese language is currently not supported for inputting a file name', { file: this.relPath(absPath), line: fileLine });
      return;
    }
    this.cmdAddFramesText('1,O', absPath, fileLine, studioLine); // :180

    const maxSaveFile = o.selectCampaignMaxSaveFileSlots; // :182
    for (let i = 0; i < maxSaveFile; i++) {
      if (Math.abs(Math.max(0, slot) - i) <= 2) {
        this.cmdAddFramesText('3', absPath, fileLine, studioLine); // :186
      }
    }
    this.cmdAddFramesText('32', absPath, fileLine, studioLine); // :189

    const result = this.selectCampaignWriteName(saveFileName, studioLine, absPath, fileLine);
    if (!result) {
      // Without font metrics the BFS cursor path is unknown. Everything else in
      // InputName is deterministic, so it is still emitted (SelectCampaignCommand.cs:355-356).
      this.warn(
        'SelectCampaign name entry: only the cursor movement frames are unresolved '
        + '(ActiveFont.Measure, SelectCampaignCommand.cs:202-215, is needed for the BFS grid). '
        + 'The per-character confirm presses and the trailing "1,S" + "48" are emitted.',
      );
    }

    // :355-356
    this.cmdAddFramesText('1,S', absPath, fileLine, studioLine);
    this.cmdAddFramesText('48', absPath, fileLine, studioLine);
  }

  /**
   * The BFS grid from SelectCampaignCommand.cs:191-352. Requires real font
   * metrics (`ActiveFont.Measure`); returns false when they are unavailable, in
   * which case only the per-character confirm presses are emitted.
   */
  selectCampaignWriteName(saveFileName, studioLine, absPath, fileLine) {
    const o = this.options;
    const widths = o.fontWidths;
    if (!widths) {
      for (const _char of saveFileName) {
        this.selectCampaignConfirmPress(studioLine, absPath, fileLine);
      }
      return false;
    }

    const measure = (ch) => (widths[ch] === undefined ? widths.default : widths[ch]);
    const letters0 = o.nameLetters;
    const maxLetterLength = letters0.reduce((m, l) => Math.max(m, l.length), 0);
    const widestLetter = Math.max(
      ...letters0.flatMap((l) => l.split('')).map((c) => measure(c)),
    );
    const optionsScale = 0.75;
    const cancelWidth = measure('\u0001') * optionsScale; // name_back
    const spaceWidth = measure(' ') * optionsScale; // name_space
    const backspaceWidth = measure('\u0002') * optionsScale; // name_backspace
    const beginWidth = measure('\u0003') * optionsScale * 1.25; // name_accept
    const optionsWidth = cancelWidth + spaceWidth + backspaceWidth + beginWidth + widestLetter * 3;
    const boxPadding = widestLetter;
    const boxWidth = Math.max(maxLetterLength * widestLetter, optionsWidth) + boxPadding * 2;
    const innerWidth = boxWidth - boxPadding * 2;

    const spaceLocations = new Set();
    for (let index = 0; index < maxLetterLength; index++) {
      const realX = index * widestLetter;
      if (!(realX < cancelWidth + (innerWidth - cancelWidth - beginWidth - backspaceWidth - spaceWidth - widestLetter * 3) / 2)
          && realX < innerWidth - beginWidth - backspaceWidth - widestLetter * 2) {
        spaceLocations.add(`${index},${letters0.length}`);
      }
    }
    const spaceExitLocation = {
      x: Math.trunc((innerWidth - beginWidth - backspaceWidth - spaceWidth / 2 - widestLetter * 2) / widestLetter),
      y: letters0.length,
    };

    const MARK = '\uFFFF';
    let letters = letters0.map((l) => l.replace(/ /g, MARK));
    letters = letters.concat([
      MARK.repeat(spaceExitLocation.x) + ' ' + MARK.repeat(maxLetterLength - spaceExitLocation.x - 1),
    ]);

    // grid[line][char] -> index, for neighbour lookups
    const gridLen = letters.map((l) => l.length);

    let start = { x: 0, y: 0 };
    for (const targetChar of saveFileName) {
      const targetLine = letters.findIndex((l) => l.includes(targetChar));
      if (targetLine === -1) {
        this.abort(`Character '${targetChar}' not available in current language`, { file: this.relPath(absPath), line: fileLine });
        return false;
      }
      const targetIndex = letters[targetLine].indexOf(targetChar);
      if (targetIndex === -1) {
        this.abort(`Character '${targetChar}' not available in current language`, { file: this.relPath(absPath), line: fileLine });
        return false;
      }

      const end = { x: targetIndex, y: targetLine };
      const key = (p) => `${p.x},${p.y}`;
      const queue = [start];
      const visited = new Set([key(start)]);
      const parent = new Map();

      while (queue.length > 0) {
        const current = queue.shift();
        if (current.x === end.x && current.y === end.y) break;

        for (const [dx, dy] of [[-1, 0], [1, 0], [0, 1], [0, -1]]) {
          let nx = current.x;
          let ny = current.y;
          let skip = false;
          do {
            if (dx !== 0) nx = mod(nx + dx, gridLen[ny]);
            ny += dy;
            if (spaceLocations.has(`${nx},${ny}`)) {
              nx = spaceExitLocation.x;
              ny = spaceExitLocation.y;
              break;
            }
            if (ny < 0 || ny >= letters.length) { skip = true; break; }
          } while (nx >= gridLen[ny] || letters[ny][nx] === MARK);
          if (skip) continue;

          const next = { x: nx, y: ny };
          if (!visited.has(key(next))) {
            visited.add(key(next));
            queue.push(next);
            parent.set(key(next), current);
          }
        }
      }

      if (!visited.has(key(end))) {
        this.abort(`Failed to write out name '${saveFileName}'`, { file: this.relPath(absPath), line: fileLine });
        return false;
      }

      // :305-332  reconstruct + reverse
      const movePath = [];
      let currentCell = end;
      while (currentCell.x !== start.x || currentCell.y !== start.y) {
        const prevCell = parent.get(key(currentCell));
        if (prevCell.x === 0 && currentCell.x === letters[currentCell.y].length - 1) {
          movePath.push({ x: -1, y: currentCell.y - prevCell.y });
        } else if (currentCell.x === 0 && prevCell.x === letters[currentCell.y].length - 1) {
          movePath.push({ x: 1, y: currentCell.y - prevCell.y });
        } else if (currentCell.x === spaceExitLocation.x && currentCell.y === spaceExitLocation.y) {
          movePath.push({ x: 0, y: 1 });
        } else if (prevCell.x === spaceExitLocation.x && prevCell.y === spaceExitLocation.y) {
          movePath.push({ x: 0, y: -1 });
        } else {
          movePath.push({ x: currentCell.x - prevCell.x, y: currentCell.y - prevCell.y });
        }
        currentCell = prevCell;
      }
      movePath.reverse();
      start = end;

      // :337-349
      for (const mv of movePath) {
        if (mv.x > 0) {
          this.cmdAddFramesText(this.totalFrames % 2 === 0 ? '1,R' : '1,F,90', absPath, fileLine, studioLine);
        } else if (mv.x < 0) {
          this.cmdAddFramesText(this.totalFrames % 2 === 0 ? '1,L' : '1,F,270', absPath, fileLine, studioLine);
        }
        if (mv.y > 0) {
          this.cmdAddFramesText(this.totalFrames % 2 === 0 ? '1,D' : '1,F,180', absPath, fileLine, studioLine);
        } else if (mv.y < 0) {
          this.cmdAddFramesText(this.totalFrames % 2 === 0 ? '1,U' : '1,F,0', absPath, fileLine, studioLine);
        }
      }

      // :351
      this.selectCampaignConfirmPress(studioLine, absPath, fileLine);
    }
    return true;
  }

  /**
   * SelectCampaignCommand.cs:351 -- press confirm on the letter, or jump when
   * the previous input already carried Confirm.
   */
  selectCampaignConfirmPress(studioLine, absPath, fileLine) {
    const last = this.inputs.length > 0 ? this.inputs[this.inputs.length - 1] : null;
    const acts = last ? actionsOfRawLine(last.action) : Actions.None;
    this.cmdAddFramesText((acts & Actions.Confirm) !== 0 ? '1,J' : '1,O', absPath, fileLine, studioLine);
  }

  // SelectCampaignCommand.cs:359-438  ChangeSelectedCampaign
  selectCampaignChangeSelectedCampaign(campaignName, studioLine, absPath, fileLine) {
    const o = this.options;
    const areas = o.areaLevelSets;
    let startingLevelSet = 'Celeste';
    if (areas.includes(o.defaultStartingLevelSet)) startingLevelSet = o.defaultStartingLevelSet; // :361

    // :366-385 / :387-406
    const countMoves = (step) => {
      let moves = 0;
      let current = startingLevelSet;
      let guard = 0;
      while (current !== campaignName && guard++ < 10000) {
        let id = step(current);
        if (id >= areas.length) id = 0;
        if (id < 0) id = areas.length - 1;
        current = areas[id];
        moves++;
      }
      return moves;
    };
    const movesLeft = countMoves((cur) => areas.indexOf(cur) - 1);
    const movesRight = countMoves((cur) => areas.lastIndexOf(cur) + 1);

    if (movesLeft === 0 && movesRight === 0) {
      this.cmdAddFramesText('1,U', absPath, fileLine, studioLine); // :410
      return;
    }

    this.cmdAddFramesText('1,D', absPath, fileLine, studioLine); // :415
    this.cmdAddFramesText('1,F,180', absPath, fileLine, studioLine); // :416
    if (o.variantsUnlocked) {
      this.cmdAddFramesText('1,D', absPath, fileLine, studioLine); // :418
    }

    if (movesRight <= movesLeft) {
      for (let i = 0; i < movesRight; i++) {
        this.cmdAddFramesText(i % 2 === 0 ? '1,R' : '1,F,90', absPath, fileLine, studioLine); // :423
      }
    } else {
      for (let i = 0; i < movesLeft; i++) {
        this.cmdAddFramesText(i % 2 === 0 ? '1,L' : '1,F,270', absPath, fileLine, studioLine); // :427
      }
    }

    this.cmdAddFramesText('1,U', absPath, fileLine, studioLine); // :432
    this.cmdAddFramesText('1,F,0', absPath, fileLine, studioLine); // :433
    this.cmdAddFramesText('1,U', absPath, fileLine, studioLine); // :434
    if (o.variantsUnlocked) {
      this.cmdAddFramesText('1,F,0', absPath, fileLine, studioLine); // :436
    }
  }

  // InputController.cs:141 + [ParseFileEnd] handlers
  parseFileEnd() {
    // RepeatCommand.cs:24-34
    if (this.repeatStack.length > 0) {
      const a = this.repeatStack[this.repeatStack.length - 1];
      this.abort(`Repeat command does not have a paired EndRepeat command`, {
        file: this.relPath(a.startFilePath), line: a.startFileLine,
      });
    }
    // AutoInputCommand.cs:47-63
    for (const [key, a] of this.autoInputArgs) {
      if (a.inputs === null) {
        this.abort('AutoInput command does not have a paired StartAutoInput command', {
          file: this.relPath(key), line: a.startLine - 1,
        });
      } else {
        this.abort(`${a.stunPause ? 'StunPause' : 'StartAutoInput'} command does not have a paired ${a.stunPause ? 'EndStunPause' : 'EndAutoInput'} command`, {
          file: this.relPath(key), line: a.startLine - 1,
        });
      }
    }
  }
}

/**
 * ActionLine.ToString() -- ActionLine.cs:301-316.
 *
 * This is CelesteTAS's own canonical form of an input line (stored as
 * `InputFrame.actionLineString`, InputFrame.cs:62), which is what a game-side
 * trace of `InputController.Inputs` reports. `inputs[].action` instead keeps the
 * literal source text, so this helper lets the two be compared.
 */
export function actionLineToString(al) {
  const flags = [
    Actions.Left, Actions.Right, Actions.Up, Actions.Down,
    Actions.Jump, Actions.Jump2, Actions.Dash, Actions.Dash2,
    Actions.DemoDash, Actions.DemoDash2, Actions.Grab, Actions.Grab2,
    Actions.Start, Actions.Restart, Actions.Journal, Actions.Confirm,
    Actions.DashOnly, Actions.MoveOnly, Actions.PressedKey, Actions.Feather,
  ];
  const dashOnlyOrder = [Actions.LeftDashOnly, Actions.RightDashOnly, Actions.UpDashOnly, Actions.DownDashOnly];
  const moveOnlyOrder = [Actions.LeftMoveOnly, Actions.RightMoveOnly, Actions.UpMoveOnly, Actions.DownMoveOnly];

  const customBindings = [...al.customBindings].sort();

  let actions = '';
  for (const flag of flags) {
    if ((al.actions & flag) === 0) continue;
    let text;
    if (flag === Actions.DashOnly) {
      text = 'A' + dashOnlyOrder.filter((f) => al.actions & f).map(charForAction).join('');
    } else if (flag === Actions.MoveOnly) {
      text = 'M' + moveOnlyOrder.filter((f) => al.actions & f).map(charForAction).join('');
    } else if (flag === Actions.PressedKey) {
      text = 'P' + customBindings.join('');
    } else {
      text = charForAction(flag);
    }
    actions += `,${text}`;
  }

  const featherAngle = (al.actions & Actions.Feather) !== 0 ? `,${al.featherAngle ?? ''}` : '';
  const featherMagnitude = (al.actions & Actions.Feather) !== 0 && al.featherMagnitude !== null
    ? `,${al.featherMagnitude}`
    : '';

  // $"{Frames,MaxFramesDigits}" with MaxFramesDigits = 4 (ActionLine.cs:11)
  const frames = String(al.frameCount).padStart(4, ' ');
  return `${frames}${actions}${featherAngle}${featherMagnitude}`;
}

/** Actions.cs:111-134  CharForAction */
export function charForAction(a) {
  switch (a) {
    case Actions.Right: case Actions.RightDashOnly: case Actions.RightMoveOnly: return 'R';
    case Actions.Left: case Actions.LeftDashOnly: case Actions.LeftMoveOnly: return 'L';
    case Actions.Up: case Actions.UpDashOnly: case Actions.UpMoveOnly: return 'U';
    case Actions.Down: case Actions.DownDashOnly: case Actions.DownMoveOnly: return 'D';
    case Actions.Jump: return 'J';
    case Actions.Jump2: return 'K';
    case Actions.Dash: return 'X';
    case Actions.Dash2: return 'C';
    case Actions.DemoDash: return 'Z';
    case Actions.DemoDash2: return 'V';
    case Actions.Grab: return 'G';
    case Actions.Grab2: return 'H';
    case Actions.Start: return 'S';
    case Actions.Restart: return 'Q';
    case Actions.Journal: return 'N';
    case Actions.Confirm: return 'O';
    case Actions.DashOnly: return 'A';
    case Actions.MoveOnly: return 'M';
    case Actions.PressedKey: return 'P';
    case Actions.Feather: return 'F';
    default: return ' ';
  }
}

/** Canonical CelesteTAS text for a raw input-line, or null if it does not parse. */
export function canonicalActionText(line) {
  const al = parseActionLine(line);
  return al === null ? null : actionLineToString(al);
}

function mod(a, n) {
  return ((a % n) + n) % n;
}

/**
 * Structural invariants of a resolved TAS. Returns a list of human readable
 * problems; an empty list means the result is internally consistent.
 *
 * These are *not* the frame-count checks against the `FileTime` header -- see
 * `parseFileTimeHeader` and docs/tas-format.md for why those two numbers are
 * different quantities.
 */
export function selfCheck(result) {
  const problems = [];
  if (result.version !== RESOLVE_VERSION) problems.push(`unexpected version ${result.version}`);
  if (typeof result.entry !== 'string' || result.entry.length === 0) problems.push('entry is empty');

  let sum = 0;
  result.inputs.forEach((entry, i) => {
    if (!Number.isInteger(entry.frames) || entry.frames <= 0) {
      problems.push(`inputs[${i}].frames must be a positive integer, got ${entry.frames}`);
    }
    if (typeof entry.action !== 'string') problems.push(`inputs[${i}].action is not a string`);
    if (!Number.isInteger(entry.line) || entry.line < 1) problems.push(`inputs[${i}].line invalid`);
    if (typeof entry.file !== 'string' || entry.file.length === 0) problems.push(`inputs[${i}].file is empty`);
    sum += entry.frames;
  });
  if (sum !== result.totalFrames) {
    problems.push(`totalFrames ${result.totalFrames} != sum(inputs.frames) ${sum}`);
  }

  let prevFrame = -1;
  result.events.forEach((ev, i) => {
    if (!Number.isInteger(ev.frame) || ev.frame < 0 || ev.frame > result.totalFrames) {
      problems.push(`events[${i}].frame ${ev.frame} out of range [0, ${result.totalFrames}]`);
    }
    // Parsing is strictly forward-only, so directive frames never move backwards.
    if (ev.frame < prevFrame) {
      problems.push(`events[${i}].frame ${ev.frame} < previous event frame ${prevFrame}`);
    }
    prevFrame = ev.frame;
    if (typeof ev.type !== 'string' || ev.type.length === 0) problems.push(`events[${i}].type is empty`);
    if (!Array.isArray(ev.args) || ev.args.some((a) => typeof a !== 'string')) {
      problems.push(`events[${i}].args must be an array of strings`);
    }
    if (typeof ev.file !== 'string' || ev.file.length === 0) problems.push(`events[${i}].file is empty`);
    if (!Number.isInteger(ev.line) || ev.line < 1) problems.push(`events[${i}].line invalid`);
  });

  return problems;
}

/** Total frame count encoded in a `FileTime: h:mm:ss.mmm(N)` header. */
export function headerFrames(text) {
  const h = parseFileTimeHeader(text);
  return h === null ? null : h.frames;
}

/** Actions of an already-parsed input entry (used by StunPause / SelectCampaign). */
function actionsOfRawLine(text) {
  const al = parseActionLine(text);
  return al ? al.actions : Actions.None;
}

/**
 * Flattens a Celeste `.tas` file into a canonical per-frame input stream.
 *
 * @param {string} rootDir   directory that contains the TAS files
 * @param {string} entryName entry file name, relative to `rootDir`
 * @param {object} [options] see DEFAULT_OPTIONS
 */
export function resolveTas(rootDir, entryName, options = {}) {
  const resolver = new Resolver(rootDir, options);
  return resolver.resolve(entryName);
}

/**
 * `FileTime: h:mm:ss.mmm(N)`.
 *
 * `frames` (N) is `(SaveData.Time - startTicks) / Engine.RawDeltaTime.SecondsToTicks()`,
 * i.e. the number of frames in which Celeste's save-file timer advanced -- NOT
 * `InputController.Inputs.Count`. See docs/tas-format.md.
 *
 * Returns null when the line is absent or unparseable.
 */
export function parseFileTimeHeader(text) {
  const m = /^[ \t]*FileTime\s*:\s*(?:(\d+):)?(\d+):(\d+)\.(\d+)\s*\(\s*(\d+)\s*\)/m.exec(text);
  if (!m) return null;
  return {
    raw: m[0].trim(),
    hours: m[1] === undefined ? 0 : Number(m[1]),
    minutes: Number(m[2]),
    seconds: Number(m[3]),
    millis: Number(m[4]),
    frames: Number(m[5]),
  };
}

export default resolveTas;
