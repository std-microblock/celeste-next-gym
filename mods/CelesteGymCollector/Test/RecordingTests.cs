using Xunit;
using Monocle;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Net;
using System.Net.Sockets;
using Microsoft.Xna.Framework;

namespace Celeste.Mod.CelesteGymCollector;

public sealed class RecordingTests : IDisposable {
    private static class FakeRandomizerInterop {
        public static bool Generate(object settings) => settings is not null;
    }

    private readonly string temporaryRoot = Path.Combine(
        Path.GetTempPath(),
        "celeste-gym-recording-tests",
        Guid.NewGuid().ToString("N")
    );

    [Theory]
    [InlineData(true, false, true)]
    [InlineData(true, true, true)]
    [InlineData(false, true, true)]
    [InlineData(false, false, false)]
    public void InitialEntryWipeIsCancelledOnlyForAnActiveCaptureOrSkippedTransitions(
        bool captureActive,
        bool skipTransitions,
        bool expected
    ) {
        Assert.Equal(
            expected,
            RecordingLifecycle.ShouldCancelInitialEntryWipe(captureActive, skipTransitions)
        );
    }

    [Fact]
    public void AuthenticationRequiresExactNonceAndProcessId() {
        RecordingSecurity.Authenticate("nonce", 42, "nonce", 42);
        Assert.Throws<UnauthorizedAccessException>(
            () => RecordingSecurity.Authenticate("nonce", 42, "wrong", 42)
        );
        Assert.Throws<UnauthorizedAccessException>(
            () => RecordingSecurity.Authenticate("nonce", 42, "nonce", 43)
        );
    }

    [Fact]
    public void TokenAndScenarioRejectInjectionCharacters() {
        string token = new('a', 32);
        Assert.Equal(token, RecordingSecurity.ValidateToken(token));
        Assert.Throws<InvalidOperationException>(() => RecordingSecurity.ValidateToken("short"));
        Assert.Throws<InvalidOperationException>(
            () => RecordingSecurity.ValidateToken(new string('a', 31) + "/")
        );
        Assert.Equal("dash-tech.2-4", RecordingSecurity.ValidateScenarioId("dash-tech.2-4"));
        Assert.Throws<InvalidOperationException>(
            () => RecordingSecurity.ValidateScenarioId("../escape")
        );
    }

    [Fact]
    public void SessionDirectoryCannotEscapeFixedRootOrReuseToken() {
        Directory.CreateDirectory(temporaryRoot);
        string root = RecordingSecurity.ResolveRecordingRoot(temporaryRoot);
        string token = new('b', 32);
        string session = RecordingSecurity.CreateSessionDirectory(root, "scenario", token);
        RecordingSecurity.EnsureContained(root, session);
        Assert.Throws<InvalidOperationException>(
            () => RecordingSecurity.CreateSessionDirectory(root, "scenario", token)
        );
        Assert.Throws<UnauthorizedAccessException>(
            () => RecordingSecurity.EnsureContained(root, Path.Combine(root, "..", "escape"))
        );
    }

    [Fact]
    public void PresentationTimelineRecordsSkippedUpdatesAndRepeatedStates() {
        RecordingTimeline timeline = new(0, 5);
        RecordingFrameManifest first = timeline.AddFrame(0, 10, "frames/0.bgra", "a", 4);
        RecordingFrameManifest skipped = timeline.AddFrame(3, 20, "frames/1.bgra", "b", 4);
        RecordingFrameManifest repeated = timeline.AddFrame(3, 30, "frames/2.bgra", "c", 4);
        RecordingFrameManifest final = timeline.AddFrame(5, 40, "frames/3.bgra", "d", 4);

        Assert.Equal(0, first.RenderIndex);
        Assert.Equal(new StateIndexRange(1, 2), skipped.UnpresentedUpdatesBefore);
        Assert.True(repeated.RepeatedStatePresentation);
        Assert.Equal(new StateIndexRange(4, 4), final.UnpresentedUpdatesBefore);
        Assert.Equal(1, timeline.RepeatedPresentationCount);
        Assert.True(timeline.FinalStatePresented);
        Assert.Equal([new StateIndexRange(1, 2), new StateIndexRange(4, 4)], timeline.UnpresentedUpdateRanges);
    }

