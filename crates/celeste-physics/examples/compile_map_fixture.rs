use std::{env, fs, path::Path, process::ExitCode};

use celeste_physics::{
    PLAYGROUND_PACKAGE, PLAYGROUND_ROOM, encode_celeste_map, encode_map_fixture,
    mechanics_playground, parse_map_fixture,
};

fn main() -> ExitCode {
    match run() {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let mut check = false;
    let mut legacy_playground = false;
    let mut paths = Vec::new();
    for argument in env::args_os().skip(1) {
        match argument.to_str() {
            Some("--check") => check = true,
            Some("--legacy-playground") => legacy_playground = true,
            Some(value) if value.starts_with('-') => {
                return Err(format!("unknown option {value:?}\n{}", usage()));
            }
            _ => paths.push(argument),
        }
    }
    if paths.len() < 2 {
        return Err(usage().to_owned());
    }
    let fixture_path = Path::new(&paths[0]);
    let output_paths = paths[1..].iter().map(Path::new).collect::<Vec<_>>();
    let fixture_json = fs::read(fixture_path)
        .map_err(|error| format!("failed to read {}: {error}", fixture_path.display()))?;
    let fixture = parse_map_fixture(&fixture_json).map_err(|error| error.to_string())?;
    let bytes = encode_map_fixture(&fixture).map_err(|error| error.to_string())?;
    let repeated = encode_map_fixture(&fixture).map_err(|error| error.to_string())?;
    if bytes != repeated {
        return Err("fixture generated different bytes on the second pass".to_owned());
    }
    if legacy_playground {
        if fixture.package != PLAYGROUND_PACKAGE || fixture.sid != "CelesteGymPlayground/Playground"
        {
            return Err("--legacy-playground requires the CelesteGymPlayground fixture".to_owned());
        }
        let legacy =
            encode_celeste_map(&mechanics_playground(), PLAYGROUND_PACKAGE, PLAYGROUND_ROOM)
                .map_err(|error| error.to_string())?;
        if bytes != legacy {
            return Err(format!(
                "{} compiles to different bytes than mechanics_playground()",
                fixture_path.display()
            ));
        }
    }

    if check {
        for output_path in &output_paths {
            let committed = fs::read(output_path)
                .map_err(|error| format!("failed to read {}: {error}", output_path.display()))?;
            if committed != bytes {
                return Err(format!(
                    "{} is stale (fixture compiled to {} bytes, file has {} bytes)",
                    output_path.display(),
                    bytes.len(),
                    committed.len()
                ));
            }
        }
        Ok(format!(
            "fixture check passed: {} rooms, {} bytes, {} synchronized outputs",
            fixture.rooms.len(),
            bytes.len(),
            output_paths.len()
        ))
    } else {
        for output_path in &output_paths {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
            }
            fs::write(output_path, &bytes)
                .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
        }
        Ok(format!(
            "compiled {} rooms to {} bytes in {} outputs",
            fixture.rooms.len(),
            bytes.len(),
            output_paths.len()
        ))
    }
}

fn usage() -> &'static str {
    "usage: compile_map_fixture [--check] [--legacy-playground] <fixture.json> <output.bin> [mirror.bin ...]"
}
