# Celeste `.tas` format - everything `tas-replay` implements

This document describes exactly what `src/resolve.mjs` implements, one construct at
a time, with a `file:line` citation into the reference sources for every
behavioural claim.

## Reference sources

| shorthand | path |
| --- | --- |
| `CT/` | `MicroblocksQolUtils/.work/celestetas-src/CelesteTAS-EverestInterop/Source/` |
| `SC/` | `MicroblocksQolUtils/.work/celestetas-src/StudioCommunication/` |
| `ST/` | `MicroblocksQolUtils/.work/celestetas-src/Studio/CelesteStudio/` |
| `CE/` | `celeste-next-gym/vendor/celeste-fna/` (Celeste 1.4.0.0 FNA source) |

The reference checkout is at commit `fd1e267`. The corpus under test is the
`.tas` input repository pinned at `074e71a9` (a different repository: it holds TAS
inputs, not tool code).

## Output schema

```js
resolveTas(rootDir, entryName, options) => {
  version: 1,
  entry: "0 - 100%.tas",
  totalFrames: <int>,          // == sum(inputs[].frames) == InputController.Inputs.Count
  inputs: [ { frames, action, line, file } ],
  events: [ { frame, type, args, file, line } ],
}
```

* `inputs` is the flat play order of `InputController.Inputs`
  (`CT/TAS/Input/InputController.cs:39,326-332`). One entry per `AddFrames` call:
  `frames` is that call's frame count and `action` is the literal source text of
  the input line (or the generated action text for inputs a command synthesised).
  `line`/`file` locate it. `celeste>AutoInputCommand.TryInsert` can split a single
  source line into several entries, which is why the unit is the `AddFrames` call
  rather than the source line.
* `events` records every non-input directive in the order it took effect:
  parsed commands (`type` = the registered command name, lowercased, so aliases
  like `EnforceMaingame` normalise to `enforcelegal`) and labels (`type: "label"`,
  `args: [name]`). `frame` is `CurrentParsingFrame` at the moment the directive
  was parsed, i.e. the index of the first frame it precedes
  (`CT/TAS/Input/InputController.cs:278-284`). Plain comments are not emitted:
  they have no effect (`CT/TAS/Input/InputController.cs:302-310`).
* `actionLineToString()` / `canonicalActionText()` reproduce CelesteTAS's own
  canonical text (`SC/ActionLine.cs:301-316`, stored as
  `InputFrame.actionLineString` at `CT/TAS/Input/InputFrame.cs:62`), which is the
  form a game-side trace of `Inputs` reports.

## 1. Parse pipeline and dispatch order

`InputController.ReadLine` (`CT/TAS/Input/InputController.cs:274-316`) is the whole
grammar. For each line, with `lineText = line.Trim()`:

1. `Command.TryParse` (`CT/TAS/Input/Command.cs:93-116`) - the line must start with
   a letter (`char.IsLetter`) and `CommandLine.TryParse` must succeed and the
   command name must match a registered `[TasCommand]` (case-insensitively,
   `Command.cs:66-69`). If it matches, the command is recorded at
   `CurrentParsingFrame` and, when its `ExecuteTiming` includes `Parse`
   (`Command.cs:119-125`), executed immediately.
   * `Play` additionally stops reading the current file (`InputController.cs:286-290`).
2. `FastForwardLine.TryParse` (`SC/FastForwardLine.cs:10-36`) - a line starting with
   `***`. Breakpoints produce no inputs. (Disabled inside `Read`, see below, and
   absent from the whole corpus.)
3. `lineText.StartsWith("#")` (`InputController.cs:302-310`) - comment; if
   `CommentLine.IsLabel` (`SC/CommentLine.cs:14-18`) it is also a fast-forward label.
4. `AutoInputCommand.TryInsert` (`CT/TAS/Input/Commands/AutoInputCommand.cs:162-217`).
5. `InputController.AddFrames` (`InputController.cs:319-323`) - the ordinary input
   path.

Note that the trim happens *before* the `#` test, so an indented `   #Start` **is**
a label, even though `CommentLine.IsLabel`'s own doc comment only promises the
untrimmed form.

`ReadLines` (`InputController.cs:250-271`) walks the line array, skips
`fileLine < startLine`, stops when `ReadLine` returns false, increments
`studioLine` for every processed line of the *main* file, and finally records a
hidden label at the end of the block.

## 2. Command token splitter (`SC/CommandLine.cs:29-150`)

* The line must start (after leading whitespace) with a letter, otherwise it is not
  a command line at all (`:30-34`).
* The separator is decided by **one** regex search over the trimmed line:
  `/(?:\s+)|(?:\s*,\s*)/` (`:24`). The first match, and only that match, becomes
  `ArgumentSeparator`; the same literal string is then used to split the rest
  (`:38-41,53,66-74`).
  * `Read, 0 - Prologue, Start` -> separator `", "` -> args `["0 - Prologue", "Start"]`.
  * `Read,0 - Prologue,Start` -> separator `","` -> the same two args.
  * `console load 1 lvl_9b` -> separator `" "` -> args `["load", "1", "lvl_9b"]`.
  * Because only the first separator is remembered, mixing styles
    (`Set, A,0,B`) makes `A,0,B` a single argument.
