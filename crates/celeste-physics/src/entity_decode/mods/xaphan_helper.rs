use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "XaphanHelper/InGameMapTilesController"
            | "XaphanHelper/InGameMapRoomController"
            | "XaphanHelper/CustomTorch"
    )
    .then(Registration::decoration)
}
