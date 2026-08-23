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

Planned/in progress:

- Disk-backed successful-attempt recording and SpeedrunTool timeline splicing.
