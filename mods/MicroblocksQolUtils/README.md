# microblock's QoL Utils

Windows-first Everest utility mod. Optional integrations are detected at
runtime; MiaoNet+ and SpeedrunTool are not hard dependencies.

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
- A Rust `cdylib` capture backend built directly on `scap`/WGC. Captured BGRA
  frames use an intentional CPU copy, stay outside managed memory, and pass
  through a fixed-capacity latest-frame queue; a slow encoder cannot grow
  memory without bound.
- Streaming H.264 encoding through FFmpeg shared libraries, with automatic
  NVENC, QSV, AMF, Media Foundation, then OpenH264 fallback. No `ffmpeg.exe`,
  `gdigrab`, managed frame buffer, or subprocess is used.
- One WGC/encoder session remains alive for the whole room. Deaths,
  SpeedrunTool loads, and respawn changes only move logical start/end markers;
  they never restart capture or grow an in-memory recording buffer.
- Native background finalization decodes only the retained ranges from the
  continuous room file and re-encodes them into a gapless MP4. This permits
  exact non-keyframe cuts while failed attempts and load freezes are omitted.
- Pass-through FMOD DSP taps capture `bus:/gameplay_sfx` and optionally
  `bus:/ui_sfx`, while deliberately excluding `bus:/music`. Mixer callbacks
  feed a fixed pool of 32 native PCM chunks with non-blocking `try_lock`
  semantics, and a writer thread streams them to a timestamped `.sfxchunks`
  sidecar instead of buffering room audio in managed or native memory. Exact
  zero-filled idle blocks are represented as timestamp gaps rather than stored.
- Timeline cuts for SpeedrunTool save/load and respawn-point triggers. A saved
  prefix is trimmed at its exact timestamp, so deaths and load freezes are not
  included in the final video.

Planned/in progress:

- Timeline-aware SFX mixing/AAC muxing from the captured sidecar, plus
  automatic BGM reconstruction from event and timeline metadata.

## Recorder setup

The native capture bridge selects the window configured by
`RecordingWindowTitle` (default `Celeste`). During development, the Everest
commands `qol_capture_probe_start`, `qol_capture_probe_stats`, and
`qol_capture_probe_stop` exercise scap/WGC without enabling automatic
recording.

`scripts/build-qol-mod.mjs` downloads the current FFmpeg 8.1 LGPL shared build
from BtbN, verifies GitHub's SHA-256 digest, links the Rust encoder against its
import libraries, and packages only the required DLLs and license beside the
mod's native DLL. The FFmpeg executable in the development archive is not
packaged or invoked.

For the planned automatic BGM reconstruction, a JSON event map will resolve
FMOD event paths to clean music files, for example:

```json
{
  "event:/music/lvl1/main": "D:/Celeste-BGM/first_steps.flac"
}
```

Each retained clip already records the FMOD event path and timeline position so
the future audio finalizer can align these files without recording the gameplay
music bus.
