use std::{env, fs, process::ExitCode};

use celeste_physics::{BinaryElement, BinaryValue, parse_celeste_bin};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let (Some(path), room) = (args.next(), args.next()) else {
        eprintln!("usage: inspect_bin_tree <map.bin> [room]");
        return ExitCode::from(2);
    };
    let Ok(bytes) = fs::read(&path) else {
        eprintln!("failed to read {path:?}");
        return ExitCode::FAILURE;
    };
    let Ok(root) = parse_celeste_bin(&bytes) else {
        eprintln!("failed to parse {path:?}");
        return ExitCode::FAILURE;
    };
    println!("package={:?} root={}", root.package, root.name);
    for child in &root.children {
        println!(
            "top {} attrs={:?} children={}",
            child.name,
            child.attributes,
            child.children.len()
        );
        if room.is_none() {
            for grandchild in &child.children {
                println!(
                    "  {} attrs={:?} children={}",
                    grandchild.name,
                    grandchild.attributes,
                    grandchild.children.len()
                );
            }
        }
    }
    if let Some(room) = room.and_then(|room| room.into_string().ok()) {
        let Some(levels) = root.children.iter().find(|child| child.name == "levels") else {
            return ExitCode::FAILURE;
        };
        let Some(level) = levels.children.iter().find(|level| {
            matches!(
                level.attributes.get("name"),
                Some(BinaryValue::String(name))
                    if name == &room || name.strip_prefix("lvl_") == Some(room.as_str())
            )
        }) else {
            eprintln!("room {room} was not found");
            return ExitCode::FAILURE;
        };
        print_element(level, 0, 2);
    }
    ExitCode::SUCCESS
}

fn print_element(element: &BinaryElement, depth: usize, max_depth: usize) {
    let indent = "  ".repeat(depth);
    println!(
        "{indent}{} attrs={:?} children={}",
        element.name,
        element.attributes,
        element.children.len()
    );
    if depth >= max_depth {
        return;
    }
    for child in &element.children {
        print_element(child, depth + 1, max_depth);
    }
}
