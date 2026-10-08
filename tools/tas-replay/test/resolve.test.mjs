// test/resolve.test.mjs -- `node --test`
//
// Two kinds of test:
//
//  1. Synthetic fixtures (`mkTas`) whose expected frame counts are derived by
//     hand from the CelesteTAS reference source. These pin the exact semantics of
//     Read / Repeat / Play / Add / AutoInput / StunPause / SaveAndQuitReenter and
//     of the action-line grammar.
//
//  2. Corpus tests against the 127-file CelesteTAS input corpus. They pin the
//     resolved frame counts, the file headers, and the exact difference between
//     the two. See docs/tas-format.md, section "The FileTime header", for why
//     those two numbers are different quantities.

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import {
  resolveTas,
  selfCheck,
  parseActionLine,
  parseCommandLine,
  parseFileTimeHeader,
  isLabel,
  tryGetLineTarget,
  findReadTargetFile,
  splitDotNetLines,
  intTryParse,
  Actions,
  ACTION_CHARS,
  actionForChar,
  canonicalActionText,
} from '../src/resolve.mjs';

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/** Writes `files` (name -> text) into a fresh temp dir and returns the dir. */
function mkTas(files) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'tas-replay-'));
  for (const [name, text] of Object.entries(files)) {
    const abs = path.join(dir, name);
    fs.mkdirSync(path.dirname(abs), { recursive: true });
    fs.writeFileSync(abs, text);
  }
  return dir;
}

const CORPUS = process.env.TAS_CORPUS
  ?? 'D:/celeste-research/.tmp/tas/CelesteTAS/CelesteTAS-074e71a93a073ec8940d45161c76484a33684841';
const hasCorpus = fs.existsSync(CORPUS);
const corpusSkip = hasCorpus ? false : `corpus not found at ${CORPUS} (set TAS_CORPUS)`;

function corpusText(rel) {
  return fs.readFileSync(path.join(CORPUS, rel), 'utf8');
}

// ---------------------------------------------------------------------------
// 1. unit: primitive ports
// ---------------------------------------------------------------------------

test('splitDotNetLines mimics File.ReadAllLines', () => {
  assert.deepEqual(splitDotNetLines('a\nb\n'), ['a', 'b']);
  assert.deepEqual(splitDotNetLines('a\r\nb\r\n'), ['a', 'b']);
  assert.deepEqual(splitDotNetLines('a\rb\r'), ['a', 'b']);
  assert.deepEqual(splitDotNetLines('a\n\n'), ['a', '']);
  assert.deepEqual(splitDotNetLines(''), []);
  assert.deepEqual(splitDotNetLines('a'), ['a']);
});

test('intTryParse matches C# int.TryParse (Integer NumberStyles)', () => {
  assert.equal(intTryParse('42'), 42);
  assert.equal(intTryParse(' 42 '), 42);
  assert.equal(intTryParse('-7'), -7);
  assert.equal(intTryParse('+7'), 7);
  assert.equal(intTryParse(''), null);
  assert.equal(intTryParse('abc'), null);
  assert.equal(intTryParse('4.2'), null);
  assert.equal(intTryParse('2147483648'), null); // overflow
});

test('CommandLine.TryParse: separator is taken from the first separator run', () => {
  // CommandLine.cs:24-150
  let cl = parseCommandLine('Read, 0 - Prologue, Start');
  assert.equal(cl.command, 'Read');
  assert.deepEqual(cl.args, ['0 - Prologue', 'Start']); // separator ", "
  assert.equal(cl.argumentSeparator, ', ');

  cl = parseCommandLine('Read,0 - Prologue,Start');
  assert.deepEqual(cl.args, ['0 - Prologue', 'Start']); // separator ","
  assert.equal(cl.argumentSeparator, ',');

  cl = parseCommandLine('console load 1 lvl_9b');
  assert.equal(cl.command, 'console');
  assert.deepEqual(cl.args, ['load', '1', 'lvl_9b']); // separator " "

  // FileTime: is the command name (it is registered as an alias)
  cl = parseCommandLine('FileTime: 1:08:45.679(242687)');
  assert.equal(cl.command, 'FileTime:');
  assert.deepEqual(cl.args, ['1:08:45.679(242687)']);

  // no-argument command: the whole trimmed line is the command name
  cl = parseCommandLine('EndRepeat');
  assert.equal(cl.command, 'EndRepeat');
  assert.deepEqual(cl.args, []);

  // quotes and bracket groups protect separators (CommandLine.cs:57-58,77-109)
  cl = parseCommandLine('Set,X,"a,b"');
  assert.deepEqual(cl.args, ['X', 'a,b']);
  cl = parseCommandLine('Assert,Equal,{a,b},c');
  assert.deepEqual(cl.args, ['Equal', '{a,b}', 'c']);

  // not a command line at all
  assert.equal(parseCommandLine('  1,R,J'), null); // must start with a letter
  assert.equal(parseCommandLine(''), null);
});

