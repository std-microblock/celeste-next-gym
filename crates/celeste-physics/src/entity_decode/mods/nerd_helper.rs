use super::Registration;
use crate::{
    BinaryElement,
    entity_decode::{Placement, attr_bool},
};

pub(super) fn lookup(name: &str) -> Option<Registration> {
    let placement = match name {
        "NerdHelper/DashThroughSpikesUp" => Placement::SpikeUp,
        "NerdHelper/DashThroughSpikesDown" => Placement::SpikeDown,
        "NerdHelper/DashThroughSpikesLeft" => Placement::SpikeLeft,
        "NerdHelper/DashThroughSpikesRight" => Placement::SpikeRight,
        _ => return None,
    };
    Some(Registration::dash_through_spike(placement))
}

pub(super) fn compatible(entity: &BinaryElement) -> bool {
    !entity.name.starts_with("NerdHelper/DashThroughSpikes")
        || (attr_bool(entity, "red_boosters_count_as_dash", true)
            && !attr_bool(entity, "invert", false)
            && attr_bool(entity, "along", true)
            && attr_bool(entity, "into", true)
            && attr_bool(entity, "diag", true)
            && !attr_bool(entity, "zero_speed_only", false))
}
