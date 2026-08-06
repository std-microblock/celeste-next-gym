use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "coloredlights/hanginglamp").then(Registration::decoration)
}
