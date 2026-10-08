using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Reflection;
using System.Text;
using Celeste;
using Microsoft.Xna.Framework;
using Monocle;
using StudioCommunication;
using TAS.InfoHUD;
using TAS.Input;
using TAS.Module;
using TAS.Utils;
using CelesteInput = Celeste.Input;

namespace TAS;

/// <summary>
/// Dumps one JSON object per executed TAS frame, containing the raw virtual-input state
/// observed by the game plus every declared instance field of the current Player.
/// Used as the ground-truth trace for frame-by-frame fidelity comparisons.
/// </summary>
public static class TasFrameTrace {
    private class Meta : ITasCommandMeta {
        public string Insert => $"TasFrameTrace{CommandInfo.Separator}[0;frame-trace.jsonl]";
        public bool HasArguments => true;
    }

    /// <summary>
    /// Captured once per executed TAS frame, from the same call site as CelesteTAS's own
    /// <c>ExportGameInfo.ExportInfo()</c> — immediately before <c>InputHelper.FeedInputs</c>.
    /// <c>CurrentFrameInTas</c> is still the frame about to run, and
    /// <c>InputController.AdvanceFrame</c> always ends with exactly one <c>CurrentFrameInTas++</c>,
    /// so the executed frame index is <c>FrameInTas + 1</c>.
    ///
    /// The press-edge fields are filled later, by <see cref="CaptureInput"/>, which
    /// <c>InputController.AdvanceFrame</c> calls immediately after
    /// <c>InputHelper.FeedInputs(Current!)</c>. A class (not a record struct) so that call can
    /// mutate the entry already queued here.
    /// </summary>
    private sealed class PendingFrame {
        public InputFrame Frame = null!;
        public int FrameInTas;

        /// <summary>
        /// `VirtualButton.Pressed` for each button, sampled before anything in the frame could
        /// consume it. `Player.BoostUpdate` calls `Input.Dash.ConsumePress()` (`Player.cs:4721-4731`),
        /// which sets Monocle's `VirtualButton.consumed` and makes `Pressed` return false for the
        /// rest of the frame (`Monocle/VirtualButton.cs:55-58`, `:153-157`). Because the row is
        /// written after the engine update, the historical `in.dashP`/`in.cdashP`/`in.jumpP`/
        /// `in.talkP` keys are post-consumption on every such frame; these are the true edges.
        /// </summary>
        public bool PressedCaptured;
        public bool JumpPressed;
        public bool DashPressed;
        public bool CrouchDashPressed;
        public bool TalkPressed;
    }

    private static StreamWriter? writer;
    // A queue rather than a single slot: `AdvanceFrame` runs at most once per `Engine.Update`, but a
    // single slot would silently drop a row if that ever changed. `SaveAndQuitReenter` and
    // `SelectCampaign` jump `CurrentFrameInTas` forward *inside* one AdvanceFrame
    // (InputController.cs:178-184), which shows up as a legitimate gap in `f` — it is not row loss.
    private static readonly Queue<PendingFrame> pending = new();
    // The entry `CaptureInput` fills. `ExportInfo` hands it over, so a frame whose press edges are
    // never captured (for example `Current` is null, or `AdvanceFrame` returns before
    // `FeedInputs`) cannot inherit the previous frame's edges.
    private static PendingFrame? captureTarget;
    private static bool exporting;
    private static string targetPath = "";
    private static long rows;
    private static long errors;
    private static int flushEvery = 512;
    private static readonly StringBuilder sb = new(8192);
    private static readonly List<FieldInfo> playerFields = new();
    private static Type? fieldsType;
    // `Celeste.Level.windController` (Level.cs:101) and the WindController's own
    // `targetSpeed`/`pattern` (WindController.cs:45-47) are all private fields, so
    // reflection is the only reader. Cached because the write path is per frame.
    private static FieldInfo? windControllerField;
    private static FieldInfo? windTargetField;
    private static FieldInfo? windPatternField;

