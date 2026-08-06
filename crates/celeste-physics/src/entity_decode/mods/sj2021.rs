use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "SJ2021/HintController"
            | "SJ2021/GlowController"
            | "SJ2021/CassetteMusicTransitionController"
    )
    .then(Registration::decoration)
}
