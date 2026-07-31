import { defineMapPart } from "../map-parts.js";
import type { FixtureEntity, MapPart, Rect } from "../types.js";
import { REFORM_MAP_PARTS } from "./reform-parts.js";
import { CASSETTE_SPINNER_MAP_PARTS } from "./cassette-spinner-parts.js";
import { CORE_HEART_SQUISH_MAP_PARTS } from "./core-heart-squish-parts.js";
import { LOOKOUT_MAP_PARTS } from "./lookout-parts.js";
import { BUBS_MAP_PARTS } from "./bubs-parts.js";

const PACKAGE = "CelesteGymPlayground";
const SID = "CelesteGymPlayground/Playground";
const ROOM = "playground";

function fixturePart(
  id: string,
  options: {
    solids?: readonly Rect[];
    entities?: readonly FixtureEntity[];
  } = {},
): MapPart {
  return defineMapPart({
    id,
    dependencies: id === "playground.base" ? [] : ["playground.base"],
    package: PACKAGE,
    sid: SID,
    rooms: [
      {
        name: ROOM,
        ...(options.solids ? { solids: options.solids } : {}),
        ...(options.entities ? { entities: options.entities } : {}),
      },
    ],
  });
}

export const PLAYGROUND_BASE = defineMapPart({
  id: "playground.base",
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [
    {
      name: ROOM,
      bounds: [0, 0, 960, 544],
      spawn: [64, 496],
      solids: [
        [0, 496, 960, 48],
        [0, 0, 24, 496],
        [936, 0, 24, 496],
        [272, 304, 32, 192],
        [400, 80, 24, 200],
        [480, 240, 96, 24],
        [688, 360, 24, 136],
        [720, 304, 120, 8],
        [720, 320, 120, 16],
        [800, 248, 120, 16],
        [864, 240, 24, 256],
      ],
    },
  ],
});

