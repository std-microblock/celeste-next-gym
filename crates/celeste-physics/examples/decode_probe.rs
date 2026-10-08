//! Read-only decode probe for workstream Y2 ("mapdecode").
//!
//! Every claim about what `map.rs` decodes is checkable here by printing the
//! decoded value next to the raw BinaryPacker attribute it came from.
//!
//! ```text
//! usage: decode_probe room     <map.bin> <room>   decoded room vs raw element attrs
//!        decode_probe unknown  <maps-dir>         EntityKind::Unknown counts per room
//!        decode_probe names    <maps-dir>         unhandled entity-name counts
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{env, fs, process::ExitCode};

use celeste_physics::{
    BinaryElement, BinaryValue, EntityKind, Map, audit_celeste_map, celeste_map_rooms,
    decode_map_room_local, parse_celeste_bin,
};

fn value_string(value: &BinaryValue) -> String {
    match value {
        BinaryValue::Bool(v) => v.to_string(),
        BinaryValue::Byte(v) => v.to_string(),
        BinaryValue::Short(v) => v.to_string(),
        BinaryValue::Int(v) => v.to_string(),
        BinaryValue::Float(v) => v.to_string(),
        BinaryValue::String(v) => format!("{v:?}"),
    }
}

fn attribute<'a>(element: &'a BinaryElement, key: &str) -> Option<&'a BinaryValue> {
    element.attributes.get(key)
}

fn find_child<'a>(element: &'a BinaryElement, name: &str) -> Option<&'a BinaryElement> {
    element.children.iter().find(|child| child.name == name)
}

fn find_room<'a>(root: &'a BinaryElement, room: &str) -> Option<&'a BinaryElement> {
    let levels = find_child(root, "levels")?;
    levels.children.iter().find(|level| {
        attribute(level, "name").is_some_and(|value| match value {
            BinaryValue::String(name) => name == room || name.strip_prefix("lvl_") == Some(room),
            _ => false,
        })
    })
}

