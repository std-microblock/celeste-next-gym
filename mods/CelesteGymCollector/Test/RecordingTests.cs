using Xunit;

namespace Celeste.Mod.CelesteGymCollector;

public sealed class RecordingTests : IDisposable {
    private readonly string temporaryRoot = Path.Combine(
        Path.GetTempPath(),
        "celeste-gym-recording-tests",
        Guid.NewGuid().ToString("N")
    );

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

    public void Dispose() {
        if (Directory.Exists(temporaryRoot)) Directory.Delete(temporaryRoot, recursive: true);
    }
}