    [Fact]
    public void PresentationTimelineRejectsBackwardStateOrTimestamp() {
        RecordingTimeline timeline = new(0, 2);
        timeline.AddFrame(1, 20, "frame", "hash", 4);
        Assert.Throws<InvalidOperationException>(
            () => timeline.AddFrame(0, 30, "frame", "hash", 4)
        );
        Assert.Throws<InvalidOperationException>(
            () => timeline.AddFrame(1, 19, "frame", "hash", 4)
        );
    }

    [Fact]
    public void PresentationTimelineWaitsSixtyFramesAfterTheFinalState() {
        RecordingTimeline timeline = new(0, 1);
        timeline.AddFrame(1, 0, "frames/0.bgra", "a", 4);

        Assert.False(timeline.HasPresentedFinalStateTail(60));
        for (int frame = 1; frame < 60; frame++) {
            timeline.AddFrame(1, frame, $"frames/{frame}.bgra", "a", 4);
        }
        Assert.False(timeline.HasPresentedFinalStateTail(60));

        timeline.AddFrame(1, 60, "frames/60.bgra", "a", 4);
        Assert.True(timeline.HasPresentedFinalStateTail(60));
        Assert.Throws<ArgumentOutOfRangeException>(
            () => timeline.HasPresentedFinalStateTail(-1)
        );
    }

    [Fact]
    public void DirectJumpConsumptionClearsTheEntireScriptedBuffer() {
        Assert.Equal(0, ScriptedInputBuffer.Consume(5));
        Assert.Equal(0, ScriptedInputBuffer.Consume(0));
        Assert.Throws<ArgumentOutOfRangeException>(() => ScriptedInputBuffer.Consume(-1));
    }

    [Fact]
    public async Task TcpConnectionProtocolProcessesMultipleJsonLinesInOrder() {
        using StringReader reader = new("{\"request\":1}\n{\"request\":2}\n");
        using StringWriter writer = new();
        int sequence = 0;
        await CollectorConnectionProtocol.RunAsync(
            reader,
            writer,
            (line, _) => Task.FromResult($"{{\"sequence\":{++sequence},\"echo\":{line}}}"),
            CancellationToken.None
        );

        string[] responses = writer.ToString().Split(
            Environment.NewLine,
            StringSplitOptions.RemoveEmptyEntries
        );
        Assert.Equal(2, responses.Length);
        Assert.Contains("\"sequence\":1", responses[0]);
        Assert.Contains("\"request\":1", responses[0]);
        Assert.Contains("\"sequence\":2", responses[1]);
        Assert.Contains("\"request\":2", responses[1]);
    }

    [Theory]
    [InlineData(1)]
    [InlineData(16)]
    [InlineData(4096)]
    public void FastLoopAcceptsSupportedGymBatchesInBoundedOuterTickBursts(int frameCount) {
        string episodeId = new('a', 32);
        CollectorRequest request = new() {
            Command = "gym_step",
            EpisodeId = episodeId,
            Inputs = Enumerable.Range(0, frameCount).Select(_ => new FrameInput()).ToList()
        };

        Assert.Equal(
            Math.Min(frameCount, GymFastLoopPolicy.MaximumUpdatesPerOuterTick),
            GymFastLoopPolicy.SelectFrameCount(true, false, episodeId, request)
        );
    }

    [Theory]
    [InlineData(1, 1)]
    [InlineData(256, 256)]
    [InlineData(257, 256)]
    [InlineData(4096, 256)]
    public void FastLoopContinuesAnActiveStepInBoundedBursts(int remaining, int expected) {
        Assert.Equal(
            expected,
            GymFastLoopPolicy.SelectActiveStepFrameCount(true, remaining)
        );
        Assert.Equal(0, GymFastLoopPolicy.SelectActiveStepFrameCount(false, remaining));
    }

    [Fact]
    public void FastLoopCannotAffectDefaultOrUnrelatedGameRequests() {
        string episodeId = new('a', 32);
        CollectorRequest step = new() {
            Command = "gym_step",
            EpisodeId = episodeId,
            Inputs = [new FrameInput()]
        };
        CollectorRequest observe = new() {
            Command = "gym_observe",
            EpisodeId = episodeId
        };

        Assert.Equal(0, GymFastLoopPolicy.SelectFrameCount(false, false, episodeId, step));
        Assert.Equal(0, GymFastLoopPolicy.SelectFrameCount(true, true, episodeId, step));
        Assert.Equal(0, GymFastLoopPolicy.SelectFrameCount(true, false, "different", step));
        Assert.Equal(0, GymFastLoopPolicy.SelectFrameCount(true, false, episodeId, observe));
        Assert.Equal(0, GymFastLoopPolicy.SelectFrameCount(true, false, episodeId, null));
    }

