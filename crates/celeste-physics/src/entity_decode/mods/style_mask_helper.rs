use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    matches!(
        name,
        "StyleMaskHelper/StylegroundMask"
            | "StyleMaskHelper/AllInOneMask"
            | "SJ2021/StylegroundMask"
            | "StyleMaskHelper/LightingMask"
    )
    .then(Registration::decoration)
}
