use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "CrossoverCollab/CompassController").then(Registration::decoration)
}