export const PLAYGROUND_JUMP_THRU = fixturePart("playground.jump-thru", {
  entities: [
    {
      id: "entity-0000",
      kind: "jump_thru",
      bounds: [112, 400, 112, 8],
      name: "jumpThru",
    },
  ],
});
export const PLAYGROUND_ZIP_MOVER = defineMapPart({
  id: "playground.zip-mover",
  dependencies: ["playground.base", "playground.jump-thru"],
  package: PACKAGE,
  sid: SID,
  rooms: [
    {
      name: ROOM,
      solids: [[112, 416, 8, 80]],
      entities: [
        {
          id: "entity-0001",
          kind: "zip_mover",
          bounds: [32, 440, 64, 16],
          nodes: [[32, 320]],
          name: "zipMover",
        },
      ],
    },
  ],
});
export const PLAYGROUND_BOOSTER = fixturePart("playground.booster", {
  entities: [
    {
      id: "entity-0002",
      kind: "booster",
      bounds: [252, 384, 16, 16],
      name: "booster",
    },
    {
      id: "entity-0003",
      kind: "booster",
      bounds: [230, 384, 16, 16],
      name: "booster",
    },
    {
      id: "entity-0011",
      kind: "booster",
      bounds: [712, 312, 16, 16],
      name: "booster",
    },
    {
      id: "entity-0012",
      kind: "booster",
      bounds: [752, 432, 16, 16],
      name: "booster",
    },
    {
      id: "entity-0013",
      kind: "red_booster",
      bounds: [816, 432, 16, 16],
      name: "booster",
    },
  ],
});
export const PLAYGROUND_SPIKES = fixturePart("playground.spikes", {
  entities: [
    {
      id: "entity-0004",
      kind: "spikes",
      bounds: [328, 493, 96, 3],
      direction: [0, -1],
      name: "spikesUp",
    },
    {
      id: "entity-0005",
      kind: "spikes",
      bounds: [397, 160, 3, 32],
      direction: [-1, 0],
      name: "spikesLeft",
    },
  ],
});
export const PLAYGROUND_SPRING = fixturePart("playground.spring", {
  entities: [
    {
      id: "entity-0006",
      kind: "spring",
      bounds: [72, 490, 16, 6],
      direction: [0, -1],
      name: "spring",
    },
  ],
});
export const PLAYGROUND_BERRY = fixturePart("playground.berry", {
  entities: [
    {
      id: "entity-0007",
      kind: "strawberry",
      bounds: [153, 461, 14, 14],
      name: "strawberry",
    },
    {
      id: "entity-0008",
      kind: "strawberry",
      bounds: [153, 461, 14, 14],
      name: "strawberry",
    },
  ],
});
export const PLAYGROUND_SWIM = fixturePart("playground.swim", {
  entities: [
    {
      id: "entity-0009",
      kind: "water",
      bounds: [448, 416, 112, 80],
      name: "water",
    },
  ],
});
export const PLAYGROUND_DREAM_BLOCK = fixturePart("playground.dream-block", {
  entities: [
    {
      id: "entity-0010",
      kind: "dream_block",
      bounds: [600, 352, 64, 144],
      name: "dreamBlock",
    },
  ],
});
export const PLAYGROUND_STAR_FLY = fixturePart("playground.star-fly", {
  entities: [
    {
      id: "entity-0014",
      kind: "fly_feather",
      bounds: [110, 190, 20, 20],
      name: "infiniteStar",
    },
    {
      id: "entity-0015",
      kind: "fly_feather",
      bounds: [150, 30, 20, 20],
      name: "infiniteStar",
    },
    {
      id: "entity-0016",
      kind: "fly_feather",
      bounds: [110, 110, 20, 20],
      shielded: true,
      name: "infiniteStar",
    },
    {
      id: "entity-0017",
      kind: "fly_feather",
      bounds: [350, 390, 20, 20],
      name: "infiniteStar",
    },
    {
      id: "entity-0018",
      kind: "fly_feather",
      bounds: [110, 310, 20, 20],
      name: "infiniteStar",
    },
    {
      id: "entity-0019",
      kind: "fly_feather",
      bounds: [240, 310, 20, 20],
      name: "infiniteStar",
    },
    {
      id: "entity-0020",
      kind: "fly_feather",
      bounds: [310, 110, 20, 20],
      name: "infiniteStar",
    },
    {
      id: "entity-0021",
      kind: "fly_feather",
      bounds: [890, 474, 20, 20],
      name: "infiniteStar",
    },
  ],
});
export const PLAYGROUND_BUMPER = fixturePart("playground.bumper", {
  entities: [
    {
      id: "entity-0022",
      kind: "bumper",
      bounds: [588, 188, 24, 24],
      name: "bigSpinner",
    },
  ],
});
export const PLAYGROUND_ICE_BALL = fixturePart("playground.ice-ball", {
  entities: [
    {
      id: "entity-0023",
      kind: "ice_ball",
      bounds: [314, 154, 12, 12],
      nodes: [[336, 160]],
      singleUse: true,
      name: "fireBall",
    },
  ],
});
export const TECH_ENTITY_4_3_BUMPER_CLIP = fixturePart(
  "tech.entity-4.3-bumper-clip",
  {
    solids: [[560, 176, 16, 48]],
    entities: [
      {
        id: "tech-4.3-bumper",
        kind: "bumper",
        bounds: [588, 188, 24, 24],
        name: "bigSpinner",
      },
    ],
  },
);
export const TECH_ENTITY_4_4_EXPLOSION_BOOST = fixturePart(
  "tech.entity-4.4-explosion-boost",
  {
    entities: [
      {
        id: "tech-4.4-bumper",
        kind: "bumper",
        bounds: [588, 188, 24, 24],
        name: "bigSpinner",
      },
    ],
  },
);
export const TECH_ENTITY_4_5_BOUNCE_JUMP = fixturePart(
  "tech.entity-4.5-bounce-jump",
  {
    entities: [
      {
        id: "tech-4.5-ice-ball",
        kind: "ice_ball",
        bounds: [314, 154, 12, 12],
        nodes: [[336, 160]],
        singleUse: true,
        name: "fireBall",
      },
    ],
  },
);
export const TECH_ENTITY_4_6_CLOUD_JUMP = fixturePart(
  "tech.entity-4.6-cloud-jump",
  {
    entities: [
      {
        id: "tech-4.6-cloud",
        kind: "cloud",
        bounds: [600, 440, 32, 5],
        name: "cloud",
      },
      {
        id: "tech-4.6-spikes",
        kind: "spikes",
        bounds: [600, 493, 32, 3],
        direction: [0, -1],
        name: "spikesUp",
      },
    ],
  },
);
export const TECH_ENTITY_4_12_FEATHERBOOST = fixturePart(
  "tech.entity-4.12-featherboost",
  {
    entities: [
      {
        id: "tech-4.12-feather",
        kind: "fly_feather",
        bounds: [110, 190, 20, 20],
        name: "infiniteStar",
      },
    ],
  },
);
export const TECH_ENTITY_4_13_FEATHER_SUPER = fixturePart(
  "tech.entity-4.13-feather-super",
  {
    entities: [
      {
        id: "tech-4.13-feather",
        kind: "fly_feather",
        bounds: [890, 474, 20, 20],
        name: "infiniteStar",
      },
    ],
  },
);
export const TECH_ENTITY_4_15_1_FEATHER_CLIP = fixturePart(
  "tech.entity-4.15.1-feather-clip",
  {
    entities: [
      {
        id: "tech-4.15.1-feather",
        kind: "fly_feather",
        bounds: [150, 30, 20, 20],
        name: "infiniteStar",
      },
      {
        id: "tech-4.15.1-jump-thru",
        kind: "jump_thru",
        bounds: [152, 400, 16, 8],
        name: "jumpThru",
      },
    ],
  },
);
export const TECH_ENTITY_4_15_2_HITBOX_PRESERVATION = fixturePart(
  "tech.entity-4.15.2-hitbox-preservation",
  {
    entities: [
      {
        id: "tech-4.15.2-feather",
        kind: "fly_feather",
        bounds: [310, 110, 20, 20],
        name: "infiniteStar",
      },
      {
        id: "tech-4.15.2-ice-ball",
        kind: "ice_ball",
        bounds: [314, 154, 12, 12],
        nodes: [[336, 160]],
        singleUse: true,
        name: "fireBall",
      },
    ],
  },
);
export const TECH_ENTITY_4_6_1_CLOUD_HYPER_SUPER = fixturePart(
  "tech.entity-4.6.1-cloud-hyper-super",
  {
    entities: [
      {
        id: "tech-4.6.1-cloud",
        kind: "cloud",
        bounds: [600, 440, 32, 5],
        name: "cloud",
      },
    ],
  },
);
export const TECH_ENTITY_4_6_2_CLOUD_HYPER_BUNNYHOP = fixturePart(
  "tech.entity-4.6.2-cloud-hyper-bunnyhop",
  {
    // Keep the apex platform one grid cell past the cloud's right edge.  Starting
    // it flush with the cloud adds a horizontal wall collision on the departure
    // frame, which is not part of the cloud-apex bunnyhop.
    solids: [[544, 416, 160, 8]],
    entities: [
      {
        id: "tech-4.6.2-cloud",
        kind: "cloud",
        bounds: [504, 434, 32, 5],
        name: "cloud",
      },
    ],
  },
);
export const TECH_ENTITY_4_10_3_DREAM_SMUGGLE = fixturePart(
  "tech.entity-4.10.3-dream-smuggle",
  {
    entities: [
      {
        id: "tech-4.10.3-dream",
        kind: "dream_block",
        bounds: [231, 432, 104, 64],
        name: "dreamBlock",
      },
      {
        id: "tech-4.10.3-theo",
        kind: "theo_crystal",
        bounds: [228, 486, 8, 10],
        name: "theoCrystal",
      },
    ],
  },
);
export const TECH_ENTITY_4_10_3_1_DREAM_GRAB_HYPER = fixturePart(
  "tech.entity-4.10.3.1-dream-grab-hyper",
  {
    entities: [
      {
        id: "tech-4.10.3.1-dream",
        kind: "dream_block",
        bounds: [231, 432, 104, 64],
        name: "dreamBlock",
      },
    ],
  },
);
export const TECH_ENTITY_4_10_3_2_HOLDABLE_DREAM_HYPER = fixturePart(
  "tech.entity-4.10.3.2-holdable-dream-hyper",
  {
    entities: [
      {
        id: "tech-4.10.3.2-dream",
        kind: "dream_block",
        bounds: [231, 432, 104, 64],
        name: "dreamBlock",
      },
      {
        id: "tech-4.10.3.2-theo",
        kind: "theo_crystal",
        bounds: [228, 486, 8, 10],
        name: "theoCrystal",
      },
    ],
  },
);
export const TECH_ENTITY_4_10_4_HOLDABLE_GRABLESS_DREAM_HYPER = fixturePart(
  "tech.entity-4.10.4-holdable-grabless-dream-hyper",
  {
    entities: [
      {
        id: "tech-4.10.4-dream",
        kind: "dream_block",
        bounds: [231, 432, 104, 64],
        name: "dreamBlock",
      },
      {
        id: "tech-4.10.4-theo",
        kind: "theo_crystal",
        bounds: [228, 486, 8, 10],
        name: "theoCrystal",
      },
    ],
  },
);
export const PLAYGROUND_BADELINE = fixturePart("playground.badeline", {
  entities: [
    {
      id: "entity-0024",
      kind: "badeline_boost",
      bounds: [304, 384, 32, 32],
      nodes: [[320, 288]],
      name: "badelineBoost",
    },
    {
      id: "entity-0025",
      kind: "badeline_boost",
      bounds: [432, 384, 32, 32],
      name: "badelineBoost",
    },
  ],
});
export const PLAYGROUND_THEO = fixturePart("playground.theo", {
  entities: [
    {
      id: "entity-0026",
      kind: "theo_crystal",
      bounds: [846, 486, 8, 10],
      name: "theoCrystal",
    },
  ],
});
export const PLAYGROUND_BOUNCE_BLOCK = fixturePart("playground.bounce-block", {
  entities: [
    {
      id: "entity-0027",
      kind: "bounce_block",
      bounds: [352, 360, 64, 16],
      name: "bounceBlock",
    },
  ],
});
export const PLAYGROUND_WIND = fixturePart("playground.wind", {
  entities: [
    {
      id: "entity-0028",
      kind: "wind",
      bounds: [640, 128, 280, 120],
      direction: [400, 0],
      name: "windTrigger",
    },
  ],
});
export const PLAYGROUND_DASHLESS = fixturePart("playground.dashless", {
  solids: [
    [168, 120, 72, 64],
    [240, 112, 8, 8],
    [296, 120, 16, 8],
    [328, 120, 48, 8],
    [400, 120, 80, 64],
    [480, 104, 8, 16],
    [544, 112, 80, 8],
    [688, 120, 80, 64],
    [768, 112, 8, 8],
    [840, 120, 80, 8],
  ],
});
export const PLAYGROUND_LAUNCH = fixturePart("playground.launch");
export const PLAYGROUND_MISC = fixturePart("playground.misc");
export const PLAYGROUND_TRANSITION = defineMapPart({
  id: "playground.transition",
  dependencies: ["playground.base"],
  package: PACKAGE,
  sid: SID,
  rooms: [
    { name: "transition_0", bounds: [0, -544, 960, 544], spawn: [24, -16] },
  ],
});
export const PLAYGROUND_OTHER_HALF_STAMINA = fixturePart(
  "playground.other-half-stamina",
  {
    solids: [[624, 272, 8, 184]],
  },
);
export const PLAYGROUND_OTHER_KERMIT = defineMapPart({
  id: "playground.other-kermit",
  dependencies: ["playground.base", "playground.transition"],
  package: PACKAGE,
  sid: SID,
  rooms: [{ name: "transition_0" }],
});
export const PLAYGROUND_OTHER_SUBPIXEL = fixturePart(
  "playground.other-subpixel",
);
export const TECH_OTHER_5_13_UNDEMO_DASHING = fixturePart(
  "tech.other-5.13-undemo-dashing",
);
export const TECH_OTHER_5_8_ROBOBOOST = fixturePart(
  "tech.other-5.8-roboboost",
  {
    // The 8px-grid corner stops the ascending MoveBlock at its lower edge.  Its
    // retained upward lift makes the post-hyper ClimbJump clear the wall within
    // Player.WallSpeedRetentionTime.
    solids: [[448, 432, 8, 8]],
    entities: [
      {
        id: "tech-5.8-move-block",
        kind: "move_block",
        bounds: [400, 464, 64, 16],
        direction: [0, -1],
        name: "moveBlock",
      },
    ],
  },
);