    // "TasFrameTrace"
    // "TasFrameTrace Path"
    [TasCommand("TasFrameTrace", Aliases = ["StartTasFrameTrace"], CalcChecksum = false, MetaDataProvider = typeof(Meta))]
    private static void StartExportCommand(CommandLine commandLine, int studioLine, string filePath, int fileLine) {
        string path = commandLine.Arguments.Length > 0 && commandLine.Arguments[0].IsNotEmpty()
            ? commandLine.Arguments[0]
            : "frame-trace.jsonl";
        Begin(path);
    }

    [TasCommand("EndTasFrameTrace", Aliases = ["FinishTasFrameTrace"], CalcChecksum = false)]
    private static void FinishExportCommand(CommandLine commandLine, int studioLine, string filePath, int fileLine) {
        Finish();
    }

    [DisableRun]
    private static void Finish() {
        exporting = false;
        pending.Clear();
        captureTarget = null;
        if (writer != null) {
            try {
                writer.Flush();
                writer.Dispose();
            } catch {
                // ignored
            }

            writer = null;
        }
    }

    [Load]
    private static void Load() {
        On.Monocle.Engine.Update += EngineOnUpdate;
    }

    [Unload]
    private static void Unload() {
        On.Monocle.Engine.Update -= EngineOnUpdate;
    }

    private static void EngineOnUpdate(On.Monocle.Engine.orig_Update orig, Engine self, GameTime gameTime) {
        orig(self, gameTime);

        while (pending.Count > 0) {
            PendingFrame frame = pending.Dequeue();
            if (ReferenceEquals(captureTarget, frame)) {
                captureTarget = null;
            }

            try {
                WriteFrame(frame);
            } catch (Exception e) {
                errors++;
                if (errors <= 8) {
                    Console.Error.WriteLine($"[TasFrameTrace] frame {rows}: {e}");
                }
            }
        }
    }

    /// <summary>
    /// Called once per executed TAS frame, immediately before the inputs are fed. The write happens
    /// after the matching <c>Engine.Update</c>, the same point <c>ExportGameInfo</c> records at.
    /// </summary>
    public static void ExportInfo() {
        captureTarget = null;
        if (!exporting) {
            return;
        }

        InputController controller = Manager.Controller;
        if (controller.Current is { } currentInput) {
            var entry = new PendingFrame {
                Frame = currentInput,
                FrameInTas = controller.CurrentFrameInTas,
            };
            pending.Enqueue(entry);
            captureTarget = entry;
        }
    }

    /// <summary>
    /// Snapshots the frame's press edges into the entry <see cref="ExportInfo"/> just queued.
    ///
    /// `InputController.AdvanceFrame` calls this immediately after
    /// <c>InputHelper.FeedInputs(Current!)</c> (`InputController.cs:220`), which ends with
    /// <c>MInput.UpdateVirtualInputs()</c> (`InputHelper.cs:33`). At that instant the fed input has
    /// just recomputed every `VirtualButton` and no `Scene.Update` has run yet, so the edges are
    /// pre-consumption — unlike the post-frame reads the `in.*P` keys have always used.
    /// </summary>
    public static void CaptureInput() {
        if (!exporting || captureTarget == null) {
            return;
        }

        PendingFrame target = captureTarget;
        target.JumpPressed = CelesteInput.Jump.Pressed;
        target.DashPressed = CelesteInput.Dash.Pressed;
        target.CrouchDashPressed = CelesteInput.CrouchDash.Pressed;
        target.TalkPressed = CelesteInput.Talk.Pressed;
        target.PressedCaptured = true;
    }

    private static void Begin(string path) {
        Finish();
        targetPath = path;
        if (Path.GetDirectoryName(path) is { } dir && dir.IsNotEmpty()) {
            Directory.CreateDirectory(dir);
        }

        try {
            writer = new StreamWriter(path, false, new UTF8Encoding(false), 1 << 20) { AutoFlush = false };
        } catch (Exception e) {
            Console.Error.WriteLine($"[TasFrameTrace] cannot open '{path}': {e.Message}");
            writer = null;
            exporting = false;
            return;
        }

        exporting = true;
        rows = 0;
        errors = 0;
        Console.WriteLine($"[TasFrameTrace] writing to {path}");
    }

