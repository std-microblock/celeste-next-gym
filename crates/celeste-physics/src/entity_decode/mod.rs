//! Small, data-only entity compatibility registry.
//!
//! The simulator keeps stateful mechanics in `sim.rs`. This registry is for
//! entities that can faithfully reuse an existing physics primitive, plus
//! presentation-only entities whose source has no gameplay collider. Mod
//! registrations live in one file per mod so compatibility does not turn into
//! one unreviewable name table.

mod mods;
mod vanilla;

use crate::{BinaryElement, BinaryValue, EntityKind, Rect, Vec2};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Placement {
    TopLeft,
    SpikeUp,
    SpikeDown,
    SpikeLeft,
    SpikeRight,
    SpringFloor,
    SpringLeft,
    SpringRight,
    Refill,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Registration {
    pub kind: EntityKind,
    pub placement: Placement,
    pub default_width: f32,
    pub default_height: f32,
    pub shielded: bool,
}

impl Registration {
    pub(crate) const fn decoration() -> Self {
        Self::new(EntityKind::Decoration, Placement::TopLeft, 8.0, 8.0)
    }

    pub(crate) const fn water() -> Self {
        Self::new(EntityKind::Water, Placement::TopLeft, 8.0, 8.0)
    }

    pub(crate) const fn jump_thru() -> Self {
        Self::new(EntityKind::JumpThru, Placement::TopLeft, 8.0, 8.0)
    }

    pub(crate) const fn dream_block() -> Self {
        Self::new(EntityKind::DreamBlock, Placement::TopLeft, 16.0, 16.0)
    }

    pub(crate) const fn refill() -> Self {
        Self::new(EntityKind::Refill, Placement::Refill, 16.0, 16.0)
    }

    pub(crate) const fn spike(placement: Placement) -> Self {
        Self::new(EntityKind::Spikes, placement, 8.0, 8.0)
    }

    pub(crate) const fn dash_through_spike(placement: Placement) -> Self {
        Self {
            shielded: true,
            ..Self::spike(placement)
        }
    }

    pub(crate) const fn spring(placement: Placement) -> Self {
        let (width, height) = match placement {
            Placement::SpringFloor => (16.0, 6.0),
            Placement::SpringLeft | Placement::SpringRight => (6.0, 16.0),
            _ => (8.0, 8.0),
        };
        Self::new(EntityKind::Spring, placement, width, height)
    }

    const fn new(
        kind: EntityKind,
        placement: Placement,
        default_width: f32,
        default_height: f32,
    ) -> Self {
        Self {
            kind,
            placement,
            default_width,
            default_height,
            shielded: false,
        }
    }

    pub(crate) fn bounds_and_direction(
        self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        two_dash: bool,
    ) -> (Rect, Vec2) {
        match self.placement {
            Placement::TopLeft => (Rect::new(x, y, width, height), Vec2::default()),
            Placement::SpikeUp => (Rect::new(x, y - 3.0, width, 3.0), Vec2::new(0.0, -1.0)),
            Placement::SpikeDown => (Rect::new(x, y, width, 3.0), Vec2::new(0.0, 1.0)),
            Placement::SpikeLeft => (Rect::new(x - 3.0, y, 3.0, height), Vec2::new(-1.0, 0.0)),
            Placement::SpikeRight => (Rect::new(x, y, 3.0, height), Vec2::new(1.0, 0.0)),
            Placement::SpringFloor => {
                (Rect::new(x - 8.0, y - 6.0, 16.0, 6.0), Vec2::new(0.0, -1.0))
            }
            Placement::SpringLeft => (Rect::new(x, y - 8.0, 6.0, 16.0), Vec2::new(1.0, 0.0)),
            Placement::SpringRight => {
                (Rect::new(x - 6.0, y - 8.0, 6.0, 16.0), Vec2::new(-1.0, 0.0))
            }
            Placement::Refill => (
                Rect::new(x - 8.0, y - 8.0, 16.0, 16.0),
                Vec2::new(if two_dash { 1.0 } else { 0.0 }, 0.0),
            ),
        }
    }
}

pub(crate) fn lookup(entity: &BinaryElement) -> Option<Registration> {
    vanilla::lookup(&entity.name)
        .or_else(|| mods::lookup(&entity.name))
        .filter(|_| mods::compatible(entity))
}

pub(crate) fn additional_solids(entity: &BinaryElement, x: f32, y: f32) -> Vec<Rect> {
    mods::additional_solids(entity, x, y)
}

pub(super) fn attr_f32(entity: &BinaryElement, key: &str, default: f32) -> f32 {
    match entity.attributes.get(key) {
        Some(BinaryValue::Byte(value)) => *value as f32,
        Some(BinaryValue::Short(value)) => *value as f32,
        Some(BinaryValue::Int(value)) => *value as f32,
        Some(BinaryValue::Float(value)) => *value,
        Some(BinaryValue::String(value)) => value.parse().unwrap_or(default),
        _ => default,
    }
}

pub(super) fn attr_bool(entity: &BinaryElement, key: &str, default: bool) -> bool {
    match entity.attributes.get(key) {
        Some(BinaryValue::Bool(value)) => *value,
        Some(BinaryValue::Byte(value)) => *value != 0,
        _ => default,
    }
}

pub(super) fn attr_text<'a>(entity: &'a BinaryElement, key: &str) -> Option<&'a str> {
    match entity.attributes.get(key) {
        Some(BinaryValue::String(value)) => Some(value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(name: &str) -> BinaryElement {
        BinaryElement {
            package: None,
            name: name.to_owned(),
            attributes: Default::default(),
            children: vec![],
        }
    }

    #[test]
    fn registrations_keep_mod_physics_aliases_exact() {
        let water = lookup(&raw("pandorasBox/coloredWater")).unwrap();
        assert_eq!(water.kind, EntityKind::Water);
        assert_eq!(water.placement, Placement::TopLeft);

        let spring = lookup(&raw("FrostHelper/SpringRight")).unwrap();
        assert_eq!(spring.kind, EntityKind::Spring);
        assert_eq!(spring.placement, Placement::SpringRight);

        let spikes = lookup(&raw("VivHelper/RainbowSpikesUp")).unwrap();
        assert_eq!(spikes.kind, EntityKind::Spikes);
        assert_eq!(spikes.placement, Placement::SpikeUp);

        let dash_spikes = lookup(&raw("NerdHelper/DashThroughSpikesLeft")).unwrap();
        assert_eq!(dash_spikes.kind, EntityKind::Spikes);
        assert!(dash_spikes.shielded);
    }

    #[test]
    fn registrations_do_not_claim_unknown_gameplay_entities() {
        assert_eq!(lookup(&raw("MaxHelpingHand/SidewaysJumpThru")), None);
        assert_eq!(lookup(&raw("seekerBarrier")), None);

        let mut redirected = raw("FrostHelper/CustomDreamBlock");
        redirected
            .attributes
            .insert("allowRedirects".to_owned(), BinaryValue::Bool(true));
        assert_eq!(lookup(&redirected), None);
    }
}
