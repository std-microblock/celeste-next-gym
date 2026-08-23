# microblock's QoL Utils

Windows-first Everest utility mod. The implementation is intentionally split
into optional subsystems so MiaoNet+, SpeedrunTool and FFmpeg are not hard
dependencies.

Current foundation:

- HUD entity and settings model.
- Direct Windows TTF/OTF glyph rasterization with a bounded, lazy GPU cache.
- Rolling FPS display.
- Persistent watched-player list and Everest console commands.

Planned/in progress:

- Room minimap, cached route distance and MiaoNet player markers.
- MiaoNet chat commands and background Windows notifications.
- Zero-duration room transitions and per-mod/entity frame spike profiles.
- Disk-backed successful-attempt recording and SpeedrunTool timeline splicing.

