use crate::{Entity, EntityKind, Map, Rect, Vec2};

pub const PLAYGROUND_PACKAGE: &str = "CelesteGymPlayground";
pub const PLAYGROUND_SID: &str = "CelesteGymPlayground/Playground";
pub const PLAYGROUND_ROOM: &str = "playground";

pub fn mechanics_playground() -> Map {
    Map {
        bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
        spawn: Vec2::new(64.0, 496.0),
        solids: vec![
            Rect::new(0.0, 496.0, 960.0, 48.0),
            Rect::new(0.0, 0.0, 24.0, 496.0),
            Rect::new(936.0, 0.0, 24.0, 496.0),
            // Isolated wall for Delayed Blockboost. Its eight-pixel gap from
            // the ZipMover keeps the later wall jump clear of the platform's
            // own side collider.
            Rect::new(112.0, 416.0, 8.0, 80.0),
            Rect::new(272.0, 304.0, 32.0, 192.0),
            Rect::new(688.0, 360.0, 24.0, 136.0),
            Rect::new(864.0, 240.0, 24.0, 256.0),
            Rect::new(480.0, 240.0, 96.0, 24.0),
            Rect::new(800.0, 248.0, 120.0, 16.0),
            Rect::new(400.0, 80.0, 24.0, 200.0),
        ],
        entities: vec![
            Entity {
                kind: EntityKind::JumpThru,
                bounds: Rect::new(112.0, 400.0, 112.0, 8.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "jumpThru".to_owned(),
            },
            // Vanilla ZipMover booth for Liftboost. A player whose bottom is
            // y=440 starts the source 0.1-second delay, then rides it upward.
            Entity {
                kind: EntityKind::ZipMover,
                bounds: Rect::new(32.0, 440.0, 64.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![Vec2::new(32.0, 320.0)],
                name: "zipMover".to_owned(),
            },
            // Adjacent jumpthrough boosters for coyote-time bubble
            // super/hyper setups.
            Entity {
                kind: EntityKind::Booster,
                bounds: Rect::new(252.0, 384.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "booster".to_owned(),
            },
            Entity {
                kind: EntityKind::Booster,
                bounds: Rect::new(230.0, 384.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "booster".to_owned(),
            },
            Entity {
                kind: EntityKind::Spikes,
                bounds: Rect::new(328.0, 493.0, 96.0, 3.0),
                direction: Vec2::new(0.0, -1.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "spikesUp".to_owned(),
            },
            Entity {
                kind: EntityKind::Water,
                bounds: Rect::new(448.0, 416.0, 112.0, 80.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "water".to_owned(),
            },
            Entity {
                kind: EntityKind::DreamBlock,
                bounds: Rect::new(600.0, 352.0, 64.0, 144.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dreamBlock".to_owned(),
            },
            // Isolated Archie booth. The horizontal demo reaches this booster
            // several live frames after StartDash without touching a floor,
            // wall, or either coyote booster.
            Entity {
                kind: EntityKind::Booster,
                bounds: Rect::new(712.0, 312.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "booster".to_owned(),
            },
            Entity {
                kind: EntityKind::Booster,
                bounds: Rect::new(752.0, 432.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "booster".to_owned(),
            },
            Entity {
                kind: EntityKind::RedBooster,
                bounds: Rect::new(816.0, 432.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "booster".to_owned(),
            },
            Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(110.0, 190.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            },
            // High isolated feather aligned with the long jumpthrough below;
            // a downward flight reaches it close to StarFly expiry for the
            // Feather Clip setup without drifting into the booster booths.
            Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(150.0, 30.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            },
            Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(110.0, 110.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: true,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            },
            Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(350.0, 390.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            },
            Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(110.0, 310.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            },
            Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(240.0, 310.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            },
            // Feather aligned above the stationary Ice Ball for the
            // Player.Bounce StarFly-collider preservation setup.
            Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(310.0, 110.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            },
            // Grounded feather booth for Feather Super without injecting a
            // StarFly state into the initial snapshot.
            Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(890.0, 474.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            },
            Entity {
                kind: EntityKind::Bumper,
                bounds: Rect::new(588.0, 188.0, 24.0, 24.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "bigSpinner".to_owned(),
            },
            // Stationary cold Core fireball. The zero path speed keeps its
            // top-bounce position deterministic across fresh E2E room loads.
            Entity {
                kind: EntityKind::IceBall,
                bounds: Rect::new(314.0, 154.0, 12.0, 12.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: true,
                nodes: vec![Vec2::new(336.0, 160.0)],
                name: "fireBall".to_owned(),
            },
            Entity {
                kind: EntityKind::BadelineBoost,
                bounds: Rect::new(304.0, 384.0, 32.0, 32.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![Vec2::new(320.0, 288.0)],
                name: "badelineBoost".to_owned(),
            },
            Entity {
                kind: EntityKind::BadelineBoost,
                bounds: Rect::new(432.0, 384.0, 32.0, 32.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "badelineBoost".to_owned(),
            },
            Entity {
                kind: EntityKind::Wind,
                bounds: Rect::new(640.0, 128.0, 280.0, 120.0),
                direction: Vec2::new(400.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "windTrigger".to_owned(),
            },
        ],
        source_package: Some(PLAYGROUND_PACKAGE.to_owned()),
    }
}
