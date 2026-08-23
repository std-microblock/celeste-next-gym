using System.ComponentModel;

namespace Celeste.Mod.MicroblocksQolUtils;

public enum MiniMapShape {
    Circle,
    Square
}

public enum MiniMapNameMode {
    None,
    WatchedOnly,
    Everyone
}

public enum RecordingPolicy {
    EveryRoom,
    GoldenRunsOnly
}

public enum BgmRecordingMode {
    CaptureGameMix,
    SfxOnlyWithPostMix
}

public sealed class QolSettings : EverestModuleSettings {
    [DefaultValue(true)]
    public bool Enabled { get; set; } = true;

    [DefaultValue("Microsoft YaHei UI")]
    public string FontFamily { get; set; } = "Microsoft YaHei UI";

    [DefaultValue("")]
    public string FontFile { get; set; } = "";

    [DefaultValue(true)]
    public bool MiniMapEnabled { get; set; } = true;

    [SettingRange(96, 384)]
    [DefaultValue(220)]
    public int MiniMapSize { get; set; } = 220;

    [SettingRange(1, 8)]
    [DefaultValue(3)]
    public int MiniMapZoom { get; set; } = 3;

    [DefaultValue(MiniMapShape.Circle)]
    public MiniMapShape MiniMapShape { get; set; } = MiniMapShape.Circle;

    [DefaultValue(true)]
    public bool MiniMapBackground { get; set; } = true;

    [SettingRange(0, 10)]
    [DefaultValue(6)]
    public int MiniMapBackgroundOpacity { get; set; } = 6;

    [DefaultValue(true)]
    public bool ShowMiaoNetPlayers { get; set; } = true;

    [DefaultValue(MiniMapNameMode.WatchedOnly)]
    public MiniMapNameMode MiniMapNames { get; set; } = MiniMapNameMode.WatchedOnly;

    [DefaultValue(true)]
    public bool HideMiaoNetOffscreenNames { get; set; } = true;

    [DefaultValue(true)]
    public bool ShowRoomsRemaining { get; set; } = true;

    [DefaultValue(true)]
    public bool ShowMapPlayerCount { get; set; } = true;

    [DefaultValue(true)]
    public bool ShowClock { get; set; } = true;

    [DefaultValue(true)]
    public bool WatchedPlayerNotifications { get; set; } = true;

    public List<string> WatchedPlayers { get; set; } = [];

    [DefaultValue(false)]
    public bool RemoveRoomTransitions { get; set; }

    [DefaultValue(true)]
    public bool ShowFps { get; set; } = true;

    [DefaultValue(false)]
    public bool EnableFrameProfiler { get; set; }

    [SettingRange(20, 250)]
    [DefaultValue(34)]
    public int FrameSpikeThresholdMs { get; set; } = 34;

    [DefaultValue(false)]
    public bool AutoRecorderEnabled { get; set; }

    [DefaultValue(RecordingPolicy.EveryRoom)]
    public RecordingPolicy RecordingPolicy { get; set; } = RecordingPolicy.EveryRoom;

    [DefaultValue(BgmRecordingMode.CaptureGameMix)]
    public BgmRecordingMode BgmMode { get; set; } = BgmRecordingMode.CaptureGameMix;

    [DefaultValue(60)]
    [SettingRange(30, 120)]
    public int RecordingFrameRate { get; set; } = 60;

    [DefaultValue(12000)]
    [SettingRange(2000, 50000)]
    public int RecordingBitrateKbps { get; set; } = 12000;

    [DefaultValue("")]
    public string RecordingDirectory { get; set; } = "";

    [DefaultValue("Celeste")]
    public string RecordingWindowTitle { get; set; } = "Celeste";

    [DefaultValue("auto")]
    public string RecordingEncoder { get; set; } = "auto";

    [DefaultValue("")]
    public string BgmEventMapFile { get; set; } = "";
}
