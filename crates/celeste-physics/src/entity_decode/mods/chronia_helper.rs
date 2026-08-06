use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "ChroniaHelper/CustomTorch").then(Registration::decoration)
}
