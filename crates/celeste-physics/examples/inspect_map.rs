use std::{env, fs, process::ExitCode};

use celeste_physics::{BinaryValue, decode_map_room, parse_celeste_bin};

fn main() -> ExitCode {
    let Some(path) = env::args_os().nth(1) else {
        eprintln!("usage: inspect_map <map.bin>");
        return ExitCode::from(2);
    };
    let Ok(bytes) = fs::read(&path) else {
        eprintln!("failed to read {path:?}");
        return ExitCode::FAILURE;
    };
    if env::args_os().nth(2).is_some_and(|arg| arg == "--rooms") {
        let Ok(root) = parse_celeste_bin(&bytes) else {
            eprintln!("failed to parse {path:?}");
            return ExitCode::FAILURE;
        };
        let Some(levels) = root.children.iter().find(|child| child.name == "levels") else {
            eprintln!("map has no levels");
            return ExitCode::FAILURE;
        };
        for level in &levels.children {
            let name = match level.attributes.get("name") {
                Some(BinaryValue::String(value)) => value.as_str(),
                _ => "<unnamed>",
            };
            let entities = level.children.iter().find(|child| child.name == "entities");
            let count = |kind: &str| {
                entities
                    .map(|entities| {
                        entities
                            .children
                            .iter()
                            .filter(|entity| entity.name == kind)
                            .count()
                    })
                    .unwrap_or(0)
            };
            let dream_blocks = count("dreamBlock");
            let water = count("water");
            let boosters = count("booster");
            let red_boosters = count("redBooster");
            let wind = count("windTrigger");
            if dream_blocks + water + boosters + red_boosters + wind > 0 {
                println!(
                    "room={name} dream_blocks={dream_blocks} water={water} boosters={boosters} red_boosters={red_boosters} wind={wind}"
                );
            }
        }
        return ExitCode::SUCCESS;
    }
    let room = env::args().nth(2).filter(|arg| arg != "--rooms");
    let Ok(map) = decode_map_room(&bytes, room.as_deref()) else {
        eprintln!("failed to decode {path:?}");
        return ExitCode::FAILURE;
    };
    println!("bounds={:?} spawn={:?}", map.bounds, map.spawn);
    for solid in &map.solids {
        println!("solid {solid:?}");
    }
    for entity in &map.entities {
        println!("entity {entity:?}");
    }
    ExitCode::SUCCESS
}
