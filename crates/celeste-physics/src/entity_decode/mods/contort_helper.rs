use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "ContortHelper/LightSource"
            | "ContortHelper/AlphaLerpLightSource"
            | "ContortHelper/LightSourceZone"
    )
    .then(Registration::decoration)
}
