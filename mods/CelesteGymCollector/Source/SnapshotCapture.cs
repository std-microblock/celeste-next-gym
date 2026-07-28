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
    private static readonly FieldInfo? playerHurtbox = typeof(Player).GetField(
        "hurtbox",
        BindingFlags.Instance | BindingFlags.NonPublic
    );
    private static readonly FieldInfo? heartGemCollected = typeof(HeartGem).GetField(
        "collected",
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
        values["playerCollider"] = ColliderGeometry(player.Collider);
        values["engineTimeRate"] = Engine.TimeRate;
        if (playerHurtbox?.GetValue(player) is Collider hurtbox) {
            values["playerHurtbox"] = ColliderGeometry(hurtbox);
        }
        if (player.Scene is Level level) {
            values["levelWind"] = new[] { level.Wind.X, level.Wind.Y };
            if (level.Entities.FindFirst<ZipMover>() is ZipMover zipMover) {
                values["zipMoverPosition"] = Simplify(zipMover.Position);
                values["zipMoverLiftSpeed"] = Simplify(zipMover.LiftSpeed);
                values["zipMoverMovementCounter"] = Simplify(platformMovementCounter?.GetValue(zipMover));
            }
            HeartGem? heartGem = null;
            foreach (Entity entity in level.Entities) {
                if (entity is HeartGem candidate) {
                    heartGem = candidate;
                    break;
                }
            }
            if (heartGem is not null) {
                values["heartGemPosition"] = Simplify(heartGem.Position);
                values["heartGemCollected"] = heartGemCollected?.GetValue(heartGem) as bool? ?? false;
                values["heartGemCollidable"] = heartGem.Collidable;
                values["heartGemVisible"] = heartGem.Visible;
            }
            Entity? reformBlock = level.Entities.FindFirst<MoveBlock>();
            reformBlock ??= level.Entities.FindFirst<BounceBlock>();
            if (reformBlock is not null) {
                values["reformBlockKind"] = reformBlock.GetType().Name;
                values["reformBlockPosition"] = Simplify(reformBlock.Position);
                values["reformBlockCollidable"] = reformBlock.Collidable;
                values["reformBlockVisible"] = reformBlock.Visible;

                Spikes? attachedSpike = level.Entities.FindAll<Spikes>()
                    .Find(spike => spike.Get<StaticMover>()?.Platform == reformBlock);
                if (attachedSpike is not null) {
                    values["reformSpikePosition"] = Simplify(attachedSpike.Position);
                    values["reformSpikeCollidable"] = attachedSpike.Collidable;
                }
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
            HoldingGlider = player.Holding?.Entity is Glider,
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
        HoldingGlider = previous.HoldingGlider,
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

    private static float[] ColliderGeometry(Collider collider) => [
        collider.Left,
        collider.Top,
        collider.Width,
        collider.Height
    ];
}
