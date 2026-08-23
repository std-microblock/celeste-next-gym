using Microsoft.Xna.Framework;

namespace Celeste.Mod.MicroblocksQolUtils;

public static class AutoRecorder {
    private static readonly List<RecordingClip> ActivePrefix = [];
    private static readonly List<Task> PendingStops = [];
    private static readonly HashSet<string> TemporaryFiles = new(StringComparer.OrdinalIgnoreCase);
    private static readonly HashSet<string> ProtectedSources = new(StringComparer.OrdinalIgnoreCase);
    private static NativeRecordingSegment? current;
    private static RecordingTimelineSnapshot? respawnAnchor;
    private static Vector2? observedRespawnPoint;
    private static string roomKey = "";
    private static string roomName = "";
    private static string areaSid = "";
    private static bool completing;

    public static void Load(string directory) {
        _ = directory;
        On.Celeste.Player.Die += PlayerDie;
        On.Celeste.Level.TransitionTo += LevelTransitionTo;
        On.Celeste.Level.RegisterAreaComplete += RegisterAreaComplete;
        SpeedrunToolBridge.Load();
    }

    public static void Unload() {
        SpeedrunToolBridge.Unload();
        On.Celeste.Level.RegisterAreaComplete -= RegisterAreaComplete;
        On.Celeste.Level.TransitionTo -= LevelTransitionTo;
        On.Celeste.Player.Die -= PlayerDie;
        StopAndReset(deleteAll: true);
    }

    public static void Update(Level level) {
        QolSettings settings = MicroblocksQolUtilsModule.Settings;
        if (!settings.AutoRecorderEnabled || !OperatingSystem.IsWindows()) {
            if (current is not null || roomKey.Length > 0) StopAndReset(deleteAll: true);
            return;
        }

        Player? player = level.Tracker.GetEntity<Player>();
        if (player is null) return;
        string key = $"{level.Session.Area.SID}|{(int)level.Session.Area.Mode}|{level.Session.Level}";
        if (!string.Equals(key, roomKey, StringComparison.Ordinal)) {
            if (roomKey.Length > 0 && !completing) StopAndReset(deleteAll: true);
            BeginRoom(level);
        }
        if (!ShouldRecord(player, settings)) return;

        Vector2? respawn = level.Session.RespawnPoint;
        if (observedRespawnPoint.HasValue && respawn.HasValue && Vector2.DistanceSquared(observedRespawnPoint.Value, respawn.Value) > 0.01f) {
            respawnAnchor = CaptureTimeline(level);
            if (respawnAnchor is not null) Protect(respawnAnchor);
        }
        observedRespawnPoint = respawn;

        if (current is null && !player.Dead && !level.Transitioning) {
            StartSegment();
        }
    }

    public static RecordingTimelineSnapshot? CaptureTimeline(Level level) {
        if (current is null || !string.Equals(level.Session.Level, roomName, StringComparison.Ordinal)) return null;
        List<RecordingClip> clips = [.. ActivePrefix, CurrentClip()];
        RecordingTimelineSnapshot snapshot = new(clips);
        Protect(snapshot);
        return snapshot;
    }

    public static void RestoreTimeline(Level level, RecordingTimelineSnapshot snapshot) {
        if (!MicroblocksQolUtilsModule.Settings.AutoRecorderEnabled) return;
        StopCurrent(deleteIfUnprotected: true);
        ActivePrefix.Clear();
        ActivePrefix.AddRange(snapshot.Clips);
        Protect(snapshot);
        roomKey = $"{level.Session.Area.SID}|{(int)level.Session.Area.Mode}|{level.Session.Level}";
        roomName = level.Session.Level;
        areaSid = level.Session.Area.SID;
        observedRespawnPoint = level.Session.RespawnPoint;
    }

    private static PlayerDeadBody? PlayerDie(
        On.Celeste.Player.orig_Die orig,
        Player self,
        Vector2 direction,
        bool evenIfInvincible,
        bool registerDeathInStats
    ) {
        PlayerDeadBody? body = orig(self, direction, evenIfInvincible, registerDeathInStats);
        if (body is null || current is null) return body;
        StopCurrent(deleteIfUnprotected: true);
        ActivePrefix.Clear();
        if (respawnAnchor is not null) ActivePrefix.AddRange(respawnAnchor.Clips);
        return body;
    }

    private static void LevelTransitionTo(On.Celeste.Level.orig_TransitionTo orig, Level self, LevelData next, Vector2 direction) {
        Complete(self);
        orig(self, next, direction);
    }

