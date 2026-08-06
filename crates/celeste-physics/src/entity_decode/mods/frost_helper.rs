use super::Registration;
use crate::{
    BinaryElement,
    entity_decode::{Placement, attr_bool, attr_f32, attr_text},
};

pub(super) fn lookup(name: &str) -> Option<Registration> {
    match name {
        // These are reskinned vanilla springs; their gameplay component and
        // collider geometry are the vanilla floor/wall spring geometry.
        "FrostHelper/SpringFloor" => Some(Registration::spring(Placement::SpringFloor)),
        "FrostHelper/SpringLeft" => Some(Registration::spring(Placement::SpringLeft)),
        "FrostHelper/SpringRight" => Some(Registration::spring(Placement::SpringRight)),
        "FrostHelper/CustomDreamBlock" => Some(Registration::dream_block()),
        "FrostHelper/ArbitraryLight"
        | "FrostHelper/BloomPoint"
        | "FrostHelper/DustSprite"
        | "FrostHelper/WireLamps"
        | "FrostHelper/RainbowTilesetController"
        | "FrostHelper/EntityRainbowifyController"
        | "FrostHelper/ColoredLightbeam"
        | "FrostHelper/ColoredHangingLamp" => Some(Registration::decoration()),
        _ => None,
    }
}

pub(super) fn compatible(entity: &BinaryElement) -> bool {
    match entity.name.as_str() {
        "FrostHelper/SpringFloor" | "FrostHelper/SpringLeft" | "FrostHelper/SpringRight" => {
            attr_bool(entity, "playerCanUse", true)
                && !attr_bool(entity, "oneUse", false)
                && attr_f32(entity, "dashRecovery", 10_000.0) == 10_000.0
                && attr_f32(entity, "staminaRecovery", 10_000.0) == 10_000.0
                && attr_f32(entity, "jumpRecovery", 10_001.0) == 10_001.0
                && attr_f32(entity, "attachGroup", -1.0) == -1.0
                && attr_text(entity, "recovery").is_none_or(|value| value == "10000;10000;10001")
                && attr_text(entity, "speedMult").map_or_else(
                    || attr_f32(entity, "speedMult", 1.0) == 1.0,
                    |value| matches!(value.trim(), "1" | "1.0" | "1,1" | "1.0,1.0"),
                )
        }
        "FrostHelper/CustomDreamBlock" => {
            entity.children.iter().all(|child| child.name != "node")
                && attr_f32(entity, "speed", 240.0) == 240.0
                && attr_f32(entity, "sameDirectionSpeedMultiplier", 2.0) == 2.0
                && attr_f32(entity, "moveSpeedMult", 1.0) == 1.0
                && !attr_bool(entity, "oneUse", false)
                && !attr_bool(entity, "conserveSpeed", false)
                && !attr_bool(entity, "allowRedirects", false)
                && !attr_bool(entity, "allowSameDirectionDash", false)
                && !attr_bool(entity, "connected", false)
        }
        _ => true,
    }
}