    private static void WriteFrame(PendingFrame record) {
        InputFrame inputFrame = record.Frame;
        sb.Clear();
        sb.Append('{');

        sb.Append("\"n\":").Append(rows.ToString(CultureInfo.InvariantCulture));
        sb.Append(",\"f\":").Append((record.FrameInTas + 1).ToString(CultureInfo.InvariantCulture));
        sb.Append(",\"fi\":").Append(Manager.Controller.CurrentFrameInInput.ToString(CultureInfo.InvariantCulture));
        sb.Append(",\"line\":").Append((inputFrame.StudioLine + 1).ToString(CultureInfo.InvariantCulture));

        sb.Append(",\"dt\":"); AppendFloat(Engine.DeltaTime);
        sb.Append(",\"rawDt\":"); AppendFloat(Engine.RawDeltaTime);
        sb.Append(",\"timeRate\":"); AppendFloat(Engine.TimeRate);

        AppendInputState(record);

        sb.Append(",\"a\":").Append(((int) inputFrame.Actions).ToString(CultureInfo.InvariantCulture));
        sb.Append(",\"aStr\":");
        AppendString(inputFrame.ToString());
        sb.Append(",\"nf\":").Append(inputFrame.Frames.ToString(CultureInfo.InvariantCulture));
        sb.Append(",\"rep\":[").Append(inputFrame.RepeatIndex.ToString(CultureInfo.InvariantCulture)).Append(',')
            .Append(inputFrame.RepeatCount.ToString(CultureInfo.InvariantCulture)).Append(']');

        if (Engine.Scene is Level level) {
            sb.Append(",\"scene\":\"Level\"");
            AppendStringField("sid", level.Session.Area.GetSID());
            sb.Append(",\"area\":").Append(level.Session.Area.ID.ToString(CultureInfo.InvariantCulture));
            sb.Append(",\"mode\":").Append(((int) level.Session.Area.Mode).ToString(CultureInfo.InvariantCulture));
            AppendStringField("room", level.Session.Level);
            sb.Append(",\"deaths\":").Append(level.Session.Deaths.ToString(CultureInfo.InvariantCulture));
            sb.Append(",\"levelTime\":").Append(level.Session.Time.ToString(CultureInfo.InvariantCulture));

            Player? player = level.Tracker.GetEntity<Player>();
            if (player != null) {
                sb.Append(",\"state\":");
                AppendString(PlayerStates.GetCurrentStateName(player));
                AppendPlayerFields(player);
            }

            // Append-only tail. Every key added here is new and lands *after* all
            // pre-existing keys of the `Level` branch, so no earlier key moves and
            // no earlier key changes meaning; readers of the older traces are
            // unaffected because they ignore unknown top-level keys.
            AppendLevelTail(level, player);
        } else if (Engine.Scene is Overworld overworld) {
            sb.Append(",\"scene\":");
            AppendString($"Overworld {(overworld.Current ?? overworld.Next).GetType().Name}");
        } else if (Engine.Scene == null) {
            sb.Append(",\"scene\":\"null\"");
        } else {
            sb.Append(",\"scene\":");
            AppendString(Engine.Scene.GetType().Name);
        }

        sb.Append('}');
        writer!.Write(sb);
        writer.Write('\n');
        rows++;

        if (rows % flushEvery == 0) {
            writer.Flush();
        }
    }

