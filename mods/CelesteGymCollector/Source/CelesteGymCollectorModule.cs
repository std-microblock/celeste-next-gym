using Microsoft.Xna.Framework;
using Monocle;
using System.Reflection;

namespace Celeste.Mod.CelesteGymCollector;

public sealed class CelesteGymCollectorModule : EverestModule {
    private readonly CollectorServer server = new();
    private SimulationJob? job;
    private readonly Dictionary<VirtualButton, List<VirtualButton.Node>> originalButtonNodes = [];
    private static readonly PropertyInfo FeatherValueProperty = typeof(VirtualJoystick)
        .GetProperty(nameof(VirtualJoystick.Value), BindingFlags.Instance | BindingFlags.Public)!;
    private static readonly PropertyInfo AimValueProperty = typeof(VirtualJoystick)
        .GetProperty(nameof(VirtualJoystick.Value), BindingFlags.Instance | BindingFlags.Public)!;

    public override void Load() {
        On.Monocle.Engine.Update += EngineUpdate;
        On.Celeste.Player.Update += PlayerUpdate;
        On.Celeste.Player.StartDash += PlayerStartDash;
        On.Celeste.Player.Die += PlayerDie;
        On.Celeste.Input.GetAimVector += GetAimVector;
        server.Start();
    }

    public override void Unload() {
        server.Dispose();
        On.Monocle.Engine.Update -= EngineUpdate;
        On.Celeste.Player.Update -= PlayerUpdate;
        On.Celeste.Player.StartDash -= PlayerStartDash;
        On.Celeste.Player.Die -= PlayerDie;
        On.Celeste.Input.GetAimVector -= GetAimVector;
    }

    private void EngineUpdate(On.Monocle.Engine.orig_Update orig, Engine self, GameTime gameTime) {
        ProcessRequests();
        PrepareSimulationFrame();
        orig(self, gameTime);
        FinishSimulationFrame();
    }

    private void ProcessRequests() {
        if (job is null && server.TryDequeue(out PendingRequest? pending) && pending is not null) {
            if (pending.Request.Command == "ping") {
                bool ready = Celeste.LoadTimer is null
                    && Engine.Scene is not null
                    && AreaData.Areas is { Count: > 0 }
                    && Settings.Instance is not null;
                pending.Completion.SetResult(new CollectorResponse {
                    Success = ready,
                    Version = "0.1.0",
                    Error = ready ? null : "game content is still loading"
                });
                return;
            }
            if (pending.Request.Command != "simulate_area") {
                pending.Completion.SetResult(new CollectorResponse { Success = false, Error = "unknown command" });
                return;
            }
            try {
                if (SaveData.Instance is null) SaveData.InitializeDebugMode(loadExisting: false);
                int areaId = ResolveAreaId(pending.Request);
                job = new SimulationJob(pending, Engine.Scene, areaId);
                // Always create a fresh level/player. Merely overwriting public
                // Player fields leaves old dash coroutines and entity state
                // alive, so requests would not be pure or isolated.
                // `fromSaveData: true` skips postcards/vignettes that wait for human confirmation.
                Session session = new(new AreaKey(areaId));
                if (pending.Request.DreamDash) session.Inventory.DreamDash = true;
                if (!string.IsNullOrWhiteSpace(pending.Request.Room)) {
                    session.Level = pending.Request.Room;
                    session.RespawnPoint = null;
                }
                if (pending.Request.SkipTransitions) {
                    Engine.Scene = new LevelLoader(session) {
                        PlayerIntroTypeOverride = Player.IntroTypes.None
                    };
                } else {
                    LevelEnter.Go(session, fromSaveData: true);
                }
            } catch (Exception error) {
                job = null;
                pending.Completion.SetResult(new CollectorResponse { Success = false, Error = error.ToString() });
            }
        }
    }

    private void PlayerUpdate(On.Celeste.Player.orig_Update orig, Player self) {
        FrameInput? input = job?.CurrentInput;
        if (input is not null) {
            Input.MoveX.Value = Math.Clamp(input.MoveX, -1, 1);
            Input.MoveY.Value = Math.Clamp(input.MoveY, -1, 1);
            Vector2 joystick = new Vector2(input.MoveX, input.MoveY).SafeNormalize();
            FeatherValueProperty.SetValue(Input.Feather, joystick);
            AimValueProperty.SetValue(Input.Aim, joystick);
        }
        int beforeState = self.StateMachine.State;
        float beforeSpeedY = self.Speed.Y;
        orig(self);
        if (job is { JumpBufferFrames: > 0 }
            && self.Speed.Y < 0f
            && (beforeSpeedY >= 0f || (beforeState != 0 && self.StateMachine.State == 0))) {
            job.JumpBufferFrames = 0;
        }
    }

