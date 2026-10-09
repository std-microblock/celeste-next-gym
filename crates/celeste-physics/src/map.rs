use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    BinaryElement, BinaryPackerWriteError, BinaryValue, Vec2, encode_celeste_bin, parse_celeste_bin,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn right(self) -> f32 {
        self.x + self.width
    }
    pub fn bottom(self) -> f32 {
        self.y + self.height
    }
    pub fn intersects(self, other: Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    /// Presentation-only entity or a non-solid reveal sensor. It is retained
    /// so room compatibility scans do not report it as silently ignored, but
    /// it deliberately has no physics interaction.
    Decoration,
    JumpThru,
    DreamBlock,
    Spikes,
    Water,
    Booster,
    RedBooster,
    FlyFeather,
    Bumper,
    IceBall,
    Puffer,
    AngryOshiro,
    Seeker,
    Snowball,
    /// Vanilla bottom-of-room hazard with a fixed 32 px PlayerCollider. It
    /// enables only after the player is sufficiently above it and disables
    /// again after the player has fallen sufficiently below it.
    Killbox,
    Cloud,
    BadelineBoost,
    Spring,
    Strawberry,
    /// Vanilla Refill PlayerCollider entity. direction.x stores the
    /// `twoDash` flag; `single_use` stores `oneUse`.
    Refill,
    /// Vanilla FallingBlock Solid. direction.x stores the `climbFall` flag,
    /// direction.y the `behind` depth flag; width/height come from attributes.
    FallingBlock,
    /// Vanilla passage that is disabled when the player starts inside it and
    /// becomes a permanent Solid after the player leaves.
    ExitBlock,
    /// Vanilla invisible safe Solid. It starts non-collidable, then either
    /// becomes permanent on its first Update or disables itself forever when
    /// that first update finds the player overlapping it. Its edge
    /// ClimbBlocker prevents grabs, wall slides, and wall jumps.
    InvisibleBarrier,
    Wind,
    /// Vanilla hot-state Core BounceBlock Solid.
    BounceBlock,
    /// Vanilla TheoCrystal Actor with a Holdable component.
    TheoCrystal,
    /// Vanilla Crystal Heart / HeartGem PlayerCollider and collection routine.
    HeartGem,
    /// Core chapter camera-following bottom lava hazard.
    RisingLava,
    /// Core chapter persistent top-and-bottom sandwich lava hazard.
    SandwichLava,
    /// Vanilla Farewell Glider Actor with a Holdable component.
    Glider,
    /// Vanilla Celeste ZipMover Solid. The first node is its target position.
    ZipMover,
    /// Vanilla steerable MoveBlock ("moon block") Solid.
    MoveBlock,
    /// Vanilla 8px-wide TempleGate using CloseBehindPlayerAlways.
    TempleGate,
    /// Vanilla beat-indexed CassetteBlock Solid. `direction.x` stores its
    /// integer index and `direction.y` stores its tempo multiplier.
    CassetteBlock,
    /// Vanilla CrystalStaticSpinner hazard.
    CrystalStaticSpinner,
    /// Vanilla binocular entity. direction.x/y persist onlyY/summit flags.
    Lookout,
    /// Vanilla `Celeste.CrushBlock : Solid` (`CrushBlock.cs:9`). Its data
    /// `position`/`width`/`height` go straight into `Solid(Position, width,
    /// height, safe: false)` (`CrushBlock.cs:85-87`, `146-149`), so the raw
    /// rectangle is the collider. `direction.x` stores the `axes` enum
    /// (`Both`=0, `Horizontal`=1, `Vertical`=2) and `direction.y` the `chillout`
    /// flag; `OnDashCollide` turns on the crusher's attack when the axes allow
    /// the dash direction (`CrushBlock.cs:274-303`).
    CrushBlock,
    /// Vanilla `Celeste.DashBlock : Solid` (`DashBlock.cs:7`). `Solid(position,
    /// width, height, safe: true)` again takes the raw rectangle
    /// (`DashBlock.cs:30-32`). `direction.x` stores `canDash` and `direction.y`
    /// `permanent`; its `OnDashCollide` breaks the block and rebounds
    /// (`DashBlock.cs:131-139`).
    DashBlock,
    /// Vanilla `Celeste.WallBooster : Entity` (`WallBooster.cs:8`), the Core
    /// conveyor. Not a `Solid`: a 2 px `Hitbox` strip that `Player.ClimbUpdate`
    /// speeds the player along at `WallBoosterSpeed` (`Player.cs:3093-3099`,
    /// `3154-3167`). `direction.x` is `Facing` (`-1` left, `+1` right) and
    /// `direction.y` the `notCoreMode` flag, which forces `IceMode` regardless of
    /// the room's `CoreMode` (`WallBooster.cs:26-39`, `:74-101`). In `IceMode` the
    /// entity's own `ClimbBlocker(edge: false)` is `Blocking`, so the strip
    /// refuses the grab instead of driving it (`WallBooster.cs:42`, `:85-101`;
    /// `ClimbBlocker.cs:28-38`).
    WallBooster,
    /// Vanilla `Celeste.CoreModeToggle : Entity` (`CoreModeToggle.cs:6`), the Core's ice/fire
    /// switch. Not a `Solid`: a `Hitbox(16f, 24f, -8f, -12f)` (`CoreModeToggle.cs:46`) that flips
    /// `Level.CoreMode` when the player's hurtbox touches it (`:103-126`). `direction.x` is
    /// `onlyFire` and `direction.y` `onlyIce`, the `Usable` gate (`:24-38`); `single_use` is
    /// `persistent`, which decides whether the flip is also written back to `Session.CoreMode`
    /// (`:117-120`).
    CoreModeToggle,
    /// Vanilla `Celeste.AscendManager : Entity` (`AscendManager.cs:9`), written by the map as
    /// `SummitBackgroundManager`. Its `Routine` (`:249-273`) waits until the player's `Y` is no
    /// greater than its own and then takes the player over as a dummy: `Speed = Vector2.Zero`,
    /// `StateMachine.State = 11` (`StDummy`), `DummyGravity = false` (`:270-273`).
    /// `direction.x` carries the map's `index` (only `index == 9` delays the takeover by 1.6 s,
    /// `:257-260`).
    SummitBackgroundManager,
    /// Vanilla `Solid` subclasses that never move and are collidable from the moment the room
    /// loads, so the raw rectangle (plus whatever the constructor applies to its collider) is the
    /// whole physics model. Currently `BridgeFixed` (`BridgeFixed.cs:9-11`,
    /// `Solid(data.Position + offset, data.Width, 8f, safe: true)`) and `Plateau`
    /// (`Plateau.cs:12-15`, `Solid(e.Position + offset, 104f, 4f, safe: true)` with
    /// `Collider.Left += 8f`). Keeping them apart from `EntityKind::MovingSolid` matters: the
    /// simulator must never treat one as carried by the player.
    StaticSolid,
    /// Vanilla `Celeste.CrumblePlatform : Solid` (`CrumblePlatform.cs:8`, map name
    /// `crumbleBlock`): `base(position, width, 8f, safe: false)` - the same eight-pixel height as
    /// the raw rectangle, so no collider adjustment. Treated as a plain solid for now:
    /// `CrumblePlatform.Sequence` (`:94-169`) collapses it 0.2 s after a player stands on top plus
    /// up to 0.4 s of further standing, sets `Collidable = false`, waits 2 s and re-arms once
    /// nothing overlaps it. Standing *on* one is the case that matters for the trace.
    CrumbleBlock,
    /// Vanilla `Celeste.FloatySpaceBlock : Solid` (`FloatySpaceBlock.cs:9`, map name
    /// `floatySpaceBlock`): the bobbing Farewell block. `base(position, width, height, safe: true)`
    /// (`:44`) takes the raw map rectangle as the collider, and `Depth = -9000` (`:47`) puts its
    /// update *after* `Player.Update`. Each frame the group master recomputes a target
    /// `Lerp(originalY, originalY + 12, Ease.SineInOut(yLerp)) + sin(sineWave) * 4` plus a dash
    /// offset and moves every group member there through `MoveToY`/`MoveToX` (`:288-289`), i.e.
    /// whole-pixel `MoveHExact`/`MoveVExact` that carries riders. `direction` carries the two
    /// constructor inputs a replay needs: `x` is `data.Bool("disableSpawnOffset")` (which selects
    /// `sineWave = 0` instead of the random spawn draw, `:50-57`) and `y` is the `tiletype` char
    /// code, which groups touching blocks (`:189`).
    FloatySpaceBlock,
    /// Vanilla `Celeste.SwitchGate : Solid` (map name `switchGate`): a solid gate that waits for
    /// every `Switch` component in the room to finish (`Switch.FinishedCheck`, `Switch.cs:99-110`),
    /// then runs `SwitchGate.Sequence` (`SwitchGate.cs:102-145`): `yield 0.1`, a 0.5 s icon ramp
    /// (`icon.Rate += dt * 2`), `yield 0.1`, then a 2 s `Ease.CubeOut` tween of `MoveTo(nodes[0])`,
    /// then `yield 1.8`. `Awake` (`:68-82`) jumps straight to the target when the room's
    /// `switches_<room>` session flag is already set, which is why the flag is threaded.
    SwitchGate,
    /// Vanilla `Celeste.TouchSwitch : Entity` (map name `touchSwitch`): `Switch(groundReset: false)`
    /// plus `PlayerCollider(OnPlayer, null, new Hitbox(30f, 30f, -15f, -15f))`
    /// (`TouchSwitch.cs:44-45`), so its activation box is 30x30 centred on the entity position, not
    /// the 8x8 map rectangle.
    TouchSwitch,
    /// Vanilla `Celeste.DashSwitch : Solid` (map names `dashSwitchH`/`dashSwitchV`): the Mirror
    /// Temple button that a dash presses. The constructor's `Solid(position, 0f, 0f, safe: true)`
    /// is immediately resized to `16x8` for `Sides.Up`/`Down` and `8x16` for `Sides.Left`/`Right`
    /// at the entity position, with collider offset `(0, 0)` (`DashSwitch.cs:52-71`), so *the map
    /// rectangle is not the collider* - every vanilla element carries a 0-sized one. `direction`
    /// is `pressDirection` (`DashSwitch.cs:72-99`: `UnitY`, `-UnitY`, `UnitX`, `-UnitX`) and
    /// `single_use` carries `persistent`, which decides whether the press also writes the
    /// `dashSwitch_<id>` session flag (`DashSwitch.cs:223-226`) and whether `Awake` (`:124-149`)
    /// restores the pushed state at room load - see `Simulator::set_pressed_dash_switches`.
    DashSwitch,
    /// Simulator-native constant-velocity Solid used to exercise Monocle
    /// carrying, pushing, and Player LiftSpeed inheritance independently of a
    /// specific vanilla entity state machine.
    MovingSolid,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub kind: EntityKind,
    pub bounds: Rect,
    #[serde(default)]
    pub direction: Vec2,
    #[serde(default)]
    pub shielded: bool,
    #[serde(default)]
    pub single_use: bool,
    #[serde(default)]
    pub nodes: Vec<Vec2>,
    #[serde(default)]
    pub name: String,
}

/// Visual/skin metadata for one decoded entity, kept separate from the physics
/// model so decode-only extras (mod custom spinners, falling-block tiletype)
/// never change collision semantics.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EntityVisual {
    #[serde(default)]
    pub texture: Option<String>,
    #[serde(default)]
    pub tile: Option<char>,
    #[serde(default)]
    pub tint: Option<String>,
    #[serde(default)]
    pub variant: Option<String>,
}

/// Runtime data for one Celeste room. `Level.LoadLevel` replaces room-local
/// solids and entities during a transition while the session-wide state
/// (notably CassetteBlockManager) remains alive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoomRuntime {
    pub bounds: Rect,
    #[serde(default)]
    pub spawns: Vec<Vec2>,
    #[serde(default)]
    pub solids: Vec<Rect>,
    #[serde(default)]
    pub entities: Vec<Entity>,
    /// This room's own `LevelData.LoadSeed`: a transition into it re-runs `Level.LoadLevel`
    /// (`Level.cs:1509`) and therefore re-seeds `Calc.Random` (`:386`) from *its* name.
    #[serde(default)]
    pub load_seed: i32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Map {
    /// The `.bin` `id` attribute of each entity, in `entities` order. `Session.DoNotLoad` is
    /// keyed by `"<Level>:<ID>"`, so this lets `Simulator::new` skip an entity the game never
    /// constructed (`Level.cs:472`, `:1188`). `-1` means the map carried no id.
    #[serde(default)]
    pub entity_ids: Vec<i32>,
    pub bounds: Rect,
    /// Bounds of the other rooms in the same Celeste map. These are retained
    /// when decoding one room so Level.EnforceBounds can resolve transitions.
    #[serde(default)]
    pub transition_rooms: Vec<Rect>,
    /// Decoded room-local data for every room in the same map, including the
    /// room initially loaded into `solids` and `entities`. Keeping the source
    /// room is necessary when a player transitions back into it, such as a
    /// Bubsdrop: `Level.LoadLevel` must restore its collision and choose from
    /// its own spawn set rather than retaining the upper room's data.
    #[serde(default)]
    pub transition_runtime: Vec<RoomRuntime>,
    /// Static `LevelData.Spawns` for the presently loaded room. `spawn` is
    /// the session respawn selected from this set during a room transition.
    #[serde(default)]
    pub room_spawns: Vec<Vec2>,
    #[serde(default)]
    pub spawn: Vec2,
    #[serde(default)]
    pub solids: Vec<Rect>,
    #[serde(default)]
    pub entities: Vec<Entity>,
    #[serde(default)]
    pub source_package: Option<String>,
    /// Source per-room solid char grid (one string per row). Decode-only extra
    /// used by the web renderer to autotile each cell with its own tileset.
    #[serde(default)]
    pub tile_grid: Vec<String>,
    /// Visual/skin metadata aligned by index with entities.
    #[serde(default)]
    pub entity_visuals: Vec<EntityVisual>,
    /// `LevelData.LoadSeed` (`LevelData.cs:99-111`): the sum of the level name's char codes with
    /// the first four (`lvl_`) stripped. `Level.LoadLevel` pushes `Calc.Random` onto a
    /// `System.Random` seeded with it for the whole room build (`Level.cs:386-1386`), which is what
    /// `FloatySpaceBlock`'s constructor draws its `sineWave` from (`FloatySpaceBlock.cs:50-57`).
    #[serde(default)]
    pub load_seed: i32,
}

/// Raw per-room contents used by offline compatibility scanners. Unlike
/// [`Map`], this preserves every original entity and trigger name so callers
/// can reject rooms whose gameplay depends on something the simulator ignores.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CelesteRoomAudit {
    pub name: String,
    pub bounds: Rect,
    #[serde(default)]
    pub spawns: Vec<Vec2>,
    #[serde(default)]
    pub entity_names: BTreeMap<String, u32>,
    #[serde(default)]
    pub trigger_names: BTreeMap<String, u32>,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            transition_rooms: vec![],
            transition_runtime: vec![],
            room_spawns: vec![],
            spawn: Vec2::new(24.0, 160.0),
            solids: vec![],
            entities: vec![],
            entity_ids: vec![],
            source_package: None,
            tile_grid: vec![],
            entity_visuals: vec![],
            load_seed: 0,
        }
    }
}

