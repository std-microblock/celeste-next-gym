use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "OzmasOddities/IdleSoundController").then(Registration::decoration)
}
