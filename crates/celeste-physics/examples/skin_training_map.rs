use std::{collections::BTreeMap, env, fs, process::ExitCode};

use celeste_physics::{BinaryElement, BinaryValue, encode_celeste_bin, parse_celeste_bin};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        eprintln!("usage: skin_training_map <input.bin> <output.bin>");
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("usage: skin_training_map <input.bin> <output.bin>");
        return ExitCode::from(2);
    }
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut root = parse_celeste_bin(&fs::read(&input)?)?;
        let start_level = apply_beginner_gym_tiles(&mut root)?;
        root.children.retain(|child| child.name != "meta");
        root.children.push(metadata(start_level));
        let bytes = encode_celeste_bin(&root)?;
        if let Some(parent) = std::path::Path::new(&output).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output, bytes)?;
        Ok(())
    })();
    match result {
        Ok(()) => {
            println!("applied bundled Strawberry Jam Beginner Gym skin to {output:?}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("failed to skin training map: {error}");
            ExitCode::FAILURE
        }
    }
}

fn apply_beginner_gym_tiles(root: &mut BinaryElement) -> Result<String, &'static str> {
    let levels = root
        .children
        .iter_mut()
        .find(|child| child.name == "levels")
        .ok_or("map has no levels element")?;
    let mut start_level = None;
    for level in &mut levels.children {
        if start_level.is_none()
            && let Some(BinaryValue::String(name)) = level.attributes.get("name")
        {
            start_level = Some(name.strip_prefix("lvl_").unwrap_or(name).to_owned());
        }
        let solids = level
            .children
            .iter_mut()
            .find(|child| child.name == "solids")
            .ok_or("level has no solids layer")?;
        let Some(BinaryValue::String(grid)) = solids.attributes.get_mut("innerText") else {
            return Err("solids layer has no string grid");
        };
        *grid = grid.replace('1', "Y");
    }
    start_level.ok_or("map has no level")
}

fn metadata(start_level: String) -> BinaryElement {
    BinaryElement {
        package: None,
        name: "meta".to_owned(),
        attributes: BTreeMap::from([
            (
                "ForegroundTiles".to_owned(),
                BinaryValue::String("Graphics/CelesteGymTraining/ForegroundTiles.xml".to_owned()),
            ),
            ("BloomBase".to_owned(), BinaryValue::Float(0.0)),
            ("BloomStrength".to_owned(), BinaryValue::Float(1.0)),
            ("DarknessAlpha".to_owned(), BinaryValue::Float(0.05)),
            ("Dreaming".to_owned(), BinaryValue::Bool(false)),
            ("Interlude".to_owned(), BinaryValue::Bool(false)),
            ("IntroType".to_owned(), BinaryValue::String("None".to_owned())),
            ("OverrideASideMeta".to_owned(), BinaryValue::Bool(true)),
            ("TitleAccentColor".to_owned(), BinaryValue::String("7eb2dd".to_owned())),
            ("TitleBaseColor".to_owned(), BinaryValue::String("1a2438".to_owned())),
            ("TitleTextColor".to_owned(), BinaryValue::String("ffffff".to_owned())),
        ]),
        children: vec![BinaryElement {
            package: None,
            name: "mode".to_owned(),
            attributes: BTreeMap::from([
                ("HeartIsEnd".to_owned(), BinaryValue::Bool(false)),
                ("Inventory".to_owned(), BinaryValue::String("Default".to_owned())),
                ("SeekerSlowdown".to_owned(), BinaryValue::Bool(false)),
                ("StartLevel".to_owned(), BinaryValue::String(start_level)),
                ("TheoInBubble".to_owned(), BinaryValue::Bool(false)),
            ]),
            children: vec![],
        }],
    }
}
