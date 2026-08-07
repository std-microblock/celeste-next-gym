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
- default `NerdHelper/DashThroughSpikes*` -> directional spikes that suppress
  their normal lethal callback during a non-zero live or lingering dash
  attack; non-default inversion/direction filters remain unknown;
- `FancyTileEntities/FancySolidTiles` -> exact 8 px solid rectangles decoded
  from its comma-separated `tileData`; `loadGlobally` remains unsupported;
- `CherryHelper/AssistRect` -> presentation-only decoration.
- vanilla `exitBlock` -> source `Awake` overlap pass-through followed by the
  permanent Solid close once the player clears the original rectangle.
- vanilla `invisibleBarrier` -> constructor-time non-collidable Solid whose
  first `Update` either enables it permanently or deactivates it forever when
  the player overlaps it; its edge `ClimbBlocker` also rejects grabs, wall
  slides, and wall-jump probes.

Stateful flag systems, seekers/barriers, sideways or upside-down jumpthroughs,
trigger spikes, crumble blocks, and custom moving solids intentionally remain
unknown until their update and collision order is implemented.

## 2026-08-07 catalog measurement

The pinned `../celeste-next-gym-ai` catalog contains 10,785 rooms from 480
decoded maps. Re-scanning the same sources after this registry changed the
compatibility totals as follows:

| Verdict | Before | After |
| --- | ---: | ---: |
| supported | 198 | 422 |
| terrain | 224 | 92 |
| unsupported | 10,363 | 10,271 |

Official supported rooms increased from 56 to 132; Mod supported rooms
increased from 142 to 290. All 514 resulting `supported` or `terrain` rooms
completed the deterministic native runtime smoke sequence without a failure.
This catalog result is a compatibility filter, not a claim of real
Celeste/Everest frame-perfect equivalence.

The second registry batch did not change the room verdict totals because every
affected room still had at least one other unsupported gameplay dependency. It
did remove silent-ignore status from 1,890 default DashThroughSpikes across 806
rooms, 454 FancySolidTiles across 106 rooms, and 1,601 AssistRect decorations
across 498 rooms. Fourteen rooms with non-default DashThroughSpikes parameters
correctly remain unknown.

The ExitBlock batch directly added 19 supported rooms and 2 terrain rooms.
Its map-order runtime state remains composable across split simulation, and
the AI geometry consumes the exported parked/restored rectangle rather than
assuming the block is always closed.

The InvisibleBarrier batch directly added 6 supported rooms and 6 terrain
rooms. The catalog contains 2,797 barriers across 1,241 rooms; rooms that still
have other unknown dependencies remain unsupported. The first-update state is
portable across split simulation and room transitions, and the AI fine grid
uses the live parked/restored runtime rectangle instead of painting every
barrier as permanently solid.
