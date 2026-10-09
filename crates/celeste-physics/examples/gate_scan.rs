use std::{collections::BTreeMap, env, fs, process::ExitCode};

use celeste_physics::{BinaryElement, BinaryValue, parse_celeste_bin};

fn attr<'a>(el: &'a BinaryElement, name: &str) -> Option<&'a str> {
    match el.attributes.get(name) {
        Some(BinaryValue::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn int_attr(el: &BinaryElement, name: &str) -> i32 {
    match el.attributes.get(name) {
        Some(BinaryValue::Int(value)) => *value,
        _ => 0,
    }
}

fn bool_attr(el: &BinaryElement, name: &str) -> bool {
    matches!(el.attributes.get(name), Some(BinaryValue::Bool(true)))
}

fn main() -> ExitCode {
    let path = env::args_os().nth(1).expect("usage: gate_scan <map.bin> [room]");
    let wanted: Vec<String> = env::args().skip(2).collect();
    let bytes = fs::read(&path).unwrap();
    let root = parse_celeste_bin(&bytes).unwrap();
    let levels = root.children.iter().find(|c| c.name == "levels").unwrap();
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for level in &levels.children {
        let room = attr(level, "name").unwrap_or("<unnamed>").to_owned();
        let Some(entities) = level.children.iter().find(|c| c.name == "entities") else {
            continue;
        };
        let mut lines: Vec<String> = Vec::new();
        for entity in &entities.children {
            match entity.name.as_str() {
                "templeGate" => {
                    let ty = attr(entity, "type").unwrap_or("<none>").to_owned();
                    *counts.entry(ty.clone()).or_default() += 1;
                    lines.push(format!(
                        "    templeGate type={ty} x={} y={} h={} id={}",
                        int_attr(entity, "x"),
                        int_attr(entity, "y"),
                        int_attr(entity, "height"),
                        int_attr(entity, "id"),
                    ));
                }
                "dashSwitchH" | "dashSwitchV" => {
                    lines.push(format!(
                        "    {} x={} y={} id={} persistent={} allGates={} leftSide={} ceiling={}",
                        entity.name,
                        int_attr(entity, "x"),
                        int_attr(entity, "y"),
                        int_attr(entity, "id"),
                        bool_attr(entity, "persistent"),
                        bool_attr(entity, "allGates"),
                        bool_attr(entity, "leftSide"),
                        bool_attr(entity, "ceiling"),
                    ));
                }
                "touchSwitch" => {
                    lines.push(format!(
                        "    touchSwitch x={} y={} id={}",
                        int_attr(entity, "x"),
                        int_attr(entity, "y"),
                        int_attr(entity, "id"),
                    ));
                }
                _ => {}
            }
        }
        if lines.is_empty() {
            continue;
        }
        if wanted.is_empty() || wanted.iter().any(|w| *w == room) {
            println!("{room}:");
            for line in lines {
                println!("{line}");
            }
        }
    }
    if wanted.is_empty() {
        println!("totals {counts:?}");
    }
    ExitCode::SUCCESS
}