* Quotes (`"`), brackets (`[`), and braces (`{`) all participate in one
  `groupStack`; a separator only splits when the stack is empty (`:57-58,66`).
  `[`/`{` push, `]`/`}` pop (and an unmatched closer makes the whole line fail,
  `:90-109`). A `"` toggles only when the stack is empty or already on a `"`
  (`:78-84`); otherwise it is a literal character.
* Quotation marks themselves are **not** kept in the argument (`:78-84`).
* `\` escapes the next character; `\n` becomes a newline, anything else is taken
  literally, and a trailing `\` fails the line (`:110-128`).
* Command matching is case-insensitive (`Command.cs:66-69`), and the command name
  is `lineTrimmed[..separatorIndex]` - so `FileTime:` (with the colon) is the
  command token, which is why the metadata commands register `FileTime:` as an
  alias (`CT/TAS/Input/Commands/MetadataCommands.cs:142`). The fullwidth colon
  variants are also registered.

## 3. Input lines (`SC/ActionLine.cs`)

### 3.1 Frame-count prefix rule

* The line is trimmed and split on `,` with `TrimEntries` (`:47-50`).
* If `tokens[0]` is empty/whitespace **or** parses as an `int`, it becomes `Frames`;
  otherwise the strict parse fails and the loose parse is tried (`:53-57`).
* `Frames` is stored trimmed and its integer value is `int.TryParse` or **0** on
  failure (`:18-24`). So `,R` is a *zero-frame* input line, and `0,R` likewise.
* `AddFrames` appends `Frames` copies of the frame (`InputController.cs:326-332`),
  so a zero or negative count appends nothing - but the line still parses and is
  still recorded as a `repeat`-visible input line when it does have frames.
* A line with an empty frame prefix parses only if some other action/feather/key
  is present (`:131-139`); otherwise it is not an input line at all. Empty lines
  and comments therefore contribute nothing.
* There is no upper bound at parse time: `ActionLine.MaxFrames = 9999` (`:10-11`)
  is only used for Studio's editing clamp and `ToString()` padding.
* Real frame counts in the corpus go up to a few hundred; the numbers are
  *time*, i.e. `N` means "hold this input for `N` frames".

### 3.2 Action characters (`SC/Actions.cs`)

`ActionForChar` (`Actions.cs:86-109`) upper-cases the character first, so lower
case letters work. The complete table (the enum is at `Actions.cs:9-39`, the
`Chars` dictionary at `Actions.cs:42-64`):

| char | `Actions` flag | what `InputHelper` presses | Celeste `Input` effect |
| --- | --- | --- | --- |
| `L` | `Left` | `GamePadDPad.Left` pressed (`CT/TAS/InputHelper.cs:74`) + `BindingHelper.Left` = `Buttons.DPadLeft` (`CT/TAS/BindingHelper.cs:24`) | `Settings.Left`, threshold 0.3 (`CE/Celeste/Input.cs:145`) -> `Input.MoveX == -1` |
| `R` | `Right` | DPad right (`InputHelper.cs:75`), `BindingHelper.Right` (`BindingHelper.cs:25`) | `Input.MoveX == +1` (`CE/Celeste/Input.cs:145`) |
| `U` | `Up` | DPad up (`InputHelper.cs:72`), `BindingHelper.Up` (`BindingHelper.cs:22`) | `Input.MoveY == -1` (threshold 0.7, `CE/Celeste/Input.cs:147`) |
| `D` | `Down` | DPad down (`InputHelper.cs:73`), `BindingHelper.Down` (`BindingHelper.cs:23`) | `Input.MoveY == +1` (`CE/Celeste/Input.cs:147`) |
| `J` | `Jump` | `BindingHelper.JumpAndConfirm` = `Buttons.A` (`InputHelper.cs:85`, `BindingHelper.cs:14`) | `Input.Jump` (`BindingHelper.cs:62`) and `Input.Confirm` (`BindingHelper.cs:67`); edge buffer 0.08 s (`CE/Celeste/Input.cs:153`) |
| `K` | `Jump2` | `Buttons.Y` (`InputHelper.cs:86`, `BindingHelper.cs:15`) | `Input.Jump` (`BindingHelper.cs:62`) - a second, independent jump button, so `J` and `K` can alternate to re-press Jump |
| `Z` | `DemoDash` | `Buttons.RightShoulder` (`InputHelper.cs:87`, `BindingHelper.cs:27`) | `Input.CrouchDash` (`BindingHelper.cs:73`), and `CrouchDashMode` is forced to `Press` (`BindingHelper.cs:158`), so `Input.CrouchDashPressed == CrouchDash.Pressed` (`CE/Celeste/Input.cs:120-131`) |
| `V` | `DemoDash2` | `Buttons.RightStick` (`InputHelper.cs:88`, `BindingHelper.cs:28`) | `Input.CrouchDash` (`BindingHelper.cs:73`) |
| `X` | `Dash` | `Buttons.B` (`InputHelper.cs:89`, `BindingHelper.cs:16`) | `Input.Dash` (`BindingHelper.cs:63`), `Input.Talk` (`BindingHelper.cs:64`) and `Input.Cancel` (`BindingHelper.cs:68`). `CrouchDashMode = Press` makes `Input.DashPressed == Dash.Pressed` (`CE/Celeste/Input.cs:103-114`) |
| `C` | `Dash2` | `Buttons.X` (`InputHelper.cs:90`, `BindingHelper.cs:17`) | `Input.Dash` and `Input.Cancel` (`BindingHelper.cs:63,68`) |
| `G` | `Grab` | `Buttons.LeftStick` (`InputHelper.cs:91`, `BindingHelper.cs:18`) | `Input.Grab` (`BindingHelper.cs:61`); `GrabMode` is forced to `Hold` (`BindingHelper.cs:161`) so `Input.GrabCheck == Grab.Check` (`CE/Celeste/Input.cs:96-101`) |
| `H` | `Grab2` | `Buttons.Back` (`InputHelper.cs:92`, `BindingHelper.cs:19`) | `Input.Grab` (`BindingHelper.cs:61`) - second grab button |
| `S` | `Start` | `Buttons.Start` (`InputHelper.cs:93`, `BindingHelper.cs:20`) | `Input.Pause` (`BindingHelper.cs:66`) / `MenuPause` |
| `Q` | `Restart` | `Buttons.LeftShoulder` (`InputHelper.cs:94`, `BindingHelper.cs:21`) | `Input.QuickRestart` (`BindingHelper.cs:71`) |
| `N` | `Journal` | `Buttons.LeftTrigger` (`InputHelper.cs:99`) plus analog trigger `1f` (`InputHelper.cs:83`), `BindingHelper.JournalAndTalk` (`BindingHelper.cs:26`) | `Input.Journal` (`BindingHelper.cs:70`) and `Input.Talk` (`BindingHelper.cs:64`) |
| `O` | `Confirm` | **keyboard only**: `BindingHelper.Confirm2 = Keys.NumPad0` (`InputHelper.cs:40-42`, `BindingHelper.cs:33,67`) | `Input.Confirm` (`BindingHelper.cs:67`). Note `O` does *not* press a gamepad button - unlike `J`, which also sets Confirm |
| `F` | `Feather` | all DPad buttons released and both sticks replaced: left stick = `StickPosition`, right stick = `DashOnlyStickPosition` (`InputHelper.cs:18-22,65-68`) | `Input.Feather` (a `VirtualJoystick` built from the *MoveOnly* bindings, `CE/Celeste/Input.cs:151`) and `Input.Aim` (built from the *DashOnly* bindings, `CE/Celeste/Input.cs:149`) |
| `A` | `DashOnly` | prefix; each following `L/R/U/D` sets `LeftDashOnly`/`RightDashOnly`/`UpDashOnly`/`DownDashOnly` (`SC/ActionLine.cs:66-71`, `SC/Actions.cs:66-72,197-205`), which map to the right-thumbstick buttons (`BindingHelper.cs:29-32,75-78`) | `Input.Aim` from the right stick only (`CE/Celeste/Input.cs:149`, `CT/TAS/Input/InputFrame.cs:75-85`) |
| `M` | `MoveOnly` | prefix; each following `L/R/U/D` sets `LeftMoveOnly`/... (`SC/ActionLine.cs:72-77`, `SC/Actions.cs:207-215`), which map to the arrow keys (`InputHelper.cs:44-58`, `BindingHelper.cs:34-37,80-83`) | the `*MoveOnly` halves of `Input.MoveX` / `Input.MoveY` / `Input.Feather` (`CE/Celeste/Input.cs:145,147,151`) |
| `P` | `PressedKey` | prefix; every following character is added to the keyboard state (`SC/ActionLine.cs:78-81`, `InputHelper.cs:60`) | arbitrary custom bindings (used by `Press`, `CT/TAS/Input/Commands/PressCommand.cs`) |

Anything not in the table maps to `Actions.None` (`SC/Actions.cs:108`).
`ActionLine.Sorted()` (`Actions.cs:136-157`) fixes the canonical output order:
Left, Right, Up, Down, Jump, Jump2, Dash, Dash2, DemoDash, DemoDash2, Grab, Grab2,
Start, Restart, Journal, Confirm, DashOnly, MoveOnly, PressedKey, Feather.

### 3.3 Feather angle / magnitude

`F,angle[,magnitude]` (`SC/ActionLine.cs:87-128`). The two values stay as strings
and are clamped: angle to `[0, 360]`, magnitude to `[0, 1]` (`:89-111`, loose path
`:283-296`). They are resolved to a stick vector by
`AnalogHelper.ComputeAngleVector` (`CT/TAS/AnalogHelper.cs:38`, called from
`CT/TAS/Input/InputFrame.cs:65-73`): `x = sin(angle)`, `y = cos(angle)` with the
cardinal values short-circuited (`AnalogHelper.cs:55-75`), so **0 degrees is
straight up**, 90 right, 180 down, 270 left; the default `AnalogMode.Ignore`
(`AnalogHelper.cs:36`) multiplies by the magnitude and keeps the analog
dead-zone `0.239532471` (`AnalogHelper.cs:30`). An unparseable angle falls back to
0 degrees with magnitude 1 (`InputFrame.cs:70-72`).

### 3.4 Strict vs loose parse

`ActionLine.TryParse = TryParseStrict || TryParseLoose` (`SC/ActionLine.cs:39`).
Strict handles the comma form; loose (`:146-299`) is a character scanner that also
accepts comma-less forms such as `1J` or `1gd`, and clamps feather values at the
end. Loose returns false when the scanner never left the frame-digit state
(`:298`). The resolver implements both, including the subtle cases:

* a token longer than one character is only legal for `A`, `M` and `P`
  (`:66-85`), so `1,RJ` fails strict and is re-read loosely as `R` then `J`;
* feather arguments are consumed so that `1,F,90,0.5` has one frame, not three;
* `1,F,90,abc` fails strict (the `abc` token is length 3) and parses loosely as
  one frame of Feather.

## 4. Labels and comments (`SC/CommentLine.cs:14-18`)

A line is a label iff its trimmed text is at least two characters, starts with
`#`, and the second character is a letter. `#Start`, `#lvl_1`, `#cycle_a` are
labels; `#`, `##Start`, `# Start` are not.

