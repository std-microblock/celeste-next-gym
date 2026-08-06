use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "GameHelper/DebugDecalController"
            | "GameHelper/TileDebugDecalConverter"
            | "GameHelper/ColorfulDebugController"
    )
    .then(Registration::decoration)
}