    private int PlayerStartDash(On.Celeste.Player.orig_StartDash orig, Player self) {
        int nextState = orig(self);
        if (job is not null) {
            job.DashBufferFrames = 0;
            job.CrouchDashBufferFrames = 0;
        }
        return nextState;
    }

    private PlayerDeadBody? PlayerDie(
        On.Celeste.Player.orig_Die orig,
        Player self,
        Vector2 direction,
        bool evenIfInvincible,
        bool registerDeathInStats
    ) {
        PlayerDeadBody? body = orig(self, direction, evenIfInvincible, registerDeathInStats);
        if (body is not null && job is { Started: true, CurrentInput: not null }) {
            job.DeathFrame = SnapshotCapture.Capture(self, job.Index + 1);
        }
        return body;
    }

    private void PrepareSimulationFrame() {
        if (job is null) return;
        if (!job.Started) {
            if (ReferenceEquals(Engine.Scene, job.SceneAtRequest)) return;
            Player? player = ActivePlayer(job.AreaId);
            if (player is null) return;
            if (job.Pending.Request.SkipTransitions && Engine.Scene is Level level) {
                level.Wipe?.Cancel();
            }
            InstallScriptedButtons();
            ApplyInitialSnapshot(player, job.Pending.Request.InitialSnapshot);
            job.States.Add(SnapshotCapture.Capture(player, 0));
            job.Started = true;
        }
        if (job.Index >= job.Pending.Request.Inputs.Count) {
            CompleteJob();
            return;
        }
        job.PreviousInput = job.LastInput;
        job.CurrentInput = job.Pending.Request.Inputs[job.Index];
        if (job.CurrentInput.JumpPressed) job.JumpBufferFrames = 5;
        if (job.CurrentInput.DashPressed) job.DashBufferFrames = 5;
        if (job.CurrentInput.CrouchDashPressed) job.CrouchDashBufferFrames = 5;
    }

    private void FinishSimulationFrame() {
        if (job is null || !job.Started || job.CurrentInput is null) return;
        job.Index++;
        Player? player = ActivePlayer(job.AreaId);
        PlayerFrame frame = player is null
            ? job.DeathFrame ?? SnapshotCapture.CaptureMissing(job.States[^1], job.Index)
            : SnapshotCapture.Capture(player, job.Index);
        job.DeathFrame = null;
        job.States.Add(frame);
        job.LastInput = job.CurrentInput;
        job.CurrentInput = null;
        if (job.JumpBufferFrames > 0) job.JumpBufferFrames--;
        if (job.DashBufferFrames > 0) job.DashBufferFrames--;
        if (job.CrouchDashBufferFrames > 0) job.CrouchDashBufferFrames--;
        if (job.Index >= job.Pending.Request.Inputs.Count) CompleteJob();
    }

    private static Player? ActivePlayer(int areaId) {
        if (Engine.Scene is not Level level || level.Session.Area.ID != areaId) return null;
        return level.Tracker.GetEntity<Player>();
    }

    private static int ResolveAreaId(CollectorRequest request) {
        if (string.IsNullOrWhiteSpace(request.AreaSid)) return request.AreaId;
        AreaData? area = AreaData.Get(request.AreaSid);
        if (area is null) {
            ModAsset? plainAsset = Everest.Content.Get($"Maps/{request.AreaSid}");
            ModAsset? binAsset = Everest.Content.Get($"Maps/{request.AreaSid}.bin");
            string assets = $"plain={DescribeAsset(plainAsset)}, bin={DescribeAsset(binAsset)}";
            throw new InvalidOperationException(
                $"unknown area SID: {request.AreaSid}; assets: {assets}"
            );
        }
        return area.ID;
    }

    private static string DescribeAsset(ModAsset? asset) {
        return asset is null ? "missing" : $"{asset.PathVirtual}:{asset.Type?.Name}:{asset.Format}";
    }

    private void CompleteJob() {
        SimulationJob completed = job!;
        job = null;
        RestoreButtons();
        completed.Pending.Completion.SetResult(new CollectorResponse { Success = true, States = completed.States });
    }

