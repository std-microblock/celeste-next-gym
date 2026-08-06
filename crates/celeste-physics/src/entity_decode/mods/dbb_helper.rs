use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "DBBHelper/GodLight2D"
            | "DBBHelper/AlignedText"
            | "DBBHelper/FogEffect"
            | "DBBHelper/ScanLineJitterGlitchEffect"
    )
    .then(Registration::decoration)
}
