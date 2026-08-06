use super::Registration;

pub(crate) fn lookup(name: &str) -> Option<Registration> {
    // These vanilla classes are visual/audio entities and do not add a Solid,
    // PlayerCollider, HoldableCollider, or hazard collider. FakeWall/FakeBlock
    // are reveal sensors implemented as Entity (not Solid), so they likewise
    // do not alter player movement in the physics-only simulator.
    matches!(
        name,
        "floatingDebris"
            | "foregroundDebris"
            | "wire"
            | "torch"
            | "hanginglamp"
            | "cobweb"
            | "lightbeam"
            | "soundSource"
            | "moonCreature"
            | "flutterbird"
            | "waterfall"
            | "playbackBillboard"
            | "cliffside_flag"
            | "cliffflag"
            | "SummitBackgroundManager"
            | "fakeWall"
            | "fakeBlock"
    )
    .then(Registration::decoration)
}