    private static void AppendInputState(PendingFrame record) {
        sb.Append(",\"in\":{");
        sb.Append("\"mx\":").Append(CelesteInput.MoveX.Value.ToString(CultureInfo.InvariantCulture));
        sb.Append(",\"my\":").Append(CelesteInput.MoveY.Value.ToString(CultureInfo.InvariantCulture));
        sb.Append(",\"gly\":").Append(CelesteInput.GliderMoveY.Value.ToString(CultureInfo.InvariantCulture));
        AppendBoolField("jump", CelesteInput.Jump.Check);
        AppendBoolField("jumpP", CelesteInput.Jump.Pressed);
        AppendBoolField("dash", CelesteInput.Dash.Check);
        AppendBoolField("dashP", CelesteInput.Dash.Pressed);
        AppendBoolField("cdash", CelesteInput.CrouchDash.Check);
        AppendBoolField("cdashP", CelesteInput.CrouchDash.Pressed);
        AppendBoolField("grab", CelesteInput.Grab.Check);
        AppendBoolField("talk", CelesteInput.Talk.Check);
        AppendBoolField("talkP", CelesteInput.Talk.Pressed);
        // Append-only: the same edges sampled before any consumer ran, i.e. what
        // `InputHelper.FeedInputs` produced and what the frame's state callbacks saw before they
        // could call `VirtualButton.ConsumePress()`. `in.*P` above stay exactly as they were.
        AppendBoolField("jumpP0", record.PressedCaptured ? record.JumpPressed : CelesteInput.Jump.Pressed);
        AppendBoolField("dashP0", record.PressedCaptured ? record.DashPressed : CelesteInput.Dash.Pressed);
        AppendBoolField("cdashP0", record.PressedCaptured ? record.CrouchDashPressed : CelesteInput.CrouchDash.Pressed);
        AppendBoolField("talkP0", record.PressedCaptured ? record.TalkPressed : CelesteInput.Talk.Pressed);
        sb.Append(",\"aim\":");
        AppendVector(CelesteInput.Aim.Value);
        sb.Append(",\"feather\":");
        AppendVector(CelesteInput.Feather.Value);
        sb.Append('}');
    }

    /// <summary>
    /// Scene-, engine-, session- and computed-property state that lives outside the
    /// player's declared field base chain, so <see cref="AppendPlayerFields"/> cannot
    /// see it. Emitted at the end of the `Level` branch only.
    /// </summary>
    private static void AppendLevelTail(Level level, Player? player) {
        // Celeste.Level.Wind (Level.cs:149). WindController.Update rewrites it every
        // frame with `Calc.Approach(level.Wind, targetSpeed, 1000f * Engine.DeltaTime)`
        // (WindController.cs:194) and then displaces every WindMover component by
        // `level.Wind * 0.1f * Engine.DeltaTime` (WindController.cs:201). The Player
        // owns one of those components (Player.cs:1180), so this value is a physical
        // per-frame displacement the player snapshot cannot otherwise recover.
        sb.Append(",\"wind\":");
        AppendVector(level.Wind);

        // Celeste.Level.WindSine / WindSineTimer (Level.cs:151-153). Visual only, but
        // exported so the ramp can be followed without guessing.
        sb.Append(",\"windSine\":");
        AppendFloat(level.WindSine);
        sb.Append(",\"windSineTimer\":");
        AppendFloat(level.WindSineTimer);

        // Celeste.Level.Transitioning => `transition != null` (Level.cs:221). While it
        // is true `Level.Update` runs the transition coroutine and `Player.Update`
        // never runs, which is why the head of every room segment is a stale window.
        sb.Append(",\"transitioning\":").Append(level.Transitioning ? "true" : "false");

        // Monocle.Engine.FreezeTimer (Monocle/Engine.cs:28). While positive,
        // `Engine.Update` only decrements it and skips `Scene.Update` entirely
        // (Engine.cs:266-269), so the frame is a stale copy of the previous one.
        sb.Append(",\"freezeTimer\":");
        AppendFloat(Engine.FreezeTimer);

        AppendWindController(level);

        // Celeste.Level.InSpace = levelData.Space (Level.cs:449). A per-room map property read by
        // Player.cs:3703-3706, 3718-3722 and 3778-3781, which scale the boost/feather/climb speed
        // components by 0.6f. `crates/celeste-physics/src/map.rs` does not decode the `.bin` level
        // element's `space` attribute, so the trace is the only ground-truth source for it.
        sb.Append(",\"inSpace\":").Append(level.InSpace ? "true" : "false");

        // Celeste.Session.CoreMode (Session.cs:22-27, 111). Read by the Core's ice factor
        // (Player.cs:3681-3684) and by the CoreModeListener entities; it is session state, not a
        // `Player` field. `Session.CoreModes` is `None = 0, Hot = 1, Cold = 2`.
        sb.Append(",\"coreMode\":")
            .Append(((int) level.Session.CoreMode).ToString(CultureInfo.InvariantCulture));

        // Celeste.Session.Inventory is a `PlayerInventory` struct (Session.cs:35,
        // PlayerInventory.cs:6-36). `Session.Level` and `Session.Deaths` already have
        // their own keys above (`room`, `deaths`).
        AppendInventory(level);

        if (player == null) {
            return;
        }

        // Player.Ducking is a computed property over `Entity.Collider`
        // (Player.cs:1005-1028), not a declared field, so `p` never carries it and the
        // anchors can hold the 11 px normal hitbox where the game has the 6 px
        // duck hitbox.
        sb.Append(",\"ducking\":").Append(player.Ducking ? "true" : "false");

        // `Monocle.Entity.Collider` (Monocle/Entity.cs:73), read through
        // `Collider.AbsoluteLeft/AbsoluteTop/Width/Height` (Monocle/Collider.cs:229,
        // 205, 12, 14) so the active hitbox is exported exactly, in world pixels.
        sb.Append(",\"collider\":");
        AppendCollider(player.Collider);
    }