    [Fact]
    public void ActiveGymEpisodeParksBetweenExplicitResetAndStepWork() {
        Assert.False(GymIdlePolicy.ShouldPark(false, false, false, false));
        Assert.True(GymIdlePolicy.ShouldPark(true, false, false, false));
        Assert.False(GymIdlePolicy.ShouldPark(true, true, false, false));
        Assert.False(GymIdlePolicy.ShouldPark(true, false, true, false));
        Assert.False(GymIdlePolicy.ShouldPark(true, false, false, true));
    }

    [Theory]
    [InlineData(0)]
    [InlineData(4097)]
    public void FastLoopRejectsUnsupportedBatchSizes(int frameCount) {
        string episodeId = new('a', 32);
        CollectorRequest request = new() {
            Command = "gym_step",
            EpisodeId = episodeId,
            Inputs = Enumerable.Range(0, frameCount).Select(_ => new FrameInput()).ToList()
        };

        Assert.Equal(
            0,
            GymFastLoopPolicy.SelectFrameCount(true, false, episodeId, request)
        );
    }

    [Fact]
    public void FastLoopServicesWallClockSubsystemsOncePerOuterTick() {
        bool consumed = false;
        Assert.True(GymFastLoopPolicy.ConsumeOuterTickService(true, ref consumed));
        Assert.True(consumed);
        Assert.False(GymFastLoopPolicy.ConsumeOuterTickService(true, ref consumed));

        // Outside an accelerated Tick the policy must not modify normal game behavior.
        consumed = true;
        Assert.True(GymFastLoopPolicy.ConsumeOuterTickService(false, ref consumed));
        Assert.True(consumed);
    }

    [Fact]
    public void GymEpisodeSuppressesAutoSplitterAcrossAllTicks() {
        bool consumed = false;
        Assert.False(GymFastLoopPolicy.ShouldRunAutoSplitter(true, false, ref consumed));
        Assert.False(consumed);
        Assert.False(GymFastLoopPolicy.ShouldRunAutoSplitter(true, true, ref consumed));
        Assert.False(consumed);

        Assert.True(GymFastLoopPolicy.ShouldRunAutoSplitter(false, true, ref consumed));
        Assert.True(consumed);
        Assert.False(GymFastLoopPolicy.ShouldRunAutoSplitter(false, true, ref consumed));
    }

    [Fact]
    public void GymEpisodeSuppressesNativeControllerRumble() {
        Assert.False(GymFastLoopPolicy.ShouldRunRumble(true));
        Assert.True(GymFastLoopPolicy.ShouldRunRumble(false));
    }

    [Fact]
    public void GymRandomResetRestoresTheAuthoritativeSequenceAndClearsNestedState() {
        const int seed = 8675309;
        Calc.PushRandom(1234);

        GymRandomPolicy.Reset(seed);
        int[] first = Enumerable.Range(0, 8).Select(_ => Calc.Random.Next()).ToArray();

        Calc.PushRandom(4321);
        _ = Calc.Random.Next();
        GymRandomPolicy.Reset(seed);
        int[] second = Enumerable.Range(0, 8).Select(_ => Calc.Random.Next()).ToArray();

        Assert.Equal(first, second);
        Assert.Throws<InvalidOperationException>(() => Calc.PopRandom());
    }

    [Fact]
    public void GymAreaModeDefaultsToAAndMatchesBothIdAndMode() {
        CollectorRequest request = new();
        Assert.Equal(0, request.AreaMode);

        AreaKey bSide = GymAreaIdentity.CreateKey(4, 1);
        Assert.Equal(4, bSide.ID);
        Assert.Equal(1, (int) bSide.Mode);
        Assert.True(GymAreaIdentity.Matches(bSide, 4, 1));
        Assert.False(GymAreaIdentity.Matches(bSide, 4, 0));
        Assert.False(GymAreaIdentity.Matches(bSide, 2, 1));
        Assert.Throws<InvalidOperationException>(() => GymAreaIdentity.CreateKey(4, -1));
        Assert.Throws<InvalidOperationException>(() => GymAreaIdentity.CreateKey(4, 3));
    }

