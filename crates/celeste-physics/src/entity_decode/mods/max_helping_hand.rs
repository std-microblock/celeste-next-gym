use super::Registration;
use crate::{BinaryElement, entity_decode::attr_f32};

pub(super) fn lookup(name: &str) -> Option<Registration> {
    match name {
        "MaxHelpingHand/CustomizableRefill" => Some(Registration::refill()),
        "MaxHelpingHand/Comment"
        | "MaxHelpingHand/FlagDecal"
        | "MaxHelpingHand/FlagDecalXML"
        | "MaxHelpingHand/ReskinnableFloatingDebris"
        | "MaxHelpingHand/StylegroundFadeController"
        | "MaxHelpingHand/RainbowSpinnerColorController"
        | "MaxHelpingHand/RainbowSpinnerColorControllerDisabler"
        | "MaxHelpingHand/FlagRainbowSpinnerColorController"
        | "MaxHelpingHand/ParallaxFadeOutController"
        | "MaxHelpingHand/ParallaxFadeSpeedController" => Some(Registration::decoration()),
        _ => None,
    }
}

pub(super) fn compatible(entity: &BinaryElement) -> bool {
    entity.name != "MaxHelpingHand/CustomizableRefill"
        || attr_f32(entity, "respawnTime", 2.5) == 2.5
}
