use super::Registration;
use crate::{
    BinaryElement, Rect,
    entity_decode::{attr_bool, attr_text},
};

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "FancyTileEntities/FancySolidTiles").then(Registration::decoration)
}

pub(super) fn compatible(entity: &BinaryElement) -> bool {
    entity.name != "FancyTileEntities/FancySolidTiles"
        || (!attr_bool(entity, "loadGlobally", false)
            && attr_text(entity, "tileData").is_some_and(|data| !data.is_empty()))
}

pub(super) fn additional_solids(entity: &BinaryElement, x: f32, y: f32) -> Vec<Rect> {
    if entity.name != "FancyTileEntities/FancySolidTiles" || !compatible(entity) {
        return vec![];
    }
    let data = attr_text(entity, "tileData").expect("compatible tile data");
    let mut solids = Vec::new();
    for (row, tiles) in data.split(',').enumerate() {
        let chars = tiles.as_bytes();
        let mut column = 0usize;
        while column < chars.len() {
            if chars[column] == b'0' {
                column += 1;
                continue;
            }
            let start = column;
            while column < chars.len() && chars[column] != b'0' {
                column += 1;
            }
            solids.push(Rect::new(
                x + start as f32 * 8.0,
                y + row as f32 * 8.0,
                (column - start) as f32 * 8.0,
                8.0,
            ));
        }
    }
    solids
}
