use super::Registration;

pub(super) fn lookup(name: &str) -> Option<Registration> {
    (name == "LunaticHelper/InvisibleLightSource").then(Registration::decoration)
}
