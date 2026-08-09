using Microsoft.Xna.Framework;
using Monocle;
using System.Reflection;

namespace Celeste.Mod.CelesteGymCollector;

public sealed class CelesteGymCollectorModule : EverestModule {
    private readonly CollectorServer server = new();
    private readonly CollectorPortConfiguration collectorPortConfiguration = ReadCollectorPort();
    private int collectorPort;
    private readonly string runNonce = Environment.GetEnvironmentVariable("CELESTE_GYM_RUN_NONCE") ?? "";
    private readonly bool headlessActor = string.Equals(
        Environment.GetEnvironmentVariable("CELESTE_GYM_HEADLESS"),
        "1",
        StringComparison.Ordinal
    );
    private readonly string? configuredRecordingRoot =
        Environment.GetEnvironmentVariable("CELESTE_GYM_RECORDING_ROOT");
    private readonly HashSet<string> usedCaptureTokens = new(StringComparer.Ordinal);
    private SimulationJob? job;
    private GymResetJob? gymResetJob;
    private GymStepJob? gymStepJob;
    private GymEpisode? gymEpisode;
    private GymFastLoopPatch? gymFastLoopPatch;
    private PresentationCaptureSession? captureSession;
    private InteractiveRecordingSession? interactiveSession;
    private readonly Dictionary<VirtualButton, List<VirtualButton.Node>> originalButtonNodes = [];
    private static readonly PropertyInfo FeatherValueProperty = typeof(VirtualJoystick)
        .GetProperty(nameof(VirtualJoystick.Value), BindingFlags.Instance | BindingFlags.Public)!;
    private static readonly PropertyInfo AimValueProperty = typeof(VirtualJoystick)
        .GetProperty(nameof(VirtualJoystick.Value), BindingFlags.Instance | BindingFlags.Public)!;

    public override void Load() {
        gymFastLoopPatch = new GymFastLoopPatch(
            SelectFastLoopFrameCount,
            () => gymStepJob is not null,
            () => gymResetJob is not null || gymStepJob is not null || gymEpisode is not null,
            () => headlessActor
        );
        On.Monocle.Engine.Update += EngineUpdate;
        On.Celeste.Celeste.RenderCore += CelesteRenderCore;
        On.Celeste.Player.Update += PlayerUpdate;
        On.Celeste.Player.Jump += PlayerJump;
        On.Celeste.Player.StartDash += PlayerStartDash;
        On.Celeste.Player.Die += PlayerDie;
        On.Celeste.Lookout.Removed += LookoutRemoved;
        On.Celeste.Input.GetAimVector += GetAimVector;
        collectorPort = server.Start(
            collectorPortConfiguration.PreferredPort,
            collectorPortConfiguration.AllowFallback
        );
    }

    public override void Unload() {
        server.Dispose();
        On.Monocle.Engine.Update -= EngineUpdate;
        On.Celeste.Celeste.RenderCore -= CelesteRenderCore;
        On.Celeste.Player.Update -= PlayerUpdate;
        On.Celeste.Player.Jump -= PlayerJump;
        On.Celeste.Player.StartDash -= PlayerStartDash;
        On.Celeste.Player.Die -= PlayerDie;
        On.Celeste.Lookout.Removed -= LookoutRemoved;
        On.Celeste.Input.GetAimVector -= GetAimVector;
        gymFastLoopPatch?.Dispose();
        gymFastLoopPatch = null;
    }

