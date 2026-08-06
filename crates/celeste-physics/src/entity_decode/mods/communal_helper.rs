use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "CommunalHelper/HintController"
            | "CommunalHelper/GlowController"
            | "CommunalHelper/UnderwaterMusicController"
    )
    .then(Registration::decoration)
}