    private void InstallScriptedButtons() {
        ReplaceNodes(Input.Jump,
            () => job?.CurrentInput?.JumpHeld ?? false,
            () => job?.JumpBufferFrames > 0,
            () => job?.PreviousInput?.JumpHeld == true && job?.CurrentInput?.JumpHeld != true);
        ReplaceNodes(Input.Dash,
            () => job?.CurrentInput?.DashPressed ?? false,
            () => job?.DashBufferFrames > 0,
            () => job?.PreviousInput?.DashPressed == true && job?.CurrentInput?.DashPressed != true);
        ReplaceNodes(Input.CrouchDash,
            () => job?.CurrentInput?.CrouchDashPressed ?? false,
            () => job?.CrouchDashBufferFrames > 0,
            () => job?.PreviousInput?.CrouchDashPressed == true && job?.CurrentInput?.CrouchDashPressed != true);
        ReplaceNodes(Input.Grab,
            () => job?.CurrentInput?.GrabHeld ?? false,
            () => job?.PreviousInput?.GrabHeld != true && job?.CurrentInput?.GrabHeld == true,
            () => job?.PreviousInput?.GrabHeld == true && job?.CurrentInput?.GrabHeld != true);
    }

    private void ReplaceNodes(VirtualButton button, Func<bool> check, Func<bool> pressed, Func<bool> released) {
        originalButtonNodes[button] = [.. button.Nodes];
        button.Nodes.Clear();
        ScriptedButtonNode node = new(check, pressed, released) {
            // The collector owns the explicit five-frame counters above.
            // Letting VirtualButton buffer this scripted node as well would
            // re-press dash after Player.StartDash consumed the source buffer.
            Bufferable = false
        };
        button.Nodes.Add(node);
    }

    private void RestoreButtons() {
        foreach ((VirtualButton button, List<VirtualButton.Node> nodes) in originalButtonNodes) {
            button.Nodes.Clear();
            button.Nodes.AddRange(nodes);
        }
        originalButtonNodes.Clear();
    }

    private Vector2 GetAimVector(On.Celeste.Input.orig_GetAimVector orig, Facings defaultFacing) {
        FrameInput? input = job?.CurrentInput;
        if (input is null) return orig(defaultFacing);
        Vector2 aim = new(input.MoveX, input.MoveY);
        return aim == Vector2.Zero ? Vector2.UnitX * (float) defaultFacing : aim.SafeNormalize();
    }

    private static void ApplyInitialSnapshot(Player player, InitialSnapshot? snapshot) {
        if (snapshot is null) return;
        if (snapshot.Pos is { Length: >= 2 }) player.Position = new Vector2(snapshot.Pos[0], snapshot.Pos[1]);
        if (snapshot.Speed is { Length: >= 2 }) player.Speed = new Vector2(snapshot.Speed[0], snapshot.Speed[1]);
        if (snapshot.Dashes is int dashes) player.Dashes = dashes;
        if (snapshot.Stamina is float stamina) player.Stamina = stamina;
        if (snapshot.State is int state && state >= 0 && state < 26) player.StateMachine.State = state;
        if (snapshot.Facing is bool facing) player.Facing = facing ? Facings.Right : Facings.Left;
        if (snapshot.Ducking is bool ducking) player.Ducking = ducking;
    }

    private sealed class SimulationJob(PendingRequest pending, Scene? sceneAtRequest, int areaId) {
        public PendingRequest Pending { get; } = pending;
        public Scene? SceneAtRequest { get; } = sceneAtRequest;
        public int AreaId { get; } = areaId;
        public List<PlayerFrame> States { get; } = [];
        public bool Started { get; set; }
        public int Index { get; set; }
        public FrameInput? CurrentInput { get; set; }
        public FrameInput? PreviousInput { get; set; }
        public FrameInput? LastInput { get; set; }
        public PlayerFrame? DeathFrame { get; set; }
        public int JumpBufferFrames { get; set; }
        public int DashBufferFrames { get; set; }
        public int CrouchDashBufferFrames { get; set; }
    }

    private sealed class ScriptedButtonNode(Func<bool> check, Func<bool> pressed, Func<bool> released) : VirtualButton.Node {
        public override bool Check => check();
        public override bool Pressed => pressed();
        public override bool Released => released();
    }
}
