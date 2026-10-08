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
    /// </summary>
    private readonly record struct PendingFrame(InputFrame Frame, int FrameInTas);

    private static StreamWriter? writer;
    // A queue rather than a single slot: `AdvanceFrame` runs at most once per `Engine.Update`, but a
    // single slot would silently drop a row if that ever changed. `SaveAndQuitReenter` and
    // `SelectCampaign` jump `CurrentFrameInTas` forward *inside* one AdvanceFrame
    // (InputController.cs:178-184), which shows up as a legitimate gap in `f` — it is not row loss.
    private static readonly Queue<PendingFrame> pending = new();
    private static bool exporting;
    private static string targetPath = "";
    private static long rows;
    private static long errors;
    private static int flushEvery = 512;
    private static readonly StringBuilder sb = new(8192);
    private static readonly List<FieldInfo> playerFields = new();
    private static Type? fieldsType;

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
        if (!exporting) {
            return;
        }

        InputController controller = Manager.Controller;
        if (controller.Current is { } currentInput) {
            pending.Enqueue(new PendingFrame(currentInput, controller.CurrentFrameInTas));
        }
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

        AppendInputState();

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

    private static void AppendInputState() {
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
        sb.Append(",\"aim\":");
        AppendVector(CelesteInput.Aim.Value);
        sb.Append(",\"feather\":");
        AppendVector(CelesteInput.Feather.Value);
        sb.Append('}');
    }

    private static void AppendPlayerFields(Player player) {
        if (fieldsType != typeof(Player)) {
            fieldsType = typeof(Player);
            playerFields.Clear();
            const BindingFlags flags = BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.DeclaredOnly;
            playerFields.AddRange(fieldsType.GetFields(flags));
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
