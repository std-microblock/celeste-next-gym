use super::Registration;
use crate::{
    BinaryElement,
    entity_decode::{Placement, attr_bool},
};

pub(super) fn lookup(name: &str) -> Option<Registration> {
    match name {
        // Rainbow spikes only replace rendering/color selection. Preserve the
        // vanilla three-pixel directional hazard collider.
        "VivHelper/RainbowSpikesUp" => Some(Registration::spike(Placement::SpikeUp)),
        "VivHelper/RainbowSpikesDown" => Some(Registration::spike(Placement::SpikeDown)),
        "VivHelper/RainbowSpikesLeft" => Some(Registration::spike(Placement::SpikeLeft)),
        "VivHelper/RainbowSpikesRight" => Some(Registration::spike(Placement::SpikeRight)),
        "VivHelper/CustomHangingLamp"
        | "VivHelper/CustomLightbeam"
        | "VivHelper/HideRoomInMap"
        | "VivHelper/DebrisLimiter"
        | "VivHelper/CustomTorch" => Some(Registration::decoration()),
        _ => None,
    }
}

pub(super) fn compatible(entity: &BinaryElement) -> bool {
    !entity.name.starts_with("VivHelper/RainbowSpikes") || !attr_bool(entity, "groundRefill", false)
}
