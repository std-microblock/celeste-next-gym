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
    BadelineBoost,
    Wind,
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
    /// Bounds of the other rooms in the same Celeste map. These are retained
    /// when decoding one room so Level.EnforceBounds can resolve transitions.
    #[serde(default)]
    pub transition_rooms: Vec<Rect>,
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
        || map.transition_rooms.iter().any(|room| {
            room.width <= 0.0
                || room.height <= 0.0
                || room.width % 8.0 != 0.0
                || room.height % 8.0 != 0.0
        })
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
            EntityKind::Unknown => None,
        };
        if let Some(encoded) = encoded {
            entities.push(encoded);
        }
    }

    let level = encoded_level(
        format!("lvl_{room}"),
        map.bounds,
        triggers,
        entities,
        solids_text(map),
    );
    let mut levels = vec![level];
    for (index, bounds) in map.transition_rooms.iter().copied().enumerate() {
        let blank = Map {
            bounds,
            solids: map.solids.clone(),
            ..Map::default()
        };
        // LevelData marks rooms without a player spawn as Dummy, and
        // MapData.CanTransitionTo rejects Dummy targets. A transition room
        // therefore needs a valid spawn even though screen transitions keep
        // the existing player instead of respawning at it.
        let transition_spawn = element(
            "player",
            [
                ("id", BinaryValue::Int(0)),
                ("originX", BinaryValue::Int(4)),
                ("originY", BinaryValue::Int(8)),
                ("width", BinaryValue::Int(8)),
                ("x", BinaryValue::Int(24)),
                ("y", BinaryValue::Int(bounds.height as i32 - 16)),
            ],
            vec![],
        );
        levels.push(encoded_level(
            format!("lvl_transition_{index}"),
            bounds,
            vec![],
            vec![transition_spawn],
            solids_text(&blank),
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
                "badelineBoost" => EntityKind::BadelineBoost,
                "windTrigger" => EntityKind::Wind,
                _ => EntityKind::Unknown,
            };
            let default_w = match kind {
                EntityKind::Booster | EntityKind::RedBooster => 16.0,
                EntityKind::FlyFeather => 20.0,
                EntityKind::Bumper => 24.0,
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
                | "badelineBoost" => (
                    Rect::new(
                        ex - raw_width * 0.5,
                        ey - raw_height * 0.5,
                        raw_width,
                        raw_height,
                    ),
                    Vec2::default(),
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

    for room_level in &levels.children {
        let room_x = attr_f32(room_level, "x", 0.0);
        let room_y = attr_f32(room_level, "y", 0.0);
        if let Some(solids) = room_level.children.iter().find(|e| e.name == "solids")
            && let Some(text) = attr_text(solids, "innerText")
        {
            map.solids.extend(tile_rects(text, room_x, room_y));
        }
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
        self.static_solid_at(rect) || self.dream_block_at(rect)
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
        let upper = decode_map_room(&bytes, Some("transition_0")).unwrap();
        assert_eq!(upper.bounds, adjacent);
        assert_eq!(upper.transition_rooms, vec![map.bounds]);
        assert_eq!(upper.spawn, Vec2::new(24.0, -16.0));
        let decoded_solids = vec![
            Rect::new(0.0, 176.0, 320.0, 8.0),
            Rect::new(160.0, -16.0, 8.0, 8.0),
            Rect::new(160.0, -8.0, 8.0, 8.0),
        ];
        assert_eq!(lower.solids, decoded_solids);
        assert_eq!(upper.solids, decoded_solids);
        assert!(lower.solid_at(Rect::new(160.0, -12.0, 1.0, 1.0)));
        assert!(upper.solid_at(Rect::new(160.0, -12.0, 1.0, 1.0)));
    }
}
