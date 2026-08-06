use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "FlaglinesAndSuch/BonfireLight").then(Registration::decoration)
}
