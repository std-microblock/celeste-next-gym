namespace Celeste.Mod.MicroblocksQolUtils;

/// <summary>Optional MiaoNet integration. Implemented without a hard DLL dependency.</summary>
public static class MiaoNetBridge {
    public static void Update(Level? level) {
        // The concrete reflection adapter is added with the minimap feature.
        _ = level;
    }
}