test('ActionLine: every action character from Actions.cs is recognised', () => {
  // StudioCommunication/Actions.cs:42-64
  assert.deepEqual(Object.keys(ACTION_CHARS).sort(), [
    'A', 'C', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'U', 'V', 'X', 'Z',
  ]);
  assert.equal(actionForChar('J'), Actions.Jump);
  assert.equal(actionForChar('K'), Actions.Jump2);
  assert.equal(actionForChar('Z'), Actions.DemoDash);
  assert.equal(actionForChar('V'), Actions.DemoDash2);
  assert.equal(actionForChar('X'), Actions.Dash);
  assert.equal(actionForChar('C'), Actions.Dash2);
  assert.equal(actionForChar('G'), Actions.Grab);
  assert.equal(actionForChar('H'), Actions.Grab2);
  assert.equal(actionForChar('S'), Actions.Start);
  assert.equal(actionForChar('Q'), Actions.Restart);
  assert.equal(actionForChar('N'), Actions.Journal);
  assert.equal(actionForChar('O'), Actions.Confirm);
  assert.equal(actionForChar('A'), Actions.DashOnly);
  assert.equal(actionForChar('M'), Actions.MoveOnly);
  assert.equal(actionForChar('P'), Actions.PressedKey);
  assert.equal(actionForChar('F'), Actions.Feather);
  assert.equal(actionForChar('!'), Actions.None);
  assert.equal(actionForChar('r'), Actions.Right); // ActionForChar upper-cases
});

test('ActionLine: frame-count prefix rule', () => {
  // ActionLine.cs:53-57 + :18-24  Frames comes from tokens[0]; a non-numeric
  // prefix makes the strict parse fail, and the final check lets a
  // frame-less line through only when some other action is present.
  assert.equal(parseActionLine(' 40').frameCount, 40);
  assert.equal(parseActionLine(''), null);
  assert.equal(parseActionLine('#Start'), null);
  assert.equal(parseActionLine('Read,sub'), null); // strict fails, loose fails

  const al = parseActionLine('1,R,J');
  assert.equal(al.frameCount, 1);
  assert.equal(al.actions, Actions.Right | Actions.Jump);

  // a comma with an empty frame count yields 0 frames but still parses
  assert.equal(parseActionLine(',R').frameCount, 0);
  assert.equal(parseActionLine(',R').actions, Actions.Right);

  // "0" is a valid (zero-frame) line
  assert.equal(parseActionLine('0,R').frameCount, 0);

  // negative counts parse but AddFrames adds nothing (for (i=0;i<Frames;i++))
  assert.equal(parseActionLine('-3,R').frameCount, -3);
});

test('ActionLine: loose parse handles comma-less lines', () => {
  // ActionLine.cs:146-299  TryParseLoose
  const al = parseActionLine('1J');
  assert.equal(al.frameCount, 1);
  assert.equal(al.actions, Actions.Jump);

  const al2 = parseActionLine('2gd');
  assert.equal(al2.frameCount, 2);
  assert.equal(al2.actions, Actions.Grab | Actions.Down);
});

test('ActionLine: feather angle and magnitude', () => {
  // ActionLine.cs:87-128
  let al = parseActionLine('1,F,90');
  assert.equal(al.featherAngle, '90');
  assert.equal(al.featherMagnitude, null);
  assert.equal(al.frameCount, 1);

  al = parseActionLine('1,F,90,0.5');
  assert.equal(al.featherAngle, '90');
  assert.equal(al.featherMagnitude, '0.5');

  al = parseActionLine('1,F,400'); // clamped to 360
  assert.equal(al.featherAngle, '360');

  al = parseActionLine('1,F,-5'); // clamped to 0
  assert.equal(al.featherAngle, '0');
});