// Refill booth: a regular diamond above the main floor. The scenario starts
// the player at (168, 496) so the first grounded frame overlaps the 16x16
// collider and Player.UseRefill restores the depleted dash and stamina.
export const PLAYGROUND_REFILL = fixturePart("playground.refill", {
  entities: [
    {
      id: "playground-refill",
      kind: "refill",
      bounds: [160, 480, 16, 16],
      direction: [0, 0],
      name: "refill",
    },
  ],
});
export const PLAYGROUND_REFILL_TWO_DASH = fixturePart(
  "playground.refill-two-dash",
  {
    entities: [
      {
        id: "playground-refill-two-dash",
        kind: "refill",
        bounds: [160, 480, 16, 16],
        direction: [1, 0],
        name: "refill",
      },
    ],
  },
);
// Falling block booth: a 32x16 climbFall block hangs over the main floor.
// The scenario spawns the player on top so the rider check starts the drop.
export const PLAYGROUND_FALLING_BLOCK = fixturePart(
  "playground.falling-block",
  {
    entities: [
      {
        id: "playground-falling-block",
        kind: "falling_block",
        bounds: [200, 400, 32, 16],
        direction: [1, 0],
        name: "fallingBlock",
      },
    ],
  },
);
export const PLAYGROUND_FALLING_BLOCK_JUMP_OFF = fixturePart(
  "playground.falling-block-jump-off",
  {
    entities: [
      {
        id: "playground-falling-block-jump-off",
        kind: "falling_block",
        bounds: [200, 400, 32, 16],
        direction: [1, 0],
        name: "fallingBlock",
      },
    ],
  },
);