    /// <summary>
    /// `Celeste.Level.windController` (Level.cs:101) with its private `targetSpeed`
    /// and `pattern` fields (WindController.cs:45-47). The target is what
    /// `WindController.Update` ramps `Level.Wind` toward, so a replayed room segment
    /// cannot reconstruct it from `Level.Wind` alone. Both keys are `null` when the
    /// level has no WindController or the fields cannot be read.
    /// </summary>
    private static void AppendWindController(Level level) {
        object? controller = null;
        try {
            windControllerField ??= typeof(Level).GetField(
                "windController",
                BindingFlags.Instance | BindingFlags.NonPublic);
            controller = windControllerField?.GetValue(level);
        } catch {
            controller = null;
        }

        Vector2? target = null;
        long? pattern = null;
        if (controller != null) {
            Type type = controller.GetType();
            try {
                windTargetField ??= type.GetField("targetSpeed", BindingFlags.Instance | BindingFlags.NonPublic);
                windPatternField ??= type.GetField("pattern", BindingFlags.Instance | BindingFlags.NonPublic);
                if (windTargetField?.GetValue(controller) is Vector2 value) {
                    target = value;
                }

                if (windPatternField?.GetValue(controller) is Enum enumValue) {
                    pattern = Convert.ToInt64(enumValue, CultureInfo.InvariantCulture);
                }
            } catch {
                // Leave both null: an unreadable target must not corrupt the row.
            }
        }

        sb.Append(",\"windTarget\":");
        if (target.HasValue) {
            AppendVector(target.Value);
        } else {
            sb.Append("null");
        }

        sb.Append(",\"windPattern\":");
        sb.Append(pattern?.ToString(CultureInfo.InvariantCulture) ?? "null");
    }

    private static void AppendInventory(Level level) {
        PlayerInventory inventory = level.Session.Inventory;
        sb.Append(",\"inventory\":{");
        sb.Append("\"Dashes\":").Append(inventory.Dashes.ToString(CultureInfo.InvariantCulture));
        AppendBoolField("DreamDash", inventory.DreamDash);
        AppendBoolField("Backpack", inventory.Backpack);
        AppendBoolField("NoRefills", inventory.NoRefills);
        sb.Append('}');
    }