Labels matter for two things: they are targets for `Read`/`Play` start and end
arguments, and they are recorded as fast-forward points
(`InputController.cs:302-305`). `Parsing.TryGetLineTarget` (`SC/Parsing.cs:83-103`)
tries a line number first and only then searches for `^#\s*<escaped>$` against
each trimmed line - so `Read,LoadA,0,Control` reads from line **0** (the whole
file), not from a label named `0`.

## 5. `Read` (`CT/TAS/Input/Commands/ReadCommand.cs:119-204`)

Forms:

```
Read, File
Read, File, StartLabelOrLine
Read, File, StartLabelOrLine, EndLabelOrLine
```

* Extra arguments beyond the third are parsed but ignored (`:155-167`).
* The resolver reads the file relative to the *directory of the file containing
  the `Read`*, via `Parsing.FindReadTargetFile` (`SC/Parsing.cs:11-80`):
  `Path.Combine(dir, arg)` with `.tas` appended unless already present; on a miss
  it walks the path components case-insensitively, allows `..`, and finally
  accepts a *unique* file whose stem starts with the requested name. Ambiguous or
  missing targets abort the TAS (`:137-143`).
* Reading yourself is refused (`:145-148`); recursion into an identical
  `Read` line already on the stack is refused (`:170-174`).