export const COMMON_MAP_PARTS = new Map<string, MapPart>(
  [
    PLAYGROUND_BASE,
    PLAYGROUND_JUMP_THRU,
    PLAYGROUND_ZIP_MOVER,
    PLAYGROUND_BOOSTER,
    PLAYGROUND_SPIKES,
    PLAYGROUND_SPRING,
    PLAYGROUND_BERRY,
    PLAYGROUND_REFILL,
    PLAYGROUND_REFILL_TWO_DASH,
    PLAYGROUND_FALLING_BLOCK,
    PLAYGROUND_FALLING_BLOCK_JUMP_OFF,
    PLAYGROUND_SWIM,
    PLAYGROUND_DREAM_BLOCK,
    PLAYGROUND_STAR_FLY,
    PLAYGROUND_BUMPER,
    PLAYGROUND_ICE_BALL,
    PLAYGROUND_BADELINE,
    TECH_ENTITY_4_3_BUMPER_CLIP,
    TECH_ENTITY_4_4_EXPLOSION_BOOST,
    TECH_ENTITY_4_5_BOUNCE_JUMP,
    TECH_ENTITY_4_6_CLOUD_JUMP,
    TECH_ENTITY_4_12_FEATHERBOOST,
    TECH_ENTITY_4_13_FEATHER_SUPER,
    TECH_ENTITY_4_15_1_FEATHER_CLIP,
    TECH_ENTITY_4_15_2_HITBOX_PRESERVATION,
    TECH_ENTITY_4_6_1_CLOUD_HYPER_SUPER,
    TECH_ENTITY_4_6_2_CLOUD_HYPER_BUNNYHOP,
    TECH_ENTITY_4_10_3_DREAM_SMUGGLE,
    TECH_ENTITY_4_10_3_1_DREAM_GRAB_HYPER,
    TECH_ENTITY_4_10_3_2_HOLDABLE_DREAM_HYPER,
    TECH_ENTITY_4_10_4_HOLDABLE_GRABLESS_DREAM_HYPER,
    PLAYGROUND_THEO,
    PLAYGROUND_BOUNCE_BLOCK,
    PLAYGROUND_WIND,
    PLAYGROUND_DASHLESS,
    PLAYGROUND_LAUNCH,
    PLAYGROUND_MISC,
    PLAYGROUND_TRANSITION,
    PLAYGROUND_OTHER_HALF_STAMINA,
    PLAYGROUND_OTHER_KERMIT,
    PLAYGROUND_OTHER_SUBPIXEL,
    TECH_OTHER_5_13_UNDEMO_DASHING,
    TECH_OTHER_5_8_ROBOBOOST,
    ...REFORM_MAP_PARTS,
    ...CASSETTE_SPINNER_MAP_PARTS,
    ...CORE_HEART_SQUISH_MAP_PARTS,
    ...LOOKOUT_MAP_PARTS,
    ...BUBS_MAP_PARTS,
  ].map((part) => [part.id, part]),
);
