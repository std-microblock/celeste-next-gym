use std::{
    collections::BTreeMap,
    env, fs,
    path::Path,
    process::ExitCode,
};

use celeste_physics::{BinaryElement, BinaryValue, encode_celeste_bin, parse_celeste_bin};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Catalog {
    projects: Vec<Project>,
}

#[derive(Deserialize)]
struct Project {
    id: String,
    room: String,
    training: Training,
}

#[derive(Deserialize)]
struct Training {
    modules: Vec<TrainingModule>,
    finish: Finish,
}

#[derive(Deserialize)]
struct TrainingModule {
    id: String,
    trigger: Trigger,
    end_trigger: Trigger,
}

#[derive(Deserialize)]
struct Finish {
    trigger: Trigger,
    require_all_modules: bool,
}

#[derive(Deserialize)]
struct Trigger {
    bounds: Bounds,
}

#[derive(Clone, Copy, Deserialize)]
struct Bounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

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
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("usage: inject_training_triggers <input.bin> <catalog.json> <output.bin>".into());
    }
    let input = Path::new(&args[0]);
    let catalog_path = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    let mut root = parse_celeste_bin(
        &fs::read(input).map_err(|error| format!("failed to read {}: {error}", input.display()))?,
    )
    .map_err(|error| format!("failed to decode {}: {error}", input.display()))?;
    let catalog: Catalog = serde_json::from_slice(
        &fs::read(catalog_path)
            .map_err(|error| format!("failed to read {}: {error}", catalog_path.display()))?,
    )
    .map_err(|error| format!("invalid training catalog: {error}"))?;

    let levels = root
        .children
        .iter_mut()
        .find(|child| child.name == "levels")
        .ok_or_else(|| "map has no levels container".to_owned())?;
    let mut injected = 0usize;
    for project in catalog.projects {
        let level_name = format!("lvl_{}", project.room);
        let level = levels
            .children
            .iter_mut()
            .find(|level| string_attr(level, "name") == Some(level_name.as_str()))
            .ok_or_else(|| format!("catalog room {} is absent from the map", project.room))?;
        let level_x = int_attr(level, "x").unwrap_or(0);
        let level_y = int_attr(level, "y").unwrap_or(0);
        let triggers = level
            .children
            .iter_mut()
            .find(|child| child.name == "triggers")
            .ok_or_else(|| format!("room {} has no triggers container", project.room))?;

        for module in project.training.modules {
            let mut attributes = trigger_attributes(module.trigger.bounds, level_x, level_y, injected as i32);
            attributes.insert("lessonId".into(), BinaryValue::String(module.id.clone()));
            attributes.insert("projectId".into(), BinaryValue::String(project.id.clone()));
            triggers.children.push(BinaryElement {
                package: None,
                name: "CelesteGymTraining/lessonTrigger".into(),
                attributes,
                children: vec![],
            });
            injected += 1;

            let mut end_attributes = trigger_attributes(module.end_trigger.bounds, level_x, level_y, injected as i32);
            end_attributes.insert("lessonId".into(), BinaryValue::String(module.id));
            end_attributes.insert("projectId".into(), BinaryValue::String(project.id.clone()));
            triggers.children.push(BinaryElement {
                package: None,
                name: "CelesteGymTraining/lessonEndTrigger".into(),
                attributes: end_attributes,
                children: vec![],
            });
            injected += 1;
        }

        let mut attributes = trigger_attributes(
            project.training.finish.trigger.bounds,
            level_x,
            level_y,
            injected as i32,
        );
        attributes.insert("projectId".into(), BinaryValue::String(project.id));
        attributes.insert(
            "requireAllModules".into(),
            BinaryValue::Bool(project.training.finish.require_all_modules),
        );
        triggers.children.push(BinaryElement {
            package: None,
            name: "CelesteGymTraining/finishTrigger".into(),
            attributes,
            children: vec![],
        });
        injected += 1;
    }

    let bytes = encode_celeste_bin(&root).map_err(|error| error.to_string())?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(output, bytes)
        .map_err(|error| format!("failed to write {}: {error}", output.display()))?;
    Ok(format!("injected {injected} training triggers into {}", output.display()))
}

fn trigger_attributes(bounds: Bounds, level_x: i32, level_y: i32, id: i32) -> BTreeMap<String, BinaryValue> {
    BTreeMap::from([
        ("height".into(), BinaryValue::Int(bounds.height.round() as i32)),
        ("id".into(), BinaryValue::Int(10_000 + id)),
        ("width".into(), BinaryValue::Int(bounds.width.round() as i32)),
        ("x".into(), BinaryValue::Int(bounds.x.round() as i32 - level_x)),
        ("y".into(), BinaryValue::Int(bounds.y.round() as i32 - level_y)),
    ])
}

fn string_attr<'a>(element: &'a BinaryElement, key: &str) -> Option<&'a str> {
    match element.attributes.get(key) {
        Some(BinaryValue::String(value)) => Some(value),
        _ => None,
    }
}

fn int_attr(element: &BinaryElement, key: &str) -> Option<i32> {
    match element.attributes.get(key) {
        Some(BinaryValue::Int(value)) => Some(*value),
        Some(BinaryValue::Short(value)) => Some(i32::from(*value)),
        Some(BinaryValue::Byte(value)) => Some(i32::from(*value)),
        _ => None,
    }
}