* Range semantics: `startLine` is a 1-based line number that is **included**;
  `endLine` is passed to `File.ReadLines(path).Take(endLine)`
  (`InputController.cs:240`), so the first `endLine` lines are read and the end
  label's own line is included (it is a comment, so it contributes no frames).
* When the start argument is exactly `Start`, CelesteTAS additionally injects an
  `Assert,Equal,"<SID>",{Session.Area.SID}` line (`ReadCommand.cs:179-190`) -
  zero frames, but it does appear in `events`.
* `EnableBreakpointParsing` is turned off for the duration of the nested read
  (`ReadCommand.cs:196-198`), so `***` lines inside a read file are ignored.
* After the nested read, an `AnalogMode` restore line is parsed
  (`ReadCommand.cs:203`) - zero frames.

## 6. `Repeat` / `EndRepeat` (`CT/TAS/Input/Commands/RepeatCommand.cs:42-119`)

* `Repeat, Count`: `Count` must parse as an int and be in `[1, 10_000_000]`
  (`:47-64`). It pushes `{StartFrame: CurrentParsingFrame, Count, file, line}`
  (`:66`). `CurrentParsingFrame` is the number of frames parsed so far, i.e. the
  frame index where the block starts.
* `EndRepeat`: pops the stack (an unpaired `EndRepeat` aborts, `:73-76`) and then
  (`:79-93`):
  * `startLine = RepeatLine + 1`, `endLine = EndRepeatLine - 1` - the `Repeat`
    and `EndRepeat` lines themselves are excluded;
  * if `endLine < startLine`, the file is missing, or `Count <= 1`, nothing is
    re-read;
  * otherwise the *same file* is re-read from disk (`File.ReadLines(file).Take(endLine)`
    then `ReadLines(..., startLine, ...)`) once for each `i` in `2..Count`.
* Consequently the block's inputs and **all commands inside it** are re-executed
  `Count - 1` extra times, including nested `Repeat`s and `Read`s. The first pass
  is the ordinary in-line parse, so total frames for a block of `B` frames is
  `B * Count`.
* The stack is global to the parse, so a `Repeat` opened in a read file must be
  closed in the same file for the ranges to make sense; an unclosed `Repeat` is
  reported by the `[ParseFileEnd]` hook (`:24-34`).
* `RepeatIndex`/`RepeatCount` are only bookkeeping for Studio's display
  (`:98-105`) and are not exposed by this resolver.

## 7. `Play` (`CT/TAS/Input/Commands/PlayCommand.cs:41-59`)

* `Play, StartLabel[, FramesToWait]`. `StartLabel` is resolved with
  `TryGetLineTarget` against the *current* file (`:44`).
* If the second argument parses as an int it is fed as a plain input line first
  (`:49-51`).
* If `startLine <= studioLine + 1` the command is a no-op with a warning
  (`:53-56`) - this is the recursion guard, expressed in studio lines.
