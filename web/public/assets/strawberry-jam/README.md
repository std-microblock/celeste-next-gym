# Strawberry Jam 2021 theme assets

The compact theme atlas in `gameplay/` was selected from the user's locally installed
`1.0.12-Map_Pack-StrawberryJam2021.zip`. It contains the five official Gym palettes
(Beginner through Grandmaster) and five distinct lobby combinations used by the theme picker.

The source archive was found through Steam's registered library at:

`C:/SteamLibrary/steamapps/common/Celeste/Mods/1.0.12-Map_Pack-StrawberryJam2021.zip`

To regenerate the atlas, extract that archive and run:

```bash
node scripts/pack-strawberry-jam-themes.mjs <extracted-root> public/assets/strawberry-jam/gameplay
```

`theme-selected.json` records the exact source path for every packed texture. Atlas keys are the
original Gameplay paths (e.g. `danger/spikes/SJ2021/Gym/beg_up00`, `tilesets/SJ2021/Gym/BeginnerGym`)
so themes can reuse the same values a Celeste map would use (`Spike` meta / entity type, tileset
paths, spinner prefixes) without a renaming step. Strawberry Jam,
Celeste, and their visual assets belong to their respective creators and rights holders; these
files are not intended as independently redistributable project assets.
