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
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Map {
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
            source_package: None,
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
                        ("tiletype", BinaryValue::Int(b'3' as i32)),
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
                            BinaryValue::String("CloseBehindPlayerAlways".to_owned()),
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
                EntityKind::CrystalStaticSpinner => Some(element(
                    "spinner",
                    [
                        ("attachToSolid", BinaryValue::Bool(false)),
                        ("id", BinaryValue::Int(id)),
                        ("x", BinaryValue::Int(x + width / 2)),
                        ("y", BinaryValue::Int(y + height / 2)),
                    ],
                    vec![],
                )),
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
                EntityKind::Unknown => None,
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

pub fn decode_map_room(bytes: &[u8], room: Option<&str>) -> Result<Map, MapError> {
    if let Ok(map) = rmp_serde::from_slice::<Map>(bytes) {
        return Ok(map);
    }
    let root = parse_celeste_bin(bytes).map_err(|e| MapError::Unsupported(e.to_string()))?;
    map_from_binary(root, room)
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
        bounds: Rect::new(x, y, width, height),
        transition_rooms: levels
            .children
            .iter()
            .filter(|candidate| !std::ptr::eq(*candidate, level))
            .map(|candidate| {
                Rect::new(
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
                "cloud" if !attr_bool(el, "fragile", false) => EntityKind::Cloud,
                "badelineBoost" => EntityKind::BadelineBoost,
                "spring" | "wallSpringLeft" | "wallSpringRight" => EntityKind::Spring,
                "strawberry" => EntityKind::Strawberry,
                "refill" => EntityKind::Refill,
                "fallingBlock" => EntityKind::FallingBlock,
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
                "spinner" => EntityKind::CrystalStaticSpinner,
                "towerviewer" | "lookout" => EntityKind::Lookout,
                "celesteGymMovingSolid" => EntityKind::MovingSolid,
                _ => EntityKind::Unknown,
            };
            let default_w = match kind {
                EntityKind::Booster | EntityKind::RedBooster => 16.0,
                EntityKind::FlyFeather => 20.0,
                EntityKind::Bumper => 24.0,
                EntityKind::IceBall => 12.0,
                EntityKind::Puffer => 12.0,
                EntityKind::AngryOshiro => 28.0,
                EntityKind::Seeker => 12.0,
                EntityKind::Snowball => 12.0,
                EntityKind::Cloud => 32.0,
                EntityKind::BadelineBoost => 32.0,
                EntityKind::Strawberry => 14.0,
                EntityKind::Refill => 16.0,
                EntityKind::TheoCrystal | EntityKind::Glider | EntityKind::TempleGate => 8.0,
                EntityKind::CrystalStaticSpinner => 16.0,
                EntityKind::Lookout => 4.0,
                EntityKind::HeartGem => 16.0,
                EntityKind::RisingLava | EntityKind::SandwichLava => 340.0,
                _ => 8.0,
            };
            let default_h = match kind {
                EntityKind::TheoCrystal | EntityKind::Glider | EntityKind::Puffer => 10.0,
                EntityKind::Snowball => 9.0,
                EntityKind::Cloud => 5.0,
                EntityKind::RisingLava | EntityKind::SandwichLava => 120.0,
                EntityKind::CrystalStaticSpinner => 12.0,
                EntityKind::Lookout => 4.0,
                _ => default_w,
            };
            let raw_width = attr_f32(el, "width", default_w);
            let raw_height = attr_f32(el, "height", default_h);
            let (bounds, direction) = match el.name.as_str() {
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
                "bounceBlock" | "zipMover" | "templeGate" => {
                    (Rect::new(ex, ey, raw_width, raw_height), Vec2::default())
                }
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
                "risingLava" | "sandwichLava" => (Rect::new(ex, ey, 340.0, 120.0), Vec2::default()),
                "celesteGymMovingSolid" => (
                    Rect::new(ex, ey, raw_width, raw_height),
                    Vec2::new(attr_f32(el, "speedX", 0.0), attr_f32(el, "speedY", 0.0)),
                ),
                _ => (Rect::new(ex, ey, raw_width, raw_height), Vec2::default()),
            };
            map.entities.push(Entity {
                kind,
                bounds,
                direction,
                shielded: attr_bool(el, "shielded", false),
                single_use: match kind {
                    EntityKind::RisingLava => attr_bool(el, "intro", false),
                    EntityKind::Refill => attr_bool(el, "oneUse", false),
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
                        | EntityKind::FallingBlock
                        | EntityKind::MoveBlock
                        | EntityKind::MovingSolid
                        | EntityKind::ZipMover
                        | EntityKind::TempleGate
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
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(16.0, 176.0, 32.0, 8.0)],
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
    fn selected_room_retains_adjacent_transition_bounds() {
        let adjacent = Rect::new(0.0, -184.0, 320.0, 184.0);
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
        assert_eq!(lower.bounds, map.bounds);
        assert_eq!(lower.transition_rooms, vec![adjacent]);
        assert_eq!(lower.transition_runtime.len(), 2);
        assert!(
            lower
                .transition_runtime
                .iter()
                .any(|room| room.bounds == map.bounds && room.spawns == vec![lower.spawn])
        );
        let upper = decode_map_room(&bytes, Some("transition_0")).unwrap();
        assert_eq!(upper.bounds, adjacent);
        assert_eq!(upper.transition_rooms, vec![map.bounds]);
        assert_eq!(upper.transition_runtime.len(), 2);
        assert!(
            upper
                .transition_runtime
                .iter()
                .any(|room| room.bounds == map.bounds && room.spawns == vec![lower.spawn])
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
}