fn bin_files(directory: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![directory.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "bin") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Print a `Map`'s bounds, solids and entities in a diffable form.
fn dump_map(map: &Map) {
    println!(
        "decoded bounds=({},{},{},{}) spawn=({},{}) room_spawns={} solids={} entities={}",
        map.bounds.x,
        map.bounds.y,
        map.bounds.width,
        map.bounds.height,
        map.spawn.x,
        map.spawn.y,
        map.room_spawns.len(),
        map.solids.len(),
        map.entities.len()
    );
    for (index, solid) in map.solids.iter().enumerate() {
        println!(
            "  decoded solid[{index}]=({},{},{},{})",
            solid.x, solid.y, solid.width, solid.height
        );
    }
    for (index, entity) in map.entities.iter().enumerate() {
        println!(
            "  decoded entity[{index}] name={:?} kind={:?} bounds=({},{},{},{}) dir=({},{})",
            entity.name,
            entity.kind,
            entity.bounds.x,
            entity.bounds.y,
            entity.bounds.width,
            entity.bounds.height,
            entity.direction.x,
            entity.direction.y
        );
    }
}

fn room_command(map_path: &Path, room: &str) -> ExitCode {
    let Ok(bytes) = fs::read(map_path) else {
        eprintln!("failed to read {}", map_path.display());
        return ExitCode::FAILURE;
    };
    let Ok(root) = parse_celeste_bin(&bytes) else {
        eprintln!("failed to parse {}", map_path.display());
        return ExitCode::FAILURE;
    };
    let Some(level) = find_room(&root, room) else {
        eprintln!("no room {room:?} in {}", map_path.display());
        return ExitCode::FAILURE;
    };
    println!(
        "raw level name={:?}",
        attribute(level, "name").map(value_string)
    );
    for key in ["x", "y", "width", "height"] {
        println!(
            "raw level {key}={}",
            attribute(level, key)
                .map(value_string)
                .unwrap_or_else(|| "<absent>".into())
        );
    }
    if let Some(entities) = find_child(level, "entities") {
        println!("raw entities count={}", entities.children.len());
        for entity in &entities.children {
            let attributes = entity
                .attributes
                .iter()
                .map(|(key, value)| format!("{key}={}", value_string(value)))
                .collect::<Vec<_>>()
                .join(" ");
            println!(
                "  raw entity name={:?} children={} {attributes}",
                entity.name,
                entity.children.len()
            );
        }
    }
    if let Some(solids) = find_child(level, "solids") {
        println!(
            "raw solids offsetX={} offsetY={}",
            attribute(solids, "offsetX")
                .map(value_string)
                .unwrap_or_else(|| "<absent>".into()),
            attribute(solids, "offsetY")
                .map(value_string)
                .unwrap_or_else(|| "<absent>".into())
        );
        let text = attribute(solids, "innerText").and_then(|value| match value {
            BinaryValue::String(text) => Some(text.clone()),
            _ => None,
        });
        match text {
            Some(text) => {
                let rows: Vec<&str> = text.lines().collect();
                println!(
                    "raw solids innerText chars={} lines={} row_widths={:?}",
                    text.chars().count(),
                    rows.len(),
                    rows.iter()
                        .map(|row| row.chars().count())
                        .collect::<Vec<_>>()
                );
                for (index, row) in rows.iter().enumerate() {
                    println!(
                        "  raw solids row[{index}] y={} len={} {row}",
                        index as f32 * 8.0,
                        row.chars().count()
                    );
                }
            }
            None => println!("raw solids innerText=<absent>"),
        }
    }
    let Ok(map) = decode_map_room_local(&bytes, Some(room)) else {
        eprintln!("failed to decode room {room:?}");
        return ExitCode::FAILURE;
    };
    dump_map(&map);
    ExitCode::SUCCESS
}

fn unknown_command(directory: &Path) -> ExitCode {
    let mut total_unknown = 0usize;
    let mut total_entities = 0usize;
    let mut rooms_with_unknown = 0usize;
    let mut rooms = 0usize;
    let mut per_name: BTreeMap<String, u32> = BTreeMap::new();
    for path in bin_files(directory) {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(names) = celeste_map_rooms(&bytes) else {
            continue;
        };
        for room in names {
            rooms += 1;
            let Ok(map) = decode_map_room_local(&bytes, Some(&room)) else {
                continue;
            };
            let mut room_unknown = 0usize;
            for entity in &map.entities {
                total_entities += 1;
                if entity.kind == EntityKind::Unknown {
                    total_unknown += 1;
                    room_unknown += 1;
                    *per_name.entry(entity.name.clone()).or_insert(0) += 1;
                }
            }
            if room_unknown > 0 {
                rooms_with_unknown += 1;
                println!(
                    "unknown room={room} file={} count={room_unknown}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
            }
        }
    }
    println!(
        "TOTAL rooms={rooms} rooms_with_unknown={rooms_with_unknown} entities={total_entities} unknown={total_unknown}"
    );
    for (name, count) in &per_name {
        println!("  unknown name={name:?} count={count}");
    }
    ExitCode::SUCCESS
}

fn names_command(directory: &Path) -> ExitCode {
    let mut per_name: BTreeMap<String, u64> = BTreeMap::new();
    let mut instances = 0u64;
    for path in bin_files(directory) {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(audits) = audit_celeste_map(&bytes) else {
            continue;
        };
        for audit in audits {
            for (name, count) in &audit.entity_names {
                *per_name.entry(name.clone()).or_insert(0) += u64::from(*count);
                instances += u64::from(*count);
            }
        }
    }
    println!(
        "TOTAL entity instances={instances} distinct names={}",
        per_name.len()
    );
    for (name, count) in &per_name {
        println!("  name={name:?} count={count}");
    }
    ExitCode::SUCCESS
}

/// `LevelLoader.cs:100` splits the tile grid with
/// `new Regex("\\r\\n|\\n\\r|\\n|\\r")`. Reproduce that exactly so a claim about
/// row alignment can be checked against what the game really builds.
fn dotnet_regex_split(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut rows = Vec::new();
    let mut current = String::new();
    let mut index = 0;
    while index < bytes.len() {
        let two = bytes.get(index..index + 2);
        let separator_len = match two {
            Some([b'\r', b'\n']) | Some([b'\n', b'\r']) => 2,
            Some([b'\r']) | Some([b'\n']) => 1,
            _ => 0,
        };
        if separator_len > 0 {
            rows.push(std::mem::take(&mut current));
            index += separator_len;
            continue;
        }
        let ch = text[index..].chars().next().expect("in bounds");
        current.push(ch);
        index += ch.len_utf8();
    }
    rows.push(current);
    rows
}

fn escaped(text: &str, limit: usize) -> String {
    let mut out = String::new();
    for (count, ch) in text.chars().enumerate() {
        if count >= limit {
            out.push_str("...");
            break;
        }
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn text_command(map_path: &Path, room: &str) -> ExitCode {
    let Ok(bytes) = fs::read(map_path) else {
        eprintln!("failed to read {}", map_path.display());
        return ExitCode::FAILURE;
    };
    let Ok(root) = parse_celeste_bin(&bytes) else {
        eprintln!("failed to parse {}", map_path.display());
        return ExitCode::FAILURE;
    };
    let Some(level) = find_room(&root, room) else {
        eprintln!("no room {room:?} in {}", map_path.display());
        return ExitCode::FAILURE;
    };
    let x = match attribute(level, "x") {
        Some(BinaryValue::Short(value)) => *value as f32,
        Some(BinaryValue::Int(value)) => *value as f32,
        Some(BinaryValue::Float(value)) => *value,
        Some(BinaryValue::Byte(value)) => *value as f32,
        _ => 0.0,
    };
    let y = match attribute(level, "y") {
        Some(BinaryValue::Short(value)) => *value as f32,
        Some(BinaryValue::Int(value)) => *value as f32,
        Some(BinaryValue::Float(value)) => *value,
        Some(BinaryValue::Byte(value)) => *value as f32,
        _ => 0.0,
    };
    println!("level x={x} y={y}");
    if let Some(filler) = find_child(&root, "Filler") {
        println!("Filler children={}", filler.children.len());
        for child in &filler.children {
            let attributes = child
                .attributes
                .iter()
                .map(|(key, value)| format!("{key}={}", value_string(value)))
                .collect::<Vec<_>>()
                .join(" ");
            println!("  Filler rect {attributes} (tile coords)");
        }
    } else {
        println!("Filler <absent>");
    }
    println!(
        "level children: {:?}",
        level
            .children
            .iter()
            .map(|child| child.name.as_str())
            .collect::<Vec<_>>()
    );
    for solids in level.children.iter().filter(|child| child.name == "solids") {
        let Some(BinaryValue::String(text)) = attribute(solids, "innerText") else {
            println!("solids without innerText");
            continue;
        };
        let rust_rows: Vec<&str> = text.lines().collect();
        let dotnet_rows = dotnet_regex_split(text);
        println!(
            "solids chars={} rust_lines={} dotnet_rows={}",
            text.chars().count(),
            rust_rows.len(),
            dotnet_rows.len()
        );
        let positions = |needle: char| {
            text.chars()
                .enumerate()
                .filter(|(_, ch)| *ch == needle)
                .map(|(index, _)| index)
                .collect::<Vec<_>>()
        };
        let cr = positions('\r');
        let lf = positions('\n');
        println!("  CR count={} at {:?}", cr.len(), &cr[..cr.len().min(40)]);
        println!("  LF count={} at {:?}", lf.len(), &lf[..lf.len().min(40)]);
        println!(
            "  rust row widths  = {:?}",
            rust_rows
                .iter()
                .map(|row| row.chars().count())
                .collect::<Vec<_>>()
        );
        println!(
            "  dotnet row widths= {:?}",
            dotnet_rows
                .iter()
                .map(|row| row.chars().count())
                .collect::<Vec<_>>()
        );
        for (index, row) in dotnet_rows.iter().enumerate() {
            println!(
                "  dotnet row[{index}] world_y={} len={} {}",
                y + index as f32 * 8.0,
                row.chars().count(),
                escaped(row, 130)
            );
        }
    }
    ExitCode::SUCCESS
}

fn rooms_command(map_path: &Path) -> ExitCode {
    let Ok(bytes) = fs::read(map_path) else {
        eprintln!("failed to read {}", map_path.display());
        return ExitCode::FAILURE;
    };
    let Ok(root) = parse_celeste_bin(&bytes) else {
        eprintln!("failed to parse {}", map_path.display());
        return ExitCode::FAILURE;
    };
    let Some(levels) = find_child(&root, "levels") else {
        eprintln!("no levels");
        return ExitCode::FAILURE;
    };
    println!(
        "package={:?} levels={}",
        root.package,
        levels.children.len()
    );
    for level in &levels.children {
        let name = match attribute(level, "name") {
            Some(BinaryValue::String(value)) => value.clone(),
            other => format!("<{other:?}>"),
        };
        let field = |key: &str| match attribute(level, key) {
            Some(BinaryValue::Byte(value)) => *value as f64,
            Some(BinaryValue::Short(value)) => *value as f64,
            Some(BinaryValue::Int(value)) => *value as f64,
            Some(BinaryValue::Float(value)) => *value as f64,
            Some(BinaryValue::String(value)) => value.parse().unwrap_or(f64::NAN),
            _ => f64::NAN,
        };
        println!(
            "room name={name:?} x={} y={} width={} height={}",
            field("x"),
            field("y"),
            field("width"),
            field("height")
        );
    }
    ExitCode::SUCCESS
}

/// Print every raw attribute set seen for the requested entity names across
/// every room of every `.bin` under a directory. This pins decoder defaults to
/// real vanilla data instead of a guess.
fn attrs_command(directory: &Path, names: &[String]) -> ExitCode {
    let mut seen: BTreeMap<String, (u64, Vec<String>)> = BTreeMap::new();
    let mut rooms = 0u64;
    for path in bin_files(directory) {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(root) = parse_celeste_bin(&bytes) else {
            continue;
        };
        let Some(levels) = find_child(&root, "levels") else {
            continue;
        };
        for level in &levels.children {
            let room = match attribute(level, "name") {
                Some(BinaryValue::String(value)) => value.clone(),
                _ => continue,
            };
            rooms += 1;
            let Some(entities) = find_child(level, "entities") else {
                continue;
            };
            for entity in &entities.children {
                if !names.iter().any(|name| name == &entity.name) {
                    continue;
                }
                let mut attributes = entity
                    .attributes
                    .iter()
                    .filter(|(key, _)| {
                        key.as_str() != "id" && key.as_str() != "x" && key.as_str() != "y"
                    })
                    .map(|(key, value)| format!("{key}={}", value_string(value)))
                    .collect::<Vec<_>>();
                attributes.sort();
                let key = format!("{} :: {}", entity.name, attributes.join(" "));
                let entry = seen.entry(key).or_insert_with(|| (0, Vec::new()));
                entry.0 += 1;
                if entry.1.len() < 3 {
                    entry.1.push(format!(
                        "{}/{}: {}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        room,
                        attributes.join(" ")
                    ));
                }
            }
        }
    }
    println!("rooms scanned={rooms}");
    let mut rows: Vec<_> = seen.into_iter().collect();
    rows.sort_by(|a, b| b.1.0.cmp(&a.1.0));
    for (key, (count, samples)) in rows {
        println!("{key}  count={count}");
        for sample in samples {
            println!("    {sample}");
        }
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("room") if args.len() == 4 => room_command(Path::new(&args[2]), &args[3]),
        Some("tex") if args.len() == 4 => text_command(Path::new(&args[2]), &args[3]),
        Some("rooms") if args.len() == 3 => rooms_command(Path::new(&args[2])),
        Some("unknown") if args.len() == 3 => unknown_command(Path::new(&args[2])),
        Some("names") if args.len() == 3 => names_command(Path::new(&args[2])),
        Some("attrs") if args.len() == 4 => attrs_command(
            Path::new(&args[2]),
            &args[3].split(',').map(str::to_owned).collect::<Vec<_>>(),
        ),
        _ => {
            eprintln!(
                "usage: decode_probe room <map.bin> <room>\n       \
                 decode_probe tex <map.bin> <room>\n       \
                 decode_probe rooms <map.bin>\n       \
                 decode_probe unknown <maps-dir>\n       \
                 decode_probe names <maps-dir>\n       \
                 decode_probe attrs <maps-dir> <name,name>"
            );
            ExitCode::from(2)
        }
    }
}
