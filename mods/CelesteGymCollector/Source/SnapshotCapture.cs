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
    private static readonly FieldInfo? bounceBlockState = typeof(BounceBlock).GetField(
        "state",
        BindingFlags.Instance | BindingFlags.NonPublic
    );
    private static readonly FieldInfo? bounceBlockRespawnTimer = typeof(BounceBlock).GetField(
        "respawnTimer",
        BindingFlags.Instance | BindingFlags.NonPublic
    );
    private static readonly FieldInfo? bounceBlockReformed = typeof(BounceBlock).GetField(
        "reformed",
        BindingFlags.Instance | BindingFlags.NonPublic
    );
    private static readonly FieldInfo? bounceBlockStartPosition = typeof(BounceBlock).GetField(
        "startPos",
        BindingFlags.Instance | BindingFlags.NonPublic
    );
    private static readonly Dictionary<BounceBlock, BounceBlockReformProbe> bounceBlockReformProbes = [];
    // This is a diagnostic lattice around BounceBlock.startPos, not a
    // substitute respawn rule. Entries are sorted by squared distance, then
    // Y/X, so the first clear entry is the nearest legal candidate among the
    // documented -8/-1/0/+1/+8 offsets.
    private static readonly Vector2[] bounceBlockResetScanOffsets = BuildBounceBlockResetScanOffsets();
    // `Lookout.Removed` restores StNormal but intentionally does not call
    // StopInteracting. The entity is gone by the next PlayerFrame, so retain
    // this source-side observation for a transition trace to prove the split
    // lifecycle without treating a missing Lookout as `interacting = false`.
    private static bool lookoutRemovalObserved;
    private static bool lookoutRemovedWhileInteracting;
    private static int? lookoutRemovalPlayerState;

    private sealed class BounceBlockReformProbe {
        public Vector2 PreviousPosition;
        public Vector2? TargetPosition;
        public float RespawnTimer;
        public bool ActorBlocked;
        public bool? SolidBlocked;
        public string Result = "probe-error";
        public List<Dictionary<string, object?>> Actors = [];
        public List<Dictionary<string, object?>> Solids = [];
        public List<BounceBlockResetCollisionCandidate> CollisionScan = [];
        public Vector2? NearestLegalTarget;
        public Vector2? NearestLegalOffset;
        public int? PostState;
        public bool? PostCollidable;
        public Vector2? PostPosition;
        public string? Error;
    }

    private sealed class BounceBlockResetCollisionCandidate {
        public Vector2 Offset;
        public Vector2 Target;
        public bool ActorBlocked;
        public bool SolidBlocked;
        public List<Dictionary<string, object?>> Actors = [];
        public List<Dictionary<string, object?>> Solids = [];
    }

    public static void ResetLookoutLifecycleObservation() {
        lookoutRemovalObserved = false;
        lookoutRemovedWhileInteracting = false;
        lookoutRemovalPlayerState = null;
    }

    // The collector calls this before each Engine update. A successful
    // BounceBlock reform changes state during that update, so retaining a
    // previous frame's probe would falsely attribute an old BlockedCheck to a
    // later Waiting frame.
    public static void BeginEngineFrame() => bounceBlockReformProbes.Clear();

    public static void ObserveBounceBlockReformProbe(BounceBlock bounceBlock) {
        object? rawState;
        object? rawTimer;
        try {
            rawState = bounceBlockState?.GetValue(bounceBlock);
            rawTimer = bounceBlockRespawnTimer?.GetValue(bounceBlock);
        } catch {
            return;
        }
        if (rawState is not Enum state || Convert.ToInt32(state) != 4 || rawTimer is not float timer || timer > 0f) {
            return;
        }

        BounceBlockReformProbe probe = new() {
            PreviousPosition = bounceBlock.Position,
            RespawnTimer = timer,
        };
        bounceBlockReformProbes[bounceBlock] = probe;
        try {
            if (bounceBlockStartPosition?.GetValue(bounceBlock) is not Vector2 target) {
                probe.Error = "start-position-unavailable";
                return;
            }
            probe.TargetPosition = target;
            // This is source-equivalent to BounceBlock.Update's next branch:
            // it calls CollideCheck<Actor>() at startPos, then only checks
            // Solid when no Actor blocks. The position overload preserves the
            // live broken-block position while reporting the actual colliders
            // the source check would see.
            probe.CollisionScan = ScanBounceBlockResetCandidates(bounceBlock, target);
            foreach (BounceBlockResetCollisionCandidate candidate in probe.CollisionScan) {
                if (!candidate.ActorBlocked && !candidate.SolidBlocked) {
                    probe.NearestLegalTarget = candidate.Target;
                    probe.NearestLegalOffset = candidate.Offset;
                    break;
                }
            }
            if (probe.CollisionScan.Count == 0) {
                probe.Error = "collision-scan-unavailable";
                return;
            }
            BounceBlockResetCollisionCandidate sourceCandidate = probe.CollisionScan[0];
            probe.Actors = sourceCandidate.Actors;
            probe.ActorBlocked = sourceCandidate.ActorBlocked;
            if (probe.ActorBlocked) {
                probe.Result = "actor";
                return;
            }

            probe.Solids = sourceCandidate.Solids;
            probe.SolidBlocked = sourceCandidate.SolidBlocked;
            probe.Result = probe.SolidBlocked == true ? "solid" : "clear";
        } catch (Exception error) {
            // Telemetry must never take the game down. Keep the source-side
            // failure visible on the frame instead of allowing it to become a
            // runner backend timeout.
            probe.Error = error.GetType().Name;
        }
    }

    public static void ObserveBounceBlockReformResult(BounceBlock bounceBlock) {
        if (!bounceBlockReformProbes.TryGetValue(bounceBlock, out BounceBlockReformProbe? probe)) return;
        try {
            object? state = bounceBlockState?.GetValue(bounceBlock);
            probe.PostState = state is Enum value ? Convert.ToInt32(value) : null;
            probe.PostCollidable = bounceBlock.Collidable;
            probe.PostPosition = bounceBlock.Position;
        } catch (Exception error) {
            probe.Error ??= error.GetType().Name;
        }
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
        Collider? hurtbox = playerHurtbox?.GetValue(player) as Collider;
        if (hurtbox is not null) {
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
                // CED needs to distinguish three otherwise identical-looking
                // frames: the Broken respawn countdown, body-only reform, and
                // the later StaticMover alarm callback. These fields are
                // intentionally limited to BounceBlock, whose private state
                // and respawnTimer own that ordering.
                if (reformBlock is BounceBlock bounceBlock) {
                    values["reformBlockColliderPresent"] = bounceBlock.Collider is not null;
                    if (bounceBlock.Collider is Collider reformBlockCollider) {
                        values["reformBlockCollider"] = ColliderGeometry(reformBlockCollider);
                        values["reformBlockWorldBounds"] = ColliderWorldGeometry(reformBlockCollider, bounceBlock);
                        values["reformPlayerColliderWorldBounds"] = ColliderWorldGeometry(player.Collider, player);
                        if (hurtbox is not null) {
                            values["reformPlayerHurtboxWorldBounds"] = ColliderWorldGeometry(hurtbox, player);
                        }
                        // This is an observable BlockedCheck proxy, not an
                        // attempt to reproduce BounceBlock's private check: it
                        // records whether the live Player body/hurtbox intersects
                        // the source body, and whether the player is geometrically
                        // grounded on its top face this frame.
                        values["reformBlockPlayerColliderOverlaps"] = WorldCollidersOverlap(
                            player.Collider,
                            player,
                            reformBlockCollider,
                            bounceBlock
                        );
                        values["reformBlockPlayerHurtboxOverlaps"] = hurtbox is not null
                            && WorldCollidersOverlap(hurtbox, player, reformBlockCollider, bounceBlock);
                        values["reformBlockPlayerGroundedOnSource"] = PlayerGroundedOnCollider(
                            player,
                            reformBlockCollider,
                            bounceBlock
                        );
                    }
                    object? state = bounceBlockState?.GetValue(bounceBlock);
                    values["reformBounceBlockState"] = Simplify(state);
                    values["reformBounceBlockStateName"] = state?.ToString();
                    values["reformBounceBlockRespawnTimer"] = Simplify(
                        bounceBlockRespawnTimer?.GetValue(bounceBlock)
                    );
                    values["reformBounceBlockReformed"] = bounceBlockReformed?.GetValue(bounceBlock) as bool? ?? false;
                    AppendBounceBlockReformProbe(values, bounceBlock);

                    Alarm? staticMoverAlarm = bounceBlock.Get<Alarm>();
                    values["reformStaticMoverAlarmPresent"] = staticMoverAlarm is not null;
                    if (staticMoverAlarm is not null) {
                        values["reformStaticMoverAlarmActive"] = staticMoverAlarm.Active;
                        values["reformStaticMoverAlarmDuration"] = staticMoverAlarm.Duration;
                        values["reformStaticMoverAlarmTimeLeft"] = staticMoverAlarm.TimeLeft;
                    }

                    Spikes? attachedSpike = level.Entities.FindAll<Spikes>()
                        .Find(spike => spike.Get<StaticMover>()?.Platform == bounceBlock);
                    if (attachedSpike is not null) {
                        StaticMover? staticMover = attachedSpike.Get<StaticMover>();
                        values["reformSpikePosition"] = Simplify(attachedSpike.Position);
                        values["reformSpikeActive"] = attachedSpike.Active;
                        values["reformSpikeCollidable"] = attachedSpike.Collidable;
                        values["reformSpikeColliderPresent"] = attachedSpike.Collider is not null;
                        values["reformSpikeStaticMoverEnabled"] = staticMover?.Active ?? false;
                    }
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

    // Player.hurtbox is a standalone Hitbox, so it has no Collider.Entity.
    // Capture owns both sides of this CED comparison and supplies the source
    // entity explicitly instead of inferring it from the collider.
    private static float[] ColliderWorldGeometry(Collider collider, Entity owner) => [
        WorldLeft(collider, owner),
        WorldTop(collider, owner),
        collider.Width,
        collider.Height
    ];

    private static float WorldLeft(Collider collider, Entity owner) => owner.Position.X + collider.Left;

    private static float WorldTop(Collider collider, Entity owner) => owner.Position.Y + collider.Top;

    private static float WorldRight(Collider collider, Entity owner) => WorldLeft(collider, owner) + collider.Width;

    private static float WorldBottom(Collider collider, Entity owner) => WorldTop(collider, owner) + collider.Height;

    private static bool WorldCollidersOverlap(
        Collider first,
        Entity firstOwner,
        Collider second,
        Entity secondOwner
    ) =>
        WorldLeft(first, firstOwner) < WorldRight(second, secondOwner)
        && WorldRight(first, firstOwner) > WorldLeft(second, secondOwner)
        && WorldTop(first, firstOwner) < WorldBottom(second, secondOwner)
        && WorldBottom(first, firstOwner) > WorldTop(second, secondOwner);

    private static bool PlayerGroundedOnCollider(Player player, Collider source, Entity sourceOwner) =>
        player.OnGround()
        && WorldLeft(player.Collider, player) < WorldRight(source, sourceOwner)
        && WorldRight(player.Collider, player) > WorldLeft(source, sourceOwner)
        && MathF.Abs(WorldBottom(player.Collider, player) - WorldTop(source, sourceOwner)) <= 1f;

    private static void AppendBounceBlockReformProbe(
        Dictionary<string, object?> values,
        BounceBlock bounceBlock
    ) {
        bool observed = bounceBlockReformProbes.TryGetValue(bounceBlock, out BounceBlockReformProbe? probe);
        values["reformResetBlockedCheckObserved"] = observed;
        if (!observed || probe is null) return;

        values["reformResetPreviousPosition"] = Simplify(probe.PreviousPosition);
        values["reformResetTargetPosition"] = probe.TargetPosition is Vector2 target ? Simplify(target) : null;
        values["reformResetRespawnTimer"] = probe.RespawnTimer;
        values["reformResetActorBlocked"] = probe.ActorBlocked;
        values["reformResetSolidBlocked"] = probe.SolidBlocked;
        values["reformResetBlockedCheckResult"] = probe.Result;
        values["reformResetBlockingActors"] = probe.Actors;
        values["reformResetBlockingSolids"] = probe.Solids;
        values["reformResetCollisionScanTarget"] = probe.TargetPosition is Vector2 scanTarget
            ? Simplify(scanTarget)
            : null;
        values["reformResetCollisionScan"] = DescribeBounceBlockResetCollisionScan(probe.CollisionScan);
        values["reformResetNearestLegalTarget"] = probe.NearestLegalTarget is Vector2 legalTarget
            ? Simplify(legalTarget)
            : null;
        values["reformResetNearestLegalOffset"] = probe.NearestLegalOffset is Vector2 legalOffset
            ? Simplify(legalOffset)
            : null;
        if (probe.PostState is int state) values["reformResetPostState"] = state;
        if (probe.PostCollidable is bool collidable) values["reformResetPostCollidable"] = collidable;
        if (probe.PostPosition is Vector2 position) values["reformResetPostPosition"] = Simplify(position);
        if (probe.Error is not null) values["reformResetProbeError"] = probe.Error;
    }

    private static List<Dictionary<string, object?>> DescribeCollisionEntities<T>(IEnumerable<T> entities)
        where T : Entity {
        List<Dictionary<string, object?>> result = [];
        foreach (T entity in entities) {
            Dictionary<string, object?> description = new() {
                ["type"] = entity.GetType().Name,
                ["position"] = Simplify(entity.Position),
                ["active"] = entity.Active,
                ["collidable"] = entity.Collidable,
                ["colliderPresent"] = entity.Collider is not null,
            };
            if (entity.Collider is Collider collider) {
                description["worldBounds"] = ColliderWorldGeometry(collider, entity);
            }
            result.Add(description);
        }
        return result;
    }

    private static List<BounceBlockResetCollisionCandidate> ScanBounceBlockResetCandidates(
        BounceBlock bounceBlock,
        Vector2 target
    ) {
        List<BounceBlockResetCollisionCandidate> result = [];
        foreach (Vector2 offset in bounceBlockResetScanOffsets) {
            Vector2 candidateTarget = target + offset;
            // Use the engine's position overloads directly. In particular,
            // this lets a SolidTiles Grid decide collision tile-by-tile rather
            // than reducing the map to an inaccurate Solid bounding box.
            List<Entity> actors = bounceBlock.CollideAll<Actor>(candidateTarget);
            List<Entity> solids = bounceBlock.CollideAll<Solid>(candidateTarget);
            BounceBlockResetCollisionCandidate candidate = new() {
                Offset = offset,
                Target = candidateTarget,
                ActorBlocked = actors.Count > 0,
                SolidBlocked = solids.Count > 0,
                Actors = DescribeCollisionEntities(actors),
                Solids = DescribeCollisionEntities(solids),
            };
            result.Add(candidate);
        }
        return result;
    }

    private static List<Dictionary<string, object?>> DescribeBounceBlockResetCollisionScan(
        List<BounceBlockResetCollisionCandidate> candidates
    ) {
        List<Dictionary<string, object?>> result = [];
        foreach (BounceBlockResetCollisionCandidate candidate in candidates) {
            result.Add(new Dictionary<string, object?> {
                ["offset"] = Simplify(candidate.Offset),
                ["target"] = Simplify(candidate.Target),
                ["actorBlocked"] = candidate.ActorBlocked,
                ["solidBlocked"] = candidate.SolidBlocked,
                ["result"] = candidate.ActorBlocked
                    ? "actor"
                    : candidate.SolidBlocked ? "solid" : "clear",
                ["actors"] = candidate.Actors,
                ["solids"] = candidate.Solids,
            });
        }
        return result;
    }

    private static Vector2[] BuildBounceBlockResetScanOffsets() {
        float[] axis = [-8f, -1f, 0f, 1f, 8f];
        List<Vector2> result = [];
        foreach (float y in axis) {
            foreach (float x in axis) result.Add(new Vector2(x, y));
        }
        result.Sort((first, second) => {
            int distance = first.LengthSquared().CompareTo(second.LengthSquared());
            if (distance != 0) return distance;
            int y = first.Y.CompareTo(second.Y);
            return y != 0 ? y : first.X.CompareTo(second.X);
        });
        return [.. result];
    }
}
