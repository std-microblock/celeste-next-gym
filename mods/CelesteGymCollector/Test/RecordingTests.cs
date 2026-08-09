using Xunit;

namespace Celeste.Mod.CelesteGymCollector;

public sealed class RecordingTests : IDisposable {
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

    public void Dispose() {
        if (Directory.Exists(temporaryRoot)) Directory.Delete(temporaryRoot, recursive: true);
    }
}