test('ActionLine: dash-only / move-only / pressed-key prefixes', () => {
  // ActionLine.cs:66-81
  let al = parseActionLine('1,AL');
  assert.ok(al.actions & Actions.DashOnly);
  assert.ok(al.actions & Actions.LeftDashOnly);
  assert.equal(al.actions & Actions.Left, 0);

  al = parseActionLine('1,MU');
  assert.ok(al.actions & Actions.MoveOnly);
  assert.ok(al.actions & Actions.UpMoveOnly);

  al = parseActionLine('1,Pa');
  assert.ok(al.actions & Actions.PressedKey);
  assert.deepEqual([...al.customBindings], ['A']);
});

test('CommentLine.IsLabel', () => {
  // CommentLine.cs:14-18
  assert.equal(isLabel('#Start'), true);
  assert.equal(isLabel('#lvl_1'), true);
  assert.equal(isLabel('#cycle_a'), true);
  assert.equal(isLabel('#'), false);
  assert.equal(isLabel('##Start'), false);
  assert.equal(isLabel('# Start'), false);
});

test('Parsing.TryGetLineTarget: line numbers win over label lookup', () => {
  // Parsing.cs:83-103
  const lines = ['#Start', '  5', '#lvl_04 (2)', '  1'];
  assert.deepEqual(tryGetLineTarget('0', lines), { lineNumber: 0, isLabel: false });
  assert.deepEqual(tryGetLineTarget('3', lines), { lineNumber: 3, isLabel: false });
  assert.deepEqual(tryGetLineTarget('Start', lines), { lineNumber: 1, isLabel: true });
  assert.deepEqual(tryGetLineTarget('lvl_04 (2)', lines), { lineNumber: 3, isLabel: true });
  assert.equal(tryGetLineTarget('nope', lines), null);
});

// ---------------------------------------------------------------------------
// 2. synthetic fixtures: directive semantics
// ---------------------------------------------------------------------------

test('Read: whole file, labels define an inclusive range', () => {
  const dir = mkTas({
    'main.tas': 'FileTime: 0:00.000(0)\n  3\nRead,sub,B,E\n  2\n',
    'sub.tas': '  7\n#B\n  5,J\n  1\n#E\n  9\n',
  });
  const r = resolveTas(dir, 'main.tas');
  // 3 (own) + [ #B(0) + 5 + 1 + #E(0) ] + 2
  assert.equal(r.totalFrames, 11);
  assert.deepEqual(r.inputs.map((i) => i.frames), [3, 5, 1, 2]);
  assert.deepEqual(r.inputs.map((i) => i.file), ['main.tas', 'sub.tas', 'sub.tas', 'main.tas']);
  assert.deepEqual(r.inputs.map((i) => i.line), [2, 3, 4, 4]);
  assert.deepEqual(
    r.events.map((e) => [e.frame, e.type]),
    [[0, 'filetime'], [3, 'read'], [3, 'label'], [9, 'label']],
  );
});

test('Read: a numeric start argument is a line number, not a label', () => {
  const dir = mkTas({
    'main.tas': 'Read,sub,0,3\n',
    'sub.tas': '  10\n  20\n  30\n  40\n',
  });
  const r = resolveTas(dir, 'main.tas');
  // Take(endLine=3) -> lines 1..3; startLine=0 -> nothing skipped.
  assert.equal(r.totalFrames, 60);
});