    private void EngineUpdate(On.Monocle.Engine.orig_Update orig, Engine self, GameTime gameTime) {
        if (headlessActor || gymEpisode?.FastMode == true) self.SuppressDraw();
        ProcessRequests();
        PrepareSimulationFrame();
        if (GymIdlePolicy.ShouldPark(
                gymEpisode is not null,
                gymResetJob is not null,
                gymStepJob is not null,
                job is not null
            )) {
            return;
        }
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
        if (job is null && gymResetJob is null && gymStepJob is null
            && server.TryDequeue(out PendingRequest? pending) && pending is not null) {
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
            if (pending.Request.Command.StartsWith("gym_", StringComparison.Ordinal)) {
                HandleGymCommand(pending);
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
                gymEpisode = null;
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

    private void HandleGymCommand(PendingRequest pending) {
        try {
            if (interactiveSession?.IsRunning == true || captureSession?.IsCapturing == true) {
                throw new InvalidOperationException("cannot use the gym backend while recording is active");
            }
            CollectorRequest request = pending.Request;
            RecordingSecurity.Authenticate(
                runNonce,
                Environment.ProcessId,
                request.RunNonce,
                request.ProcessId
            );
            switch (request.Command) {
                case "gym_reset": {
                    if (request.MaxEpisodeFrames is <= 0 or > 10_000_000) {
                        throw new InvalidOperationException(
                            "max_episode_frames must be between 1 and 10000000"
                        );
                    }
                    if (SaveData.Instance is null) SaveData.InitializeDebugMode(loadExisting: false);
                    int areaId = ResolveAreaId(request);
                    gymEpisode = null;
                    gymResetJob = new GymResetJob(pending, Engine.Scene, areaId);
                    GymResetPolicy.ClearEngineUpdateBlockers();
                    if (request.Seed is int seed) GymRandomPolicy.Reset(seed);
                    Session session = new(new AreaKey(areaId));
                    if (request.DreamDash) session.Inventory.DreamDash = true;
                    if (!string.IsNullOrWhiteSpace(request.Room)) {
                        session.Level = request.Room;
                        session.RespawnPoint = null;
                    }
                    if (request.SkipTransitions
                        && Engine.Scene is Level activeLevel
                        && activeLevel.Session.Area.ID == areaId) {
                        // A new LevelLoader allocates the entire area's renderer,
                        // backdrop, particle and graphics infrastructure. Repeating
                        // that hundreds of times in a long-running RL actor grows
                        // native FNA/FMOD resources until the process terminates.
                        // For the same area, keep that infrastructure and perform
                        // Celeste's authoritative room reload against a fresh
                        // Session, so gameplay entities/coroutines are rebuilt.
                        GymResetPolicy.PrepareInPlaceLevel(activeLevel);
                        gymResetJob = new GymResetJob(
                            pending,
                            Engine.Scene,
                            areaId,
                            inPlace: true
                        );
                        activeLevel.Completed = false;
                        activeLevel.Session = session;
                        SaveData.Instance!.StartSession(session);
                        activeLevel.Reload();
                        Player? reloadedPlayer = activeLevel.Tracker.GetEntity<Player>();
                        if (reloadedPlayer?.StateMachine.State == Player.StIntroRespawn) {
                            // Level.Reload always creates the player with the
                            // 0.6 s respawn presentation. Wall-clock latency
                            // between reset and step would then change the
                            // physics state reached by identical input batches.
                            // Gym reset is already transition-free, so finish
                            // that presentation immediately and deterministically.
                            reloadedPlayer.StateMachine.State = Player.StNormal;
                            reloadedPlayer.JustRespawned = false;
                        }
                        break;
                    }
                    if (request.SkipTransitions) {
                        Engine.Scene = new LevelLoader(session) {
                            PlayerIntroTypeOverride = Player.IntroTypes.None
                        };
                    } else {
                        LevelEnter.Go(session, fromSaveData: true);
                    }
                    break;
                }
                case "gym_step": {
                    GymEpisode episode = RequireGymEpisode(request.EpisodeId);
                    if (episode.Done) {
                        throw new InvalidOperationException("gym episode is done; call gym_reset");
                    }
                    if (request.Inputs.Count is <= 0 or > 4096) {
                        throw new InvalidOperationException("gym_step inputs must contain 1 to 4096 frames");
                    }
                    if (Engine.Scene is not Level level || level.Session.Area.ID != episode.AreaId) {
                        throw new InvalidOperationException("gym episode level is no longer active");
                    }
                    gymStepJob = new GymStepJob(pending, episode, level);
                    InstallScriptedButtons();
                    break;
                }
                case "gym_observe": {
                    GymEpisode episode = RequireGymEpisode(request.EpisodeId);
                    if (Engine.Scene is not Level level) {
                        throw new InvalidOperationException("gym episode level is no longer active");
                    }
                    PlayerFrame player = CaptureGymPlayer(level, episode, episode.Frame);
                    pending.Completion.SetResult(new CollectorResponse {
                        Success = true,
                        Observation = GymCapture.Capture(
                            level,
                            episode,
                            player,
                            includeGeometry: true,
                            terminated: episode.Terminated,
                            truncated: episode.Truncated,
                            success: episode.Success,
                            terminationReason: episode.TerminationReason
                        ),
                        FramesExecuted = 0
                    });
                    break;
                }
                case "gym_close":
                    if (gymEpisode is not null && request.EpisodeId is not null) {
                        RequireGymEpisode(request.EpisodeId);
                    }
                    gymEpisode = null;
                    pending.Completion.SetResult(new CollectorResponse {
                        Success = true,
                        FramesExecuted = 0
                    });
                    break;
                default:
                    throw new InvalidOperationException("unknown gym command");
            }
        } catch (Exception error) {
            gymResetJob = null;
            gymStepJob = null;
            RestoreButtons();
            pending.Completion.SetResult(new CollectorResponse {
                Success = false,
                Error = error.Message
            });
        }
    }

    private GymEpisode RequireGymEpisode(string? episodeId) {
        if (gymEpisode is null || string.IsNullOrWhiteSpace(episodeId)
            || !string.Equals(gymEpisode.Id, episodeId, StringComparison.Ordinal)) {
            throw new InvalidOperationException("episode_id does not identify the active gym episode");
        }
        return gymEpisode;
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
                    if (job is not null || gymEpisode is not null
                        || captureSession?.IsCapturing == true) {
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

    private static CollectorPortConfiguration ReadCollectorPort() {
        string? raw = Environment.GetEnvironmentVariable("CELESTE_GYM_COLLECTOR_PORT");
        if (string.IsNullOrWhiteSpace(raw)) {
            return new CollectorPortConfiguration(32270, AllowFallback: true);
        }
        if (!int.TryParse(raw, out int port) || port is <= 0 or > 65535) {
            throw new InvalidOperationException($"CELESTE_GYM_COLLECTOR_PORT must be between 1 and 65535, got {raw}");
        }
        return new CollectorPortConfiguration(port, AllowFallback: false);
    }

    private readonly record struct CollectorPortConfiguration(int PreferredPort, bool AllowFallback);

    private void PlayerUpdate(On.Celeste.Player.orig_Update orig, Player self) {
        interactiveSession?.BeginPlayerFrame(self);
        IScriptedInputJob? inputJob = ActiveInputJob;
        FrameInput? input = inputJob?.CurrentInput;
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
        if (inputJob is { JumpBufferFrames: > 0 }
            && self.Speed.Y < 0f
            && (beforeSpeedY >= 0f
                || (beforeState != Player.StNormal && self.StateMachine.State == Player.StNormal)
                || normalJumpStarted
                || climbJumpCostPaid)) {
            inputJob.JumpBufferFrames = 0;
        }
    }

    private int PlayerStartDash(On.Celeste.Player.orig_StartDash orig, Player self) {
        int nextState = orig(self);
        if (ActiveInputJob is { } inputJob) {
            inputJob.DashBufferFrames = 0;
            inputJob.CrouchDashBufferFrames = 0;
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
        if (ActiveInputJob is { } inputJob) {
            inputJob.JumpBufferFrames = ScriptedInputBuffer.Consume(inputJob.JumpBufferFrames);
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
        if (body is not null && gymStepJob is { CurrentInput: not null } gymJob) {
            gymJob.DeathFrame = SnapshotCapture.Capture(self, gymJob.Episode.Frame + 1);
        }
        if (body is not null) interactiveSession?.CaptureDeath(self);
        return body;
    }

    private void LookoutRemoved(
        On.Celeste.Lookout.orig_Removed orig,
        Lookout self,
        Scene scene
    ) {
        // Lookout.Removed restores StNormal but does not StopInteracting. Read
        // the private flag before the original handler detaches the entity.
        bool wasInteracting = SnapshotCapture.IsLookoutInteracting(self);
        orig(self, scene);
        SnapshotCapture.ObserveLookoutRemoved(wasInteracting, scene);
    }

    private void PrepareSimulationFrame() {
        if (gymResetJob is not null) {
            PrepareGymReset();
            return;
        }
        if (job is null) {
            if (gymStepJob is not null) PrepareGymStepFrame();
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
            // token owns the entire request. Cancel only the fresh entry wipe
            // before applying the initial snapshot: waiting for it to clear
            // advances entities and clocks outside the scripted state range,
            // while later room transitions remain untouched after the job starts.
            if (RecordingLifecycle.ShouldCancelInitialEntryWipe(
                    captureSession?.IsCapturing == true,
                    job.Pending.Request.SkipTransitions)
                && Engine.Scene is Level level) {
                level.Wipe?.Cancel();
            }
            // The level loader may have removed a Lookout from a previous
            // request. Start the proof window only once this job owns its
            // freshly loaded player and room entities.
            SnapshotCapture.ResetLookoutLifecycleObservation();
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
        if (gymStepJob is not null) {
            FinishGymStepFrame();
            return;
        }
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

    private void PrepareGymReset() {
        GymResetJob reset = gymResetJob!;
        if (!reset.InPlace && ReferenceEquals(Engine.Scene, reset.SceneAtRequest)) return;
        if (Engine.Scene is not Level level || level.Session.Area.ID != reset.AreaId) return;
        Player? player = level.Tracker.GetEntity<Player>();
        if (player is null) return;

        SnapshotCapture.ResetLookoutLifecycleObservation();
        ApplyInitialSnapshot(player, reset.Pending.Request.InitialSnapshot);
        if (reset.Pending.Request.InitialSnapshot?.Pos is { Length: >= 2 }) {
            level.Camera.Position = player.CameraTarget;
        }
        GymEpisode episode = new(
            reset.AreaId,
            level.Session.Area.SID,
            level.Session.Level,
            reset.Pending.Request.MaxEpisodeFrames,
            reset.Pending.Request.IncludeEntities != false,
            reset.Pending.Request.IncludePlayerStates != false,
            reset.Pending.Request.FastMode
        );
        PlayerFrame frame = SnapshotCapture.Capture(player, 0);
        episode.LastPlayer = frame;
        gymEpisode = episode;
        gymResetJob = null;
        reset.Pending.Completion.SetResult(new CollectorResponse {
            Success = true,
            Observation = GymCapture.Capture(
                level,
                episode,
                frame,
                includeGeometry: true,
                terminated: false,
                truncated: false,
                success: false,
                terminationReason: null
            ),
            PlayerStates = [frame],
            FramesExecuted = 0
        });
    }

    private void PrepareGymStepFrame() {
        GymStepJob step = gymStepJob!;
        if (step.Index >= step.Pending.Request.Inputs.Count) {
            CompleteGymStep(
                step.LevelAtStart,
                step.Episode.LastPlayer
                    ?? throw new InvalidOperationException("gym episode has no player snapshot"),
                terminated: false,
                truncated: false,
                success: false,
                terminationReason: null
            );
            return;
        }
        step.PreviousInput = step.LastInput;
        step.CurrentInput = step.Pending.Request.Inputs[step.Index];
        if (step.CurrentInput.JumpPressed) step.JumpBufferFrames = 5;
        if (step.CurrentInput.DashPressed) step.DashBufferFrames = 5;
        if (step.CurrentInput.CrouchDashPressed) step.CrouchDashBufferFrames = 5;
    }

    private void FinishGymStepFrame() {
        GymStepJob step = gymStepJob!;
        if (step.CurrentInput is null) return;
        step.Index++;
        step.Episode.Frame++;

        Level level = Engine.Scene as Level ?? step.LevelAtStart;
        bool sceneChanged = Engine.Scene is not Level;
        bool roomChanged = sceneChanged
            || level.Session.Area.ID != step.Episode.AreaId
            || !string.Equals(level.Session.Level, step.Episode.StartRoom, StringComparison.Ordinal);
        Player? player = sceneChanged ? null : level.Tracker.GetEntity<Player>();
        bool dead = step.DeathFrame is not null || (!roomChanged && (player is null || player.Dead));
        bool truncated = !dead && !roomChanged && step.Episode.Frame >= step.Episode.MaxFrames;
        bool terminated = dead || roomChanged;
        bool success = roomChanged && !dead;
        string? reason = dead
            ? "death"
            : roomChanged
                ? "room_transition"
                : truncated
                    ? "max_episode_frames"
                    : null;

        bool completes = terminated
            || truncated
            || step.Index >= step.Pending.Request.Inputs.Count;
        PlayerFrame? frame = null;
        if (step.Episode.IncludePlayerStates || completes) {
            frame = step.DeathFrame
                ?? (player is null
                    ? SnapshotCapture.CaptureMissing(
                        step.Episode.LastPlayer
                            ?? throw new InvalidOperationException("gym episode has no player snapshot"),
                        step.Episode.Frame
                    )
                    : SnapshotCapture.Capture(player, step.Episode.Frame));
            step.Episode.LastPlayer = frame;
            if (step.Episode.IncludePlayerStates) step.PlayerStates.Add(frame);
        }
        step.DeathFrame = null;
        step.LastInput = step.CurrentInput;
        step.CurrentInput = null;
        if (step.JumpBufferFrames > 0) step.JumpBufferFrames--;
        if (step.DashBufferFrames > 0) step.DashBufferFrames--;
        if (step.CrouchDashBufferFrames > 0) step.CrouchDashBufferFrames--;

        if (completes) {
            CompleteGymStep(
                level,
                frame ?? throw new InvalidOperationException("gym step completed without a snapshot"),
                terminated,
                truncated,
                success,
                reason
            );
        }
    }

    private void CompleteGymStep(
        Level level,
        PlayerFrame frame,
        bool terminated,
        bool truncated,
        bool success,
        string? terminationReason
    ) {
        GymStepJob completed = gymStepJob!;
        gymStepJob = null;
        RestoreButtons();
        completed.Episode.Done |= terminated || truncated;
        completed.Episode.Terminated = terminated;
        completed.Episode.Truncated = truncated;
        completed.Episode.Success = success;
        completed.Episode.TerminationReason = terminationReason;
        completed.Pending.Completion.SetResult(new CollectorResponse {
            Success = true,
            Observation = GymCapture.Capture(
                level,
                completed.Episode,
                frame,
                includeGeometry: success,
                terminated,
                truncated,
                success,
                terminationReason
            ),
            PlayerStates = completed.PlayerStates,
            FramesExecuted = completed.Index
        });
    }

    private static PlayerFrame CaptureGymPlayer(Level level, GymEpisode episode, int frame) {
        Player? player = level.Tracker.GetEntity<Player>();
        PlayerFrame captured = player is null
            ? SnapshotCapture.CaptureMissing(
                episode.LastPlayer
                    ?? throw new InvalidOperationException("gym episode has no player snapshot"),
                frame
            )
            : SnapshotCapture.Capture(player, frame);
        episode.LastPlayer = captured;
        return captured;
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
            () => ActiveInputJob?.CurrentInput?.JumpHeld ?? false,
            () => ActiveInputJob?.JumpBufferFrames > 0,
            () => ActiveInputJob?.PreviousInput?.JumpHeld == true
                && ActiveInputJob?.CurrentInput?.JumpHeld != true);
        // Lookout.LookRoutine exits through MenuCancel. In the portable
        // protocol a jump press represents that shared confirm/cancel action,
        // so it must drive both virtual buttons. Unlike Jump, MenuCancel must
        // use the raw frame press: Player.Jump can consume the shared jump
        // buffer before LookRoutine reaches its own MenuCancel check.
        ReplaceNodes(Input.MenuCancel,
            () => ActiveInputJob?.CurrentInput?.JumpHeld ?? false,
            () => ActiveInputJob?.CurrentInput?.JumpPressed ?? false,
            () => ActiveInputJob?.PreviousInput?.JumpHeld == true
                && ActiveInputJob?.CurrentInput?.JumpHeld != true);
        ReplaceNodes(Input.Dash,
            () => ActiveInputJob?.CurrentInput?.DashPressed ?? false,
            () => ActiveInputJob?.DashBufferFrames > 0,
            () => ActiveInputJob?.PreviousInput?.DashPressed == true
                && ActiveInputJob?.CurrentInput?.DashPressed != true);
        ReplaceNodes(Input.CrouchDash,
            () => ActiveInputJob?.CurrentInput?.CrouchDashPressed ?? false,
            () => ActiveInputJob?.CrouchDashBufferFrames > 0,
            () => ActiveInputJob?.PreviousInput?.CrouchDashPressed == true
                && ActiveInputJob?.CurrentInput?.CrouchDashPressed != true);
        ReplaceNodes(Input.Grab,
            () => ActiveInputJob?.CurrentInput?.GrabHeld ?? false,
            () => ActiveInputJob?.PreviousInput?.GrabHeld != true
                && ActiveInputJob?.CurrentInput?.GrabHeld == true,
            () => ActiveInputJob?.PreviousInput?.GrabHeld == true
                && ActiveInputJob?.CurrentInput?.GrabHeld != true);
        ReplaceNodes(Input.Talk,
            () => ActiveInputJob?.CurrentInput?.TalkPressed ?? false,
            () => ActiveInputJob?.PreviousInput?.TalkPressed != true
                && ActiveInputJob?.CurrentInput?.TalkPressed == true,
            () => ActiveInputJob?.PreviousInput?.TalkPressed == true
                && ActiveInputJob?.CurrentInput?.TalkPressed != true);
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
        FrameInput? input = ActiveInputJob?.CurrentInput;
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
        if (snapshot.Facing is bool facing) player.Facing = facing ? Facings.Right : Facings.Left;
        if (snapshot.State is int state && state >= 0 && state < 26) player.StateMachine.State = state;
        if (snapshot.Ducking is bool ducking) player.Ducking = ducking;
    }

    private int SelectFastLoopFrameCount() {
        if (gymEpisode is null) return 0;
        if (gymStepJob is not null) {
            return GymFastLoopPolicy.SelectActiveStepFrameCount(
                gymEpisode.FastMode,
                gymStepJob.Pending.Request.Inputs.Count - gymStepJob.Index
            );
        }
        if (!server.TryPeek(out PendingRequest? pending)) return 0;
        return GymFastLoopPolicy.SelectFrameCount(
            gymEpisode.FastMode,
            stepAlreadyActive: false,
            gymEpisode.Id,
            pending?.Request
        );
    }

    private IScriptedInputJob? ActiveInputJob => job ?? (IScriptedInputJob?) gymStepJob;

    private interface IScriptedInputJob {
        FrameInput? CurrentInput { get; set; }
        FrameInput? PreviousInput { get; set; }
        FrameInput? LastInput { get; set; }
        int JumpBufferFrames { get; set; }
        int DashBufferFrames { get; set; }
        int CrouchDashBufferFrames { get; set; }
    }

    private sealed class SimulationJob(PendingRequest pending, Scene? sceneAtRequest, int areaId)
        : IScriptedInputJob {
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

    private sealed class GymResetJob(
        PendingRequest pending,
        Scene? sceneAtRequest,
        int areaId,
        bool inPlace = false
    ) {
        public PendingRequest Pending { get; } = pending;
        public Scene? SceneAtRequest { get; } = sceneAtRequest;
        public int AreaId { get; } = areaId;
        public bool InPlace { get; } = inPlace;
    }

    private sealed class GymStepJob(PendingRequest pending, GymEpisode episode, Level levelAtStart)
        : IScriptedInputJob {
        public PendingRequest Pending { get; } = pending;
        public GymEpisode Episode { get; } = episode;
        public Level LevelAtStart { get; } = levelAtStart;
        public List<PlayerFrame> PlayerStates { get; } = [];
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
