using System.Text.Json.Serialization;

namespace Celeste.Mod.CelesteGymCollector;

internal sealed class RecordingTimeline(int startStateIndex, int endStateIndex) {
    private int? previousStateIndex;
    private long? previousTimestampNanoseconds;

    public int StartStateIndex { get; } = startStateIndex;
    public int EndStateIndex { get; } = endStateIndex;
    public List<RecordingFrameManifest> Frames { get; } = [];
    public List<StateIndexRange> UnpresentedUpdateRanges { get; } = [];
    public int RepeatedPresentationCount { get; private set; }
    public bool FinalStatePresented { get; private set; }

    public RecordingFrameManifest AddFrame(
        int stateIndex,
        long timestampNanoseconds,
        string relativePath,
        string sha256,
        long byteSize
    ) {
        if (stateIndex < StartStateIndex) {
            throw new InvalidOperationException("cannot record a state before the requested range");
        }
        if (previousStateIndex is int previous && stateIndex < previous) {
            throw new InvalidOperationException("presentation state index moved backwards");
        }
        if (previousTimestampNanoseconds is long previousTimestamp
            && timestampNanoseconds < previousTimestamp) {
            throw new InvalidOperationException("presentation timestamp moved backwards");
        }

        bool repeated = previousStateIndex == stateIndex;
        if (repeated) RepeatedPresentationCount++;

        int firstExpected = previousStateIndex is int prior ? prior + 1 : StartStateIndex;
        StateIndexRange? skipped = null;
        if (stateIndex > firstExpected) {
            skipped = new StateIndexRange(firstExpected, stateIndex - 1);
            UnpresentedUpdateRanges.Add(skipped);
        }

        RecordingFrameManifest frame = new() {
            RenderIndex = Frames.Count,
            StateIndex = stateIndex,
            TimestampNanoseconds = timestampNanoseconds,
            RelativePath = relativePath.Replace('\\', '/'),
            Sha256 = sha256,
            ByteSize = byteSize,
            RepeatedStatePresentation = repeated,
            UnpresentedUpdatesBefore = skipped
        };
        Frames.Add(frame);
        previousStateIndex = stateIndex;
        previousTimestampNanoseconds = timestampNanoseconds;
        if (stateIndex == EndStateIndex) FinalStatePresented = true;
        return frame;
    }

    public void CompleteUnpresentedRanges(int latestStateIndex) {
        int firstMissing = previousStateIndex is int previous
            ? previous + 1
            : StartStateIndex;
        int lastMissing = Math.Min(latestStateIndex, EndStateIndex);
        if (lastMissing >= firstMissing) {
            StateIndexRange range = new(firstMissing, lastMissing);
            if (UnpresentedUpdateRanges.Count == 0
                || UnpresentedUpdateRanges[^1] != range) {
                UnpresentedUpdateRanges.Add(range);
            }
        }
    }
}

public sealed record StateIndexRange(
    [property: JsonPropertyName("start_state_index")] int StartStateIndex,
    [property: JsonPropertyName("end_state_index")] int EndStateIndex
);

internal sealed class RecordingFrameManifest {
    [JsonPropertyName("render_index")]
    public int RenderIndex { get; set; }

    [JsonPropertyName("state_index")]
    public int StateIndex { get; set; }

    [JsonPropertyName("timestamp_ns")]
    public long TimestampNanoseconds { get; set; }

    [JsonPropertyName("path")]
    public string RelativePath { get; set; } = "";

    [JsonPropertyName("sha256")]
    public string Sha256 { get; set; } = "";

    [JsonPropertyName("bytes")]
    public long ByteSize { get; set; }

    [JsonPropertyName("repeated_state_presentation")]
    public bool RepeatedStatePresentation { get; set; }

    [JsonPropertyName("unpresented_updates_before")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public StateIndexRange? UnpresentedUpdatesBefore { get; set; }
}