test('Read: missing file aborts', () => {
  const dir = mkTas({ 'main.tas': 'Read,nope\n' });
  assert.throws(() => resolveTas(dir, 'main.tas'), /couldn't find file/);
});

test('Repeat / EndRepeat: body is re-read (count-1) more times, boundary lines excluded', () => {
  const dir = mkTas({
    // Repeat,3 spans file lines 5..6; the Repeat and EndRepeat lines themselves
    // are excluded (RepeatCommand.cs:79-80).
    'main.tas': 'FileTime: 0:00.000(0)\n#Start\n  5,J\nRepeat,3\n  1,R\n  2\nEndRepeat\n  4\n',
  });
  const r = resolveTas(dir, 'main.tas');
  // 5 + 3 + 2*3 + 4
  assert.equal(r.totalFrames, 18);
  assert.deepEqual(
    r.events.map((e) => [e.frame, e.type]),
    [[0, 'filetime'], [0, 'label'], [5, 'repeat'], [8, 'endrepeat']],
  );
});

test('Repeat: nesting', () => {
  const dir = mkTas({
    'main.tas': '#Start\nRepeat,2\n  1,J\nRepeat,2\n  1,R\nEndRepeat\nEndRepeat\n',
  });
  const r = resolveTas(dir, 'main.tas');
  assert.equal(r.totalFrames, 6);
});

test('Repeat: unpaired EndRepeat aborts', () => {
  const dir = mkTas({ 'main.tas': 'EndRepeat\n' });
  assert.throws(() => resolveTas(dir, 'main.tas'), /does not have a paired Repeat/);
});

test('Add: pure libTAS directive, contributes no inputs', () => {
  const dir = mkTas({ 'main.tas': 'Add 100\n  4\nAdd,1\n  1\n' });
  const r = resolveTas(dir, 'main.tas');
  assert.equal(r.totalFrames, 5);
  assert.deepEqual(r.events.map((e) => [e.frame, e.type]), [[0, 'add'], [4, 'add']]);
});

test('Play: reads from a label and stops the current file', () => {
  const dir = mkTas({ 'main.tas': '#Start\n  2\nPlay,Go\n  1\n#Go\n  3\n' });
  const r = resolveTas(dir, 'main.tas');
  // The trailing "  1" is never parsed: ReadLine returns false for Play
  // (InputController.cs:286-290).
  assert.equal(r.totalFrames, 5);
});

test('Play with a wait-frames argument adds those frames first', () => {
  const dir = mkTas({ 'main.tas': '#Start\n  2\nPlay,Go,7\n  1\n#Go\n  3\n' });
  assert.equal(resolveTas(dir, 'main.tas').totalFrames, 12);
});

test('breakpoint lines (***) never produce inputs', () => {
  const dir = mkTas({ 'main.tas': '***\n  2\n***!S400\n  3\n' });
  assert.equal(resolveTas(dir, 'main.tas').totalFrames, 5);
});

test('SaveAndQuitReenter: frames depend on the active save slot', () => {
  const dir = mkTas({ 'main.tas': 'SaveAndQuitReenter\n  2\n' });
  // "31" and "14" are emitted unconditionally (SaveAndQuitReenterCommand.cs:72-73).
  // slot == -1: 45 + [1,D + 1,O + 33] = 80                  (:76-83)
  assert.equal(resolveTas(dir, 'main.tas', { saveAndQuitReenterSlot: -1 }).totalFrames, 80 + 2);
  // slot == 0: 45 + [1,O + 56 + 0 + 1,O + 14 + 1,O + 1] = 119   (:86-97)
  // This is also the library default, because the recorded in-game trace of
  // '0 - 100%.tas' shows exactly this branch (see docs/tas-format.md).
  assert.equal(resolveTas(dir, 'main.tas').totalFrames, 119 + 2);
  // slot == 2: 45 + [74 + 2] = 121
  assert.equal(resolveTas(dir, 'main.tas', { saveAndQuitReenterSlot: 2 }).totalFrames, 121 + 2);
  // The Randomizer adds "1,F,180" + "1" in the slot == -1 branch (:78-81)
  assert.equal(
    resolveTas(dir, 'main.tas', { randomizerInstalled: true, saveAndQuitReenterSlot: -1 }).totalFrames,
    80 + 2 + 2,
  );
});

test('StunPause (Input mode) injects "1,S,N" + "10,O" on a 2-frame cycle', () => {
  // StunPauseCommand.cs:147-175 + AutoInputCommand.cs:162-235
  const dir = mkTas({ 'main.tas': 'StunPause\n  2\n  2\nEndStunPause\n  3\n' });
  const r = resolveTas(dir, 'main.tas');
  // Immediate injection at StunPause: 1 + 10 = 11
  //   line 2 "  2": +2
  //   line 3 "  2": +11 + 2
  //   line 5 "  3": +3 (AutoInput removed by EndStunPause)
  assert.equal(r.totalFrames, 11 + 2 + 13 + 3);
});

test('AutoInput / StartAutoInput / EndAutoInput cycle insertion', () => {
  // AutoInputCommand.cs:80  StartLine = AutoInput line + 1
  // AutoInputCommand.cs:98  Inputs = the file's lines 1..(StartAutoInput line - 1)
  // AutoInputCommand.cs:162-217  every CycleLength-th consumed frame re-reads Inputs
  const dir = mkTas({
    'main.tas': 'AutoInput,3\n  1,J\nStartAutoInput\n  10\nEndAutoInput\n',
  });
  const r = resolveTas(dir, 'main.tas');
  // line 2 runs before StartAutoInput, so it is a plain 1-frame input.
  // line 4 ("10") walks cycleOffset 3 -> 0 three times, flushing 3+3+3 and
  // injecting the 1-frame block three times, then flushes the last frame:
  //   10 + 3 * 1 = 13
  assert.equal(r.totalFrames, 1 + 13);
  assert.deepEqual(r.inputs.map((i) => i.frames), [1, 3, 1, 3, 1, 3, 1, 1]);
  assert.deepEqual(r.inputs.map((i) => i.line), [2, 4, 2, 4, 2, 4, 2, 4]);
});

test('AutoInput: unpaired StartAutoInput aborts at parse end', () => {
  const dir = mkTas({ 'main.tas': 'AutoInput,2\n  1,J\n' });
  assert.throws(() => resolveTas(dir, 'main.tas'), /does not have a paired StartAutoInput/);
});

test('SelectCampaign: unknown campaign aborts, Celeste resolves deterministically', () => {
  const dir = mkTas({ 'main.tas': 'SelectCampaign,Celeste\n' });
  const r = resolveTas(dir, 'main.tas', { areaLevelSets: ['Celeste'] });
  // 2 + 1,O + 94 + 1,O + 62 + 1,D + [1,O + 32 visible-slot delay(3*3) + 32
  //                                     + confirm 1,O] + 1,U
  assert.equal(r.totalFrames, 2 + 1 + 94 + 1 + 62 + 1 + 1 + 9 + 32 + 3 + 1 + 48 + 1);

  const dir2 = mkTas({ 'main.tas': 'SelectCampaign,NotACampaign\n' });
  assert.throws(() => resolveTas(dir2, 'main.tas', { areaLevelSets: ['Celeste'] }), /Unknown campaign/);
});

test('labels and comments', () => {
  const dir = mkTas({ 'main.tas': '#Start\n##NotALabel\n# NotALabel\n  1\n' });
  const r = resolveTas(dir, 'main.tas');
  assert.equal(r.totalFrames, 1);
  assert.deepEqual(r.events, [
    { frame: 0, type: 'label', args: ['Start'], file: 'main.tas', line: 1 },
  ]);
});

test('zero-frame and negative-frame lines add nothing but still parse', () => {
  const dir = mkTas({ 'main.tas': '0,R\n-5,J\n,R\n  2\n' });
  const r = resolveTas(dir, 'main.tas');
  assert.equal(r.totalFrames, 2);
  assert.deepEqual(r.inputs.map((i) => i.line), [4]);
});

test('inputs entry shape', () => {
  const dir = mkTas({ 'main.tas': '#lvl_1\n  5,R,J\n' });
  const r = resolveTas(dir, 'main.tas');
  assert.deepEqual(r.inputs, [
    { frames: 5, action: '5,R,J', line: 2, file: 'main.tas' },
  ]);
  assert.equal(r.version, 1);
  assert.equal(r.entry, 'main.tas');
});

test('selfCheck rejects corrupted results', () => {
  const good = { version: 1, entry: 'x', totalFrames: 1, inputs: [{ frames: 1, action: '', line: 1, file: 'x' }], events: [] };
  assert.deepEqual(selfCheck(good), []);
  const bad = { ...good, totalFrames: 2 };
  assert.ok(selfCheck(bad).some((p) => p.includes('totalFrames')));
  const bad2 = { ...good, events: [{ frame: 0, type: 'a', args: [], file: 'x', line: 1 }, { frame: 0, type: 'b', args: [], file: 'x', line: 2 }] };
  assert.deepEqual(selfCheck(bad2), []);
});

// ---------------------------------------------------------------------------
// 3. corpus tests
// ---------------------------------------------------------------------------

test('corpus: 0 - 100%.tas resolved frame count', { skip: corpusSkip }, () => {
  const r = resolveTas(CORPUS, '0 - 100%.tas');
  // InputController.Inputs.Count: every input line's frame count, flattened in
  // play order (InputController.cs:326-332), Read + Repeat expanded.
  assert.equal(r.totalFrames, 281105);
  assert.ok(selfCheck(r).length === 0);
  assert.equal(r.inputs.reduce((a, e) => a + e.frames, 0), r.totalFrames);
});

test('corpus: 0 - 202 Berries.tas resolved frame count', { skip: corpusSkip }, () => {
  const r = resolveTas(CORPUS, '0 - 202 Berries.tas');
  assert.equal(r.totalFrames, 461114);
  assert.ok(selfCheck(r).length === 0);
});

test('corpus: FileTime header vs resolved frame count (0 - 100%.tas)', { skip: corpusSkip }, () => {
  // The header's N is (SaveData.Time - start) / RawDeltaTime.SecondsToTicks(),
  // NOT Inputs.Count. Both numbers and their exact difference are pinned here so
  // that neither can drift. See docs/tas-format.md "The FileTime header" and the
  // "KNOWN DEVIATION" test below.
  const h = parseFileTimeHeader(corpusText('0 - 100%.tas'));
  assert.ok(h);
  assert.equal(h.frames, 242687);
  const r = resolveTas(CORPUS, '0 - 100%.tas');
  assert.equal(r.totalFrames - h.frames, 38418);
});

test('corpus: FileTime header vs resolved frame count (0 - 202 Berries.tas)', { skip: corpusSkip }, () => {
  const h = parseFileTimeHeader(corpusText('0 - 202 Berries.tas'));
  assert.ok(h);
  assert.equal(h.frames, 401427);
  const r = resolveTas(CORPUS, '0 - 202 Berries.tas');
  assert.equal(r.totalFrames - h.frames, 59687);
});

test('KNOWN DEVIATION: the FileTime header (N) is not InputController.Inputs.Count', { skip: corpusSkip }, () => {
  // The task that produced this resolver asked for
  //     resolveTas(root, '0 - 100%.tas').totalFrames === 242687
  //     resolveTas(root, '0 - 202 Berries.tas').totalFrames === 401427
  // i.e. it expected the header's (N) field to equal the flattened input count.
  //
  // That cannot hold. Evidence, strongest first:
  //
  //  A. A recorded in-game frame trace of `0 - 100%.tas`
  //     (`.tmp/tasrun/trace-100pct.jsonl`, produced by running the real
  //     Celeste + CelesteTAS) covers the whole TAS and reports **281113** frames.
  //     This resolver reports 281105 for the same file: a 8-frame difference,
  //     entirely inside `SelectCampaign`'s name-entry BFS (which needs
  //     `ActiveFont.Measure` font metrics). Every other input entry matches the
  //     trace frame-for-frame. The header says 242687 -- 38426 frames fewer than
  //     the game actually fed.
  //  B. `1SHC.tas` contains nothing but a `console` line and 153 input lines. Its
  //     frame numbers sum to 1441, and InputController.AddFrames
  //     (InputController.cs:326-332) appends exactly `Frames` copies of each
  //     parsed line, with no code path that removes inputs. So Inputs.Count is
  //     1441. Its header says 1359. (Its trace-verified sibling `1A.tas` matches
  //     the game exactly: 3215 frames.)
  //  C. Every one of the 66 headers in the corpus satisfies
  //     `headerMillis === frames * 17` exactly, which is what
  //     GameInfo.FormatTime (GameInfo.cs:643-650) produces:
  //     `ShortGameplayFormat(T)(T/170000)` where 170000 =
  //     `Engine.RawDeltaTime.SecondsToTicks()` after the whole-millisecond
  //     rounding in Utils/Extensions.cs:850-855.
  //  D. T is `SaveData.Time`, advanced by `Level.UpdateTime`
  //     (celeste-fna/Celeste/Level.cs:1733) once per frame by exactly those
  //     170000 ticks, and skipped when there is no Level, when `Overlay != null`
  //     (pause menu / journal / Save & Quit menus, Level.cs:1757-1762), for
  //     `InCredits`, for `Session.Area.ID == 8` (the Epilogue) and for
  //     `TimerStopped`. So N counts *timer frames*, a strict lower bound on
  //     Inputs.Count that depends on runtime scene state a .tas file does not
  //     contain.
  //
  // The requested numbers are therefore recorded here as expectations that are
  // deliberately not asserted; the preceding tests pin the real relationship.
  assert.equal(242687, 242687); // requested: resolveTas(root, '0 - 100%.tas').totalFrames
  assert.equal(401427, 401427); // requested: resolveTas(root, '0 - 202 Berries.tas').totalFrames
  assert.notEqual(resolveTas(CORPUS, '0 - 100%.tas').totalFrames, 242687);
  assert.notEqual(resolveTas(CORPUS, '0 - 202 Berries.tas').totalFrames, 401427);
});

test('corpus: frame trace validation values (0 - 100%.tas)', { skip: corpusSkip }, () => {
  // Values measured from the recorded in-game trace; the trace file itself lives
  // outside this tool (`${root}/../tasrun/trace-100pct.jsonl`), so only the
  // resulting numbers are pinned here. See docs/tas-format.md.
  const GAME_TRACE_FRAMES = 281113;
  const RESOLVER_FRAMES = 281105;
  const FONT_DEPENDENT_RESIDUAL = 8; // SelectCampaign name-entry BFS
  assert.equal(GAME_TRACE_FRAMES - RESOLVER_FRAMES, FONT_DEPENDENT_RESIDUAL);
  assert.equal(resolveTas(CORPUS, '0 - 100%.tas').totalFrames, RESOLVER_FRAMES);
});

test('canonicalActionText reproduces ActionLine.ToString()', () => {
  assert.equal(canonicalActionText('1,R,J'), '   1,R,J');
  assert.equal(canonicalActionText('40'), '  40');
  assert.equal(canonicalActionText('1,F,90'), '   1,F,90');
  assert.equal(canonicalActionText('2,R,U,Z'), '   2,R,U,Z'); // Right before Up before DemoDash
  assert.equal(canonicalActionText('1,J,K'), '   1,J,K');
  // DashOnly is emitted as a single "A<dirs>" token (ActionLine.cs:307); the
  // bare Left/Right/Up/Down flags are not set by the "A" prefix.
  assert.equal(canonicalActionText('1,AL'), '   1,AL');
  assert.equal(canonicalActionText('not an action line'), null);
});

test('corpus: every FileTime header is <time> = N x 17ms', { skip: corpusSkip }, () => {
  const dirs = [CORPUS];
  const files = [];
  while (dirs.length > 0) {
    const dir = dirs.pop();
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) dirs.push(p);
      else if (e.name.endsWith('.tas')) files.push(p);
    }
  }
  let seen = 0;
  for (const abs of files) {
    const h = parseFileTimeHeader(fs.readFileSync(abs, 'utf8'));
    if (!h) continue;
    seen++;
    const ms = ((h.hours * 60 + h.minutes) * 60 + h.seconds) * 1000 + h.millis;
    assert.equal(ms, h.frames * 17, `${path.basename(abs)}: ${h.raw}`);
  }
  assert.ok(seen >= 60, `expected the corpus to carry >= 60 headers, saw ${seen}`);
});

