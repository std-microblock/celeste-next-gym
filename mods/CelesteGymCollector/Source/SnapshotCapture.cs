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
    private static readonly FieldInfo? lookoutInteracting = typeof(Lookout).GetField(
        "interacting",
        BindingFlags.Instance | BindingFlags.NonPublic
    );
    private static readonly FieldInfo? lookoutNode = typeof(Lookout).GetField(
        "node",
        BindingFlags.Instance | BindingFlags.NonPublic
    );
    private static readonly FieldInfo? lookoutNodePercent = typeof(Lookout).GetField(
        "nodePercent",
        BindingFlags.Instance | BindingFlags.NonPublic
    );
    // `Lookout.Removed` restores StNormal but intentionally does not call
    // StopInteracting. The entity is gone by the next PlayerFrame, so retain
    // this source-side observation for a transition trace to prove the split
    // lifecycle without treating a missing Lookout as `interacting = false`.
    private static bool lookoutRemovalObserved;
    private static bool lookoutRemovedWhileInteracting;
    private static int? lookoutRemovalPlayerState;

    public static void ResetLookoutLifecycleObservation() {
        lookoutRemovalObserved = false;
        lookoutRemovedWhileInteracting = false;
        lookoutRemovalPlayerState = null;
    }

    public static bool IsLookoutInteracting(Lookout lookout) =>
        lookoutInteracting?.GetValue(lookout) as bool? ?? false;

    public static void ObserveLookoutRemoved(bool wasInteracting, Scene scene) {
        lookoutRemovalObserved = true;
        lookoutRemovedWhileInteracting = wasInteracting;
        lookoutRemovalPlayerState = scene.Tracker.GetEntity<Player>()?.StateMachine.State;
    }

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
        values["engineDeltaTime"] = Engine.DeltaTime;
        if (playerHurtbox?.GetValue(player) is Collider hurtbox) {
            values["playerHurtbox"] = ColliderGeometry(hurtbox);
        }
        if (player.Scene is Level level) {
            values["levelWind"] = new[] { level.Wind.X, level.Wind.Y };
            values["levelCamera"] = new[] { level.Camera.Position.X, level.Camera.Position.Y };
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
            Lookout? lookout = level.Entities.FindFirst<Lookout>();
            values["lookoutRemovalObserved"] = lookoutRemovalObserved;
            values["lookoutRemovedWhileInteracting"] = lookoutRemovedWhileInteracting;
            if (lookoutRemovalPlayerState is int removedPlayerState) {
                values["lookoutRemovalPlayerState"] = removedPlayerState;
            }
            if (lookout is not null) {
                values["lookoutPosition"] = Simplify(lookout.Position);
                values["lookoutInteracting"] = lookoutInteracting?.GetValue(lookout) as bool? ?? false;
                values["lookoutNode"] = lookoutNode?.GetValue(lookout) as int? ?? 0;
                values["lookoutNodePercent"] = lookoutNodePercent?.GetValue(lookout) as float? ?? 0f;
            }
            CrystalStaticSpinner? spinner = level.Entities.FindFirst<CrystalStaticSpinner>();
            if (spinner is not null) {
                values["crystalSpinnerPosition"] = Simplify(spinner.Position);
                values["crystalSpinnerVisible"] = spinner.Visible;
                values["crystalSpinnerCollidable"] = spinner.Collidable;
            }
            Booster? booster = level.Entities.FindFirst<Booster>();
            if (booster is not null) {
                values["boosterBoostingPlayer"] = booster.BoostingPlayer;
            }
            List<Dictionary<string, object?>> cassetteBlocks = [];
            foreach (Entity entity in level.Entities) {
                if (entity is CassetteBlock cassette) {
                    cassetteBlocks.Add(new Dictionary<string, object?> {
                        ["index"] = cassette.Index,
                        ["position"] = Simplify(cassette.Position),
                        ["activated"] = cassette.Activated,
                        ["collidable"] = cassette.Collidable
                    });
                }
            }
            if (cassetteBlocks.Count > 0) values["cassetteBlocks"] = cassetteBlocks;
            TheoCrystal? theoCrystal = level.Entities.FindFirst<TheoCrystal>();
            if (theoCrystal is not null) {
                values["theoPosition"] = Simplify(theoCrystal.Position);
                values["theoCollidable"] = theoCrystal.Collidable;
            }
            List<float[]> bumperPositions = [];
            List<float> bumperSineCounters = [];
            foreach (Bumper bumper in level.Entities.FindAll<Bumper>()) {
                bumperPositions.Add(Simplify(bumper.Position) as float[] ?? [bumper.Position.X, bumper.Position.Y]);
                // SineWave.Counter is the source randomized phase. Persist it
                // with Position so a replay starts from this exact Bumper
                // Circle(12) location instead of inventing a random seed.
                bumperSineCounters.Add(bumper.Get<SineWave>()?.Counter ?? 0f);
            }
            if (bumperPositions.Count > 0) {
                values["bumperPositions"] = bumperPositions;
                values["bumperSineCounters"] = bumperSineCounters;
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

    internal static float[] ColliderGeometry(Collider collider) => [
        collider.Left,
        collider.Top,
        collider.Width,
        collider.Height
    ];
}
