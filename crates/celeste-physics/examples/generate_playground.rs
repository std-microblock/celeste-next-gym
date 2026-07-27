use std::{env, fs, process::ExitCode};

use celeste_physics::{
    PLAYGROUND_PACKAGE, PLAYGROUND_ROOM, encode_celeste_map, mechanics_playground,
};

fn main() -> ExitCode {
    let Some(path) = env::args_os().nth(1) else {
        eprintln!("usage: generate_playground <output.bin>");
        return ExitCode::from(2);
    };
    let map = mechanics_playground();
    let Ok(bytes) = encode_celeste_map(&map, PLAYGROUND_PACKAGE, PLAYGROUND_ROOM) else {
        eprintln!("failed to encode playground");
        return ExitCode::FAILURE;
    };
    if let Some(parent) = std::path::Path::new(&path).parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        eprintln!("failed to create output directory: {error}");
        return ExitCode::FAILURE;
    }
    if let Err(error) = fs::write(&path, bytes) {
        eprintln!("failed to write {path:?}: {error}");
        return ExitCode::FAILURE;
    }
    println!("wrote {path:?}");
    ExitCode::SUCCESS
}
