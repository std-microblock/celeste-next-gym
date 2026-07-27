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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    BadelineBoost,
    Wind,
    /// Vanilla Celeste ZipMover Solid. The first node is its target position.
    ZipMover,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Map {
    pub bounds: Rect,
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
    if map.bounds.width <= 0.0
        || map.bounds.height <= 0.0
        || map.bounds.width % 8.0 != 0.0
        || map.bounds.height % 8.0 != 0.0
    {
        return Err(MapEncodeError::InvalidBounds);
    }
    let mut entities = vec![element(
        "player",
        [
            ("id", BinaryValue::Int(0)),
            ("originX", BinaryValue::Int(4)),
            ("originY", BinaryValue::Int(8)),
            ("width", BinaryValue::Int(8)),
            ("x", BinaryValue::Int((map.spawn.x - map.bounds.x) as i32)),
            ("y", BinaryValue::Int((map.spawn.y - map.bounds.y) as i32)),
        ],
        vec![],
    )];
    let mut triggers = Vec::new();
    for (index, entity) in map.entities.iter().enumerate() {
        let id = index as i32 + 1;
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
                                ("x", BinaryValue::Int(node.x.round() as i32)),
                                ("y", BinaryValue::Int(node.y.round() as i32)),
                            ],
                            vec![],
                        )
                    })
                    .collect(),
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

    let level = element(
        "level",
        [
            ("alt_music", BinaryValue::String(String::new())),
            ("ambience", BinaryValue::String(String::new())),
            ("c", BinaryValue::Int(0)),
            ("cameraOffsetX", BinaryValue::Int(0)),
            ("cameraOffsetY", BinaryValue::Int(0)),
            ("dark", BinaryValue::Bool(false)),
            ("disableDownTransition", BinaryValue::Bool(false)),
            ("height", BinaryValue::Int(map.bounds.height as i32)),
            ("music", BinaryValue::String(String::new())),
            ("musicLayer1", BinaryValue::Bool(true)),
            ("musicLayer2", BinaryValue::Bool(true)),
            ("musicLayer3", BinaryValue::Bool(true)),
            ("musicLayer4", BinaryValue::Bool(true)),
            ("name", BinaryValue::String(format!("lvl_{room}"))),
            ("space", BinaryValue::Bool(false)),
            ("underwater", BinaryValue::Bool(false)),
            ("whisper", BinaryValue::Bool(false)),
            ("width", BinaryValue::Int(map.bounds.width as i32)),
            ("windPattern", BinaryValue::String("None".to_owned())),
            ("x", BinaryValue::Int(map.bounds.x as i32)),
            ("y", BinaryValue::Int(map.bounds.y as i32)),
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
            tile_layer("solids", Some(solids_text(map))),
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
    );
    let root = BinaryElement {
        package: Some(package.to_owned()),
        name: "Map".to_owned(),
        attributes: BTreeMap::new(),
        children: vec![
            element("Filler", [], vec![]),
            element("levels", [], vec![level]),
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
        source_package: root.package,
        ..Map::default()
    };

    if let Some(entities) = level.children.iter().find(|e| e.name == "entities") {
        for el in &entities.children {
            let ex = x + attr_f32(el, "x", 0.0);
            let ey = y + attr_f32(el, "y", 0.0);
            if el.name == "player" {
                map.spawn = Vec2::new(ex, ey);
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
                "badelineBoost" => EntityKind::BadelineBoost,
                "windTrigger" => EntityKind::Wind,
                "zipMover" => EntityKind::ZipMover,
                "celesteGymMovingSolid" => EntityKind::MovingSolid,
                _ => EntityKind::Unknown,
            };
            let default_w = match kind {
                EntityKind::Booster | EntityKind::RedBooster => 16.0,
                EntityKind::FlyFeather => 20.0,
                EntityKind::Bumper => 24.0,
                EntityKind::IceBall => 12.0,
                EntityKind::BadelineBoost => 32.0,
                _ => 8.0,
            };
            let default_h = default_w;
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
                | "fireBall" | "badelineBoost" => (
                    Rect::new(
                        ex - raw_width * 0.5,
                        ey - raw_height * 0.5,
                        raw_width,
                        raw_height,
                    ),
                    Vec2::default(),
                ),
                "zipMover" => (Rect::new(ex, ey, raw_width, raw_height), Vec2::default()),
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
                single_use: attr_bool(el, "singleUse", false),
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

    if let Some(solids) = level.children.iter().find(|e| e.name == "solids")
        && let Some(text) = attr_text(solids, "innerText")
    {
        map.solids.extend(tile_rects(text, x, y));
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
                matches!(entity.kind, EntityKind::MovingSolid | EntityKind::ZipMover)
                    && entity.bounds.intersects(rect)
            })
    }

    pub fn jump_thru_at(&self, rect: Rect, previous_bottom: f32) -> bool {
        self.entities.iter().any(|e| {
            e.kind == EntityKind::JumpThru
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
}