* Otherwise the current file is re-read from `startLine` to the end
  (`:58`), with `studioLine = startLine - 1` when the file is the main file.
* `Play` then makes `ReadLine` return false (`InputController.cs:286-290`), so
  the remainder of the containing file is *not* parsed.
* `Play` does not occur anywhere in the corpus (only inside comments).

## 8. `Add` (`CT/TAS/LibTasHelper.cs:258-264`)

`Add, input` joins its arguments with `,` and calls `AddInputFrame`
(`LibTasHelper.cs:85-96`), which returns immediately unless a libTAS export is
active. **`Add` therefore contributes no frames to `InputController.Inputs`**; it
only writes lines into the libTAS movie. The corpus uses `Add 131`, `Add 58`,
`Add 29`, `Add 1` and similar as libTAS-export hints between `Read` blocks. The
resolver records them as events and adds nothing.

## 9. `AutoInput` family (`CT/TAS/Input/Commands/AutoInputCommand.cs`)

* `AutoInput, CycleLength` (`:65-82`) registers a per-file argument block with
  `StartLine = line + 1` and `CycleOffset = CycleLength = CycleLength`
  (`:34-37,80`).
* `StartAutoInput` (`:84-99`) snapshots the file's lines `1..line-1` as the block
  to inject (`:98`). The block conventionally sits between the `AutoInput` and
  `StartAutoInput` lines.
* Every subsequent input line in that file goes through `TryInsert`
  (`:162-217`) instead of the plain path. While `CycleOffset` counts down, frames
  are accumulated; when it reaches 0 the block is re-parsed
  (`ParseInsertedLines`, `:219-235`) and the accumulator is flushed.
  * `SkipInput[, Frames[, WaitFrames]]` (`:122-160`) can skip the next input
    (`SkipNextInput`) or shift `CycleOffset` by `SkipFrames` after
    `SkipWaitingFrames` frames (`:185-192`).
  * Because `TryInsert` is called for *every* line, a nested `Read`/`Repeat`
    inside an active `AutoInput` block also injects.
  * `TryInsert` only takes over when `Inputs` has been snapshotted and
    `Inserting` is false (`:167-173`); during `ParseInsertedLines` it returns
    false so the injected lines take the ordinary path.
* Total frames are still `Inputs.Count`; the difference is that one source line
  can become several `inputs` entries.
* `AutoInput` is not used anywhere in the corpus.

## 10. `StunPause` / `EndStunPause` / `StunPauseMode`
(`CT/TAS/Input/Commands/StunPauseCommand.cs`)

* The mode is `EnforceLegal && parsing ? Input : LocalMode ?? GlobalModeParsing ?? Input`
  (`:68-77`); `Input` is the default.
* In `Input` mode, `StunPause` (`:133-155`) builds an `AutoInput` block with
  `CycleLength = 2` (`:162`), whose payload is the file's lines
  `1..line-1` plus `"1,S,N"` followed by a pause input computed from the last
  parsed frame (`:165-172`, `:177-194`):
  the pause input is `10,J,K`, `10,J`, `10,K,O` or `10,O` depending on whether
  the previous frame had Jump and/or Jump2 (`:182-190`).
  The block is then parsed immediately (`:173`), so `StunPause` itself injects
  `1 + 10 = 11` frames on the spot.
* Until the matching `EndStunPause` (`:196-203`, which removes the block), every
  2 consumed frames of the file inject those 11 frames again.
* `StunPause, Simulate` and `StunPauseMode` behave totally differently: they
  change how the game double-updates, i.e. they are *runtime* semantics and
  generate no inputs (`:205-217,239-247`).
* The corpus uses plain `StunPause` (Input mode) in `4B`, `5S`, `6A`, `6B`,
  `6HC`, `7A`, `7B`, `7S`, `9`, `9S`, `7BG`, `9G` and their `202/` variants.

## 11. `SaveAndQuitReenter`
(`CT/TAS/Input/Commands/SaveAndQuitReenterCommand.cs:53-129`)

At parse time the command emits, in this order:

* `Unsafe` if `SafeCommand.DisallowUnsafeInputParsing` is currently set (`:61-64`)
  (0 frames; it only flips the flag);
* `Set,Celeste.PlayMode,Debug` when the slot is `-1` and the play mode is not
  already `Debug` (`:67-69`) (0 frames);
* `AddInputFrame("58")` (`:71`) - libTAS export only, **no frame**;
* `"31"` and `"14"` unconditionally (`:72-73`);
* then either the "no save file" branch `1,D`, `1,O`, `33` = 35 frames (`:74-83`,
  plus `1,F,180` + `1` when the Randomizer mod is installed, `:78-81`),
  or the "existing save file" branch `1,O`, `56`, `slot` slot-selection frames
  (`1,D` / `1,F,180` alternating), `1,O`, `14`, `1,O`, `1` = `74 + slot` frames
  (`:84-99`);
* finally, if the slot was `-1` and the play mode was not `Debug`, two more
  `Set` lines and, if it was unsafe, a `Safe` line (`:101-108`) - all 0 frames.

Totals: `-1` -> 80, slot `n >= 0` -> `119 + n`.