    private static void AppendCollider(Collider? collider) {
        if (collider == null) {
            sb.Append("null");
            return;
        }

        sb.Append('[');
        AppendFloat(collider.AbsoluteLeft);
        sb.Append(',');
        AppendFloat(collider.AbsoluteTop);
        sb.Append(',');
        AppendFloat(collider.Width);
        sb.Append(',');
        AppendFloat(collider.Height);
        sb.Append(']');
    }

    private static void AppendPlayerFields(Player player) {
        if (fieldsType != typeof(Player)) {
            fieldsType = typeof(Player);
            playerFields.Clear();
            // Walk the whole base chain up to (but excluding) object, not just Player's own fields.
            // `Position` lives on Entity and `movementCounter` on Platform; without them a segment
            // restored mid-motion starts with the wrong sub-pixel remainder and drifts by a pixel.
            // A name declared more than once keeps the most-derived declaration.
            const BindingFlags flags = BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.DeclaredOnly;
            var seen = new HashSet<string>(StringComparer.Ordinal);
            for (Type? type = fieldsType; type != null && type != typeof(object); type = type.BaseType) {
                foreach (FieldInfo field in type.GetFields(flags)) {
                    if (seen.Add(field.Name)) {
                        playerFields.Add(field);
                    }
                }
            }

            playerFields.Sort(static (a, b) => string.CompareOrdinal(a.Name, b.Name));
        }

        sb.Append(",\"p\":{");
        bool first = true;
        foreach (FieldInfo field in playerFields) {
            object? value;
            try {
                value = field.GetValue(player);
            } catch {
                continue;
            }

            if (value == null) {
                continue;
            }

            int mark = sb.Length;
            if (!first) {
                sb.Append(',');
            }

            AppendString(field.Name);
            sb.Append(':');
            if (!AppendValue(value)) {
                sb.Length = mark;
                continue;
            }

            first = false;
        }

        sb.Append('}');
    }

    private static bool AppendValue(object value) {
        switch (value) {
            case bool b:
                sb.Append(b ? "true" : "false");
                return true;
            case float f:
                AppendFloat(f);
                return true;
            case double d:
                AppendFloat(d);
                return true;
            case Vector2 v:
                AppendVector(v);
                return true;
            case string s:
                AppendString(s);
                return true;
            case Enum e:
                sb.Append(Convert.ToInt64(e, CultureInfo.InvariantCulture).ToString(CultureInfo.InvariantCulture));
                return true;
            case byte or sbyte or short or ushort or int or uint or long or ulong:
                sb.Append(Convert.ToString(value, CultureInfo.InvariantCulture));
                return true;
            default:
                return false;
        }
    }

    private static void AppendVector(Vector2 v) {
        sb.Append('[');
        AppendFloat(v.X);
        sb.Append(',');
        AppendFloat(v.Y);
        sb.Append(']');
    }

    private static void AppendFloat(double value) {
        if (double.IsNaN(value) || double.IsInfinity(value)) {
            sb.Append("null");
            return;
        }

        sb.Append(value.ToString("R", CultureInfo.InvariantCulture));
    }

    private static void AppendBoolField(string name, bool value) {
        sb.Append(",\"");
        sb.Append(name);
        sb.Append("\":");
        sb.Append(value ? "true" : "false");
    }

    private static void AppendStringField(string name, string value) {
        sb.Append(",\"");
        sb.Append(name);
        sb.Append("\":");
        AppendString(value);
    }

    private static void AppendString(string value) {
        sb.Append('"');
        foreach (char c in value) {
            switch (c) {
                case '"': sb.Append("\\\""); break;
                case '\\': sb.Append("\\\\"); break;
                case '\n': sb.Append("\\n"); break;
                case '\r': sb.Append("\\r"); break;
                case '\t': sb.Append("\\t"); break;
                default:
                    if (c < ' ') {
                        sb.Append("\\u").Append(((int) c).ToString("x4", CultureInfo.InvariantCulture));
                    } else {
                        sb.Append(c);
                    }

                    break;
            }
        }

        sb.Append('"');
    }
}
