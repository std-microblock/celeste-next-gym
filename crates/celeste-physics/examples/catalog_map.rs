use std::{env, fs, path::PathBuf, process::ExitCode};

use celeste_physics::{EntityKind, audit_celeste_map, decode_map_room_local};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RoomRecord {
    source: String,
    room: String,
    audit: celeste_physics::CelesteRoomAudit,
    unknown_entities: Vec<String>,
    map: celeste_physics::Map,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorRecord {
    source: String,
    error: String,
}

fn main() -> ExitCode {
    let paths = env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if paths.is_empty() {
        eprintln!("usage: catalog_map <map.bin> [map.bin ...]");
        return ExitCode::from(2);
    }

    let mut failed = false;
    for path in paths {
        let source = path.to_string_lossy().into_owned();
        let result = fs::read(&path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| {
                let audits = audit_celeste_map(&bytes).map_err(|error| error.to_string())?;
                for audit in audits {
                    let map = decode_map_room_local(&bytes, Some(&audit.name))
                        .map_err(|error| error.to_string())?;
                    let unknown_entities = map
                        .entities
                        .iter()
                        .filter(|entity| entity.kind == EntityKind::Unknown)
                        .map(|entity| entity.name.clone())
                        .collect::<Vec<_>>();
                    let record = RoomRecord {
                        source: source.clone(),
                        room: audit.name.clone(),
                        audit,
                        unknown_entities,
                        map,
                    };
                    println!(
                        "{}",
                        serde_json::to_string(&record).map_err(|e| e.to_string())?
                    );
                }
                Ok(())
            });
        if let Err(error) = result {
            failed = true;
            println!(
                "{}",
                serde_json::to_string(&ErrorRecord { source, error })
                    .expect("error record should serialize")
            );
        }
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
