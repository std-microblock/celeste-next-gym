using System.Reflection;
using Microsoft.Xna.Framework;
using Monocle;

namespace Celeste.Mod.CelesteGymCollector;

internal static class SnapshotCapture {
    private static readonly FieldInfo[] fields = GetFields();
    private static readonly FieldInfo? platformMovementCounter = typeof(Platform).GetField(
        "movementCounter",
        BindingFlags.Instance | BindingFlags.NonPublic
    );

    public static PlayerFrame Capture(Player player, int frame) {
        Dictionary<string, object?> values = [];
        foreach (FieldInfo field in fields) {
            try {
                object? value = field.GetValue(player);
                object? serialized = Simplify(value);
                if (serialized is not null) values[field.Name] = serialized;
            } catch { }
        }
        if (player.Scene is Level level) {
            values["levelWind"] = new[] { level.Wind.X, level.Wind.Y };
            if (level.Entities.FindFirst<ZipMover>() is ZipMover zipMover) {
                values["zipMoverPosition"] = Simplify(zipMover.Position);
                values["zipMoverLiftSpeed"] = Simplify(zipMover.LiftSpeed);
                values["zipMoverMovementCounter"] = Simplify(platformMovementCounter?.GetValue(zipMover));
            }
        }

        return new PlayerFrame {
            Frame = frame,
            Pos = [player.Position.X, player.Position.Y],
            Speed = [player.Speed.X, player.Speed.Y],
            State = player.StateMachine.State,
            Facing = (int) player.Facing,
            Dashes = player.Dashes,
            Stamina = player.Stamina,
            OnGround = player.OnGround(),
            Ducking = player.Ducking,
            CanDreamDash = (player.Scene as Level)?.Session.Inventory.DreamDash ?? false,
            HoldingTheo = player.Holding?.Entity is TheoCrystal,
            Dead = player.Dead,
            FreezeTimer = Engine.FreezeTimer,
            Fields = values
        };
    }

    public static PlayerFrame CaptureMissing(PlayerFrame previous, int frame) => new() {
        Frame = frame,
        Pos = [.. previous.Pos],
        Speed = [0, 0],
        State = previous.State,
        Facing = previous.Facing,
        Dashes = previous.Dashes,
        Stamina = previous.Stamina,
        OnGround = false,
        Ducking = previous.Ducking,
        CanDreamDash = previous.CanDreamDash,
        HoldingTheo = previous.HoldingTheo,
        Dead = true,
        FreezeTimer = Engine.FreezeTimer,
        Fields = new Dictionary<string, object?>(previous.Fields) { ["collectorPlayerMissing"] = true }
    };

    private static FieldInfo[] GetFields() {
        List<FieldInfo> result = [];
        for (Type? type = typeof(Player); type is not null && type != typeof(object); type = type.BaseType) {
            result.AddRange(type.GetFields(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.DeclaredOnly));
        }
        return [.. result];
    }

    private static object? Simplify(object? value) => value switch {
        null => null,
        bool or byte or sbyte or short or ushort or int or uint or long or ulong or float or double or string => value,
        Enum e => Convert.ToInt32(e),
        Vector2 vector => new[] { vector.X, vector.Y },
        _ => null
    };
}
