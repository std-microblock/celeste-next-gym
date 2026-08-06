use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "SpringCollab2020/invisibleLightSource" | "SpringCollab2020/RainbowSpinnerColorController"
    )
    .then(Registration::decoration)
}