    [Fact]
    public void GymAreaIdentityRejectsSameNumericAreaWithDifferentSid() {
        AreaKey reusedKey = GymAreaIdentity.CreateKey(42, 0);
        Assert.False(
            GymAreaIdentity.CanResetInPlace(
                reusedKey,
                "randomizer/old-seed",
                reusedKey,
                "randomizer/new-seed"
            )
        );
        Assert.True(
            GymAreaIdentity.CanResetInPlace(
                reusedKey,
                "randomizer/new-seed",
                reusedKey,
                "randomizer/new-seed"
            )
        );
        Assert.True(
            GymAreaIdentity.Matches(
                42,
                0,
                "randomizer/new-seed",
                42,
                0,
                "randomizer/new-seed"
            )
        );
        Assert.False(
            GymAreaIdentity.Matches(
                42,
                0,
                "randomizer/old-seed",
                42,
                0,
                "randomizer/new-seed"
            )
        );
    }

    [Theory]
    [InlineData("seed_42", null, null, "seed_42", "Short", "Normal")]
    [InlineData(" abc-123 ", "Medium", "Hard", "abc-123", "Medium", "Hard")]
    public void RandomizerRequestPolicyNormalizesSafeSettings(
        string seed,
        string? length,
        string? difficulty,
        string expectedSeed,
        string expectedLength,
        string expectedDifficulty
    ) {
        RandomizerGenerationOptions result = RandomizerRequestPolicy.Validate(
            seed,
            length,
            difficulty
        );
        Assert.Equal(expectedSeed, result.Seed);
        Assert.Equal(expectedLength, result.Length);
        Assert.Equal(expectedDifficulty, result.Difficulty);
    }

    [Theory]
    [InlineData("")]
    [InlineData("../escape")]
    [InlineData("contains space")]
    [InlineData("seed.with.dot")]
    public void RandomizerRequestPolicyRejectsUnsafeSeeds(string seed) {
        Assert.Throws<InvalidOperationException>(
            () => RandomizerRequestPolicy.Validate(seed, null, null)
        );
    }

    [Fact]
    public void RandomizerReflectionContractRequiresExactStaticArity() {
        MethodInfo method = RandomizerReflectionApi.RequireMethod(
            typeof(FakeRandomizerInterop),
            "Generate",
            1
        );
        Assert.Equal(nameof(FakeRandomizerInterop.Generate), method.Name);
        Assert.Throws<InvalidOperationException>(
            () => RandomizerReflectionApi.RequireMethod(
                typeof(FakeRandomizerInterop),
                "Generate",
                0
            )
        );
    }

    [Fact]
    public void RandomizerGenerationWaitsUntilEmptyStagingSceneIsActive() {
        object oldGameplay = new();
        object staging = new();

        Assert.False(
            RandomizerGenerationStagingPolicy.ShouldStart(oldGameplay, staging, started: false)
        );
        Assert.True(
            RandomizerGenerationStagingPolicy.ShouldStart(staging, staging, started: false)
        );
        Assert.False(
            RandomizerGenerationStagingPolicy.ShouldStart(staging, staging, started: true)
        );
    }

    [Fact]
    public void GymExitGoalRequiresTheSelectedBoundaryAndAperture() {
        Rectangle bounds = new(100, 200, 320, 180);
        PlayerFrame selected = new() {
            Pos = [260, 199],
            Speed = [0, -90]
        };
        PlayerFrame wrongHole = new() {
            Pos = [300, 199],
            Speed = [0, -90]
        };
        PlayerFrame wrongSide = new() {
            Pos = [421, 260],
            Speed = [90, 0]
        };

        Assert.True(GymExitGoalPolicy.Matches("up", [252, 268], [260, 195], bounds, selected));
        Assert.False(GymExitGoalPolicy.Matches("up", [252, 268], [260, 195], bounds, wrongHole));
        Assert.False(GymExitGoalPolicy.Matches("up", [252, 268], [260, 195], bounds, wrongSide));
        Assert.True(GymExitGoalPolicy.Matches(null, null, null, bounds, wrongSide));
    }

    [Fact]
    public void GymExitGoalWorldSelectsOneEightPixelHoleWithoutAnAperture() {
        Rectangle bounds = new(0, 0, 320, 180);
        PlayerFrame centered = new() { Pos = [324, 84], Speed = [120, 0] };
        PlayerFrame adjacent = new() { Pos = [324, 92], Speed = [120, 0] };

        Assert.True(GymExitGoalPolicy.Matches("right", null, [324, 84], bounds, centered));
        Assert.False(GymExitGoalPolicy.Matches("right", null, [324, 84], bounds, adjacent));
    }