`ActiveFileSlot` (`:14-26`) is live game state: the file-select slot index when
the overworld's file select is open, otherwise `SaveData.Instance.FileSlot`, and
`-1` when no save is loaded. It is exposed as
`options.saveAndQuitReenterSlot`; see the trace section for how the corpus value
was determined.

## 12. `SelectCampaign`
(`CT/TAS/Input/Commands/SelectCampaignCommand.cs:96-172`)

Emits `2`, `1,O`, `94`, `1,O` (`:143-148`), then `62` for "no save slots" or
`56` + `slot` selection frames + `1,O` + `15` (`:150-160`), then `1,D` (`:165`),
then `InputName` (`:174-357`) and `ChangeSelectedCampaign` (`:359-438`).

* `InputName` presses confirm once per character of the save-file name
  (default `TAS`, `:121`), emits `maxSaveFile`-window delays of `3` frames each
  (`:182-188`), a `32`, the BFS cursor path, then `1,S` + `48` (`:355-356`).
* The BFS grid geometry comes from `ActiveFont.Measure` and the language's
  `name_letters` dialog (`:192-231`). **This is the one construct the resolver
  cannot resolve from the `.tas` text**, because font metrics are not in the file.
  With `options.fontWidths === null` (the default) the resolver emits the
  deterministic parts - the per-character confirm presses and `1,S`/`48` - and
  reports a warning through `options.onWarning`. Supplying `fontWidths` enables
  the full BFS.
* `ChangeSelectedCampaign` depends on `Settings.Instance.VariantsUnlocked`
  (`:417,435`), the area list, and `CoreModule.Settings.DefaultStartingLevelSet`
  (`:361`); it is always executed (the reference code calls it unconditionally
  after `InputName`). For the corpus (`campaignName == startingLevelSet ==
  "Celeste"`, one level set) it emits a single `1,U` (`:410`).
* Corpus usage: `StartFullGameFile.tas:9` (`SelectCampaign,Celeste`), reached from
  `0 - 100%.tas:8` and `0 - 202 Berries.tas`.

## 13. Directives that are intentionally not resolved