test('corpus: per-chapter files resolve, and their headers are pinned', { skip: corpusSkip }, () => {
  const expected = {
    '1SH0.tas': { computed: 7034, header: 6462 },
    '1SH1.tas': { computed: 432, header: null },
    '1SH2.tas': { computed: 854, header: null },
    '1SHC.tas': { computed: 1441, header: 1359 },
    '0 - Prologue.tas': { computed: 1544, header: null },
  };
  for (const [file, exp] of Object.entries(expected)) {
    const r = resolveTas(CORPUS, file);
    assert.equal(r.totalFrames, exp.computed, file);
    assert.ok(selfCheck(r).length === 0, file);
    const h = parseFileTimeHeader(corpusText(file));
    assert.equal(h === null ? null : h.frames, exp.header, file);
  }
});

test('corpus: read-name resolution', { skip: corpusSkip }, () => {
  assert.equal(path.basename(findReadTargetFile(CORPUS, '1SH0')), '1SH0.tas');
  assert.equal(path.basename(findReadTargetFile(CORPUS, '0 - Prologue')), '0 - Prologue.tas');
  assert.equal(findReadTargetFile(CORPUS, 'definitely-not-a-file'), null);
  assert.equal(
    path.basename(findReadTargetFile(path.join(CORPUS, '202'), '../1D')),
    '1D.tas',
  );
});
