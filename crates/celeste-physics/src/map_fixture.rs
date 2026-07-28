use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Entity, EntityKind, Map, MapEncodeError, Rect, Vec2, map::encode_celeste_rooms};

pub const MAP_FIXTURE_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FixtureVec2(pub [f64; 2]);

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FixtureRect(pub [f64; 4]);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixtureEntityKind {
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
    Wind,
    BounceBlock,
    TheoCrystal,
    HeartGem,
    RisingLava,
    SandwichLava,
    Glider,
    ZipMover,
    MoveBlock,
    TempleGate,
    CassetteBlock,
    CrystalStaticSpinner,
    Lookout,
    MovingSolid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FixtureEntity {
    /// Stable authoring identity. Entities are sorted by this value before
    /// numeric Celeste entity IDs are assigned.
    pub id: String,
    pub kind: FixtureEntityKind,
    pub bounds: FixtureRect,
    #[serde(default)]
    pub direction: FixtureVec2,
    #[serde(default)]
    pub shielded: bool,
    #[serde(default)]
    pub single_use: bool,
    #[serde(default)]
    pub nodes: Vec<FixtureVec2>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FixtureRoom {
    /// All coordinates are absolute Celeste map coordinates. Room-local
    /// BinaryPacker offsets are introduced only during lowering.
    pub name: String,
    pub bounds: FixtureRect,
    pub spawn: FixtureVec2,
    #[serde(default)]
    pub solids: Vec<FixtureRect>,
    #[serde(default)]
    pub entities: Vec<FixtureEntity>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CelesteMapFixture {
    pub format_version: u32,
    pub package: String,
    pub sid: String,
    pub rooms: Vec<FixtureRoom>,
}

/// A TypeScript-authored reusable part before it is merged into the canonical
/// rooms document consumed by the compiler CLI.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MapPartFixture {
    pub id: String,
    pub rooms: Vec<RoomContribution>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomContribution {
    pub name: String,
    #[serde(default)]
    pub bounds: Option<FixtureRect>,
    #[serde(default)]
    pub spawn: Option<FixtureVec2>,
    #[serde(default)]
    pub solids: Vec<FixtureRect>,
    #[serde(default)]
    pub entities: Vec<FixtureEntity>,
}

#[derive(Debug, Error)]
pub enum MapFixtureError {
    #[error("invalid fixture JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid map fixture: {0}")]
    Validation(String),
    #[error("failed to encode Celeste map: {0}")]
    Encode(#[from] MapEncodeError),
}

pub fn parse_map_fixture(json: &[u8]) -> Result<CelesteMapFixture, MapFixtureError> {
    let fixture = serde_json::from_slice(json)?;
    canonicalize_map_fixture(fixture)
}

pub fn canonical_map_fixture_json(fixture: CelesteMapFixture) -> Result<Vec<u8>, MapFixtureError> {
    let fixture = canonicalize_map_fixture(fixture)?;
    let mut bytes = serde_json::to_vec_pretty(&fixture)?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn encode_map_fixture(fixture: &CelesteMapFixture) -> Result<Vec<u8>, MapFixtureError> {
    let fixture = canonicalize_map_fixture(fixture.clone())?;
    let rooms = fixture
        .rooms
        .iter()
        .map(|room| {
            let bounds = rect(room.bounds);
            let map = Map {
                bounds,
                transition_rooms: fixture
                    .rooms
                    .iter()
                    .filter(|candidate| candidate.name != room.name)
                    .map(|candidate| rect(candidate.bounds))
                    .collect(),
                transition_runtime: vec![],
                spawn: vec2(room.spawn),
                solids: room.solids.iter().copied().map(rect).collect(),
                entities: room.entities.iter().map(entity).collect(),
                source_package: Some(fixture.package.clone()),
            };
            (room.name.clone(), map)
        })
        .collect::<Vec<_>>();
    Ok(encode_celeste_rooms(&fixture.package, &rooms)?)
}

pub fn merge_map_parts(
    format_version: u32,
    package: impl Into<String>,
    sid: impl Into<String>,
    parts: &[MapPartFixture],
) -> Result<CelesteMapFixture, MapFixtureError> {
    let mut seen_parts = BTreeSet::new();
    let mut rooms: BTreeMap<String, MergedRoom> = BTreeMap::new();
    let mut sorted_parts = parts.iter().collect::<Vec<_>>();
    sorted_parts.sort_by(|left, right| left.id.cmp(&right.id));
    for part in sorted_parts {
        require_id(&part.id, "part id")?;
        if !seen_parts.insert(part.id.clone()) {
            return validation(format!("duplicate part id {:?}", part.id));
        }
        let mut part_rooms = BTreeSet::new();
        for contribution in &part.rooms {
            require_id(&contribution.name, "room name")?;
            if !part_rooms.insert(contribution.name.clone()) {
                return validation(format!(
                    "part {:?} contributes room {:?} more than once",
                    part.id, contribution.name
                ));
            }
            let merged = rooms.entry(contribution.name.clone()).or_default();
            merge_optional(
                &mut merged.bounds,
                contribution.bounds,
                &part.id,
                &contribution.name,
                "bounds",
            )?;
            merge_optional(
                &mut merged.spawn,
                contribution.spawn,
                &part.id,
                &contribution.name,
                "spawn",
            )?;
            merged.solids.extend(contribution.solids.iter().copied());
            for entity in &contribution.entities {
                require_id(&entity.id, "entity id")?;
                match merged.entities.get(&entity.id) {
                    Some(existing) if existing != entity => {
                        return validation(format!(
                            "part {:?} conflicts with entity {:?} in room {:?}",
                            part.id, entity.id, contribution.name
                        ));
                    }
                    Some(_) => {
                        return validation(format!(
                            "part {:?} duplicates entity {:?} in room {:?}",
                            part.id, entity.id, contribution.name
                        ));
                    }
                    None => {
                        merged.entities.insert(entity.id.clone(), entity.clone());
                    }
                }
            }
        }
    }

    let rooms = rooms
        .into_iter()
        .map(|(name, merged)| {
            Ok(FixtureRoom {
                bounds: merged.bounds.ok_or_else(|| {
                    MapFixtureError::Validation(format!("room {name:?} has no bounds"))
                })?,
                spawn: merged.spawn.ok_or_else(|| {
                    MapFixtureError::Validation(format!("room {name:?} has no spawn"))
                })?,
                name,
                solids: merged.solids,
                entities: merged.entities.into_values().collect(),
            })
        })
        .collect::<Result<Vec<_>, MapFixtureError>>()?;
    canonicalize_map_fixture(CelesteMapFixture {
        format_version,
        package: package.into(),
        sid: sid.into(),
        rooms,
    })
}

#[derive(Default)]
struct MergedRoom {
    bounds: Option<FixtureRect>,
    spawn: Option<FixtureVec2>,
    solids: Vec<FixtureRect>,
    entities: BTreeMap<String, FixtureEntity>,
}

fn merge_optional<T: Copy + PartialEq + std::fmt::Debug>(
    target: &mut Option<T>,
    incoming: Option<T>,
    part: &str,
    room: &str,
    field: &str,
) -> Result<(), MapFixtureError> {
    let Some(incoming) = incoming else {
        return Ok(());
    };
    match target {
        Some(existing) if *existing != incoming => validation(format!(
            "part {part:?} conflicts with {field} for room {room:?}: {existing:?} != {incoming:?}"
        )),
        Some(_) => Ok(()),
        None => {
            *target = Some(incoming);
            Ok(())
        }
    }
}

fn canonicalize_map_fixture(
    mut fixture: CelesteMapFixture,
) -> Result<CelesteMapFixture, MapFixtureError> {
    if fixture.format_version != MAP_FIXTURE_FORMAT_VERSION {
        return validation(format!(
            "formatVersion must be {MAP_FIXTURE_FORMAT_VERSION}, got {}",
            fixture.format_version
        ));
    }
    require_id(&fixture.package, "package")?;
    require_id(&fixture.sid, "sid")?;
    if fixture.rooms.is_empty() {
        return validation("rooms must not be empty");
    }
    fixture
        .rooms
        .sort_by(|left, right| left.name.cmp(&right.name));
    let mut room_names = BTreeSet::new();
    let mut entity_ids = BTreeSet::new();
    for room in &mut fixture.rooms {
        require_id(&room.name, "room name")?;
        if !room_names.insert(room.name.clone()) {
            return validation(format!("duplicate room name {:?}", room.name));
        }
        validate_room(room, &mut entity_ids)?;
        room.solids.sort_by_key(|value| integer_rect(*value));
        room.solids.dedup();
        room.entities.sort_by(|left, right| left.id.cmp(&right.id));
    }
    Ok(fixture)
}

fn validate_room(
    room: &FixtureRoom,
    entity_ids: &mut BTreeSet<String>,
) -> Result<(), MapFixtureError> {
    let bounds = validate_rect(room.bounds, &format!("room {:?} bounds", room.name), true)?;
    let spawn = validate_vec2(room.spawn, &format!("room {:?} spawn", room.name))?;
    if !point_inside(spawn, bounds) {
        return validation(format!(
            "room {:?} spawn must be inside its bounds",
            room.name
        ));
    }
    for (index, solid) in room.solids.iter().copied().enumerate() {
        let solid = validate_rect(
            solid,
            &format!("room {:?} solids[{index}]", room.name),
            true,
        )?;
        require_inside(
            solid,
            bounds,
            &format!("room {:?} solids[{index}]", room.name),
        )?;
    }
    for (index, entity) in room.entities.iter().enumerate() {
        require_id(&entity.id, "entity id")?;
        if !entity_ids.insert(entity.id.clone()) {
            return validation(format!("duplicate entity id {:?}", entity.id));
        }
        let entity_bounds = validate_rect(
            entity.bounds,
            &format!("room {:?} entities[{index}].bounds", room.name),
            false,
        )?;
        require_inside(
            entity_bounds,
            bounds,
            &format!("room {:?} entity {:?}", room.name, entity.id),
        )?;
        validate_vec2(
            entity.direction,
            &format!("room {:?} entity {:?} direction", room.name, entity.id),
        )?;
        for (node_index, node) in entity.nodes.iter().copied().enumerate() {
            let point = validate_vec2(
                node,
                &format!(
                    "room {:?} entity {:?} nodes[{node_index}]",
                    room.name, entity.id
                ),
            )?;
            if !point_inside(point, bounds) {
                return validation(format!(
                    "room {:?} entity {:?} node {node_index} must be inside its bounds",
                    room.name, entity.id
                ));
            }
        }
        validate_entity_fields(entity, &room.name)?;
    }
    Ok(())
}

fn validate_entity_fields(entity: &FixtureEntity, room: &str) -> Result<(), MapFixtureError> {
    let direction = integer_vec2(entity.direction);
    match entity.kind {
        FixtureEntityKind::Spikes | FixtureEntityKind::Spring | FixtureEntityKind::MoveBlock => {
            if !matches!(direction, [-1 | 1, 0] | [0, -1 | 1]) {
                return validation(format!(
                    "room {room:?} entity {:?} requires a unit cardinal direction",
                    entity.id
                ));
            }
        }
        FixtureEntityKind::Wind => {
            if (direction[0] == 0) == (direction[1] == 0) {
                return validation(format!(
                    "room {room:?} entity {:?} requires one non-zero direction axis",
                    entity.id
                ));
            }
        }
        FixtureEntityKind::ZipMover if entity.nodes.len() != 1 => {
            return validation(format!(
                "room {room:?} entity {:?} requires exactly one node",
                entity.id
            ));
        }
        FixtureEntityKind::IceBall if entity.nodes.len() > 1 => {
            return validation(format!(
                "room {room:?} entity {:?} accepts at most one node",
                entity.id
            ));
        }
        FixtureEntityKind::CassetteBlock
            if direction[0] < 0 || direction[0] > 3 || direction[1] <= 0 =>
        {
            return validation(format!(
                "room {room:?} entity {:?} requires cassette direction metadata [index 0..3, positive integer tempo]",
                entity.id
            ));
        }
        _ => {}
    }
    if direction != [0, 0]
        && !matches!(
            entity.kind,
            FixtureEntityKind::Spikes
                | FixtureEntityKind::Spring
                | FixtureEntityKind::Wind
                | FixtureEntityKind::MoveBlock
                | FixtureEntityKind::CassetteBlock
                | FixtureEntityKind::Lookout
                | FixtureEntityKind::MovingSolid
        )
    {
        return validation(format!(
            "room {room:?} entity {:?} kind does not accept direction",
            entity.id
        ));
    }
    if !entity.nodes.is_empty()
        && !matches!(
            entity.kind,
            FixtureEntityKind::BadelineBoost
                | FixtureEntityKind::IceBall
                | FixtureEntityKind::ZipMover
                | FixtureEntityKind::Lookout
        )
    {
        return validation(format!(
            "room {room:?} entity {:?} kind does not accept nodes",
            entity.id
        ));
    }
    if entity.shielded && entity.kind != FixtureEntityKind::FlyFeather {
        return validation(format!(
            "room {room:?} entity {:?} kind does not accept shielded",
            entity.id
        ));
    }
    if entity.single_use
        && !matches!(
            entity.kind,
            FixtureEntityKind::FlyFeather | FixtureEntityKind::IceBall
        )
    {
        return validation(format!(
            "room {room:?} entity {:?} kind does not accept singleUse",
            entity.id
        ));
    }
    if let Some(name) = &entity.name {
        require_id(name, "entity name")?;
    }
    Ok(())
}

fn validate_rect(
    value: FixtureRect,
    path: &str,
    tile_aligned: bool,
) -> Result<[i32; 4], MapFixtureError> {
    let values = value.0;
    let mut integers = [0; 4];
    for (index, value) in values.into_iter().enumerate() {
        integers[index] = exact_i32(value, &format!("{path}[{index}]"))?;
    }
    if integers[2] <= 0 || integers[3] <= 0 {
        return validation(format!("{path} width and height must be positive"));
    }
    if tile_aligned && integers.iter().any(|value| value % 8 != 0) {
        return validation(format!("{path} must be aligned to the 8-pixel tile grid"));
    }
    Ok(integers)
}

fn validate_vec2(value: FixtureVec2, path: &str) -> Result<[i32; 2], MapFixtureError> {
    Ok([
        exact_i32(value.0[0], &format!("{path}[0]"))?,
        exact_i32(value.0[1], &format!("{path}[1]"))?,
    ])
}

fn exact_i32(value: f64, path: &str) -> Result<i32, MapFixtureError> {
    if !value.is_finite()
        || value.fract() != 0.0
        || value < i32::MIN as f64
        || value > i32::MAX as f64
        || (value as f32) as f64 != value
    {
        return validation(format!(
            "{path} must be a finite integer exactly representable by f32 and i32"
        ));
    }
    Ok(value as i32)
}

fn require_inside(inner: [i32; 4], outer: [i32; 4], path: &str) -> Result<(), MapFixtureError> {
    let inner = inner.map(i64::from);
    let outer = outer.map(i64::from);
    if inner[0] < outer[0]
        || inner[1] < outer[1]
        || inner[0] + inner[2] > outer[0] + outer[2]
        || inner[1] + inner[3] > outer[1] + outer[3]
    {
        return validation(format!("{path} must be contained by its room bounds"));
    }
    Ok(())
}

fn point_inside(point: [i32; 2], bounds: [i32; 4]) -> bool {
    let point = point.map(i64::from);
    let bounds = bounds.map(i64::from);
    point[0] >= bounds[0]
        && point[0] < bounds[0] + bounds[2]
        && point[1] >= bounds[1]
        && point[1] < bounds[1] + bounds[3]
}

fn require_id(value: &str, field: &str) -> Result<(), MapFixtureError> {
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        return validation(format!(
            "{field} must be non-empty, trimmed, and contain no control characters"
        ));
    }
    Ok(())
}

fn validation<T>(message: impl Into<String>) -> Result<T, MapFixtureError> {
    Err(MapFixtureError::Validation(message.into()))
}

fn integer_rect(value: FixtureRect) -> [i32; 4] {
    value.0.map(|value| value as i32)
}

fn integer_vec2(value: FixtureVec2) -> [i32; 2] {
    value.0.map(|value| value as i32)
}

fn rect(value: FixtureRect) -> Rect {
    Rect::new(
        value.0[0] as f32,
        value.0[1] as f32,
        value.0[2] as f32,
        value.0[3] as f32,
    )
}

fn vec2(value: FixtureVec2) -> Vec2 {
    Vec2::new(value.0[0] as f32, value.0[1] as f32)
}

fn entity(value: &FixtureEntity) -> Entity {
    Entity {
        kind: match value.kind {
            FixtureEntityKind::JumpThru => EntityKind::JumpThru,
            FixtureEntityKind::DreamBlock => EntityKind::DreamBlock,
            FixtureEntityKind::Spikes => EntityKind::Spikes,
            FixtureEntityKind::Water => EntityKind::Water,
            FixtureEntityKind::Booster => EntityKind::Booster,
            FixtureEntityKind::RedBooster => EntityKind::RedBooster,
            FixtureEntityKind::FlyFeather => EntityKind::FlyFeather,
            FixtureEntityKind::Bumper => EntityKind::Bumper,
            FixtureEntityKind::IceBall => EntityKind::IceBall,
            FixtureEntityKind::Puffer => EntityKind::Puffer,
            FixtureEntityKind::AngryOshiro => EntityKind::AngryOshiro,
            FixtureEntityKind::Seeker => EntityKind::Seeker,
            FixtureEntityKind::Snowball => EntityKind::Snowball,
            FixtureEntityKind::Cloud => EntityKind::Cloud,
            FixtureEntityKind::BadelineBoost => EntityKind::BadelineBoost,
            FixtureEntityKind::Spring => EntityKind::Spring,
            FixtureEntityKind::Strawberry => EntityKind::Strawberry,
            FixtureEntityKind::Wind => EntityKind::Wind,
            FixtureEntityKind::BounceBlock => EntityKind::BounceBlock,
            FixtureEntityKind::TheoCrystal => EntityKind::TheoCrystal,
            FixtureEntityKind::HeartGem => EntityKind::HeartGem,
            FixtureEntityKind::RisingLava => EntityKind::RisingLava,
            FixtureEntityKind::SandwichLava => EntityKind::SandwichLava,
            FixtureEntityKind::Glider => EntityKind::Glider,
            FixtureEntityKind::ZipMover => EntityKind::ZipMover,
            FixtureEntityKind::MoveBlock => EntityKind::MoveBlock,
            FixtureEntityKind::TempleGate => EntityKind::TempleGate,
            FixtureEntityKind::CassetteBlock => EntityKind::CassetteBlock,
            FixtureEntityKind::CrystalStaticSpinner => EntityKind::CrystalStaticSpinner,
            FixtureEntityKind::Lookout => EntityKind::Lookout,
            FixtureEntityKind::MovingSolid => EntityKind::MovingSolid,
        },
        bounds: rect(value.bounds),
        direction: vec2(value.direction),
        shielded: value.shielded,
        single_use: value.single_use,
        nodes: value.nodes.iter().copied().map(vec2).collect(),
        name: value.name.clone().unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        PLAYGROUND_PACKAGE, PLAYGROUND_ROOM, decode_map_room, encode_celeste_map,
        mechanics_playground, parse_celeste_bin,
    };
    use sha2::{Digest, Sha256};

    fn room_part(
        id: &str,
        bounds: Option<FixtureRect>,
        spawn: Option<FixtureVec2>,
    ) -> MapPartFixture {
        MapPartFixture {
            id: id.to_owned(),
            rooms: vec![RoomContribution {
                name: "room".to_owned(),
                bounds,
                spawn,
                solids: vec![],
                entities: vec![],
            }],
        }
    }

    #[test]
    fn multi_room_fixture_preserves_custom_spawns_and_entities() {
        let fixture = CelesteMapFixture {
            format_version: 1,
            package: "FixtureTest".to_owned(),
            sid: "FixtureTest/Map".to_owned(),
            rooms: vec![
                FixtureRoom {
                    name: "right".to_owned(),
                    bounds: FixtureRect([320.0, 0.0, 320.0, 184.0]),
                    spawn: FixtureVec2([344.0, 160.0]),
                    solids: vec![FixtureRect([320.0, 176.0, 320.0, 8.0])],
                    entities: vec![FixtureEntity {
                        id: "right-water".to_owned(),
                        kind: FixtureEntityKind::Water,
                        bounds: FixtureRect([400.0, 128.0, 32.0, 48.0]),
                        direction: FixtureVec2::default(),
                        shielded: false,
                        single_use: false,
                        nodes: vec![],
                        name: None,
                    }],
                },
                FixtureRoom {
                    name: "left".to_owned(),
                    bounds: FixtureRect([0.0, 0.0, 320.0, 184.0]),
                    spawn: FixtureVec2([24.0, 160.0]),
                    solids: vec![FixtureRect([0.0, 176.0, 320.0, 8.0])],
                    entities: vec![FixtureEntity {
                        id: "left-booster".to_owned(),
                        kind: FixtureEntityKind::Booster,
                        bounds: FixtureRect([80.0, 80.0, 16.0, 16.0]),
                        direction: FixtureVec2::default(),
                        shielded: false,
                        single_use: false,
                        nodes: vec![],
                        name: None,
                    }],
                },
            ],
        };
        let first = encode_map_fixture(&fixture).unwrap();
        let second = encode_map_fixture(&fixture).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            parse_celeste_bin(&first).unwrap().package.as_deref(),
            Some("FixtureTest")
        );

        let left = decode_map_room(&first, Some("left")).unwrap();
        assert_eq!(left.spawn, Vec2::new(24.0, 160.0));
        assert_eq!(left.entities.len(), 1);
        assert_eq!(left.entities[0].kind, EntityKind::Booster);
        let right = decode_map_room(&first, Some("right")).unwrap();
        assert_eq!(right.spawn, Vec2::new(344.0, 160.0));
        assert_eq!(right.entities.len(), 1);
        assert_eq!(right.entities[0].kind, EntityKind::Water);
    }

    #[test]
    fn part_merge_is_order_independent_and_rejects_conflicts() {
        let bounds = FixtureRect([0.0, 0.0, 320.0, 184.0]);
        let spawn = FixtureVec2([24.0, 160.0]);
        let mut geometry = MapPartFixture {
            id: "geometry".to_owned(),
            rooms: vec![RoomContribution {
                name: "room".to_owned(),
                bounds: Some(bounds),
                spawn: None,
                solids: vec![FixtureRect([0.0, 176.0, 320.0, 8.0])],
                entities: vec![],
            }],
        };
        geometry.rooms[0]
            .entities
            .push(test_booster("z-booster", 80.0));
        let mut player = room_part("player", Some(bounds), Some(spawn));
        player.rooms[0]
            .entities
            .push(test_booster("a-booster", 40.0));
        let one = merge_map_parts(
            1,
            "FixtureTest",
            "FixtureTest/Map",
            &[geometry.clone(), player.clone()],
        )
        .unwrap();
        let two =
            merge_map_parts(1, "FixtureTest", "FixtureTest/Map", &[player, geometry]).unwrap();
        assert_eq!(one, two);
        assert_eq!(
            one.rooms[0]
                .entities
                .iter()
                .map(|entity| entity.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a-booster", "z-booster"]
        );
        assert_eq!(
            encode_map_fixture(&one).unwrap(),
            encode_map_fixture(&two).unwrap()
        );

        let conflict = room_part(
            "conflict",
            Some(FixtureRect([8.0, 0.0, 320.0, 184.0])),
            None,
        );
        let error = merge_map_parts(
            1,
            "FixtureTest",
            "FixtureTest/Map",
            &[room_part("base", Some(bounds), Some(spawn)), conflict],
        )
        .unwrap_err();
        assert!(error.to_string().contains("conflicts with bounds"));
    }

    fn test_booster(id: &str, x: f64) -> FixtureEntity {
        FixtureEntity {
            id: id.to_owned(),
            kind: FixtureEntityKind::Booster,
            bounds: FixtureRect([x, 80.0, 16.0, 16.0]),
            direction: FixtureVec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: None,
        }
    }

    #[test]
    fn rejects_unknown_fields_unknown_kinds_fractions_and_misaligned_solids() {
        let base = |entity: &str, solid: &str| {
            format!(
                r#"{{"formatVersion":1,"package":"P","sid":"P/M","rooms":[{{"name":"r","bounds":[0,0,320,184],"spawn":[24,160],"solids":[{solid}],"entities":[{entity}]}}]}}"#
            )
        };
        assert!(
            parse_map_fixture(
                base(
                    r#"{"id":"e","kind":"nope","bounds":[8,8,8,8]}"#,
                    "[0,176,320,8]"
                )
                .as_bytes()
            )
            .is_err()
        );
        assert!(
            parse_map_fixture(
                base(
                    r#"{"id":"e","kind":"booster","bounds":[8.5,8,16,16]}"#,
                    "[0,176,320,8]"
                )
                .as_bytes()
            )
            .is_err()
        );
        assert!(
            parse_map_fixture(
                base(
                    r#"{"id":"e","kind":"booster","bounds":[8,8,16,16],"extra":1}"#,
                    "[0,176,320,8]"
                )
                .as_bytes()
            )
            .is_err()
        );
        assert!(
            parse_map_fixture(
                base(
                    r#"{"id":"e","kind":"booster","bounds":[8,8,16,16]}"#,
                    "[4,176,312,8]"
                )
                .as_bytes()
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_duplicate_ids_non_finite_values_and_out_of_bounds_spawns() {
        let duplicate = br#"{
            "formatVersion":1,"package":"P","sid":"P/M","rooms":[{
                "name":"r","bounds":[0,0,320,184],"spawn":[24,160],"solids":[],
                "entities":[
                    {"id":"same","kind":"booster","bounds":[8,8,16,16]},
                    {"id":"same","kind":"booster","bounds":[32,8,16,16]}
                ]
            }]
        }"#;
        assert!(parse_map_fixture(duplicate).is_err());

        let mut invalid_number = CelesteMapFixture {
            format_version: 1,
            package: "P".to_owned(),
            sid: "P/M".to_owned(),
            rooms: vec![FixtureRoom {
                name: "r".to_owned(),
                bounds: FixtureRect([0.0, 0.0, 320.0, 184.0]),
                spawn: FixtureVec2([24.0, 160.0]),
                solids: vec![],
                entities: vec![],
            }],
        };
        invalid_number.rooms[0].spawn.0[0] = f64::INFINITY;
        assert!(encode_map_fixture(&invalid_number).is_err());
        invalid_number.rooms[0].spawn = FixtureVec2([320.0, 160.0]);
        assert!(encode_map_fixture(&invalid_number).is_err());

        let repeated_part = room_part(
            "repeated",
            Some(FixtureRect([0.0, 0.0, 320.0, 184.0])),
            Some(FixtureVec2([24.0, 160.0])),
        );
        assert!(merge_map_parts(1, "P", "P/M", &[repeated_part.clone(), repeated_part]).is_err());
    }

    #[test]
    fn committed_playground_fixture_source_and_mirrors_are_byte_deterministic() {
        let fixture_json = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/e2e/playground.map.fixture.json"
        ));
        let fixture = parse_map_fixture(fixture_json).unwrap();
        assert_eq!(
            canonical_map_fixture_json(fixture.clone()).unwrap(),
            fixture_json
        );

        let first = encode_map_fixture(&fixture).unwrap();
        let second = encode_map_fixture(&fixture).unwrap();
        assert_eq!(first, second);
        assert_eq!(Sha256::digest(&first), Sha256::digest(&second));

        let legacy =
            encode_celeste_map(&mechanics_playground(), PLAYGROUND_PACKAGE, PLAYGROUND_ROOM)
                .unwrap();
        assert_eq!(first, legacy);
        assert_eq!(
            first.as_slice(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../mods/CelesteGymPlayground/Maps/CelesteGymPlayground/Playground.bin"
            ))
        );
        assert_eq!(
            first.as_slice(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../web/public/assets/original/maps/CelesteGymPlayground-Playground.bin"
            ))
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&first)),
            "042a6ead85c005e69909b25c7f966757fec55c25d7e610aa654efac38604731a"
        );
    }
}
