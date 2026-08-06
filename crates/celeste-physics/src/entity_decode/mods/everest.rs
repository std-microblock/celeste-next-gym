use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "everest/starClimbGraphicsController").then(Registration::decoration)
}
