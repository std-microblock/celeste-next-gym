use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    match name {
        // ColoredWater subclasses the vanilla Water interaction and changes
        // rendering colors only. Waterfalls are presentation particles.
        "pandorasBox/coloredWater" => Some(Registration::water()),
        "pandorasBox/coloredWaterfall"
        | "pandorasBox/coloredBigWaterfall"
        | "pandorasBox/lamp"
        | "pandorasBox/dustSpriteColorController" => Some(Registration::decoration()),
        _ => None,
    }
}
