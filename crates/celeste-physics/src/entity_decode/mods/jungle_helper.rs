use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    match name {
        "JungleHelper/InvisibleJumpthruPlatform" => Some(Registration::jump_thru()),
        "JungleHelper/Firefly" => Some(Registration::decoration()),
        _ => None,
    }
}
