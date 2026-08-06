mod adams_addons;
mod chronia_helper;
mod colored_lights;
mod communal_helper;
mod contort_helper;
mod crossover_collab;
mod dbb_helper;
mod everest;
mod femto_helper;
mod flaglines_and_such;
mod frost_helper;
mod game_helper;
mod honly_helper;
mod jungle_helper;
mod kyfex_helper;
mod lunatic_helper;
mod max_helping_hand;
mod ozmas_oddities;
mod pandoras_box;
mod sj2021;
mod sorbet_helper;
mod spring_collab_2020;
mod style_mask_helper;
mod viv_helper;
mod xaphan_helper;

use super::Registration;
use crate::BinaryElement;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    game_helper::lookup(name)
        .or_else(|| adams_addons::lookup(name))
        .or_else(|| chronia_helper::lookup(name))
        .or_else(|| colored_lights::lookup(name))
        .or_else(|| communal_helper::lookup(name))
        .or_else(|| max_helping_hand::lookup(name))
        .or_else(|| femto_helper::lookup(name))
        .or_else(|| contort_helper::lookup(name))
        .or_else(|| crossover_collab::lookup(name))
        .or_else(|| everest::lookup(name))
        .or_else(|| flaglines_and_such::lookup(name))
        .or_else(|| lunatic_helper::lookup(name))
        .or_else(|| spring_collab_2020::lookup(name))
        .or_else(|| sj2021::lookup(name))
        .or_else(|| sorbet_helper::lookup(name))
        .or_else(|| style_mask_helper::lookup(name))
        .or_else(|| frost_helper::lookup(name))
        .or_else(|| viv_helper::lookup(name))
        .or_else(|| honly_helper::lookup(name))
        .or_else(|| jungle_helper::lookup(name))
        .or_else(|| dbb_helper::lookup(name))
        .or_else(|| kyfex_helper::lookup(name))
        .or_else(|| ozmas_oddities::lookup(name))
        .or_else(|| xaphan_helper::lookup(name))
        .or_else(|| pandoras_box::lookup(name))
}

pub(super) fn compatible(entity: &BinaryElement) -> bool {
    frost_helper::compatible(entity)
        && max_helping_hand::compatible(entity)
        && viv_helper::compatible(entity)
}
