use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "HonlyHelper/Moth" | "HonlyHelper/FireFly" | "HonlyHelper/FlagSoundSource"
    )
    .then(Registration::decoration)
}
