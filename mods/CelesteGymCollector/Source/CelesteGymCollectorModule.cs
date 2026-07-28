using Microsoft.Xna.Framework;
using Monocle;
using System.Reflection;

namespace Celeste.Mod.CelesteGymCollector;

public sealed class CelesteGymCollectorModule : EverestModule {
    private readonly CollectorServer server = new();
    private readonly int collectorPort = ReadCollectorPort();
    private readonly string runNonce = Environment.GetEnvironmentVariable("CELESTE_GYM_RUN_NONCE") ?? "";
    private readonly string? configuredRecordingRoot =
        Environment.GetEnvironmentVariable("CELESTE_GYM_RECORDING_ROOT");
    private readonly HashSet<string> usedCaptureTokens = new(StringComparer.Ordinal);
    private SimulationJob? job;
    private PresentationCaptureSession? captureSession;
    private InteractiveRecordingSession? interactiveSession;
    private readonly Dictionary<VirtualButton, List<VirtualButton.Node>> originalButtonNodes = [];
    private static readonly PropertyInfo FeatherValueProperty = typeof(VirtualJoystick)
        .GetProperty(nameof(VirtualJoystick.Value), BindingFlags.Instance | BindingFlags.Public)!;
    private static readonly PropertyInfo AimValueProperty = typeof(VirtualJoystick)
        .GetProperty(nameof(VirtualJoystick.Value), BindingFlags.Instance | BindingFlags.Public)!;

    public override void Load() {
        On.Monocle.Engine.Update += EngineUpdate;
        On.Celeste.Celeste.RenderCore += CelesteRenderCore;
        On.Celeste.Player.Update += PlayerUpdate;
        On.Celeste.Player.Jump += PlayerJump;
        On.Celeste.Player.StartDash += PlayerStartDash;
        On.Celeste.Player.Die += PlayerDie;
        On.Celeste.Input.GetAimVector += GetAimVector;
        server.Start(collectorPort);
    }

    public override void Unload() {
        server.Dispose();
        On.Monocle.Engine.Update -= EngineUpdate;
        On.Celeste.Celeste.RenderCore -= CelesteRenderCore;
        On.Celeste.Player.Update -= PlayerUpdate;
        On.Celeste.Player.Jump -= PlayerJump;
        On.Celeste.Player.StartDash -= PlayerStartDash;
        On.Celeste.Player.Die -= PlayerDie;
        On.Celeste.Input.GetAimVector -= GetAimVector;
    }

    private void EngineUpdate(On.Monocle.Engine.orig_Update orig, Engine self, GameTime gameTime) {
        ProcessRequests();
        PrepareSimulationFrame();
        orig(self, gameTime);
        FinishSimulationFrame();
        interactiveSession?.FinishPlayerFrame();
    }

