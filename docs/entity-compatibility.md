# Entity compatibility registry

`celeste-physics` keeps low-complexity map compatibility in
`crates/celeste-physics/src/entity_decode/` instead of extending the large
`map.rs` name match indefinitely.

## Rules

- Vanilla registrations live in `vanilla.rs`.
- Each Mod has its own file under `entity_decode/mods/`.
- `EntityKind::Decoration` is only for presentation/audio entities and
  non-solid reveal sensors that do not change player physics. The original
  entity name and bounds remain available to renderers and observations.
- A Mod entity may reuse an existing physics kind only when its gameplay
  parameters match the implemented primitive. Non-default parameters remain
  `EntityKind::Unknown`; compatibility is never granted by name alone when the
  Mod exposes behavior-changing options.

The first conditional aliases are:

- `pandorasBox/coloredWater` -> vanilla `Water`;
- `VivHelper/RainbowSpikes*` -> directional vanilla `Spikes`, unless
  `groundRefill` changes gameplay;
- `FrostHelper/SpringFloor/Left/Right` -> vanilla springs only with vanilla
  speed, recovery, one-use, attachment-group, and player-use settings;
- `FrostHelper/CustomDreamBlock` -> vanilla `DreamBlock` only with the vanilla
  240 speed and no redirects, conservation, connection, one-use, or movement
  node;
- `MaxHelpingHand/CustomizableRefill` -> vanilla refill only with the vanilla
  2.5 second respawn; one-use and two-dash variants are retained;
- `JungleHelper/InvisibleJumpthruPlatform` -> vanilla `JumpThru`.

Stateful flag systems, seekers/barriers, sideways or upside-down jumpthroughs,
trigger spikes, crumble blocks, and custom moving solids intentionally remain
unknown until their update and collision order is implemented.

## 2026-08-06 catalog measurement

The pinned `../celeste-next-gym-ai` catalog contains 10,785 rooms from 480
decoded maps. Re-scanning the same sources after this registry changed the
compatibility totals as follows:

| Verdict | Before | After |
| --- | ---: | ---: |
| supported | 198 | 397 |
| terrain | 224 | 84 |
| unsupported | 10,363 | 10,304 |

Official supported rooms increased from 56 to 126; Mod supported rooms
increased from 142 to 271. All 481 resulting `supported` or `terrain` rooms
completed the deterministic native runtime smoke sequence without a failure.
This catalog result is a compatibility filter, not a claim of real
Celeste/Everest frame-perfect equivalence.