| directive | why it contributes no frames |
| --- | --- |
| `Add` | libTAS-export only (`CT/TAS/LibTasHelper.cs:85-96`) |
| `ExportLibTAS` / `EndExportLibTAS`, `Skip`, `Marker` | libTAS movie generation only (`LibTasHelper.cs:237-283`) |
| `console`, `Set`, `Invoke`, `EvalLua`, `Assert`, `Gun`, `Mouse`, `Press`, `ExitGame`, `CompleteInfo`, `SeedRandom`, `RequireDependency`, `StartRecording`, `StopRecording`, `EnforceLegal`, `Safe`, `Unsafe`, `AnalogMode` | runtime-timing commands (`Command.cs:38`) or parse-time commands that only set flags; none of them call `AddFrames`/`ReadFile`/`ReadLines` (verified by grepping every `AddFrames(` / `ReadFile(` / `ReadLines(` call site in `CT/`) |
| `RecordCount`, `FileTime`, `ChapterTime`, `RealTime`, `Midway*`, `ActivatedLobbyWarps`, `Author:`, `FrameCount:`, `TotalRecordCount:` | metadata dummies (`MetadataCommands.cs:137-193`, `DummyCommands.cs`) |
| `***` breakpoints | fast-forward only (`InputController.cs:291-301`); none in the corpus |
| `StunPauseMode`, `StunPause, Simulate` | runtime double-update behaviour, no inputs |
| `SelectCampaign` name-entry cursor movement | needs `ActiveFont.Measure` + dialog metrics; see section 12 |
| `SelectCampaign` slot selection / `EmptyFileSlot`, `MaxSaveFileSlots` | live save-directory state; exposed as options |
| `SaveAndQuitReenter` slot | live game state; exposed as `options.saveAndQuitReenterSlot` |
| anything else starting with a letter that is not a registered command | `Command.TryParse` fails (`Command.cs:104-108`), the line is not a label and not an action line, so it contributes nothing (e.g. `TasFrameTrace,...` in the corpus' `_trace-*.tas` helpers) |

## 14. The `FileTime` header

Every corpus file may carry, usually as its first line:

```
FileTime: 1:08:45.679(242687)
```

**`(N)` is not `InputController.Inputs.Count`, and the task's expectation that
`totalFrames === 242687` for `0 - 100%.tas` therefore cannot hold.** The chain:

1. `MetadataCommands` writes the header for the main file only
   (`MetadataCommands.cs:195-220,226-241`), formatting
   `SaveData.Instance.Time - TasStartInfo.Value.FileTimeTicks`
   (`MetadataCommands.cs:72`) with `GameInfo.FormatTime`.
2. `GameInfo.FormatTime` (`CT/TAS/GameInfo.cs:643-650`) is
   `$"{TimeSpan.FromTicks(T).ShortGameplayFormat()}({T / Engine.RawDeltaTime.SecondsToTicks()})"`.
   So the number in parentheses is the tick delta divided by one frame's ticks.
3. `RawDeltaTime.SecondsToTicks()` (`CT/Utils/Extensions.cs:850-855`) deliberately
   rounds to whole milliseconds: `((long)(seconds * 1000 + 0.5)) * 10000`. With
   `RawDeltaTime = 1/60 s` (`CE/Monocle/Engine.cs:226`) that is
   `(long)17.1666 * 10000 = ` **170000 ticks**.
4. `SaveData.Time` grows in `Level.UpdateTime` by
   `TimeSpan.FromSeconds(Engine.RawDeltaTime).Ticks` (`CE/Celeste/Level.cs:1733-1734`),
   i.e. by exactly those 170000 ticks, once per frame in which that method
   actually runs. It early-returns when `InCredits || Session.Area.ID == 8 ||
   TimerStopped` (`Level.cs:1729-1732`), and `Level.Update` returns before line
   1819 - where `UpdateTime()` is called - whenever a pause `Overlay` is open
   (`Level.cs:1757-1762`), i.e. the pause menu, the journal, and the
   Save & Quit menus. No `Level` scene at all (title screen, file select,
   overworld, `LevelLoader`, `LevelExit`, credits, Epilogue area 8) means no
   `UpdateTime` either.
5. Therefore `T` is always a whole multiple of 170000, the printed time is always
   exactly `N * 17 ms`, and **`N` counts frames in which the save-file timer was
   running** - a strict lower bound on `Inputs.Count` that a `.tas` file does not
   determine.

Two independent confirmations:

* All 66 headers in the corpus satisfy `headerMillis == N * 17` to the
  millisecond (a test asserts this).
* `1SHC.tas` has no directives at all, only 153 input lines whose frame numbers
  sum to **1441**; `AddFrames` cannot drop inputs, so `Inputs.Count == 1441`,
  while its header says `1359`.

The gap is real, not a rounding artefact: for `0 - 100%.tas` the game fed
**281113** frames while the header records 242687, i.e. 38426 frames were spent
outside a timer-running level.

## 15. Validating against a real game trace

This workspace contains a recorded frame trace produced by running the real
Celeste + CelesteTAS over the corpus entry
(`.tmp/tasrun/trace-100pct.jsonl`, 281113 lines, one JSON object per fed frame,
recording the frame's `InputFrame` text, its frame count and the live
`Input.*` booleans).

Comparing that trace with this resolver, run-length encoded on both sides and
aligned, gives **exactly one divergence region** in the whole 0 - 100% TAS:

```
game runs=30880 frames=281113   my runs=30872 frames=281105
divergence regions: 1
#0 @gameFrame 203: 10 game runs (10f) vs 2 my runs (2f)  delta=8
     game: 1,F,90  1,D  1,F,180  1,O  1,F,270  1,U  1,F,0  1,O  1,F,180  1,D
     mine: 1,O     1,J
total delta: 8
```

That region is `SelectCampaign`'s BFS cursor movement (section 12) - the only
font-metric-dependent construct. Everything else - 281105 of the 281113 frames,
including all 8 `SaveAndQuitReenter` expansions, `Repeat`, `Read` ranges, the
`StunPause`/`AutoInput` injections and every input entry - matches the game
frame-for-frame.

The trace also settles the two game-state options for this corpus:

* the 8 `SaveAndQuitReenter` sites show the `slot >= 0` branch with **no**
  `1,D`/`1,F,180` selection frames, so `ActiveFileSlot == 0`. That is why the
  default `saveAndQuitReenterSlot` is `0`.
* `1A.tas` (a chapter with no state-dependent construct) matches its own trace
  exactly: 3215 frames in both.

## 16. Options

| option | default | meaning |
| --- | --- | --- |
| `saveAndQuitReenterSlot` | `0` | `ActiveFileSlot`; `-1` = no save file (see §11, §15) |
| `randomizerInstalled` | `false` | `ModUtils.IsInstalled("Randomizer")` (`SaveAndQuitReenterCommand.cs:78`) |
| `playMode` | `'Normal'` | `Celeste.Celeste.PlayMode` (`SaveAndQuitReenterCommand.cs:67`) |
| `selectCampaignEmptyFileSlot` | `-1` | `EmptyFileSlot` (`SelectCampaignCommand.cs:68-93`) |
| `selectCampaignMaxSaveFileSlots` | `3` | `MaxSaveFileSlots` (`SelectCampaignCommand.cs:39-65`) |
| `variantsUnlocked` | `false` | `Settings.Instance.VariantsUnlocked` (`SelectCampaignCommand.cs:417`) |
| `nameLetters` | English letter grid | `Dialog.Clean("name_letters")` rows |
| `fontWidths` | `null` | `ActiveFont.Measure` values; `null` disables the name-entry BFS |
| `language` | `'english'` | `Settings.Instance.Language` |
| `defaultStartingLevelSet` | `'Celeste'` | `CoreModule.Settings.DefaultStartingLevelSet` |
| `areaLevelSets` | `['Celeste']` | `AreaData.Areas` level sets, in order |
| `onWarning` | `null` | receives a message per unresolved/game-state-dependent construct |
| `selfCheck` | `true` | run `selfCheck(result)` and throw on an invariant violation |
| `ignoreAborts` | `false` | ignore `AbortTas` instead of throwing `TasAbortError` |

## 17. Frame-count table (computed vs each file's own `FileTime` header)

The resolver's `totalFrames` is `Inputs.Count`. The header's `(N)` is the
timer-frame count of §14. `delta = computed - N` is therefore the number of
frames the file spends outside a timer-running level, and is not derivable from
the `.tas` text alone. Generated by
`node src/cli.mjs <tasRoot> <entryFile> --table --quiet`:

```
file                       computed     header  computed-header
------------------------  ---------  ---------  ---------------
0 - 100%.tas                 281105     242687            38418
0 - 202 Berries.tas          461114     401427            59687
0 - All A Sides.tas          106131      89315            16816
0 - All B Sides.tas           74178      62355            11823
0 - All C Sides.tas           15858      10723             5135
0 - All Cassettes.tas        106813      89545            17268
0 - All Chapters.tas         213536     181422            32114
0 - All Hearts.tas           158318     134814            23504
0 - All Red Berries.tas      145872     125469            20403
0 - Any% Pure.tas              2104        238             1866
0 - Any%.tas                  89610      75350            14260
0 - Bny%.tas                 103859      88180            15679
0 - True Ending.tas          107986      90830            17156
1HC.tas                        4288       3920              368
1SH0.tas                       7034       6462              572
1SHC.tas                       1441       1359               82
202/1BG.tas                    3970       3807              163
202/1CG.tas                     954        875               79
202/1DG.tas                    4628       4612               16
202/1SH0G.tas                  6932       6363              569
202/2AG.tas                    5666       5146              520
202/2BG.tas                    5052       4559              493
202/2CG.tas                    1274       1153              121
202/2SHCG.tas                 10612       9671              941
202/3BG.tas                    5596       5151              445
202/3CG.tas                    1057        933              124
202/3SHCG.tas                 18606      17158             1448
202/4BG.tas                    7540       7062              478
202/4CG.tas                    1545       1445              100
202/4SHCG.tas                 12621      11581             1040
202/5AG.tas                   12630      11831              799
202/5BG.tas                    6479       5998              481
202/5CG.tas                    1006        939               67
202/5SHCG.tas                 25514      23503             2011
202/6AG.tas                   15327      13916             1411
202/6BG.tas                   12365      11320             1045
202/6CG.tas                    1365       1262              103
202/7AG.tas                   27567      25124             2443
202/7BG.tas                   18158      16672             1486
202/7SHCG.tas                 35905      32579             3326
202/8AG.tas                    7961       7510              451
202/8BG.tas                    7566       6995              571
202/8CG.tas                    1511       1378              133
202/8SCG.tas                  11136      10345              791
202/9G.tas                     6624       5343             1281
2A.tas                         5748       5185              563
2HC.tas                        7727       7056              671
2SHC.tas                      11335      10398              937
3HC.tas                       14198      13068             1130
3SH.tas                       18188      16752             1436
3SHC.tas                      18874      17405             1469
4HC.tas                        8549       7827              722
4SH0.tas                      13076      12006             1070
4SHC.tas                      13795      12692             1103
5A.tas                        11841      11014              827
5AC.tas                        3891       3635              256
5B.tas                         6638       6057              581
5HC.tas                       15015      13875             1140
5S.tas                        24519      22650             1869
5SHC.tas                      26506      24411             2095
6AC.tas                        3395       3247              148
6HC.tas                       19855      18054             1801
7B.tas                        18066      16490             1576
7HC.tas                       30154      27422             2732
7SHC.tas                      38428      34721             3707
8AC.tas                        9469       8825              644

66 file(s) with a FileTime header
```

Per-chapter files without a `FileTime` header (all resolve successfully):

```
1A 3215   1B 4058   1C 1041   1D 4553   1SH1 432   1SH2 854
2B 5144   2C 1366   2S 9500   3A 12448  3B 5687    3C 1154
4A 6878   4B 7632   4C 1638   4SH1 599  5C 1098    6A 17640
6B 12460  6C 1458   7A 25700  7C 2323   7S 34158   8A 8838
8B 7886   8C 1477   8S 10564  9 7706    9NMG 41039 9S 35723
Load1B 559  Load1C 746  Load1DG 341  LoadA 856  LoadAFromB 551
LoadAG 227  LoadANoCollects 808  LoadB 489  LoadBCheatMode 520
LoadBFromA 454  LoadBG 486  LoadC 232  LoadCCheatMode 232
LoadCoreFromSummit 792  LoadEnableCheatMode 240  LoadJournal 1451
StartFullGameFile 291  StartFullGameFileAlt 335  StartFullGameFileOld 169
0 - Prologue 1544
```

## 18. Known limitations

* `SelectCampaign`'s name-entry cursor movement (font metrics) - section 12/15.
  Everything else matches the recorded game trace exactly.
* Anything whose frame count depends on live save-directory or mod state
  (`EmptyFileSlot`, `MaxSaveFileSlots`, `VariantsUnlocked`, `ActiveFileSlot`,
  `Randomizer`) is modelled through options; the defaults are the values the
  recorded trace proves for this corpus.
* The resolver does not simulate the game, so it cannot tell you how many of the
  frames it returns were *timer* frames - that is exactly the quantity the
  `FileTime` header records, and it needs a running game.
* `AnalogMode` affects the analog stick values but not the frame counts, so only
  its event is recorded.