    private static void RegisterAreaComplete(On.Celeste.Level.orig_RegisterAreaComplete orig, Level self) {
        Complete(self);
        orig(self);
    }

    private static void BeginRoom(Level level) {
        completing = false;
        roomKey = $"{level.Session.Area.SID}|{(int)level.Session.Area.Mode}|{level.Session.Level}";
        roomName = level.Session.Level;
        areaSid = level.Session.Area.SID;
        observedRespawnPoint = level.Session.RespawnPoint;
        respawnAnchor = null;
    }

    private static void StartSegment() {
        string tempRoot = Path.Combine(ResolveRecordingRoot(), ".working", Sanitize(roomKey));
        Directory.CreateDirectory(tempRoot);
        string path = Path.Combine(tempRoot, $"segment-{DateTime.UtcNow:yyyyMMdd-HHmmss-fff}-{Guid.NewGuid():N}.mkv");
        current = NativeRecordingSegment.Start(path);
        if (current is not null) TemporaryFiles.Add(path);
    }

    private static void StopCurrent(bool deleteIfUnprotected) {
        NativeRecordingSegment? session = current;
        current = null;
        if (session is null) return;
        Task stop = session.StopAsync();
        PendingStops.Add(stop);
        if (deleteIfUnprotected && !ProtectedSources.Contains(session.Path)) {
            _ = stop.ContinueWith(_ => {
                try { File.Delete(session.Path); } catch { }
            }, TaskScheduler.Default);
        }
    }

    private static void Complete(Level level) {
        if (completing || current is null || !string.Equals(level.Session.Level, roomName, StringComparison.Ordinal)) return;
        completing = true;
        RecordingClip finalClip = CurrentClip();
        StopCurrent(deleteIfUnprotected: false);
        List<RecordingClip> clips = [.. ActivePrefix, finalClip];
        string outputRoot = ResolveRecordingRoot();
        string output = Path.Combine(
            outputRoot,
            Sanitize(areaSid),
            $"{DateTime.Now:yyyyMMdd-HHmmss}-{Sanitize(roomName)}.mp4"
        );
        List<Task> stops = [.. PendingStops];
        string[] temporary = TemporaryFiles.ToArray();
        _ = NativeRecordingFinalizer.FinishAsync(
            clips,
            stops,
            temporary,
            output
        );
        ResetStateWithoutDeleting();
    }

    private static RecordingClip CurrentClip() {
        NativeRecordingSegment session = current ?? throw new InvalidOperationException("No active recording segment.");
        return new RecordingClip(
            session.Path,
            Math.Max(0.05, session.ElapsedSeconds),
            session.MusicStart.Event,
            session.MusicStart.TimelineMilliseconds
        );
    }

    private static bool ShouldRecord(Player player, QolSettings settings) {
        if (settings.RecordingPolicy == RecordingPolicy.EveryRoom) return true;
        return player.Leader.Followers.Any(follower => follower.Entity is Strawberry { Golden: true });
    }

    private static void Protect(RecordingTimelineSnapshot snapshot) {
        foreach (RecordingClip clip in snapshot.Clips) ProtectedSources.Add(clip.Source);
    }

    private static string ResolveRecordingRoot() {
        string configured = Environment.ExpandEnvironmentVariables(MicroblocksQolUtilsModule.Settings.RecordingDirectory.Trim());
        if (configured.Length > 0) return configured;
        string videos = Environment.GetFolderPath(Environment.SpecialFolder.MyVideos);
        if (videos.Length == 0) videos = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        return Path.Combine(videos, "Celeste", "microblocks-qol-recordings");
    }

    private static void StopAndReset(bool deleteAll) {
        StopCurrent(deleteIfUnprotected: deleteAll);
        if (deleteAll) {
            Task[] stops = PendingStops.ToArray();
            string[] files = TemporaryFiles.ToArray();
            _ = Task.WhenAll(stops).ContinueWith(_ => {
                foreach (string file in files) {
                    try { File.Delete(file); } catch { }
                }
            }, TaskScheduler.Default);
        }
        ResetStateWithoutDeleting();
    }

    private static void ResetStateWithoutDeleting() {
        current = null;
        ActivePrefix.Clear();
        PendingStops.Clear();
        TemporaryFiles.Clear();
        ProtectedSources.Clear();
        respawnAnchor = null;
        observedRespawnPoint = null;
        roomKey = "";
        roomName = "";
        areaSid = "";
        completing = false;
    }

    private static string Sanitize(string value) {
        char[] invalid = Path.GetInvalidFileNameChars();
        return new string(value.Select(character => invalid.Contains(character) || char.IsWhiteSpace(character) ? '_' : character).ToArray());
    }
}
