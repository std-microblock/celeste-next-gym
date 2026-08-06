use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "SorbetHelper/CustomLightbeam" | "SorbetHelper/BigWaterfall"
    )
    .then(Registration::decoration)
}
