namespace Celeste.Mod.MicroblocksQolUtils;

public sealed record RecordingClip(
    string Source,
    double DurationSeconds,
    string MusicEvent,
    int MusicTimelineMilliseconds
);

public sealed record RecordingTimelineSnapshot(IReadOnlyList<RecordingClip> Clips) {
    public RecordingTimelineSnapshot Copy() => new(Clips.ToArray());
}
