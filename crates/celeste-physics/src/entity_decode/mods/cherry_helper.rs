use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "CherryHelper/AssistRect").then(Registration::decoration)
}