#[derive(Debug, Error)]
pub enum MapError {
    #[error("map is neither a supported MessagePack map nor a Celeste BinaryPacker file: {0}")]
    Unsupported(String),
    #[error("Celeste map contains no level")]
    NoLevel,
    #[error("Celeste map contains no room named {0}")]
    RoomNotFound(String),
}

#[derive(Debug, Error)]
pub enum MapEncodeError {
    #[error("map bounds must be positive and aligned to the 8-pixel Celeste tile grid")]
    InvalidBounds,
    #[error(transparent)]
    Binary(#[from] BinaryPackerWriteError),
}

pub fn encode_map(map: &Map) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec_named(map)
}

pub fn encode_celeste_map(map: &Map, package: &str, room: &str) -> Result<Vec<u8>, MapEncodeError> {
    if !valid_room_bounds(map.bounds)
        || map
            .transition_rooms
            .iter()
            .copied()
            .any(|bounds| !valid_room_bounds(bounds))
    {
        return Err(MapEncodeError::InvalidBounds);
    }
    let mut rooms = vec![(room.to_owned(), map.clone())];
    for (index, bounds) in map.transition_rooms.iter().copied().enumerate() {
        // LevelData marks rooms without a player spawn as Dummy, and
        // MapData.CanTransitionTo rejects Dummy targets. A transition room
        // therefore needs a valid spawn even though screen transitions keep
        // the existing player instead of respawning at it.
        rooms.push((
            format!("transition_{index}"),
            Map {
                bounds,
                spawn: Vec2::new(bounds.x + 24.0, bounds.bottom() - 16.0),
                solids: map.solids.clone(),
                ..Map::default()
            },
        ));
    }
    encode_celeste_rooms(package, &rooms)
}

fn valid_room_bounds(bounds: Rect) -> bool {
    bounds.width > 0.0
        && bounds.height > 0.0
        && bounds.width % 8.0 == 0.0
        && bounds.height % 8.0 == 0.0
}

/// `LevelData`'s room rectangle, including its one unconditional rewrite:
///
/// ```csharp
/// case "height":
///     Bounds.Height = (int)attribute.Value;
///     if (Bounds.Height == 184) { Bounds.Height = 180; }
///     break;
/// ```
///
/// (`LevelData.cs:132-137`). Every consumer of `Level.Bounds` therefore sees
/// 180 for a room whose file says 184: `Level.EnforceBounds`
/// (`Level.cs:2725`, called from `Player.cs:1915-1917`), `Player.ClimbBoundsCheck`
/// (`Player.cs`), `Level.IsInBounds`/`MapData.CanTransitionTo`
/// (`Level.cs:2740,2761`) and the camera. `LevelData.TileBounds`
/// (`LevelData.cs:76`) uses the same clamped height, and
/// `ceil(180 / 8) == ceil(184 / 8) == 23`, so the stored 23 tile rows are
/// unaffected. Only the 4 px of `Bounds.Bottom` change.
pub(crate) const fn level_room_bounds(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(x, y, width, if height == 184.0 { 180.0 } else { height })
}

