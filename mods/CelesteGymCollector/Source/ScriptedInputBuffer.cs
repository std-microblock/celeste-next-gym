namespace Celeste.Mod.CelesteGymCollector;

internal static class ScriptedInputBuffer {
    public static int Consume(int bufferedFrames) {
        if (bufferedFrames < 0) throw new ArgumentOutOfRangeException(nameof(bufferedFrames));
        return 0;
    }
}
