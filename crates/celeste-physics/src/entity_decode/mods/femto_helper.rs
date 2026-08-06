use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "FemtoHelper/ParticleEmitter"
            | "FemtoHelper/CustomMoonCreature"
            | "FemtoHelper/CustomParallaxBigWaterfall"
    )
    .then(Registration::decoration)
}
