# microblock's QoL Utils

Windows-first Everest utility mod. The implementation is intentionally split
into optional subsystems so MiaoNet+, SpeedrunTool and FFmpeg are not hard
dependencies.

Implemented:

- HUD entity and settings model.
- Direct Windows TTF/OTF glyph rasterization with a bounded, lazy GPU cache.
- Rolling FPS display.
- Persistent watched-player list and Everest console commands.
- Circular/square current-room minimap rendered from the live solid-tile grid.
- Cached room-graph shortest distance to the heart/end room.
- Optional reflection-only MiaoNet player positions, avatars, names and map count.
- `/qol watch`, `/qol unwatch` and `/qol list` inside MiaoNet's own chat box
  (plus `qol_watch`, `qol_unwatch`, `qol_watch_list` in the Everest console).
- Background Windows balloon notifications when a watched player changes rooms.
- Optional suppression of MiaoNet's off-screen name labels.
- Near-instant room transitions (camera/player/light interpolation removed).
- Opt-in frame-spike sampling grouped by owning assembly for entity Update and
  Render, with an on-screen top offender and CSV logs under LocalAppData.
- Windows FFmpeg recording to disk (no frame buffering in managed memory),
  deleting failed attempts and asynchronously rendering only successful rooms.
- Timeline cuts for SpeedrunTool save/load and respawn-point triggers. A saved
  prefix is trimmed at its exact timestamp, so deaths and load freezes are not
  included in the final video.
- Optional SFX-only capture plus BGM post-mix. A JSON event map resolves FMOD
  event names to audio files and each retained clip records the FMOD timeline
  position used for alignment.

Planned/in progress:

## Recorder setup

Put FFmpeg at `Native/win-x64/ffmpeg.exe`, set `FfmpegPath`, or expose it on
`PATH`. Capture uses Windows `gdigrab` and records the window title configured
by `RecordingWindowTitle` (default `Celeste`). `RecordingEncoder` defaults to
`h264_nvenc`; use `h264_qsv`, `h264_amf`, or `libx264` as appropriate.

FFmpeg cannot isolate FMOD buses by itself. `RecordingAudioDevice` therefore
names a DirectShow audio capture device. For automatic BGM reconstruction,
route an SFX-only Celeste mix to that device, choose `SfxOnlyWithPostMix`, and
provide a JSON file like:

```json
{
  "event:/music/lvl1/main": "D:/Celeste-BGM/first_steps.flac"
}
```

The finalizer concatenates the successful timeline, cuts each mapped BGM file
from the captured FMOD timeline position, and mixes it with the SFX track.