    [Fact]
    public void GymOverlayIsBoundedAndOnlyRendersForVisibleActors() {
        string[] lines = GymOverlayPolicy.Normalize([
            "map=4H-GoldenRidge room=a-00",
            "checkpoint=12 targets=40",
            "attempted=12 passed=8 failed=4 pending=28",
            "outcome=retry"
        ]);

        Assert.Equal(4, lines.Length);
        Assert.True(GymOverlayPolicy.ShouldRender(headlessActor: false, lines));
        Assert.False(GymOverlayPolicy.ShouldRender(headlessActor: true, lines));
        Assert.False(GymOverlayPolicy.ShouldRender(false, []));
        Assert.Throws<InvalidOperationException>(() => GymOverlayPolicy.Normalize(
            Enumerable.Repeat("line", GymOverlayPolicy.MaximumLines + 1).ToArray()
        ));
        Assert.Throws<InvalidOperationException>(() => GymOverlayPolicy.Normalize(["bad\nline"]));
        Assert.Throws<InvalidOperationException>(() => GymOverlayPolicy.Normalize([
            new string('x', GymOverlayPolicy.MaximumCharactersPerLine + 1)
        ]));
    }

    [Fact]
    public void CollectorRequestSeedIsOptional() {
        CollectorRequest request = new();
        Assert.Null(request.Seed);

        request.Seed = int.MinValue;
        Assert.Equal(int.MinValue, request.Seed);
    }

    [Fact]
    public void GymResetClearsGlobalEngineStatesThatSkipPlayerUpdates() {
        Engine.FreezeTimer = 0.5f;
        Engine.DashAssistFreeze = true;
        Engine.DashAssistFreezePress = true;

        GymResetPolicy.ClearEngineUpdateBlockers();

        Assert.Equal(0f, Engine.FreezeTimer);
        Assert.False(Engine.DashAssistFreeze);
        Assert.False(Engine.DashAssistFreezePress);
    }

    [Fact]
    public void InPlaceGymResetCancelsTheRoomTransitionThatOwnsLevelUpdates() {
        Level level = (Level) RuntimeHelpers.GetUninitializedObject(typeof(Level));
        FieldInfo transition = typeof(Level).GetField(
            "transition",
            BindingFlags.Instance | BindingFlags.NonPublic
        ) ?? throw new MissingFieldException(typeof(Level).FullName, "transition");
        transition.SetValue(
            level,
            RuntimeHelpers.GetUninitializedObject(transition.FieldType)
        );
        level.Paused = true;

        Assert.True(level.Transitioning);
        GymResetPolicy.PrepareInPlaceLevel(level);

        Assert.False(level.Transitioning);
        Assert.False(level.Paused);
    }

    [Fact]
    public void DefaultCollectorPortFallsBackWhenThePreferredPortIsOccupied() {
        using TcpListener occupied = new(IPAddress.Loopback, 0);
        occupied.Start();
        int preferredPort = ((IPEndPoint) occupied.LocalEndpoint).Port;

        using CollectorListenerBinding binding = CollectorListenerBinder.Bind(
            preferredPort,
            allowFallback: true
        );

        Assert.True(binding.FellBack);
        Assert.Equal(SocketError.AddressAlreadyInUse, binding.PreferredPortFailure);
        Assert.NotEqual(preferredPort, binding.Port);
        Assert.InRange(binding.Port, 1, 65535);
    }

    [Fact]
    public void ExplicitCollectorPortDoesNotFallBackWhenItIsOccupied() {
        using TcpListener occupied = new(IPAddress.Loopback, 0);
        occupied.Start();
        int explicitPort = ((IPEndPoint) occupied.LocalEndpoint).Port;

        InvalidOperationException error = Assert.Throws<InvalidOperationException>(
            () => CollectorListenerBinder.Bind(explicitPort, allowFallback: false)
        );

        Assert.Contains($"127.0.0.1:{explicitPort}", error.Message);
        SocketException socketError = Assert.IsType<SocketException>(error.InnerException);
        Assert.Equal(SocketError.AddressAlreadyInUse, socketError.SocketErrorCode);
    }

    public void Dispose() {
        if (Directory.Exists(temporaryRoot)) Directory.Delete(temporaryRoot, recursive: true);
    }
}
