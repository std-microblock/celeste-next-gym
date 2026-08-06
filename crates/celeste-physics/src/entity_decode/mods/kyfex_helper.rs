use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "KyfexHelper/MapperNote").then(Registration::decoration)
}