    private void CelesteRenderCore(
        On.Celeste.Celeste.orig_RenderCore orig,
        global::Celeste.Celeste self
    ) {
        orig(self);
        captureSession?.CapturePresentation(self.GraphicsDevice, Engine.Viewport.Bounds);
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
                    Error = ready ? null : "game content is still loading",
                    RunNonce = runNonce,
                    ProcessId = Environment.ProcessId,
                    CollectorPort = collectorPort
                });
                return;
            }
            if (pending.Request.Command.StartsWith("capture_", StringComparison.Ordinal)) {
                HandleCaptureCommand(pending);
                return;
            }
            if (pending.Request.Command.StartsWith("interactive_", StringComparison.Ordinal)) {
                HandleInteractiveCommand(pending);
                return;
            }
            if (pending.Request.Command != "simulate_area") {
                pending.Completion.SetResult(new CollectorResponse { Success = false, Error = "unknown command" });
                return;
            }
            try {
                if (interactiveSession?.IsRunning == true) {
                    throw new InvalidOperationException("cannot inject a simulation while interactive recording is active");
                }
                BindCaptureToSimulation(pending.Request);
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

    private void HandleCaptureCommand(PendingRequest pending) {
        try {
            if (interactiveSession?.IsRunning == true) {
                throw new InvalidOperationException("cannot capture a scripted scenario while interactive recording is active");
            }
            CollectorRequest request = pending.Request;
            RecordingSecurity.Authenticate(
                runNonce,
                Environment.ProcessId,
                request.RunNonce,
                request.ProcessId
            );
            string token = RecordingSecurity.ValidateToken(request.CaptureToken);
            switch (request.Command) {
                case "capture_start":
                    if (captureSession is not null && captureSession.GetStatus().State != "finalized") {
                        throw new InvalidOperationException("another capture session is not finalized");
                    }
                    if (usedCaptureTokens.Contains(token)) {
                        throw new InvalidOperationException("capture token is one-time and was already used");
                    }
                    captureSession = new PresentationCaptureSession(
                        configuredRecordingRoot
                            ?? throw new InvalidOperationException(
                                "CELESTE_GYM_RECORDING_ROOT is not configured"
                            ),
                        runNonce,
                        Environment.ProcessId,
                        new CaptureStartRequest {
                            CaptureToken = token,
                            ScenarioId = request.ScenarioId ?? "",
                            StartStateIndex = request.StartStateIndex,
                            EndStateIndex = request.EndStateIndex,
                            TimeoutMilliseconds = request.TimeoutMilliseconds
                        }
                    );
                    usedCaptureTokens.Add(token);
                    break;
                case "capture_status":
                    RequireCaptureSession(token);
                    break;
                case "capture_stop":
                    RequireCaptureSession(token).Stop(request.Reason ?? "explicit_stop");
                    break;
                case "capture_finalize":
                    RequireCaptureSession(token).FinalizeManifest();
                    break;
                default:
                    throw new InvalidOperationException("unknown capture command");
            }
            pending.Completion.SetResult(new CollectorResponse {
                Success = true,
                Recording = RequireCaptureSession(token).GetStatus()
            });
        } catch (Exception error) {
            pending.Completion.SetResult(new CollectorResponse {
                Success = false,
                Error = error.Message
            });
        }
    }

    private void HandleInteractiveCommand(PendingRequest pending) {
        try {
            CollectorRequest request = pending.Request;
            RecordingSecurity.Authenticate(runNonce, Environment.ProcessId, request.RunNonce, request.ProcessId);
            string token = RecordingSecurity.ValidateToken(request.CaptureToken);
            switch (request.Command) {
                case "interactive_start":
                    if (job is not null || captureSession?.IsCapturing == true) {
                        throw new InvalidOperationException("another collector job is active");
                    }
                    if (interactiveSession?.IsRunning == true) {
                        throw new InvalidOperationException("another interactive recording is active");
                    }
                    if (usedCaptureTokens.Contains(token)) {
                        throw new InvalidOperationException("capture token is one-time and was already used");
                    }
                    if (string.IsNullOrWhiteSpace(request.AreaSid) || string.IsNullOrWhiteSpace(request.Room)) {
                        throw new InvalidOperationException("interactive recording requires area_sid and room");
                    }
                    if (!string.Equals(request.AreaSid, "CelesteGymPlayground/Playground", StringComparison.Ordinal)
                        || !string.Equals(request.Room, "playground", StringComparison.Ordinal)) {
                        throw new InvalidOperationException("interactive recording only accepts the bundled Playground map");
                    }
                    int areaId = ResolveAreaId(request);
                    interactiveSession = new InteractiveRecordingSession(
                        configuredRecordingRoot
                            ?? throw new InvalidOperationException("CELESTE_GYM_RECORDING_ROOT is not configured"),
                        token,
                        request.RecordingId ?? "manual-play",
                        areaId,
                        request.AreaSid,
                        request.Room,
                        request.MaxFrames
                    );
                    usedCaptureTokens.Add(token);
                    if (SaveData.Instance is null) SaveData.InitializeDebugMode(loadExisting: false);
                    Session session = new(new AreaKey(areaId)) {
                        Level = request.Room,
                        RespawnPoint = null
                    };
                    session.Inventory.DreamDash = true;
                    LevelEnter.Go(session, fromSaveData: true);
                    break;
                case "interactive_status":
                    RequireInteractiveSession(token);
                    break;
                case "interactive_stop":
                    RequireInteractiveSession(token).Stop(request.Reason ?? "explicit_stop");
                    break;
                default:
                    throw new InvalidOperationException("unknown interactive recording command");
            }
            pending.Completion.SetResult(new CollectorResponse {
                Success = true,
                InteractiveRecording = RequireInteractiveSession(token).GetStatus()
            });
        } catch (Exception error) {
            pending.Completion.SetResult(new CollectorResponse { Success = false, Error = error.Message });
        }
    }

    private InteractiveRecordingSession RequireInteractiveSession(string token) {
        if (interactiveSession is null
            || !string.Equals(interactiveSession.Token, token, StringComparison.Ordinal)) {
            throw new UnauthorizedAccessException("capture token does not identify the interactive recording");
        }
        return interactiveSession;
    }

    private PresentationCaptureSession RequireCaptureSession(string token) {
        if (captureSession is null
            || !string.Equals(captureSession.Token, token, StringComparison.Ordinal)) {
            throw new UnauthorizedAccessException("capture token does not identify the active session");
        }
        return captureSession;
    }

    private void BindCaptureToSimulation(CollectorRequest request) {
        if (captureSession is null || !captureSession.IsCapturing) {
            if (!string.IsNullOrEmpty(request.CaptureToken)) {
                throw new InvalidOperationException("simulation references no active capture session");
            }
            return;
        }
        string token = RecordingSecurity.ValidateToken(request.CaptureToken);
        RequireCaptureSession(token).BindToSimulation(request.Inputs.Count);
    }

    private static int ReadCollectorPort() {
        string? raw = Environment.GetEnvironmentVariable("CELESTE_GYM_COLLECTOR_PORT");
        if (string.IsNullOrWhiteSpace(raw)) return 32270;
        if (!int.TryParse(raw, out int port) || port is <= 0 or > 65535) {
            throw new InvalidOperationException($"CELESTE_GYM_COLLECTOR_PORT must be between 1 and 65535, got {raw}");
        }
        return port;
    }

    private void PlayerUpdate(On.Celeste.Player.orig_Update orig, Player self) {
        interactiveSession?.BeginPlayerFrame(self);
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
        float beforeStamina = self.Stamina;
        orig(self);
        // Jump/WallJump/ClimbJump all consume the same virtual-button buffer.
        // A Normal-state ClimbJump may start while Speed.Y is already negative,
        // and a repeated ClimbJump may already be exactly -105. The precise
        // 27.5 stamina cost covers that last case without swallowing launches.
        bool normalJumpStarted = self.StateMachine.State == Player.StNormal
            && MathF.Abs(self.Speed.Y - -105f) < 0.001f
            && MathF.Abs(beforeSpeedY - self.Speed.Y) > 0.001f;
        bool climbJumpCostPaid = beforeStamina - self.Stamina >= 27.49f;
        if (job is { JumpBufferFrames: > 0 }
            && self.Speed.Y < 0f
            && (beforeSpeedY >= 0f
                || (beforeState != Player.StNormal && self.StateMachine.State == Player.StNormal)
                || normalJumpStarted
                || climbJumpCostPaid)) {
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

    private void PlayerJump(
        On.Celeste.Player.orig_Jump orig,
        Player self,
        bool particles,
        bool playSfx
    ) {
        orig(self, particles, playSfx);
        if (job is not null) {
            job.JumpBufferFrames = ScriptedInputBuffer.Consume(job.JumpBufferFrames);
        }
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
        if (body is not null) interactiveSession?.CaptureDeath(self);
        return body;
    }

    private void PrepareSimulationFrame() {
        if (job is null) {
            if (interactiveSession?.IsRunning == true && Engine.Scene is Level interactiveLevel) {
                interactiveSession.TryBegin(interactiveLevel);
            }
            return;
        }
        if (!job.Started) {
            if (ReferenceEquals(Engine.Scene, job.SceneAtRequest)) return;
            Player? player = ActivePlayer(job.AreaId);
            if (player is null) return;
            // Recording starts before the fresh LevelEnter so that the capture
            // token owns the entire request. Do not advance scripted state 0
            // behind that entry wipe: wait for ScreenWipe to clear itself,
            // while leaving later room transitions entirely untouched.
            if (captureSession is not null
                && !job.Pending.Request.SkipTransitions
                && Engine.Scene is Level recordingLevel
                && recordingLevel.Wipe is not null) return;
            if (job.Pending.Request.SkipTransitions && Engine.Scene is Level level) {
                level.Wipe?.Cancel();
            }
            InstallScriptedButtons();
            ApplyInitialSnapshot(player, job.Pending.Request.InitialSnapshot);
            // Player.Update normally eases toward CameraTarget, but scenario
            // snapshots teleport the player after LevelEnter positioned the
            // camera at the room spawn. Snap once before state 0 so the first
            // recorded presentation uses Celeste's native camera offset,
            // bounds, anchors, and lock rules instead of showing empty space.
            if (job.Pending.Request.InitialSnapshot?.Pos is { Length: >= 2 }
                && player.Scene is Level initialLevel) {
                initialLevel.Camera.Position = player.CameraTarget;
            }
            job.States.Add(SnapshotCapture.Capture(player, 0));
            captureSession?.UpdateLatestStateIndex(0);
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
        captureSession?.UpdateLatestStateIndex(job.Index);
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
        completed.Pending.Completion.SetResult(new CollectorResponse {
            Success = true,
            States = completed.States,
            Recording = captureSession?.GetStatus()
        });
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
