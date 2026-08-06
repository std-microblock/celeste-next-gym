use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "AdamsAddons/ParticleColorController" | "AdamsAddons/FlagChangeDecal"
    )
    .then(Registration::decoration)
}