pub(crate) fn encode_celeste_rooms(
    package: &str,
    rooms: &[(String, Map)],
) -> Result<Vec<u8>, MapEncodeError> {
    if rooms.is_empty() || rooms.iter().any(|(_, map)| !valid_room_bounds(map.bounds)) {
        return Err(MapEncodeError::InvalidBounds);
    }
    let mut levels = Vec::with_capacity(rooms.len());
    for (room, map) in rooms {
        let spawns = if map.room_spawns.is_empty() {
            vec![map.spawn]
        } else {
            map.room_spawns.clone()
        };
        let mut entities = spawns
            .iter()
            .enumerate()
            .map(|(index, spawn)| {
                element(
                    "player",
                    [
                        ("id", BinaryValue::Int(index as i32)),
                        ("originX", BinaryValue::Int(4)),
                        ("originY", BinaryValue::Int(8)),
                        ("width", BinaryValue::Int(8)),
                        ("x", BinaryValue::Int((spawn.x - map.bounds.x) as i32)),
                        ("y", BinaryValue::Int((spawn.y - map.bounds.y) as i32)),
                    ],
                    vec![],
                )
            })
            .collect::<Vec<_>>();
        let mut triggers = Vec::new();
        for (index, entity) in map.entities.iter().enumerate() {
            let id = index as i32 + spawns.len() as i32;
            let x = (entity.bounds.x - map.bounds.x) as i32;
            let y = (entity.bounds.y - map.bounds.y) as i32;
            let width = entity.bounds.width as i32;
            let height = entity.bounds.height as i32;
            let encoded = match entity.kind {
                EntityKind::JumpThru => Some(element(
                    "jumpThru",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("texture", BinaryValue::String("default".to_owned())),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::DreamBlock => Some(element(
                    "dreamBlock",
                    [
                        ("fastMoving", BinaryValue::Bool(false)),
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::Spikes => {
                    let (name, x, y, width, height) = if entity.direction.y < 0.0 {
                        ("spikesUp", x, y + height, width, 0)
                    } else if entity.direction.y > 0.0 {
                        ("spikesDown", x, y, width, 0)
                    } else if entity.direction.x < 0.0 {
                        ("spikesLeft", x + width, y, 0, height)
                    } else {
                        ("spikesRight", x, y, 0, height)
                    };
                    let mut attrs = vec![
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(4)),
                        ("type", BinaryValue::String("default".to_owned())),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ];
                    if width > 0 {
                        attrs.push(("width", BinaryValue::Int(width)));
                    }
                    if height > 0 {
                        attrs.push(("height", BinaryValue::Int(height)));
                    }
                    Some(element_vec(name, attrs, vec![]))
                }
                EntityKind::Water => Some(element(
                    "water",
                    [
                        ("hasBottom", BinaryValue::Bool(false)),
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("steamy", BinaryValue::Bool(false)),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::Booster | EntityKind::RedBooster => Some(element(
                    "booster",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(4)),
                        ("originY", BinaryValue::Int(4)),
                        (
                            "red",
                            BinaryValue::Bool(entity.kind == EntityKind::RedBooster),
                        ),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::FlyFeather => Some(element(
                    "infiniteStar",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("shielded", BinaryValue::Bool(entity.shielded)),
                        ("singleUse", BinaryValue::Bool(entity.single_use)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::Bumper => Some(element(
                    "bigSpinner",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::IceBall => Some(element(
                    "fireBall",
                    [
                        ("amount", BinaryValue::Int(1)),
                        ("id", BinaryValue::Int(id)),
                        ("notCoreMode", BinaryValue::Bool(true)),
                        ("offset", BinaryValue::Float(0.0)),
                        ("speed", BinaryValue::Float(0.0)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![element(
                        "node",
                        [
                            ("x", BinaryValue::Int(x + width / 2 + 16)),
                            ("y", BinaryValue::Int(y + height / 2)),
                        ],
                        vec![],
                    )],
                )),
                EntityKind::Puffer => Some(element(
                    "puffer",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("right", BinaryValue::Bool(false)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::AngryOshiro => Some(element(
                    "oshiroBoss",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::Seeker => Some(element(
                    "seeker",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::Snowball => Some(element(
                    "snowball",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::Cloud => Some(element(
                    "cloud",
                    [
                        ("fragile", BinaryValue::Bool(false)),
                        ("id", BinaryValue::Int(id)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::BadelineBoost => Some(element(
                    "badelineBoost",
                    [
                        ("canSkip", BinaryValue::Bool(false)),
                        ("id", BinaryValue::Int(id)),
                        ("lockCamera", BinaryValue::Bool(false)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    entity
                        .nodes
                        .iter()
                        .map(|node| {
                            element(
                                "node",
                                [
                                    (
                                        "x",
                                        BinaryValue::Int((node.x - map.bounds.x).round() as i32),
                                    ),
                                    (
                                        "y",
                                        BinaryValue::Int((node.y - map.bounds.y).round() as i32),
                                    ),
                                ],
                                vec![],
                            )
                        })
                        .collect(),
                )),
                EntityKind::Spring => {
                    let (name, spring_x, spring_y) = if entity.direction.y < 0.0 {
                        ("spring", x + 8, y + 6)
                    } else if entity.direction.x > 0.0 {
                        ("wallSpringLeft", x, y + 8)
                    } else {
                        ("wallSpringRight", x + 6, y + 8)
                    };
                    Some(element(
                        name,
                        [
                            ("id", BinaryValue::Int(id)),
                            ("playerCanUse", BinaryValue::Bool(true)),
                            ("x", BinaryValue::Int(spring_x)),
                            ("y", BinaryValue::Int(spring_y)),
                        ],
                        vec![],
                    ))
                }
                EntityKind::Refill => Some(element(
                    "refill",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("oneUse", BinaryValue::Bool(entity.single_use)),
                        ("originX", BinaryValue::Int(4)),
                        ("originY", BinaryValue::Int(4)),
                        ("twoDash", BinaryValue::Bool(entity.direction.x != 0.0)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::FallingBlock => Some(element(
                    "fallingBlock",
                    [
                        ("behind", BinaryValue::Bool(entity.direction.y != 0.0)),
                        ("climbFall", BinaryValue::Bool(entity.direction.x != 0.0)),
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        (
                            "tiletype",
                            BinaryValue::Int(
                                map.entity_visuals
                                    .get(index)
                                    .and_then(|visual| visual.tile)
                                    .map(|tile| tile as i32)
                                    .unwrap_or(b'3' as i32),
                            ),
                        ),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::ExitBlock => Some(element(
                    "exitBlock",
                    [
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        (
                            "tileType",
                            BinaryValue::String(
                                map.entity_visuals
                                    .get(index)
                                    .and_then(|visual| visual.tile)
                                    .unwrap_or('3')
                                    .to_string(),
                            ),
                        ),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::InvisibleBarrier => Some(element(
                    "invisibleBarrier",
                    [
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::Killbox => Some(element(
                    "killbox",
                    [
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::Strawberry => Some(element(
                    "strawberry",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("moon", BinaryValue::Bool(false)),
                        ("winged", BinaryValue::Bool(false)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::Wind => {
                    triggers.push(element(
                        "windTrigger",
                        [
                            ("height", BinaryValue::Int(height)),
                            ("id", BinaryValue::Int(id)),
                            (
                                "pattern",
                                BinaryValue::String(wind_pattern(entity.direction)),
                            ),
                            ("width", BinaryValue::Int(width)),
                            ("x", BinaryValue::Int(x)),
                            ("y", BinaryValue::Int(y)),
                        ],
                        vec![],
                    ));
                    None
                }
                EntityKind::BounceBlock => Some(element(
                    "bounceBlock",
                    [
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::TheoCrystal => Some(element(
                    "theoCrystal",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("x", BinaryValue::Int(x + 4)),
                        ("y", BinaryValue::Int(y + 10)),
                    ],
                    vec![],
                )),
                EntityKind::HeartGem => Some(element(
                    "blackGem",
                    [
                        ("fake", BinaryValue::Bool(false)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(8)),
                        ("originY", BinaryValue::Int(8)),
                        ("removeCameraTriggers", BinaryValue::Bool(false)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
                EntityKind::RisingLava => Some(element(
                    "risingLava",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("intro", BinaryValue::Bool(entity.single_use)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::SandwichLava => Some(element(
                    "sandwichLava",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::Glider => Some(element(
                    "glider",
                    [
                        ("bubble", BinaryValue::Bool(false)),
                        ("id", BinaryValue::Int(id)),
                        ("tutorial", BinaryValue::Bool(false)),
                        ("x", BinaryValue::Int(x + 4)),
                        ("y", BinaryValue::Int(y + 10)),
                    ],
                    vec![],
                )),
                EntityKind::ZipMover => {
                    let target = entity
                        .nodes
                        .first()
                        .copied()
                        .unwrap_or(Vec2::new(entity.bounds.x, entity.bounds.y));
                    Some(element(
                        "zipMover",
                        [
                            ("height", BinaryValue::Int(height)),
                            ("id", BinaryValue::Int(id)),
                            ("originX", BinaryValue::Int(0)),
                            ("originY", BinaryValue::Int(0)),
                            ("theme", BinaryValue::String("Normal".to_owned())),
                            ("width", BinaryValue::Int(width)),
                            ("x", BinaryValue::Int(x)),
                            ("y", BinaryValue::Int(y)),
                        ],
                        vec![element(
                            "node",
                            [
                                (
                                    "x",
                                    BinaryValue::Int((target.x - map.bounds.x).round() as i32),
                                ),
                                (
                                    "y",
                                    BinaryValue::Int((target.y - map.bounds.y).round() as i32),
                                ),
                            ],
                            vec![],
                        )],
                    ))
                }
                EntityKind::MoveBlock => {
                    let direction = if entity.direction.x < 0.0 {
                        "Left"
                    } else if entity.direction.y < 0.0 {
                        "Up"
                    } else if entity.direction.y > 0.0 {
                        "Down"
                    } else {
                        "Right"
                    };
                    Some(element(
                        "moveBlock",
                        [
                            ("canSteer", BinaryValue::Bool(true)),
                            ("direction", BinaryValue::String(direction.to_owned())),
                            ("fast", BinaryValue::Bool(false)),
                            ("height", BinaryValue::Int(height)),
                            ("id", BinaryValue::Int(id)),
                            ("originX", BinaryValue::Int(0)),
                            ("originY", BinaryValue::Int(0)),
                            ("width", BinaryValue::Int(width)),
                            ("x", BinaryValue::Int(x)),
                            ("y", BinaryValue::Int(y)),
                        ],
                        vec![],
                    ))
                }
                EntityKind::TempleGate => Some(element(
                    "templeGate",
                    [
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("sprite", BinaryValue::String("default".to_owned())),
                        (
                            "type",
                            BinaryValue::String(
                                map.entity_visuals
                                    .get(index)
                                    .and_then(|visual| visual.variant.clone())
                                    .unwrap_or_else(|| {
                                        "CloseBehindPlayerAlways".to_owned()
                                    }),
                            ),
                        ),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::CassetteBlock => Some(element(
                    "cassetteBlock",
                    [
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("index", BinaryValue::Int(entity.direction.x.round() as i32)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        (
                            "tempo",
                            BinaryValue::Float(if entity.direction.y == 0.0 {
                                1.0
                            } else {
                                entity.direction.y
                            }),
                        ),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::CrystalStaticSpinner => {
                    let mut spinner_attrs = vec![
                        ("attachToSolid", BinaryValue::Bool(false)),
                        ("id", BinaryValue::Int(id)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ];
                    if let Some(variant) = map
                        .entity_visuals
                        .get(index)
                        .and_then(|visual| visual.variant.as_ref())
                    {
                        spinner_attrs.push(("type", BinaryValue::String(variant.clone())));
                    }
                    Some(element_vec("spinner", spinner_attrs, vec![]))
                }
                EntityKind::Lookout => Some(element(
                    // `Level.LoadLevel` instantiates the vanilla Lookout under the
                    // map entity name `towerviewer`; `lookout` is only the Rust
                    // fixture-facing alias and Everest rejects it.
                    "towerviewer",
                    [
                        ("x", BinaryValue::Int(x + 2)),
                        ("y", BinaryValue::Int(y + 4)),
                        ("onlyY", BinaryValue::Bool(entity.direction.x != 0.0)),
                        ("summit", BinaryValue::Bool(entity.direction.y != 0.0)),
                    ],
                    entity
                        .nodes
                        .iter()
                        .map(|node| {
                            element(
                                "node",
                                [
                                    (
                                        "x",
                                        BinaryValue::Int((node.x - map.bounds.x).round() as i32),
                                    ),
                                    (
                                        "y",
                                        BinaryValue::Int((node.y - map.bounds.y).round() as i32),
                                    ),
                                ],
                                vec![],
                            )
                        })
                        .collect(),
                )),
                EntityKind::MovingSolid => Some(element(
                    "celesteGymMovingSolid",
                    [
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("speedX", BinaryValue::Float(entity.direction.x)),
                        ("speedY", BinaryValue::Float(entity.direction.y)),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::CrushBlock => Some(element(
                    "crushBlock",
                    [
                        (
                            "axes",
                            BinaryValue::String(
                                match entity.direction.x as i32 {
                                    1 => "Horizontal",
                                    2 => "Vertical",
                                    _ => "Both",
                                }
                                .to_owned(),
                            ),
                        ),
                        ("chillout", BinaryValue::Bool(entity.direction.y != 0.0)),
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::DashBlock => Some(element(
                    "dashBlock",
                    [
                        (
                            "blendin",
                            BinaryValue::Bool(
                                map.entity_visuals
                                    .get(index)
                                    .and_then(|visual| visual.variant.as_deref())
                                    == Some("blendin"),
                            ),
                        ),
                        ("canDash", BinaryValue::Bool(entity.direction.x != 0.0)),
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("permanent", BinaryValue::Bool(entity.direction.y != 0.0)),
                        (
                            "tiletype",
                            BinaryValue::String(
                                map.entity_visuals
                                    .get(index)
                                    .and_then(|visual| visual.tile)
                                    .unwrap_or('3')
                                    .to_string(),
                            ),
                        ),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                // `CoreModeToggle`'s box is centred on the entity position, so re-encode the
                // position it was decoded from and let the constructor re-apply the offset.
                EntityKind::CoreModeToggle => Some(element(
                    "coreModeToggle",
                    [
                        ("id", BinaryValue::Int(id)),
                        (
                            "onlyFire",
                            BinaryValue::Bool(entity.direction.x != 0.0),
                        ),
                        ("onlyIce", BinaryValue::Bool(entity.direction.y != 0.0)),
                        ("originX", BinaryValue::Int(8)),
                        ("originY", BinaryValue::Int(8)),
                        ("persistent", BinaryValue::Bool(entity.single_use)),
                        ("x", BinaryValue::Int(x + 8)),
                        ("y", BinaryValue::Int(y + 12)),
                    ],
                    vec![],
                )),
                // Re-encode the conveyor at the entity-data position the decoder
                // started from: a left strip already sits on it, a right one was
                // shifted six pixels by `WallBooster`'s own collider.
                EntityKind::WallBooster => {
                    let left = entity.direction.x < 0.0;
                    Some(element(
                        "wallBooster",
                        [
                            ("height", BinaryValue::Int(height)),
                            ("id", BinaryValue::Int(id)),
                            ("left", BinaryValue::Bool(left)),
                            ("notCoreMode", BinaryValue::Bool(entity.direction.y != 0.0)),
                            ("originX", BinaryValue::Int(0)),
                            ("originY", BinaryValue::Int(0)),
                            ("x", BinaryValue::Int(if left { x } else { x - 6 })),
                            ("y", BinaryValue::Int(y)),
                        ],
                        vec![],
                    ))
                }
                // `AscendManager`'s only field the physics needs is `Position.Y`, and its map name
                // is what a re-encode has to keep (`AscendManager.cs:228-233`).
                EntityKind::SummitBackgroundManager => Some(element(
                    "SummitBackgroundManager",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("index", BinaryValue::Int(entity.direction.x as i32)),
                        ("intro_launch", BinaryValue::Bool(false)),
                        ("originX", BinaryValue::Int(8)),
                        ("originY", BinaryValue::Int(8)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                // Static solids round-trip under the name the decoder saw, so a fixture map keeps
                // whichever of `plateau`/`bridgeFixed` it used.
                EntityKind::CrumbleBlock => Some(element(
                    "crumbleBlock",
                    [
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                // `floatySpaceBlock` round-trips its raw rectangle plus the two constructor
                // inputs `EntityKind::FloatySpaceBlock` keeps in `direction` (`FloatySpaceBlock.cs:60-63`).
                EntityKind::FloatySpaceBlock => Some(element(
                    "floatySpaceBlock",
                    [
                        (
                            "disableSpawnOffset",
                            BinaryValue::Bool(entity.direction.x != 0.0),
                        ),
                        ("height", BinaryValue::Int(height)),
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        (
                            "tiletype",
                            BinaryValue::String(
                                char::from_u32(entity.direction.y as u32)
                                    .unwrap_or('3')
                                    .to_string(),
                            ),
                        ),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                EntityKind::StaticSolid => Some(element(
                    &entity.name,
                    [
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("width", BinaryValue::Int(width)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    vec![],
                )),
                // `switchGate` carries `nodes[0]` (its open target) and `touchSwitch` does not, so
                // the encoder round-trips the name and re-emits whatever nodes the decode kept, in
                // the same shape `ZipMover` uses.
                EntityKind::SwitchGate | EntityKind::TouchSwitch => Some(element(
                    &entity.name,
                    [
                        ("id", BinaryValue::Int(id)),
                        ("originX", BinaryValue::Int(0)),
                        ("originY", BinaryValue::Int(0)),
                        ("width", BinaryValue::Int(width)),
                        ("height", BinaryValue::Int(height)),
                        ("x", BinaryValue::Int(x)),
                        ("y", BinaryValue::Int(y)),
                    ],
                    entity
                        .nodes
                        .iter()
                        .map(|node| {
                            element(
                                "node",
                                [
                                    (
                                        "x",
                                        BinaryValue::Int((node.x - map.bounds.x).round() as i32),
                                    ),
                                    (
                                        "y",
                                        BinaryValue::Int((node.y - map.bounds.y).round() as i32),
                                    ),
                                ],
                                vec![],
                            )
                        })
                        .collect(),
                )),
                // `dashSwitchH`/`dashSwitchV` round-trip under the name the decoder saw, exactly
                // like `StaticSolid`, because the side lives in the name + one bool and the
                // collider width/height are derived on decode rather than read from the map
                // (`DashSwitch.cs:62-71`, `:103-122`). The `sprite` skin is presentation only and
                // is not re-emitted; `persistent` and `allGates` are, because they are the two
                // bits the simulator reads back (`single_use` and `shielded`).
                EntityKind::DashSwitch => {
                    let (left_side, ceiling) = match (entity.direction.x, entity.direction.y) {
                        (-1.0, _) => (Some(true), None),
                        (1.0, _) => (Some(false), None),
                        (_, -1.0) => (None, Some(true)),
                        _ => (None, Some(false)),
                    };
                    let mut attrs = vec![("id", BinaryValue::Int(id))];
                    attrs.push(("allGates", BinaryValue::Bool(entity.shielded)));
                    if let Some(left) = left_side {
                        attrs.push(("leftSide", BinaryValue::Bool(left)));
                    }
                    if let Some(ceiling) = ceiling {
                        attrs.push(("ceiling", BinaryValue::Bool(ceiling)));
                    }
                    attrs.push(("originX", BinaryValue::Int(0)));
                    attrs.push(("originY", BinaryValue::Int(0)));
                    attrs.push(("persistent", BinaryValue::Bool(entity.single_use)));
                    attrs.push(("width", BinaryValue::Int(width)));
                    attrs.push(("height", BinaryValue::Int(height)));
                    attrs.push(("x", BinaryValue::Int(x)));
                    attrs.push(("y", BinaryValue::Int(y)));
                    Some(element_vec(&entity.name, attrs, vec![]))
                }
                EntityKind::Decoration | EntityKind::Unknown => None,
            };
            if let Some(encoded) = encoded {
                entities.push(encoded);
            }
        }

        levels.push(encoded_level(
            format!("lvl_{room}"),
            map.bounds,
            triggers,
            entities,
            solids_text(map),
        ));
    }
    let root = BinaryElement {
        package: Some(package.to_owned()),
        name: "Map".to_owned(),
        attributes: BTreeMap::new(),
        children: vec![
            element("Filler", [], vec![]),
            element("levels", [], levels),
            element(
                "Style",
                [],
                vec![
                    element("Backgrounds", [], vec![]),
                    element("Foregrounds", [], vec![]),
                ],
            ),
        ],
    };
    Ok(encode_celeste_bin(&root)?)
}

fn encoded_level(
    name: String,
    bounds: Rect,
    triggers: Vec<BinaryElement>,
    entities: Vec<BinaryElement>,
    solids: String,
) -> BinaryElement {
    element(
        "level",
        [
            ("alt_music", BinaryValue::String(String::new())),
            ("ambience", BinaryValue::String(String::new())),
            ("c", BinaryValue::Int(0)),
            ("cameraOffsetX", BinaryValue::Int(0)),
            ("cameraOffsetY", BinaryValue::Int(0)),
            ("dark", BinaryValue::Bool(false)),
            ("disableDownTransition", BinaryValue::Bool(false)),
            ("height", BinaryValue::Int(bounds.height as i32)),
            ("music", BinaryValue::String(String::new())),
            ("musicLayer1", BinaryValue::Bool(true)),
            ("musicLayer2", BinaryValue::Bool(true)),
            ("musicLayer3", BinaryValue::Bool(true)),
            ("musicLayer4", BinaryValue::Bool(true)),
            ("name", BinaryValue::String(name)),
            ("space", BinaryValue::Bool(false)),
            ("underwater", BinaryValue::Bool(false)),
            ("whisper", BinaryValue::Bool(false)),
            ("width", BinaryValue::Int(bounds.width as i32)),
            ("windPattern", BinaryValue::String("None".to_owned())),
            ("x", BinaryValue::Int(bounds.x as i32)),
            ("y", BinaryValue::Int(bounds.y as i32)),
        ],
        vec![
            element(
                "triggers",
                [
                    ("offsetX", BinaryValue::Int(0)),
                    ("offsetY", BinaryValue::Int(0)),
                ],
                triggers,
            ),
            tile_layer("fgtiles", None),
            offset_container("fgdecals", vec![]),
            tile_layer("solids", Some(solids)),
            element(
                "entities",
                [
                    ("offsetX", BinaryValue::Int(0)),
                    ("offsetY", BinaryValue::Int(0)),
                ],
                entities,
            ),
            tile_layer("bgtiles", None),
            offset_container("bgdecals", vec![]),
            element(
                "bg",
                [
                    ("innerText", BinaryValue::String(String::new())),
                    ("offsetX", BinaryValue::Int(0)),
                    ("offsetY", BinaryValue::Int(0)),
                ],
                vec![],
            ),
            tile_layer("objtiles", None),
        ],
    )
}

fn element<const N: usize>(
    name: &str,
    attributes: [(&str, BinaryValue); N],
    children: Vec<BinaryElement>,
) -> BinaryElement {
    element_vec(name, attributes.into_iter().collect(), children)
}

fn element_vec(
    name: &str,
    attributes: Vec<(&str, BinaryValue)>,
    children: Vec<BinaryElement>,
) -> BinaryElement {
    BinaryElement {
        package: None,
        name: name.to_owned(),
        attributes: attributes
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
        children,
    }
}

fn offset_container(name: &str, children: Vec<BinaryElement>) -> BinaryElement {
    element(
        name,
        [
            ("offsetX", BinaryValue::Int(0)),
            ("offsetY", BinaryValue::Int(0)),
        ],
        children,
    )
}

fn tile_layer(name: &str, inner_text: Option<String>) -> BinaryElement {
    let mut attributes = vec![
        ("exportMode", BinaryValue::Int(0)),
        ("offsetX", BinaryValue::Int(0)),
        ("offsetY", BinaryValue::Int(0)),
        ("tileset", BinaryValue::String("Scenery".to_owned())),
    ];
    if let Some(inner_text) = inner_text {
        attributes.push(("innerText", BinaryValue::String(inner_text)));
    }
    element_vec(name, attributes, vec![])
}

fn solids_text(map: &Map) -> String {
    let width = (map.bounds.width / 8.0) as usize;
    let height = (map.bounds.height / 8.0) as usize;
    let mut cells = vec![vec!['0'; width]; height];
    for solid in &map.solids {
        let left = ((solid.x - map.bounds.x) / 8.0).floor().max(0.0) as usize;
        let top = ((solid.y - map.bounds.y) / 8.0).floor().max(0.0) as usize;
        let right = (((solid.right() - map.bounds.x) / 8.0).ceil() as usize).min(width);
        let bottom = (((solid.bottom() - map.bounds.y) / 8.0).ceil() as usize).min(height);
        for row in cells.iter_mut().take(bottom).skip(top) {
            for cell in row.iter_mut().take(right).skip(left) {
                *cell = '1';
            }
        }
    }
    cells
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn wind_pattern(direction: Vec2) -> String {
    if direction.x < 0.0 {
        "Left"
    } else if direction.x > 0.0 {
        "Right"
    } else if direction.y < 0.0 {
        "Up"
    } else {
        "Down"
    }
    .to_owned()
}

pub fn decode_map(bytes: &[u8]) -> Result<Map, MapError> {
    decode_map_room(bytes, None)
}

pub fn celeste_map_rooms(bytes: &[u8]) -> Result<Vec<String>, MapError> {
    let root = parse_celeste_bin(bytes).map_err(|e| MapError::Unsupported(e.to_string()))?;
    let levels = root
        .children
        .iter()
        .find(|element| element.name == "levels")
        .ok_or(MapError::NoLevel)?;
    let rooms = levels
        .children
        .iter()
        .filter_map(|level| attr_text(level, "name"))
        .map(|name| name.strip_prefix("lvl_").unwrap_or(name).to_owned())
        .collect::<Vec<_>>();
    if rooms.is_empty() {
        return Err(MapError::NoLevel);
    }
    Ok(rooms)
}

/// Enumerate raw room contents without discarding unsupported BinaryPacker
/// elements. This is intentionally separate from decoding: `EntityKind::Unknown`
/// is sufficient for simulation, while dataset builders need the original names
/// to make a conservative compatibility decision.
pub fn audit_celeste_map(bytes: &[u8]) -> Result<Vec<CelesteRoomAudit>, MapError> {
    let root = parse_celeste_bin(bytes).map_err(|e| MapError::Unsupported(e.to_string()))?;
    let levels = root
        .children
        .iter()
        .find(|element| element.name == "levels")
        .ok_or(MapError::NoLevel)?;
    let rooms = levels
        .children
        .iter()
        .map(|level| {
            let raw_name = attr_text(level, "name").ok_or(MapError::NoLevel)?;
            let name = raw_name.strip_prefix("lvl_").unwrap_or(raw_name).to_owned();
            let x = attr_f32(level, "x", 0.0);
            let y = attr_f32(level, "y", 0.0);
            let mut spawns = Vec::new();
            let mut entity_names = BTreeMap::new();
            if let Some(entities) = level.children.iter().find(|e| e.name == "entities") {
                for entity in &entities.children {
                    *entity_names.entry(entity.name.clone()).or_insert(0) += 1;
                    if entity.name == "player" {
                        spawns.push(Vec2::new(
                            x + attr_f32(entity, "x", 0.0),
                            y + attr_f32(entity, "y", 0.0),
                        ));
                    }
                }
            }
            let mut trigger_names = BTreeMap::new();
            if let Some(triggers) = level.children.iter().find(|e| e.name == "triggers") {
                for trigger in &triggers.children {
                    *trigger_names.entry(trigger.name.clone()).or_insert(0) += 1;
                }
            }
            Ok(CelesteRoomAudit {
                name,
                bounds: level_room_bounds(
                    x,
                    y,
                    attr_f32(level, "width", 320.0),
                    attr_f32(level, "height", 180.0),
                ),
                spawns,
                entity_names,
                trigger_names,
            })
        })
        .collect::<Result<Vec<_>, MapError>>()?;
    if rooms.is_empty() {
        return Err(MapError::NoLevel);
    }
    Ok(rooms)
}

pub fn decode_map_room(bytes: &[u8], room: Option<&str>) -> Result<Map, MapError> {
    if let Ok(map) = rmp_serde::from_slice::<Map>(bytes) {
        return Ok(map);
    }
    let root = parse_celeste_bin(bytes).map_err(|e| MapError::Unsupported(e.to_string()))?;
    map_from_binary(root, room)
}

/// Decode only the selected room. Adjacent room bounds are retained, but their
/// full runtime geometry is omitted to keep offline room catalogs linear in
/// map size rather than duplicating every room into every record.
pub fn decode_map_room_local(bytes: &[u8], room: Option<&str>) -> Result<Map, MapError> {
    if let Ok(map) = rmp_serde::from_slice::<Map>(bytes) {
        return Ok(map);
    }
    let root = parse_celeste_bin(bytes).map_err(|e| MapError::Unsupported(e.to_string()))?;
    map_from_binary_inner(root, room, false)
}

fn attr_f32(el: &BinaryElement, key: &str, default: f32) -> f32 {
    match el.attributes.get(key) {
        Some(BinaryValue::Byte(v)) => *v as f32,
        Some(BinaryValue::Short(v)) => *v as f32,
        Some(BinaryValue::Int(v)) => *v as f32,
        Some(BinaryValue::Float(v)) => *v,
        Some(BinaryValue::String(v)) => v.parse().unwrap_or(default),
        _ => default,
    }
}

fn attr_text<'a>(el: &'a BinaryElement, key: &str) -> Option<&'a str> {
    match el.attributes.get(key) {
        Some(BinaryValue::String(v)) => Some(v),
        _ => None,
    }
}

fn attr_char(el: &BinaryElement, key: &str) -> Option<char> {
    match el.attributes.get(key) {
        Some(BinaryValue::String(v)) => v.chars().next(),
        Some(BinaryValue::Int(v)) => char::from_u32(*v as u32),
        _ => None,
    }
}

fn attr_bool(el: &BinaryElement, key: &str, default: bool) -> bool {
    match el.attributes.get(key) {
        Some(BinaryValue::Bool(value)) => *value,
        Some(BinaryValue::Byte(value)) => *value != 0,
        _ => default,
    }
}

fn map_from_binary(root: BinaryElement, room: Option<&str>) -> Result<Map, MapError> {
    map_from_binary_inner(root, room, true)
}

fn map_from_binary_inner(
    root: BinaryElement,
    room: Option<&str>,
    include_transition_runtime: bool,
) -> Result<Map, MapError> {
    let levels = root
        .children
        .iter()
        .find(|e| e.name == "levels")
        .ok_or(MapError::NoLevel)?;
    let level = match room {
        Some(room) => levels
            .children
            .iter()
            .find(|level| {
                attr_text(level, "name")
                    .is_some_and(|name| name == room || name.strip_prefix("lvl_") == Some(room))
            })
            .ok_or_else(|| MapError::RoomNotFound(room.to_owned()))?,
        None => levels.children.first().ok_or(MapError::NoLevel)?,
    };
    let x = attr_f32(level, "x", 0.0);
    let y = attr_f32(level, "y", 0.0);
    let width = attr_f32(level, "width", 320.0);
    let height = attr_f32(level, "height", 180.0);
    let mut map = Map {
        bounds: level_room_bounds(x, y, width, height),
        // `Level.LoadLevel`'s `Calc.PushRandom(Session.LevelData.LoadSeed)` (`Level.cs:386`) is
        // what makes `FloatySpaceBlock`'s constructor draw reproducible (`FloatySpaceBlock.cs:52`).
        load_seed: crate::legacy_random::load_seed(&attr_text(level, "name").unwrap_or_default()),
        transition_rooms: levels
            .children
            .iter()
            .filter(|candidate| !std::ptr::eq(*candidate, level))
            .map(|candidate| {
                level_room_bounds(
                    attr_f32(candidate, "x", 0.0),
                    attr_f32(candidate, "y", 0.0),
                    attr_f32(candidate, "width", 320.0),
                    attr_f32(candidate, "height", 180.0),
                )
            })
            .collect(),
        source_package: root.package.clone(),
        ..Map::default()
    };

    if let Some(entities) = level.children.iter().find(|e| e.name == "entities") {
        for el in &entities.children {
            let ex = x + attr_f32(el, "x", 0.0);
            let ey = y + attr_f32(el, "y", 0.0);
            if el.name == "player" {
                let spawn = Vec2::new(ex, ey);
                if map.room_spawns.is_empty() {
                    map.spawn = spawn;
                }
                map.room_spawns.push(spawn);
                continue;
            }
            let registered = crate::entity_decode::lookup(el);
            map.solids
                .extend(crate::entity_decode::additional_solids(el, ex, ey));
            let kind = match el.name.as_str() {
                "jumpThru" => EntityKind::JumpThru,
                "dreamBlock" => EntityKind::DreamBlock,
                "spikesUp" | "spikesDown" | "spikesLeft" | "spikesRight" => EntityKind::Spikes,
                "water" => EntityKind::Water,
                "booster" if attr_bool(el, "red", false) => EntityKind::RedBooster,
                "booster" => EntityKind::Booster,
                "redBooster" => EntityKind::RedBooster,
                "infiniteStar" | "flyFeather" => EntityKind::FlyFeather,
                "bigSpinner" => EntityKind::Bumper,
                "fireBall" if attr_bool(el, "notCoreMode", false) => EntityKind::IceBall,
                "puffer" => EntityKind::Puffer,
                "oshiroBoss" => EntityKind::AngryOshiro,
                "seeker" => EntityKind::Seeker,
                "snowball" => EntityKind::Snowball,
                "killbox" => EntityKind::Killbox,
                "cloud" if !attr_bool(el, "fragile", false) => EntityKind::Cloud,
                "badelineBoost" => EntityKind::BadelineBoost,
                "spring" | "wallSpringLeft" | "wallSpringRight" => EntityKind::Spring,
                "strawberry" => EntityKind::Strawberry,
                "refill" => EntityKind::Refill,
                "fallingBlock" => EntityKind::FallingBlock,
                "exitBlock" => EntityKind::ExitBlock,
                "invisibleBarrier" => EntityKind::InvisibleBarrier,
                "windTrigger" => EntityKind::Wind,
                "bounceBlock" => EntityKind::BounceBlock,
                "theoCrystal" => EntityKind::TheoCrystal,
                "blackGem" | "heartGem" => EntityKind::HeartGem,
                "risingLava" => EntityKind::RisingLava,
                "sandwichLava" => EntityKind::SandwichLava,
                "glider" => EntityKind::Glider,
                "zipMover" => EntityKind::ZipMover,
                "moveBlock" => EntityKind::MoveBlock,
                "templeGate" => EntityKind::TempleGate,
                "cassetteBlock" => EntityKind::CassetteBlock,
                "spinner" | "VivHelper/CustomSpinner" | "FrostHelper/IceSpinner" => {
                    EntityKind::CrystalStaticSpinner
                }
                "towerviewer" | "lookout" => EntityKind::Lookout,
                "crushBlock" => EntityKind::CrushBlock,
                "dashBlock" => EntityKind::DashBlock,
                "wallBooster" => EntityKind::WallBooster,
                "coreModeToggle" => EntityKind::CoreModeToggle,
                "SummitBackgroundManager" => EntityKind::SummitBackgroundManager,
                "plateau" | "bridgeFixed" | "starJumpBlock" | "crumbleWallOnRumble" => EntityKind::StaticSolid,
                "resortRoofEnding" => EntityKind::StaticSolid,
                "crumbleBlock" => EntityKind::CrumbleBlock,
                "floatySpaceBlock" => EntityKind::FloatySpaceBlock,
                "switchGate" => EntityKind::SwitchGate,
                "touchSwitch" => EntityKind::TouchSwitch,
                // `DashSwitch.Create` (`DashSwitch.cs:103-122`) dispatches on these two names
                // alone; the bare `dashSwitch` this arm used to accept does not exist in any
                // vanilla map, so every real dash switch used to arrive as `Unknown`.
                "dashSwitchH" | "dashSwitchV" => EntityKind::DashSwitch,
                "celesteGymMovingSolid" => EntityKind::MovingSolid,
                _ => registered.map_or(EntityKind::Unknown, |entry| entry.kind),
            };
            let default_w = registered.map_or_else(
                || match kind {
                    EntityKind::Booster | EntityKind::RedBooster => 16.0,
                    EntityKind::FlyFeather => 20.0,
                    EntityKind::Bumper => 24.0,
                    EntityKind::IceBall => 12.0,
                    EntityKind::Puffer => 12.0,
                    EntityKind::AngryOshiro => 28.0,
                    EntityKind::Seeker => 12.0,
                    EntityKind::Snowball => 12.0,
                    EntityKind::Killbox => 32.0,
                    EntityKind::Cloud => 32.0,
                    EntityKind::BadelineBoost => 32.0,
                    EntityKind::Strawberry => 14.0,
                    EntityKind::Refill => 16.0,
                    EntityKind::TheoCrystal | EntityKind::Glider | EntityKind::TempleGate => 8.0,
                    EntityKind::CrystalStaticSpinner => 16.0,
                    EntityKind::Lookout => 4.0,
                    EntityKind::HeartGem => 16.0,
                    EntityKind::RisingLava | EntityKind::SandwichLava => 340.0,
                    // `CrushBlock` in every vanilla map carries an explicit
                    // `width`/`height` (24-48 px); `DashBlock` likewise (16-96 px).
                    EntityKind::CrushBlock => 32.0,
                    EntityKind::DashBlock => 16.0,
                    _ => 8.0,
                },
                |entry| entry.default_width,
            );
            let default_h = registered.map_or_else(
                || match kind {
                    EntityKind::TheoCrystal | EntityKind::Glider | EntityKind::Puffer => 10.0,
                    EntityKind::Snowball => 9.0,
                    EntityKind::Killbox => 32.0,
                    EntityKind::Cloud => 5.0,
                    EntityKind::RisingLava | EntityKind::SandwichLava => 120.0,
                    EntityKind::CrystalStaticSpinner => 12.0,
                    EntityKind::Lookout => 4.0,
                    EntityKind::CrushBlock => 32.0,
                    EntityKind::DashBlock => 16.0,
                    _ => default_w,
                },
                |entry| entry.default_height,
            );
            let raw_width = attr_f32(el, "width", default_w);
            let raw_height = attr_f32(el, "height", default_h);
            let (bounds, direction) = if let Some(entry) = registered {
                entry.bounds_and_direction(
                    ex,
                    ey,
                    raw_width,
                    raw_height,
                    attr_bool(el, "twoDash", false),
                )
            } else {
                match el.name.as_str() {
                    "spikesUp" => (
                        Rect::new(ex, ey - 3.0, raw_width, 3.0),
                        Vec2::new(0.0, -1.0),
                    ),
                    "spikesDown" => (Rect::new(ex, ey, raw_width, 3.0), Vec2::new(0.0, 1.0)),
                    "spikesLeft" => (
                        Rect::new(ex - 3.0, ey, 3.0, raw_height),
                        Vec2::new(-1.0, 0.0),
                    ),
                    "spikesRight" => (Rect::new(ex, ey, 3.0, raw_height), Vec2::new(1.0, 0.0)),
                    "booster" | "redBooster" | "infiniteStar" | "flyFeather" | "bigSpinner"
                    | "fireBall" | "badelineBoost" | "strawberry" | "puffer" | "oshiroBoss"
                    | "seeker" | "snowball" => (
                        Rect::new(
                            ex - raw_width * 0.5,
                            ey - raw_height * 0.5,
                            raw_width,
                            raw_height,
                        ),
                        Vec2::default(),
                    ),
                    "cloud" => (
                        Rect::new(ex - raw_width * 0.5, ey, raw_width, raw_height),
                        Vec2::default(),
                    ),
                    "refill" => (
                        Rect::new(ex - 8.0, ey - 8.0, 16.0, 16.0),
                        Vec2::new(
                            if attr_bool(el, "twoDash", false) {
                                1.0
                            } else {
                                0.0
                            },
                            0.0,
                        ),
                    ),
                    "fallingBlock" => (
                        Rect::new(ex, ey, raw_width, raw_height),
                        Vec2::new(
                            if attr_bool(el, "climbFall", true) {
                                1.0
                            } else {
                                0.0
                            },
                            if attr_bool(el, "behind", false) {
                                1.0
                            } else {
                                0.0
                            },
                        ),
                    ),
                    "spring" => (
                        Rect::new(ex - 8.0, ey - 6.0, 16.0, 6.0),
                        Vec2::new(0.0, -1.0),
                    ),
                    "wallSpringLeft" => (Rect::new(ex, ey - 8.0, 6.0, 16.0), Vec2::new(1.0, 0.0)),
                    "wallSpringRight" => (
                        Rect::new(ex - 6.0, ey - 8.0, 6.0, 16.0),
                        Vec2::new(-1.0, 0.0),
                    ),
                    // `Celeste.JumpthruPlatform` forwards only `data.Position` and `data.Width`
                    // to `JumpThru`, which replaces the entity data's 8 px `height` with
                    // `new Hitbox(width, 5f)` anchored at the entity's top-left
                    // (`JumpThru.cs:12`, `JumpthruPlatform.cs:23-26`). Using the raw attribute made
                    // the collider 3 px too tall, which fired the `Player.Update` JumpThru Assist
                    // (`Player.cs:1787-1790`) on frames where the real game skips it and stole
                    // `40 * Engine.DeltaTime` of sub-pixel budget.
                    "jumpThru" => (
                        Rect::new(
                            ex,
                            ey,
                            raw_width,
                            crate::entity_decode::JUMP_THRU_COLLIDER_HEIGHT,
                        ),
                        Vec2::default(),
                    ),
                    "bounceBlock" | "zipMover" | "templeGate" | "exitBlock"
                    | "invisibleBarrier" => {
                        (Rect::new(ex, ey, raw_width, raw_height), Vec2::default())
                    }
                    "killbox" => (Rect::new(ex, ey, raw_width, 32.0), Vec2::default()),
                    "cassetteBlock" => (
                        Rect::new(ex, ey, raw_width, raw_height),
                        Vec2::new(attr_f32(el, "index", 0.0), attr_f32(el, "tempo", 1.0)),
                    ),
                    "spinner" => (
                        Rect::new(
                            ex - raw_width * 0.5,
                            ey - raw_height * 0.5,
                            raw_width,
                            raw_height,
                        ),
                        Vec2::default(),
                    ),
                    "towerviewer" | "lookout" => (
                        Rect::new(ex - 2.0, ey - 4.0, 4.0, 4.0),
                        Vec2::new(
                            if attr_bool(el, "onlyY", false) {
                                1.0
                            } else {
                                0.0
                            },
                            if attr_bool(el, "summit", false) {
                                1.0
                            } else {
                                0.0
                            },
                        ),
                    ),
                    "moveBlock" => {
                        let direction = match attr_text(el, "direction").unwrap_or("Right") {
                            "Left" => Vec2::new(-1.0, 0.0),
                            "Up" => Vec2::new(0.0, -1.0),
                            "Down" => Vec2::new(0.0, 1.0),
                            _ => Vec2::new(1.0, 0.0),
                        };
                        (Rect::new(ex, ey, raw_width, raw_height), direction)
                    }
                    "theoCrystal" | "glider" => {
                        (Rect::new(ex - 4.0, ey - 10.0, 8.0, 10.0), Vec2::default())
                    }
                    "blackGem" | "heartGem" => {
                        (Rect::new(ex - 8.0, ey - 8.0, 16.0, 16.0), Vec2::default())
                    }
                    "risingLava" | "sandwichLava" => {
                        (Rect::new(ex, ey, 340.0, 120.0), Vec2::default())
                    }
                    "celesteGymMovingSolid" => (
                        Rect::new(ex, ey, raw_width, raw_height),
                        Vec2::new(attr_f32(el, "speedX", 0.0), attr_f32(el, "speedY", 0.0)),
                    ),
                    // `CrushBlock(EntityData data, Vector2 offset) : this(data.Position
                    // + offset, data.Width, data.Height, data.Enum("axes", Axes.Both),
                    // data.Bool("chillout"))` (`CrushBlock.cs:146-149`) forwards the raw
                    // rectangle straight into `Solid(position, width, height, safe:
                    // false)` (`CrushBlock.cs:85-87`). `axes` selects
                    // `canMoveHorizontally`/`canMoveVertically` (`CrushBlock.cs:98-114`)
                    // and `chillout` disables the return leg and the re-arm
                    // (`CrushBlock.cs:424-427,564-567`). Both are kept in `direction`
                    // because `CanActivate` (`CrushBlock.cs:284-303`) needs them at
                    // collision time.
                    "crushBlock" => (
                        Rect::new(ex, ey, raw_width, raw_height),
                        Vec2::new(
                            match attr_text(el, "axes").unwrap_or("Both") {
                                "Horizontal" => 1.0,
                                "Vertical" => 2.0,
                                _ => 0.0,
                            },
                            if attr_bool(el, "chillout", false) {
                                1.0
                            } else {
                                0.0
                            },
                        ),
                    ),
                    // `DashBlock(EntityData data, Vector2 offset, EntityID id) : this(
                    // data.Position + offset, data.Char("tiletype", '3'), data.Width,
                    // data.Height, data.Bool("blendin"), data.Bool("permanent", true),
                    // data.Bool("canDash", true), id)` (`DashBlock.cs:45-47`) also
                    // forwards the raw rectangle to `Solid(..., safe: true)`
                    // (`DashBlock.cs:30-32`). `canDash` gates the dash rebound and
                    // `permanent` decides whether a broken block is flagged as gone
                    // (`DashBlock.cs:86-139`).
                    "dashBlock" => (
                        Rect::new(ex, ey, raw_width, raw_height),
                        Vec2::new(
                            if attr_bool(el, "canDash", true) {
                                1.0
                            } else {
                                0.0
                            },
                            if attr_bool(el, "permanent", true) {
                                1.0
                            } else {
                                0.0
                            },
                        ),
                    ),
                    // `Plateau(e, offset) : base(e.Position + offset, 104f, 4f, safe: true)`
                    // (`Plateau.cs:12-14`) then `Collider.Left += 8f` (`:15`), i.e. the collider
                    // slides eight pixels right of the entity position and keeps its 104 px width;
                    // `BridgeFixed(data, offset) : base(data.Position + offset, data.Width, 8f,
                    // safe: true)` (`BridgeFixed.cs:9-11`) takes the raw rectangle with a fixed 8 px
                    // height. Neither has any state, so the rectangle is the whole model.
                    "plateau" => (Rect::new(ex + 8.0, ey, 104.0, 4.0), Vec2::default()),
                    "bridgeFixed" => (Rect::new(ex, ey, raw_width, 8.0), Vec2::default()),
                    // `CrumblePlatform(EntityData, offset) : base(position, width, 8f, safe: false)`
                    // (`CrumblePlatform.cs:8`): the raw rectangle is the collider, eight pixels high.
                    "crumbleBlock" => (Rect::new(ex, ey, raw_width, 8.0), Vec2::default()),
                    // `FloatySpaceBlock(EntityData data, Vector2 offset) : this(data.Position +
                    // offset, data.Width, data.Height, data.Char("tiletype", '3'),
                    // data.Bool("disableSpawnOffset"))` (`FloatySpaceBlock.cs:60-63`) forwards the
                    // raw rectangle to `Solid(position, width, height, safe: true)` (`:44`), whose
                    // `Collider = new Hitbox(width, height)` keeps offset `(0, 0)` - so the map
                    // rectangle *is* the collider. `direction.x` is `disableSpawnOffset`
                    // (`:50-57`) and `direction.y` the `tiletype` char code, the two constructor
                    // inputs `sim::initialize_floaty_blocks` needs.
                    "floatySpaceBlock" => (
                        Rect::new(ex, ey, raw_width, raw_height),
                        Vec2::new(
                            if attr_bool(el, "disableSpawnOffset", false) {
                                1.0
                            } else {
                                0.0
                            },
                            attr_char(el, "tiletype").unwrap_or('3') as u32 as f32,
                        ),
                    ),
                    // `SwitchGate(data, offset) : base(data.Position + offset, data.Width,
                    // data.Height, safe: false)` (`SwitchGate.cs:34-35`): the raw rectangle; its
                    // `nodes[0]` target is decoded generically and `direction.x` carries
                    // `data.Bool("persistent")`, which decides whether the sequence writes the
                    // room's `switches_<room>` session flag (`SwitchGate.cs:109-112`).
                    "switchGate" => (
                        Rect::new(ex, ey, raw_width, raw_height),
                        Vec2::new(
                            if attr_bool(el, "persistent", false) {
                                1.0
                            } else {
                                0.0
                            },
                            0.0,
                        ),
                    ),
                    // `TouchSwitch(data, offset) : this(data.Position + offset)` (`TouchSwitch.cs:86`)
                    // with a `Hitbox(30f, 30f, -15f, -15f)` (`:45`).
                    "touchSwitch" => (Rect::new(ex - 15.0, ey - 15.0, 30.0, 30.0), Vec2::default()),
                    // `DashSwitch(position, side, ...) : base(position, 0f, 0f, safe: true)`
                    // (`DashSwitch.cs:52-53`) then `Collider.Width = 16f; Height = 8f` for
                    // `Sides.Up`/`Down` and `8f`/`16f` for `Sides.Left`/`Right` (`:62-71`). The
                    // collider keeps offset `(0, 0)`, so it is `(position, 16, 8)` or
                    // `(position, 8, 16)` - *not* the map rectangle, which is 0-sized in every
                    // vanilla element. `direction` carries `pressDirection` (`:74-98`), which is
                    // also what `OnDashed` compares the dash direction against (`:197`).
                    "dashSwitchH" => (
                        Rect::new(ex, ey, 8.0, 16.0),
                        if attr_bool(el, "leftSide", false) {
                            Vec2::new(-1.0, 0.0)
                        } else {
                            Vec2::new(1.0, 0.0)
                        },
                    ),
                    "dashSwitchV" => (
                        Rect::new(ex, ey, 16.0, 8.0),
                        if attr_bool(el, "ceiling", false) {
                            Vec2::new(0.0, -1.0)
                        } else {
                            Vec2::new(0.0, 1.0)
                        },
                    ),
                    // `AscendManager(EntityData data, Vector2 offset)` (`AscendManager.cs:228-233`)
                    // reads `index` and `intro_launch`; only `Position.Y` drives the takeover
                    // condition, and `index` into `direction.x`.
                    "SummitBackgroundManager" => (
                        Rect::new(ex, ey, raw_width, raw_height),
                        Vec2::new(attr_f32(el, "index", 0.0), 0.0),
                    ),
                    // `CoreModeToggle(EntityData data, Vector2 offset) : this(data.Position + offset,
                    // data.Bool("onlyFire"), data.Bool("onlyIce"), data.Bool("persistent"))`
                    // (`CoreModeToggle.cs:53-54`). The constructor installs
                    // `new Hitbox(16f, 24f, -8f, -12f)` (`:46`), a 16x24 box centred on the entity
                    // position, `direction.x`/`direction.y` carry `onlyFire`/`onlyIce` for the
                    // `Usable` gate (`:24-38`) and `single_use` carries `persistent`.
                    "coreModeToggle" => (
                        Rect::new(ex - 8.0, ey - 12.0, 16.0, 24.0),
                        Vec2::new(
                            if attr_bool(el, "onlyFire", false) {
                                1.0
                            } else {
                                0.0
                            },
                            if attr_bool(el, "onlyIce", false) {
                                1.0
                            } else {
                                0.0
                            },
                        ),
                    ),
                    // `WallBooster(EntityData data, Vector2 offset) : this(data.Position +
                    // offset, data.Height, data.Bool("left"), data.Bool("notCoreMode"))`
                    // (`WallBooster.cs:47-50`). The constructor (`WallBooster.cs:24-39`)
                    // gives a left booster `new Hitbox(2f, height)` at the entity position
                    // and a right one `new Hitbox(2f, height, 6f)`, i.e. the same 2 px strip
                    // six pixels to the right; `left` also picks `Facing`
                    // (`WallBooster.cs:30-39`), which `Player.WallBoosterCheck`
                    // (`Player.cs:3284-3286`) matches against the player's. `notCoreMode` is
                    // packed into `direction.y` the way `crushBlock`/`dashBlock` pack their
                    // own flags, because it decides `IceMode` (`WallBooster.cs:77-101`) and
                    // with it whether the strip is a blocking `ClimbBlocker`.
                    "wallBooster" => {
                        let left = attr_bool(el, "left", false);
                        (
                            if left {
                                Rect::new(ex, ey, 2.0, raw_height)
                            } else {
                                Rect::new(ex + 6.0, ey, 2.0, raw_height)
                            },
                            Vec2::new(
                                if left { -1.0 } else { 1.0 },
                                if attr_bool(el, "notCoreMode", false) {
                                    1.0
                                } else {
                                    0.0
                                },
                            ),
                        )
                    }
                    _ => (Rect::new(ex, ey, raw_width, raw_height), Vec2::default()),
                }
            };
            let visual = match el.name.as_str() {
                "VivHelper/CustomSpinner" => {
                    let dir = attr_text(el, "Directory").unwrap_or_default();
                    let sub = attr_text(el, "Subdirectory").unwrap_or_default();
                    EntityVisual {
                        texture: (!dir.is_empty()).then(|| {
                            if sub.is_empty() {
                                format!("{dir}/fg")
                            } else {
                                format!("{dir}/fg_{sub}")
                            }
                        }),
                        ..EntityVisual::default()
                    }
                }
                "FrostHelper/IceSpinner" => {
                    let dir = attr_text(el, "directory").unwrap_or_default();
                    EntityVisual {
                        // "danger/crystal" is the vanilla crystal directory, whose
                        // fg_blue/fg_red/... sheets the custom entity does not select;
                        // let the web fall back to the theme's own crystal.
                        texture: (!dir.is_empty() && dir != "danger/crystal")
                            .then(|| format!("{dir}/fg")),
                        tint: attr_text(el, "tint").map(str::to_owned),
                        ..EntityVisual::default()
                    }
                }
                "spinner" => EntityVisual {
                    variant: attr_text(el, "type").map(str::to_owned),
                    ..EntityVisual::default()
                },
                // `ClutterSwitch(EntityData data, Vector2 offset)` forwards
                // `data.Enum("type", ClutterBlock.Colors.Green)`
                // (`ClutterSwitch.cs:65-68`), and `ClutterDoor` carries the same
                // attribute. `sim.rs` needs the colour to deactivate exactly the
                // `ClutterBlockBase` pile of the switch that was pressed
                // (`ClutterBlockGenerator.cs:78-81,136-138`).
                "colorSwitch" | "clutterDoor" => EntityVisual {
                    variant: attr_text(el, "type").map(str::to_owned),
                    ..EntityVisual::default()
                },
                // `TempleGate(EntityData data, Vector2 offset, string levelID)` forwards
                // `data.Enum("type", Types.NearestSwitch)` (`TempleGate.cs:72-74`). The whole
                // behaviour of the gate hangs off that enum - `Awake` (`:77-112`) starts
                // `NearestSwitch` gates *closed* and adds a different coroutine per other type, and
                // `DashSwitch.GetGate` (`DashSwitch.cs:231-253`) only ever claims a
                // `NearestSwitch` one - so it rides `variant` exactly like the colour switch's
                // `ClutterBlock.Colors`.
                "templeGate" => EntityVisual {
                    variant: attr_text(el, "type").map(str::to_owned),
                    ..EntityVisual::default()
                },
                _ => EntityVisual {
                    tile: match kind {
                        EntityKind::FallingBlock => Some(attr_char(el, "tiletype").unwrap_or('3')),
                        EntityKind::ExitBlock => Some(attr_char(el, "tileType").unwrap_or('3')),
                        // `DashBlock` takes `data.Char("tiletype", '3')`
                        // (`DashBlock.cs:46`) and draws itself with
                        // `FGAutotiler.GenerateBox/GenerateOverlay`
                        // (`DashBlock.cs:56-68`).
                        EntityKind::DashBlock => Some(attr_char(el, "tiletype").unwrap_or('3')),
                        // `FloatySpaceBlock` takes `data.Char("tiletype", '3')`
                        // (`FloatySpaceBlock.cs:61`) and feeds it to the autotiler and to the
                        // same-`tiletype` group test (`:189`).
                        EntityKind::FloatySpaceBlock => {
                            Some(attr_char(el, "tiletype").unwrap_or('3'))
                        }
                        _ => None,
                    },
                    // `DashBlock.blendin` is presentation only, so it is kept with
                    // the skin metadata instead of the physics rectangle, which the
                    // encoder reads back when rebuilding a `dashBlock` element.
                    variant: match kind {
                        EntityKind::DashBlock if attr_bool(el, "blendin", false) => {
                            Some("blendin".to_owned())
                        }
                        _ => None,
                    },
                    ..EntityVisual::default()
                },
            };
            map.entity_ids.push(attr_f32(el, "id", -1.0) as i32);
                map.entities.push(Entity {
                kind,
                bounds,
                direction,
                // `DashSwitch.Create` reads `data.Bool("persistent")` (`DashSwitch.cs:106`),
                // `data.Bool("allGates")` (`:107`) and `data.Attr("sprite", "default")` (`:108`).
                // `persistent` rides `single_use` (only a persistent press writes
                // `Session.SetFlag`, `:223-226`, and only that flag makes `Awake` (`:124-149`)
                // start the switch already pushed) and `allGates` rides `shielded`, the only
                // other per-kind bool left on `Entity`; no other reader of `shielded` can see a
                // `DashSwitch`, because both of them sit inside `Spikes`/`FlyFeather` match arms
                // (`sim.rs`). `allGates` decides whether one press opens its claimed gate or
                // *every* `NearestSwitch` gate of the room with the switch's own `LevelID`
                // (`DashSwitch.cs:208-221`).
                shielded: registered.is_some_and(|entry| entry.shielded)
                    || attr_bool(el, "shielded", false)
                    || (kind == EntityKind::DashSwitch && attr_bool(el, "allGates", false)),
                single_use: match kind {
                    EntityKind::RisingLava => attr_bool(el, "intro", false),
                    EntityKind::Refill => attr_bool(el, "oneUse", false),
                    EntityKind::CoreModeToggle => attr_bool(el, "persistent", false),
                    EntityKind::DashSwitch => attr_bool(el, "persistent", false),
                    _ => attr_bool(el, "singleUse", false),
                },
                nodes: el
                    .children
                    .iter()
                    .filter(|node| node.name == "node")
                    .map(|node| {
                        Vec2::new(x + attr_f32(node, "x", 0.0), y + attr_f32(node, "y", 0.0))
                    })
                    .collect(),
                name: el.name.clone(),
            });
            map.entity_visuals.push(visual);
        }
    }

    if let Some(triggers) = level.children.iter().find(|e| e.name == "triggers") {
        for trigger in &triggers.children {
            if trigger.name != "windTrigger" {
                continue;
            }
            let pattern = attr_text(trigger, "pattern").unwrap_or("None");
            let direction = match pattern {
                "Left" => Vec2::new(-400.0, 0.0),
                "Right" => Vec2::new(400.0, 0.0),
                "LeftStrong" => Vec2::new(-800.0, 0.0),
                "RightStrong" => Vec2::new(800.0, 0.0),
                "RightCrazy" => Vec2::new(1200.0, 0.0),
                "Up" => Vec2::new(0.0, -400.0),
                "Down" => Vec2::new(0.0, 300.0),
                "Space" => Vec2::new(0.0, -600.0),
                _ => Vec2::default(),
            };
            map.entity_ids.push(attr_f32(trigger, "id", -1.0) as i32);
                map.entities.push(Entity {
                kind: EntityKind::Wind,
                bounds: Rect::new(
                    x + attr_f32(trigger, "x", 0.0),
                    y + attr_f32(trigger, "y", 0.0),
                    attr_f32(trigger, "width", 8.0),
                    attr_f32(trigger, "height", 8.0),
                ),
                direction,
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: trigger.name.clone(),
            });
        }
    }

    // Level.LoadLevel keeps the source LevelData live until the transition
    // coroutine completes. Decoding destination tiles here would make them
    // collide during Player.TransitionTo, before that LoadLevel boundary.
    if let Some(solids) = level.children.iter().find(|e| e.name == "solids")
        && let Some(text) = attr_text(solids, "innerText")
    {
        map.tile_grid = text.lines().map(str::to_owned).collect();
        map.solids.extend(tile_rects(text, x, y));
    }
    if include_transition_runtime {
        map.transition_runtime = levels
            .children
            .iter()
            .map(|candidate| {
                let name = attr_text(candidate, "name").ok_or(MapError::NoLevel)?;
                let decoded = map_from_binary_inner(root.clone(), Some(name), false)?;
                Ok(RoomRuntime {
                    bounds: decoded.bounds,
                    spawns: if decoded.room_spawns.is_empty() {
                        vec![decoded.spawn]
                    } else {
                        decoded.room_spawns
                    },
                    solids: decoded.solids,
                    entities: decoded.entities,
                    load_seed: decoded.load_seed,
                })
            })
            .collect::<Result<Vec<_>, MapError>>()?;
    }
    Ok(map)
}

fn tile_rects(text: &str, ox: f32, oy: f32) -> Vec<Rect> {
    let rows: Vec<&str> = text.lines().collect();
    if rows.is_empty() {
        return vec![];
    }
    let width = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    let occupied = |x: usize, y: usize| {
        rows.get(y)
            .and_then(|r| r.chars().nth(x))
            .is_some_and(|c| c != '0' && c != ' ')
    };
    let mut seen = vec![vec![false; width]; rows.len()];
    let mut out = Vec::new();
    for (ty, row_seen) in seen.iter_mut().enumerate() {
        for tx in 0..width {
            if row_seen[tx] || !occupied(tx, ty) {
                continue;
            }
            let mut run = 1;
            while tx + run < width && !row_seen[tx + run] && occupied(tx + run, ty) {
                run += 1;
            }
            for cell in &mut row_seen[tx..tx + run] {
                *cell = true;
            }
            out.push(Rect::new(
                ox + tx as f32 * 8.0,
                oy + ty as f32 * 8.0,
                run as f32 * 8.0,
                8.0,
            ));
        }
    }
    out
}

impl Map {
    pub fn static_solid_at(&self, rect: Rect) -> bool {
        self.solids.iter().any(|solid| solid.intersects(rect))
    }

    pub fn dream_block_at(&self, rect: Rect) -> bool {
        self.entities
            .iter()
            .any(|entity| entity.kind == EntityKind::DreamBlock && entity.bounds.intersects(rect))
    }

    pub fn water_at(&self, rect: Rect) -> bool {
        self.entities
            .iter()
            .any(|entity| entity.kind == EntityKind::Water && entity.bounds.intersects(rect))
    }

    pub fn solid_at(&self, rect: Rect) -> bool {
        self.non_dream_solid_at(rect) || self.dream_block_at(rect)
    }

    pub fn non_dream_solid_at(&self, rect: Rect) -> bool {
        self.static_solid_at(rect)
            || self.entities.iter().any(|entity| {
                matches!(
                    entity.kind,
                    EntityKind::BounceBlock
                        | EntityKind::CassetteBlock
                        | EntityKind::CrushBlock
                        | EntityKind::DashBlock
                        | EntityKind::FallingBlock
                        | EntityKind::ExitBlock
                        | EntityKind::InvisibleBarrier
                        | EntityKind::MoveBlock
                        | EntityKind::MovingSolid
                        | EntityKind::ZipMover
                        | EntityKind::TempleGate
                        // Kept in step with `sim::is_solid_entity`: this list is what the
                        // ground probe consults, so a kind added only there is inert.
                        | EntityKind::CrumbleBlock
                        | EntityKind::FloatySpaceBlock
                        | EntityKind::StaticSolid
                        | EntityKind::SwitchGate
                        | EntityKind::DashSwitch
                ) && entity.bounds.intersects(rect)
            })
    }

    pub fn jump_thru_at(&self, rect: Rect, previous_bottom: f32) -> bool {
        self.entities.iter().any(|e| {
            matches!(e.kind, EntityKind::JumpThru | EntityKind::Cloud)
                && previous_bottom <= e.bounds.y
                && e.bounds.intersects(rect)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coalesces_solid_tiles() {
        assert_eq!(
            tile_rects("1110\n0010", 0.0, 0.0),
            vec![
                Rect::new(0.0, 0.0, 24.0, 8.0),
                Rect::new(16.0, 8.0, 8.0, 8.0)
            ]
        );
    }

    #[test]
    fn single_room_solids_round_trip_unchanged() {
        // 176 keeps the room tile-aligned, which is what the encoder requires.
        // The 184 -> 180 `LevelData` rewrite is covered separately by
        // `level_data_clamps_a_184_pixel_room_height_to_180`.
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 176.0),
            solids: vec![Rect::new(16.0, 168.0, 32.0, 8.0)],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "single").unwrap();
        let decoded = decode_map_room(&bytes, Some("single")).unwrap();
        assert_eq!(decoded.bounds, map.bounds);
        assert!(decoded.transition_rooms.is_empty());
        assert_eq!(decoded.solids, map.solids);
    }

    #[test]
    fn celeste_spring_entities_round_trip_with_source_colliders() {
        let springs = vec![
            Entity {
                kind: EntityKind::Spring,
                bounds: Rect::new(72.0, 90.0, 16.0, 6.0),
                direction: Vec2::new(0.0, -1.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "spring".to_owned(),
            },
            Entity {
                kind: EntityKind::Spring,
                bounds: Rect::new(120.0, 72.0, 6.0, 16.0),
                direction: Vec2::new(1.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "wallSpringLeft".to_owned(),
            },
            Entity {
                kind: EntityKind::Spring,
                bounds: Rect::new(154.0, 72.0, 6.0, 16.0),
                direction: Vec2::new(-1.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "wallSpringRight".to_owned(),
            },
        ];
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: springs.clone(),
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "springs").unwrap();
        let decoded = decode_map_room(&bytes, Some("springs")).unwrap();
        assert_eq!(decoded.entities, springs);
    }

    #[test]
    fn celeste_strawberry_round_trips_with_its_fourteen_pixel_collider() {
        let berry = Entity {
            kind: EntityKind::Strawberry,
            bounds: Rect::new(153.0, 81.0, 14.0, 14.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "strawberry".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![berry.clone()],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "berry").unwrap();
        let decoded = decode_map_room(&bytes, Some("berry")).unwrap();
        assert_eq!(decoded.entities, vec![berry]);
    }

    #[test]
    fn celeste_refill_round_trips_center_hitbox_and_dash_flags() {
        let refill = Entity {
            kind: EntityKind::Refill,
            bounds: Rect::new(136.0, 88.0, 16.0, 16.0),
            direction: Vec2::new(1.0, 0.0),
            shielded: false,
            single_use: true,
            nodes: vec![],
            name: "refill".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![refill.clone()],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "refills").unwrap();
        let decoded = decode_map_room(&bytes, Some("refills")).unwrap();
        assert_eq!(decoded.entities, vec![refill]);
    }

    #[test]
    fn celeste_falling_block_round_trips_its_solid_rect_and_climb_fall_flag() {
        let block = Entity {
            kind: EntityKind::FallingBlock,
            bounds: Rect::new(112.0, 24.0, 24.0, 40.0),
            direction: Vec2::new(1.0, 0.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "fallingBlock".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![block.clone()],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "blocks").unwrap();
        let decoded = decode_map_room(&bytes, Some("blocks")).unwrap();
        assert_eq!(decoded.entities, vec![block]);
    }

    #[test]
    fn vanilla_exit_block_round_trips_its_solid_rect_and_tiletype() {
        let block = Entity {
            kind: EntityKind::ExitBlock,
            bounds: Rect::new(112.0, 24.0, 24.0, 40.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "exitBlock".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![block.clone()],
            entity_visuals: vec![EntityVisual {
                tile: Some('9'),
                ..EntityVisual::default()
            }],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "exit-block").unwrap();
        let decoded = decode_map_room(&bytes, Some("exit-block")).unwrap();
        assert_eq!(decoded.entities, vec![block]);
        assert_eq!(decoded.entity_visuals[0].tile, Some('9'));
    }

    #[test]
    fn vanilla_invisible_barrier_round_trips_its_solid_rect() {
        let barrier = Entity {
            kind: EntityKind::InvisibleBarrier,
            bounds: Rect::new(112.0, 24.0, 24.0, 40.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "invisibleBarrier".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![barrier.clone()],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "invisible-barrier").unwrap();
        let decoded = decode_map_room(&bytes, Some("invisible-barrier")).unwrap();
        assert_eq!(decoded.entities, vec![barrier]);
    }

    #[test]
    fn vanilla_killbox_round_trips_its_fixed_thirty_two_pixel_collider() {
        let killbox = Entity {
            kind: EntityKind::Killbox,
            bounds: Rect::new(16.0, 184.0, 288.0, 32.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "killbox".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![killbox.clone()],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "killbox").unwrap();
        let decoded = decode_map_room(&bytes, Some("killbox")).unwrap();
        assert_eq!(decoded.entities, vec![killbox]);
    }

    #[test]
    fn selected_room_retains_adjacent_transition_bounds() {
        let adjacent = Rect::new(0.0, -184.0, 320.0, 184.0);
        // `LevelData.cs:132-137` rewrites a declared height of 184 into 180, so
        // both the selected room and its neighbour decode 4 px shorter than the
        // rectangle the encoder wrote.
        let decoded_bounds = Rect::new(0.0, 0.0, 320.0, 180.0);
        let decoded_adjacent = Rect::new(0.0, -184.0, 320.0, 180.0);
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            transition_rooms: vec![adjacent],
            solids: vec![
                Rect::new(0.0, 176.0, 320.0, 8.0),
                Rect::new(160.0, -16.0, 8.0, 16.0),
            ],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "lower").unwrap();
        let lower = decode_map_room(&bytes, Some("lower")).unwrap();
        assert_eq!(lower.bounds, decoded_bounds);
        assert_eq!(lower.transition_rooms, vec![decoded_adjacent]);
        assert_eq!(lower.transition_runtime.len(), 2);
        assert!(
            lower
                .transition_runtime
                .iter()
                .any(|room| room.bounds == decoded_bounds && room.spawns == vec![lower.spawn])
        );
        let upper = decode_map_room(&bytes, Some("transition_0")).unwrap();
        assert_eq!(upper.bounds, decoded_adjacent);
        assert_eq!(upper.transition_rooms, vec![decoded_bounds]);
        assert_eq!(upper.transition_runtime.len(), 2);
        assert!(
            upper
                .transition_runtime
                .iter()
                .any(|room| room.bounds == decoded_bounds && room.spawns == vec![lower.spawn])
        );
        assert_eq!(upper.spawn, Vec2::new(24.0, -16.0));
        assert_eq!(lower.solids, vec![Rect::new(0.0, 176.0, 320.0, 8.0)]);
        assert_eq!(
            upper.solids,
            vec![
                Rect::new(160.0, -16.0, 8.0, 8.0),
                Rect::new(160.0, -8.0, 8.0, 8.0),
            ]
        );
        assert!(!lower.solid_at(Rect::new(160.0, -12.0, 1.0, 1.0)));
        assert!(upper.solid_at(Rect::new(160.0, -12.0, 1.0, 1.0)));
    }

    #[test]
    fn simulator_moving_solid_round_trips_through_celeste_binary() {
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![Entity {
                kind: EntityKind::MovingSolid,
                bounds: Rect::new(16.0, 24.0, 32.0, 8.0),
                direction: Vec2::new(60.0, -120.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "celesteGymMovingSolid".to_owned(),
            }],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "moving").unwrap();
        let decoded = decode_map_room(&encoded, Some("moving")).unwrap();
        let entity = decoded.entities.first().unwrap();
        assert_eq!(entity.kind, EntityKind::MovingSolid);
        assert_eq!(entity.bounds, Rect::new(16.0, 24.0, 32.0, 8.0));
        assert_eq!(entity.direction, Vec2::new(60.0, -120.0));
    }

    #[test]
    fn vanilla_zip_mover_round_trips_through_celeste_binary() {
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
            entities: vec![Entity {
                kind: EntityKind::ZipMover,
                bounds: Rect::new(352.0, -120.0, 64.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![Vec2::new(352.0, -200.0)],
                name: "zipMover".to_owned(),
            }],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "zip").unwrap();
        let decoded = decode_map_room(&encoded, Some("zip")).unwrap();
        let entity = decoded.entities.first().unwrap();
        assert_eq!(entity.kind, EntityKind::ZipMover);
        assert_eq!(entity.bounds, Rect::new(352.0, -120.0, 64.0, 16.0));
        assert_eq!(entity.nodes, vec![Vec2::new(352.0, -200.0)]);
        assert_eq!(entity.name, "zipMover");
    }

    #[test]
    fn vanilla_bounce_block_round_trips_through_celeste_binary() {
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
            entities: vec![Entity {
                kind: EntityKind::BounceBlock,
                bounds: Rect::new(352.0, -120.0, 64.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "bounceBlock".to_owned(),
            }],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "bounce").unwrap();
        let decoded = decode_map_room(&encoded, Some("bounce")).unwrap();
        let entity = decoded.entities.first().unwrap();
        assert_eq!(entity.kind, EntityKind::BounceBlock);
        assert_eq!(entity.bounds, Rect::new(352.0, -120.0, 64.0, 16.0));
        assert_eq!(entity.name, "bounceBlock");
    }

    #[test]
    fn vanilla_move_block_round_trips_through_celeste_binary() {
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
            entities: vec![Entity {
                kind: EntityKind::MoveBlock,
                bounds: Rect::new(352.0, -120.0, 32.0, 16.0),
                direction: Vec2::new(1.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "moveBlock".to_owned(),
            }],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "move").unwrap();
        let decoded = decode_map_room(&encoded, Some("move")).unwrap();
        let entity = decoded.entities.first().unwrap();
        assert_eq!(entity.kind, EntityKind::MoveBlock);
        assert_eq!(entity.bounds, Rect::new(352.0, -120.0, 32.0, 16.0));
        assert_eq!(entity.direction, Vec2::new(1.0, 0.0));
        assert_eq!(entity.name, "moveBlock");
    }

    #[test]
    fn vanilla_close_behind_player_temple_gate_round_trips_through_celeste_binary() {
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
            entities: vec![Entity {
                kind: EntityKind::TempleGate,
                bounds: Rect::new(352.0, -120.0, 8.0, 48.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "templeGate".to_owned(),
            }],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "gate").unwrap();
        let decoded = decode_map_room(&encoded, Some("gate")).unwrap();
        let entity = decoded.entities.first().unwrap();
        assert_eq!(entity.kind, EntityKind::TempleGate);
        assert_eq!(entity.bounds, Rect::new(352.0, -120.0, 8.0, 48.0));
        assert_eq!(entity.name, "templeGate");
    }

    /// `TempleGate`'s `type` attribute (`TempleGate.cs:72-74`) decides the whole gate, so it has to
    /// survive the round trip the way the switch's two bools do.
    #[test]
    fn temple_gate_type_round_trips_through_celeste_binary() {
        for gate_type in [
            "NearestSwitch",
            "CloseBehindPlayer",
            "CloseBehindPlayerAlways",
            "HoldingTheo",
            "TouchSwitches",
            "CloseBehindPlayerAndTheo",
        ] {
            let map = Map {
                bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
                entities: vec![Entity {
                    kind: EntityKind::TempleGate,
                    bounds: Rect::new(352.0, -120.0, 8.0, 48.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "templeGate".to_owned(),
                }],
                entity_visuals: vec![EntityVisual {
                    variant: Some(gate_type.to_owned()),
                    ..EntityVisual::default()
                }],
                ..Map::default()
            };
            let encoded = encode_celeste_map(&map, "CelesteGymTest", "gate-type").unwrap();
            let decoded = decode_map_room(&encoded, Some("gate-type")).unwrap();
            assert_eq!(
                decoded.entity_visuals[0].variant.as_deref(),
                Some(gate_type),
                "{gate_type}"
            );
        }
    }

    #[test]
    fn cassette_and_spinner_round_trip_vanilla_entity_attributes() {
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![
                Entity {
                    kind: EntityKind::CassetteBlock,
                    bounds: Rect::new(64.0, 120.0, 64.0, 16.0),
                    direction: Vec2::new(2.0, 1.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "cassetteBlock".to_owned(),
                },
                Entity {
                    kind: EntityKind::CrystalStaticSpinner,
                    bounds: Rect::new(192.0, 94.0, 16.0, 12.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spinner".to_owned(),
                },
            ],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymPlayground", "cassette-spinner")
            .expect("fixture should encode");
        let decoded = decode_map_room(&bytes, Some("cassette-spinner")).unwrap();
        assert_eq!(decoded.entities[0].kind, EntityKind::CassetteBlock);
        assert_eq!(decoded.entities[0].bounds, map.entities[0].bounds);
        assert_eq!(decoded.entities[0].direction, Vec2::new(2.0, 1.0));
        assert_eq!(decoded.entities[1].kind, EntityKind::CrystalStaticSpinner);
        assert_eq!(decoded.entities[1].bounds, map.entities[1].bounds);
    }

    #[test]
    fn mod_custom_spinners_and_falling_block_tiletype_decode_with_visuals() {
        let root = BinaryElement {
            package: Some("StrawberryJam2021".to_owned()),
            name: "Map".to_owned(),
            attributes: BTreeMap::new(),
            children: vec![
                element("Filler", [], vec![]),
                element(
                    "levels",
                    [],
                    vec![element(
                        "level",
                        [
                            ("name", BinaryValue::String("a".to_owned())),
                            ("x", BinaryValue::Int(0)),
                            ("y", BinaryValue::Int(0)),
                            ("width", BinaryValue::Int(320)),
                            ("height", BinaryValue::Int(180)),
                        ],
                        vec![
                            element(
                                "solids",
                                [(
                                    "innerText",
                                    BinaryValue::String("........\n........".to_owned()),
                                )],
                                vec![],
                            ),
                            element(
                                "entities",
                                [],
                                vec![
                                    element(
                                        "VivHelper/CustomSpinner",
                                        [
                                            ("x", BinaryValue::Int(128)),
                                            ("y", BinaryValue::Int(64)),
                                            (
                                                "Directory",
                                                BinaryValue::String(
                                                    "danger/SJ2021/Gym/Orb".to_owned(),
                                                ),
                                            ),
                                            ("Subdirectory", BinaryValue::String("beg".to_owned())),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "FrostHelper/IceSpinner",
                                        [
                                            ("x", BinaryValue::Int(160)),
                                            ("y", BinaryValue::Int(64)),
                                            (
                                                "directory",
                                                BinaryValue::String(
                                                    "danger/SJ2021/Ceph/Spinner".to_owned(),
                                                ),
                                            ),
                                            ("tint", BinaryValue::String("ffe5e4".to_owned())),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "fallingBlock",
                                        [
                                            ("x", BinaryValue::Int(200)),
                                            ("y", BinaryValue::Int(64)),
                                            ("width", BinaryValue::Int(24)),
                                            ("height", BinaryValue::Int(16)),
                                            ("tiletype", BinaryValue::String(".".to_owned())),
                                        ],
                                        vec![],
                                    ),
                                ],
                            ),
                        ],
                    )],
                ),
            ],
        };
        let bytes = encode_celeste_bin(&root).expect("custom entity map should encode");
        let decoded = decode_map_room(&bytes, Some("a")).unwrap();
        assert_eq!(decoded.entities[0].kind, EntityKind::CrystalStaticSpinner);
        assert_eq!(decoded.entities[1].kind, EntityKind::CrystalStaticSpinner);
        assert_eq!(decoded.entities[2].kind, EntityKind::FallingBlock);
        assert_eq!(
            decoded.entity_visuals[0].texture.as_deref(),
            Some("danger/SJ2021/Gym/Orb/fg_beg")
        );
        assert_eq!(
            decoded.entity_visuals[1].texture.as_deref(),
            Some("danger/SJ2021/Ceph/Spinner/fg")
        );
        assert_eq!(decoded.entity_visuals[1].tint.as_deref(), Some("ffe5e4"));
        assert_eq!(decoded.entity_visuals[2].tile, Some('.'));
        assert_eq!(decoded.tile_grid, vec!["........", "........"]);
    }

    #[test]
    fn low_complexity_mod_entities_decode_to_existing_physics_primitives() {
        let root = BinaryElement {
            package: Some("CompatibilityFixture".to_owned()),
            name: "Map".to_owned(),
            attributes: BTreeMap::new(),
            children: vec![
                element("Filler", [], vec![]),
                element(
                    "levels",
                    [],
                    vec![element(
                        "level",
                        [
                            ("name", BinaryValue::String("aliases".to_owned())),
                            ("x", BinaryValue::Int(320)),
                            ("y", BinaryValue::Int(-180)),
                            ("width", BinaryValue::Int(320)),
                            ("height", BinaryValue::Int(180)),
                        ],
                        vec![
                            element(
                                "solids",
                                [("innerText", BinaryValue::String("........".to_owned()))],
                                vec![],
                            ),
                            element(
                                "entities",
                                [],
                                vec![
                                    element(
                                        "player",
                                        [("x", BinaryValue::Int(16)), ("y", BinaryValue::Int(160))],
                                        vec![],
                                    ),
                                    element(
                                        "pandorasBox/coloredWater",
                                        [
                                            ("x", BinaryValue::Int(24)),
                                            ("y", BinaryValue::Int(80)),
                                            ("width", BinaryValue::Int(40)),
                                            ("height", BinaryValue::Int(24)),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "FrostHelper/SpringRight",
                                        [
                                            ("x", BinaryValue::Int(100)),
                                            ("y", BinaryValue::Int(100)),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "VivHelper/RainbowSpikesUp",
                                        [
                                            ("x", BinaryValue::Int(128)),
                                            ("y", BinaryValue::Int(120)),
                                            ("width", BinaryValue::Int(24)),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "MaxHelpingHand/Comment",
                                        [("x", BinaryValue::Int(180)), ("y", BinaryValue::Int(40))],
                                        vec![],
                                    ),
                                    element(
                                        "MaxHelpingHand/CustomizableRefill",
                                        [
                                            ("x", BinaryValue::Int(200)),
                                            ("y", BinaryValue::Int(56)),
                                            ("twoDash", BinaryValue::Bool(true)),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "FrostHelper/CustomDreamBlock",
                                        [
                                            ("x", BinaryValue::Int(208)),
                                            ("y", BinaryValue::Int(96)),
                                            ("width", BinaryValue::Int(24)),
                                            ("height", BinaryValue::Int(16)),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "JungleHelper/InvisibleJumpthruPlatform",
                                        [
                                            ("x", BinaryValue::Int(240)),
                                            ("y", BinaryValue::Int(128)),
                                            ("width", BinaryValue::Int(32)),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "MaxHelpingHand/CustomizableRefill",
                                        [
                                            ("x", BinaryValue::Int(280)),
                                            ("y", BinaryValue::Int(56)),
                                            ("respawnTime", BinaryValue::Float(1.0)),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "MaxHelpingHand/SidewaysJumpThru",
                                        [
                                            ("x", BinaryValue::Int(220)),
                                            ("y", BinaryValue::Int(80)),
                                            ("height", BinaryValue::Int(32)),
                                        ],
                                        vec![],
                                    ),
                                ],
                            ),
                        ],
                    )],
                ),
            ],
        };

        let bytes = encode_celeste_bin(&root).expect("compatibility map should encode");
        let decoded = decode_map_room(&bytes, Some("aliases")).unwrap();

        assert_eq!(decoded.entities[0].kind, EntityKind::Water);
        assert_eq!(
            decoded.entities[0].bounds,
            Rect::new(344.0, -100.0, 40.0, 24.0)
        );
        assert_eq!(decoded.entities[1].kind, EntityKind::Spring);
        assert_eq!(
            decoded.entities[1].bounds,
            Rect::new(414.0, -88.0, 6.0, 16.0)
        );
        assert_eq!(decoded.entities[1].direction, Vec2::new(-1.0, 0.0));
        assert_eq!(decoded.entities[2].kind, EntityKind::Spikes);
        assert_eq!(
            decoded.entities[2].bounds,
            Rect::new(448.0, -63.0, 24.0, 3.0)
        );
        assert_eq!(decoded.entities[2].direction, Vec2::new(0.0, -1.0));
        assert_eq!(decoded.entities[3].kind, EntityKind::Decoration);
        assert_eq!(decoded.entities[4].kind, EntityKind::Refill);
        assert_eq!(
            decoded.entities[4].bounds,
            Rect::new(512.0, -132.0, 16.0, 16.0)
        );
        assert_eq!(decoded.entities[4].direction, Vec2::new(1.0, 0.0));
        assert_eq!(decoded.entities[5].kind, EntityKind::DreamBlock);
        assert_eq!(
            decoded.entities[5].bounds,
            Rect::new(528.0, -84.0, 24.0, 16.0)
        );
        assert_eq!(decoded.entities[6].kind, EntityKind::JumpThru);
        // `Celeste.JumpThru` replaces the entity data's 8 px `height` with
        // `new Hitbox(width, 5f)` (JumpThru.cs:12), anchored at the entity's top-left.
        assert_eq!(
            decoded.entities[6].bounds,
            Rect::new(560.0, -52.0, 32.0, 5.0)
        );
        assert_eq!(decoded.entities[7].kind, EntityKind::Unknown);
        assert_eq!(decoded.entities[8].kind, EntityKind::Unknown);
        assert!(!decoded.solid_at(decoded.entities[3].bounds));
    }

    #[test]
    fn fancy_tiles_and_default_dash_through_spikes_keep_exact_collision_semantics() {
        let root = BinaryElement {
            package: Some("CompatibilityFixture".to_owned()),
            name: "Map".to_owned(),
            attributes: BTreeMap::new(),
            children: vec![
                element("Filler", [], vec![]),
                element(
                    "levels",
                    [],
                    vec![element(
                        "level",
                        [
                            ("name", BinaryValue::String("tiles-hazards".to_owned())),
                            ("x", BinaryValue::Int(320)),
                            ("y", BinaryValue::Int(-180)),
                            ("width", BinaryValue::Int(320)),
                            ("height", BinaryValue::Int(180)),
                        ],
                        vec![
                            element(
                                "solids",
                                [("innerText", BinaryValue::String("........".to_owned()))],
                                vec![],
                            ),
                            element(
                                "entities",
                                [],
                                vec![
                                    element(
                                        "player",
                                        [("x", BinaryValue::Int(16)), ("y", BinaryValue::Int(160))],
                                        vec![],
                                    ),
                                    element(
                                        "FancyTileEntities/FancySolidTiles",
                                        [
                                            ("x", BinaryValue::Int(16)),
                                            ("y", BinaryValue::Int(24)),
                                            ("width", BinaryValue::Int(24)),
                                            ("height", BinaryValue::Int(16)),
                                            ("tileData", BinaryValue::String("110,010".to_owned())),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "NerdHelper/DashThroughSpikesUp",
                                        [
                                            ("x", BinaryValue::Int(80)),
                                            ("y", BinaryValue::Int(80)),
                                            ("width", BinaryValue::Int(24)),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "NerdHelper/DashThroughSpikesDown",
                                        [
                                            ("x", BinaryValue::Int(120)),
                                            ("y", BinaryValue::Int(80)),
                                            ("width", BinaryValue::Int(24)),
                                            ("invert", BinaryValue::Bool(true)),
                                        ],
                                        vec![],
                                    ),
                                    element(
                                        "CherryHelper/AssistRect",
                                        [
                                            ("x", BinaryValue::Int(160)),
                                            ("y", BinaryValue::Int(40)),
                                            ("width", BinaryValue::Int(32)),
                                            ("height", BinaryValue::Int(24)),
                                        ],
                                        vec![],
                                    ),
                                ],
                            ),
                        ],
                    )],
                ),
            ],
        };

        let bytes = encode_celeste_bin(&root).expect("compatibility map should encode");
        let decoded = decode_map_room(&bytes, Some("tiles-hazards")).unwrap();

        assert!(
            decoded
                .solids
                .contains(&Rect::new(336.0, -156.0, 16.0, 8.0))
        );
        assert!(decoded.solids.contains(&Rect::new(344.0, -148.0, 8.0, 8.0)));
        assert!(!decoded.solid_at(Rect::new(336.0, -148.0, 8.0, 8.0)));
        assert_eq!(decoded.entities[0].kind, EntityKind::Decoration);
        assert_eq!(decoded.entities[1].kind, EntityKind::Spikes);
        assert!(decoded.entities[1].shielded);
        assert_eq!(decoded.entities[2].kind, EntityKind::Unknown);
        assert_eq!(decoded.entities[3].kind, EntityKind::Decoration);
    }

    #[test]
    fn lookout_flags_and_nodes_round_trip_with_room_relative_binary_nodes() {
        let lookout = Entity {
            kind: EntityKind::Lookout,
            bounds: Rect::new(510.0, -68.0, 4.0, 4.0),
            direction: Vec2::new(1.0, 1.0),
            shielded: false,
            single_use: false,
            nodes: vec![Vec2::new(704.0, -160.0), Vec2::new(352.0, -224.0)],
            name: "towerviewer".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 640.0, 184.0),
            spawn: Vec2::new(344.0, -80.0),
            entities: vec![lookout.clone()],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "lookout").unwrap();
        assert!(
            encoded
                .windows(b"towerviewer".len())
                .any(|window| window == b"towerviewer"),
            "Everest Level.LoadLevel dispatches Lookout as `towerviewer`, not fixture alias `lookout`"
        );
        let decoded = decode_map_room(&encoded, Some("lookout")).unwrap();
        assert_eq!(decoded.entities, vec![lookout]);
    }

    #[test]
    fn badeline_nodes_are_room_relative_in_the_binary_and_absolute_after_decode() {
        let boost = Entity {
            kind: EntityKind::BadelineBoost,
            bounds: Rect::new(352.0, -136.0, 32.0, 32.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![Vec2::new(400.0, -200.0)],
            name: "badelineBoost".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
            spawn: Vec2::new(344.0, -80.0),
            entities: vec![boost.clone()],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "badeline").unwrap();
        let decoded = decode_map_room(&encoded, Some("badeline")).unwrap();
        assert_eq!(decoded.entities, vec![boost]);
    }

    #[test]
    fn vanilla_theo_crystal_round_trips_through_celeste_binary() {
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
            entities: vec![Entity {
                kind: EntityKind::TheoCrystal,
                bounds: Rect::new(364.0, -130.0, 8.0, 10.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "theoCrystal".to_owned(),
            }],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "theo").unwrap();
        let decoded = decode_map_room(&encoded, Some("theo")).unwrap();
        let entity = decoded.entities.first().unwrap();
        assert_eq!(entity.kind, EntityKind::TheoCrystal);
        assert_eq!(entity.bounds, Rect::new(364.0, -130.0, 8.0, 10.0));
        assert_eq!(entity.name, "theoCrystal");
    }

    #[test]
    fn vanilla_heart_gem_round_trips_through_celeste_binary() {
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
            entities: vec![Entity {
                kind: EntityKind::HeartGem,
                bounds: Rect::new(360.0, -136.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "blackGem".to_owned(),
            }],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "heart").unwrap();
        let decoded = decode_map_room(&encoded, Some("heart")).unwrap();
        let entity = decoded.entities.first().unwrap();
        assert_eq!(entity.kind, EntityKind::HeartGem);
        assert_eq!(entity.bounds, Rect::new(360.0, -136.0, 16.0, 16.0));
        assert_eq!(entity.name, "blackGem");
    }

    #[test]
    fn vanilla_core_lavas_round_trip_with_source_colliders() {
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
            entities: vec![
                Entity {
                    kind: EntityKind::RisingLava,
                    bounds: Rect::new(352.0, -120.0, 8.0, 8.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: true,
                    nodes: vec![],
                    name: "risingLava".to_owned(),
                },
                Entity {
                    kind: EntityKind::SandwichLava,
                    bounds: Rect::new(400.0, -120.0, 8.0, 8.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "sandwichLava".to_owned(),
                },
            ],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "lavas").unwrap();
        let decoded = decode_map_room(&encoded, Some("lavas")).unwrap();

        assert_eq!(decoded.entities[0].kind, EntityKind::RisingLava);
        assert_eq!(
            decoded.entities[0].bounds,
            Rect::new(352.0, -120.0, 340.0, 120.0)
        );
        assert!(decoded.entities[0].single_use);
        assert_eq!(decoded.entities[1].kind, EntityKind::SandwichLava);
        assert_eq!(
            decoded.entities[1].bounds,
            Rect::new(400.0, -120.0, 340.0, 120.0)
        );
    }

    #[test]
    fn vanilla_glider_round_trips_through_celeste_binary() {
        let map = Map {
            bounds: Rect::new(320.0, -240.0, 320.0, 184.0),
            entities: vec![Entity {
                kind: EntityKind::Glider,
                bounds: Rect::new(364.0, -130.0, 8.0, 10.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "glider".to_owned(),
            }],
            ..Map::default()
        };

        let encoded = encode_celeste_map(&map, "CelesteGymTest", "glider").unwrap();
        let decoded = decode_map_room(&encoded, Some("glider")).unwrap();
        let entity = decoded.entities.first().unwrap();
        assert_eq!(entity.kind, EntityKind::Glider);
        assert_eq!(entity.bounds, Rect::new(364.0, -130.0, 8.0, 10.0));
        assert_eq!(entity.name, "glider");
    }

    /// `LevelData.cs:132-137` rewrites a declared room height of 184 into 180, so
    /// a decoded room's `Level.Bounds` must be 180 tall even though the file and
    /// the encoder keep 184.
    #[test]
    fn level_data_clamps_a_184_pixel_room_height_to_180() {
        assert_eq!(
            level_room_bounds(0.0, -864.0, 976.0, 184.0),
            Rect::new(0.0, -864.0, 976.0, 180.0)
        );
        assert_eq!(
            level_room_bounds(0.0, 0.0, 320.0, 180.0),
            Rect::new(0.0, 0.0, 320.0, 180.0)
        );
        assert_eq!(
            level_room_bounds(0.0, 0.0, 320.0, 160.0),
            Rect::new(0.0, 0.0, 320.0, 160.0)
        );

        let map = Map {
            bounds: Rect::new(0.0, -864.0, 976.0, 184.0),
            ..Map::default()
        };
        let encoded = encode_celeste_map(&map, "CelesteGymTest", "tall").unwrap();
        assert_eq!(
            parse_celeste_bin(&encoded)
                .unwrap()
                .children
                .iter()
                .find(|child| child.name == "levels")
                .and_then(|levels| levels.children.first())
                .and_then(|level| level.attributes.get("height").cloned()),
            Some(BinaryValue::Int(184))
        );
        assert_eq!(
            decode_map_room(&encoded, Some("tall")).unwrap().bounds,
            Rect::new(0.0, -864.0, 976.0, 180.0)
        );
        // `LevelData.TileBounds` (`LevelData.cs:76`) uses the clamped height and
        // `ceil(180 / 8) == ceil(184 / 8) == 23`, so the stored tile rows are
        // unaffected by the rewrite.
        assert_eq!(audit_celeste_map(&encoded).unwrap()[0].bounds.height, 180.0);
    }

    /// `CrushBlock(EntityData, offset)` forwards the raw rectangle into
    /// `Solid(position, width, height, safe: false)` (`CrushBlock.cs:85-87,
    /// 146-149`) and `DashBlock` does the same with `safe: true`
    /// (`DashBlock.cs:30-32,45-47`), so both are Solids the player collides with.
    #[test]
    fn celeste_crush_block_and_dash_block_decode_as_solids() {
        let crusher = Entity {
            kind: EntityKind::CrushBlock,
            bounds: Rect::new(600.0, -1224.0, 24.0, 24.0),
            // `axes = Both` (0), `chillout = false`.
            direction: Vec2::new(0.0, 0.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "crushBlock".to_owned(),
        };
        let dash = Entity {
            kind: EntityKind::DashBlock,
            bounds: Rect::new(80.0, 88.0, 64.0, 40.0),
            // `canDash = false`, `permanent = false`.
            direction: Vec2::new(0.0, 0.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "dashBlock".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 176.0),
            entities: vec![crusher.clone(), dash.clone()],
            ..Map::default()
        };
        let encoded = encode_celeste_map(&map, "CelesteGymTest", "solids").unwrap();
        let decoded = decode_map_room(&encoded, Some("solids")).unwrap();
        assert_eq!(decoded.entities[0].kind, EntityKind::CrushBlock);
        assert_eq!(decoded.entities[0].bounds, crusher.bounds);
        assert_eq!(decoded.entities[0].direction, Vec2::new(0.0, 0.0));
        assert_eq!(decoded.entities[1].kind, EntityKind::DashBlock);
        assert_eq!(decoded.entities[1].bounds, dash.bounds);
        assert_eq!(decoded.entities[1].direction, Vec2::new(0.0, 0.0));
        assert!(decoded.non_dream_solid_at(Rect::new(604.0, -1220.0, 8.0, 11.0)));
        assert!(decoded.non_dream_solid_at(Rect::new(100.0, 100.0, 8.0, 11.0)));
        assert!(!decoded.solid_at(Rect::new(4.0, 4.0, 8.0, 11.0)));

        // `axes` and `chillout` survive both directions.
        let vertical = Entity {
            direction: Vec2::new(2.0, 1.0),
            ..crusher.clone()
        };
        let canvas = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 176.0),
            entities: vec![vertical],
            ..Map::default()
        };
        let encoded = encode_celeste_map(&canvas, "CelesteGymTest", "axes").unwrap();
        let decoded = decode_map_room(&encoded, Some("axes")).unwrap();
        assert_eq!(decoded.entities[0].direction, Vec2::new(2.0, 1.0));
    }

    /// `DashSwitch.Create` (`DashSwitch.cs:103-122`) takes the side from the map name plus one
    /// bool: `dashSwitchH` is `Left` with `leftSide` and `Right` otherwise, `dashSwitchV` is `Up`
    /// with `ceiling` and `Down` otherwise. `base(position, 0f, 0f, safe: true)` (`:52-53`) is
    /// immediately resized to 16x8 for `Up`/`Down` and 8x16 for `Left`/`Right` (`:62-71`) with
    /// collider offset `(0, 0)`, so the collider sits at the entity position - not at the map
    /// rectangle, which is 0-sized in every vanilla element.
    #[test]
    fn celeste_dash_switches_decode_with_source_colliders() {
        let cases = [
            // name, `pressDirection` (`:72-99`), collider
            (
                "dashSwitchH",
                Vec2::new(-1.0, 0.0),
                Rect::new(96.0, 64.0, 8.0, 16.0),
            ),
            (
                "dashSwitchH",
                Vec2::new(1.0, 0.0),
                Rect::new(96.0, 64.0, 8.0, 16.0),
            ),
            (
                "dashSwitchV",
                Vec2::new(0.0, -1.0),
                Rect::new(200.0, 120.0, 16.0, 8.0),
            ),
            (
                "dashSwitchV",
                Vec2::new(0.0, 1.0),
                Rect::new(200.0, 120.0, 16.0, 8.0),
            ),
        ];
        for (name, press_direction, collider) in cases {
            let switch = Entity {
                kind: EntityKind::DashSwitch,
                bounds: collider,
                direction: press_direction,
                // `allGates` (`DashSwitch.cs:107`), carried in `shielded`.
                shielded: true,
                // `persistent` (`DashSwitch.cs:106`), only observable through the
                // unrepresentable `dashSwitch_<id>` session flag (`:223-226`).
                single_use: true,
                nodes: vec![],
                name: name.to_owned(),
            };
            let map = Map {
                bounds: Rect::new(0.0, 0.0, 320.0, 176.0),
                entities: vec![switch],
                ..Map::default()
            };
            let encoded = encode_celeste_map(&map, "CelesteGymTest", "switches").unwrap();
            let decoded = decode_map_room(&encoded, Some("switches")).unwrap();
            assert_eq!(decoded.entities[0].kind, EntityKind::DashSwitch, "{name}");
            assert_eq!(decoded.entities[0].bounds, collider, "{name}");
            assert_eq!(decoded.entities[0].direction, press_direction, "{name}");
            assert_eq!(decoded.entities[0].name, name);
            assert!(decoded.entities[0].single_use);
            assert!(decoded.entities[0].shielded, "{name} must keep allGates");
            // The collider is what the player's own collision and the dash-collide
            // probe consult, and it is live from the moment the room loads.
            let inside = Rect::new(collider.x + 1.0, collider.y + 1.0, 2.0, 2.0);
            assert!(decoded.non_dream_solid_at(inside), "{name}");
        }

        // Every vanilla element stores a 0-sized map rectangle (`Solid(position, 0f, 0f,
        // safe: true)`, `DashSwitch.cs:53`), so the decode arm must derive the collider
        // from the name plus one bool instead of reading `width`/`height` - which would
        // leave the button with no collider at all, exactly as before this change.
        let zero_rect = Entity {
            kind: EntityKind::DashSwitch,
            bounds: Rect::new(96.0, 64.0, 0.0, 0.0),
            direction: Vec2::new(1.0, 0.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "dashSwitchH".to_owned(),
        };
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 176.0),
            entities: vec![zero_rect],
            ..Map::default()
        };
        let encoded = encode_celeste_map(&map, "CelesteGymTest", "zero").unwrap();
        let decoded = decode_map_room(&encoded, Some("zero")).unwrap();
        assert_eq!(decoded.entities[0].bounds, Rect::new(96.0, 64.0, 8.0, 16.0));
    }
}
