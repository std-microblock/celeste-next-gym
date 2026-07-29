import type { ScenarioDefinition } from "../../types.js";
import { scenario as scenario0 } from "./playground-load.js";
import { scenario as scenario1 } from "./mechanics-corner-correction-up.js";
import { scenario as scenario2 } from "./mechanics-corner-correction-horizontal.js";
import { scenario as scenario3 } from "./mechanics-directional-spikes-away.js";
import { scenario as scenario4 } from "./mechanics-directional-spikes-into.js";
import { scenario as scenario5 } from "./mechanics-berry-train.js";
import { scenario as scenario6 } from "./seven-jump.js";
import { scenario as scenario7 } from "./eight-jump.js";
import { scenario as scenario8 } from "./mechanics-screen-transition-up.js";
import { scenario as scenario9 } from "./mechanics-liftboost-zip-jump.js";
import { scenario as scenario10 } from "./dash-spring-cancel.js";
import { scenario as scenario11 } from "./dash-spiked-wallbounce.js";
import { scenario as scenario12 } from "./dash-spiked-wallbounce-late.js";
import { scenario as scenario13 } from "./dash-superwave.js";
import { scenario as scenario14 } from "./dash-demodash-gap.js";
import { scenario as scenario15 } from "./dash-ultra.js";
import { scenario as scenario16 } from "./dash-grounded-ultra.js";
import { scenario as scenario17 } from "./dash-delayed-ultra.js";
import { scenario as scenario18 } from "./dash-chained-ultras.js";
import { scenario as scenario19 } from "./nine-jump.js";
import { scenario as scenario20 } from "./entity-4.8-delayed-blockboost.js";
import { scenario as scenario21 } from "./entity-4.7-core-super.js";
import { scenario as scenario22 } from "./entity-4.7-core-hyper.js";
import { scenario as scenario23 } from "./playground-swim-idle.js";
import { scenario as scenario24 } from "./playground-swim-right.js";
import { scenario as scenario25 } from "./playground-swim-surface-idle.js";
import { scenario as scenario26 } from "./playground-swim-up.js";
import { scenario as scenario27 } from "./playground-swim-down.js";
import { scenario as scenario28 } from "./playground-swim-diagonal.js";
import { scenario as scenario29 } from "./playground-swim-exit-right.js";
import { scenario as scenario30 } from "./playground-swim-jump.js";
import { scenario as scenario31 } from "./playground-swim-dash.js";
import { scenario as scenario32 } from "./entity-4.1-archie.js";
import { scenario as scenario33 } from "./collector-startdash-buffer-consumed-through-boost.js";
import { scenario as scenario34 } from "./entity-4.2-bubble-super.js";
import { scenario as scenario35 } from "./entity-4.2-bubble-demohyper.js";
import { scenario as scenario36 } from "./entity-4.5-iceball-jump.js";
import { scenario as scenario37 } from "./entity-4.15.2-feather-hitbox-preservation.js";
import { scenario as scenario38 } from "./playground-green-booster-auto.js";
import { scenario as scenario39 } from "./playground-green-booster-right.js";
import { scenario as scenario40 } from "./playground-green-booster-up.js";
import { scenario as scenario41 } from "./playground-red-booster-auto.js";
import { scenario as scenario42 } from "./playground-red-booster-right.js";
import { scenario as scenario43 } from "./playground-red-booster-up.js";
import { scenario as scenario44 } from "./playground-wind-idle.js";
import { scenario as scenario45 } from "./playground-wind-dash-left.js";
import { scenario as scenario46 } from "./playground-wind-ground-standing.js";
import { scenario as scenario47 } from "./playground-wind-ground-ducking.js";
import { scenario as scenario48 } from "./playground-wind-wall-shield.js";
import { scenario as scenario49 } from "./mechanics-dash-attack-late-shield.js";
import { scenario as scenario50 } from "./entity-4.12-featherboost.js";
import { scenario as scenario51 } from "./entity-4.13-feather-super.js";
import { scenario as scenario52 } from "./entity-4.15.1-feather-clip.js";
import { scenario as scenario53 } from "./playground-starfly-right.js";
import { scenario as scenario54 } from "./playground-starfly-idle-timeout.js";
import { scenario as scenario55 } from "./playground-starfly-up-exit.js";
import { scenario as scenario56 } from "./playground-starfly-turn-up.js";
import { scenario as scenario57 } from "./playground-starfly-turn-back.js";
import { scenario as scenario58 } from "./playground-starfly-dash-cancel.js";
import { scenario as scenario59 } from "./playground-starfly-wall-jump-cancel.js";
import { scenario as scenario60 } from "./playground-starfly-wall-grab-cancel.js";
import { scenario as scenario61 } from "./playground-starfly-shield-bounce.js";
import { scenario as scenario62 } from "./playground-starfly-shield-dash.js";
import { scenario as scenario63 } from "./playground-starfly-renew.js";
import { scenario as scenario64 } from "./playground-launch-up.js";
import { scenario as scenario65 } from "./playground-launch-side.js";
import { scenario as scenario66 } from "./playground-launch-dash-cancel.js";
import { scenario as scenario67 } from "./playground-summit-launch.js";
import { scenario as scenario68 } from "./entity-4.3-bumper-clip.js";
import { scenario as scenario69 } from "./entity-4.4-explosion-boost.js";
import { scenario as scenario70 } from "./playground-bumper-left-idle.js";
import { scenario as scenario71 } from "./playground-bumper-left-hold.js";
import { scenario as scenario72 } from "./playground-bumper-right-hold.js";
import { scenario as scenario73 } from "./playground-badeline-boost-launch.js";
import { scenario as scenario74 } from "./playground-badeline-boost-summit.js";
import { scenario as scenario75 } from "./playground-dummy-state.js";
import { scenario as scenario76 } from "./playground-frozen-state.js";
import { scenario as scenario77 } from "./playground-temple-fall-state.js";
import { scenario as scenario78 } from "./playground-reflection-fall-state.js";
import { scenario as scenario79 } from "./entity-4.6-cloud-jump.js";
import { scenario as scenario80 } from "./eleven-jump.js";
import { scenario as scenario81 } from "./reverse-cornerboost.js";
import { scenario as scenario82 } from "./neutral-reverse-cornerboost.js";
import { scenario as scenario83 } from "./spiked-cornerboost.js";
import { scenario as scenario84 } from "./other-5.5-half-stamina-climbing.js";
import { scenario as scenario85 } from "./other-5.6-kermit-dash.js";
import { scenario as scenario86 } from "./other-5.12-subpixel-manipulation.js";
import { scenario as scenario87 } from "./spike-climb.js";
import { scenario as scenario88 } from "./narrow-spiked-climb.js";
import { scenario as scenario89 } from "./spike-clip.js";
import { scenario as scenario90 } from "./spike-jump.js";
import { scenario as scenario91 } from "./cornerboost-wallboost.js";
import { scenario as scenario92 } from "./cornerslip.js";
import { scenario as scenario93 } from "./dash-grounded-ultra-cancel.js";
import { scenario as scenario94 } from "./dash-grounded-ultra-cancel-control.js";
import { scenario as scenario95 } from "./entity-4.6.1-cloud-super.js";
import { scenario as scenario96 } from "./entity-4.6.1-cloud-hyper.js";
import { scenario as scenario97 } from "./entity-4.6.2-cloud-hyper-bunnyhop.js";
import { scenario as scenario98 } from "./entity-4.10.3-dream-smuggle.js";
import { scenario as scenario99 } from "./entity-4.10.3.1-dream-grab-hyper.js";
import { scenario as scenario100 } from "./entity-4.10.3.2-holdable-dream-hyper.js";
import { scenario as scenario101 } from "./entity-4.10.4-holdable-grabless-dream-hyper.js";
import { scenario as scenario102 } from "./entity-4.20-theo-regrab.js";
import { scenario as scenario103 } from "./entity-4.22-neutral-drop.js";
import { scenario as scenario104 } from "./entity-4.22.2-holdable-climb.js";
import { scenario as scenario105 } from "./entity-4.22.3-holdable-neutral-jump.js";
import { scenario as scenario106 } from "./entity-4.17-moon-boost.js";
import { scenario as scenario107 } from "./entity-4.18-reform-tech.js";
import { scenario as scenario108 } from "./entity-4.18.1-reform-kick.js";
import { scenario as scenario109 } from "./entity-4.18.3-core-block-entity-displacement.js";
import { scenario as scenario110 } from "./entity-4.23-theo-ultra.js";
import { scenario as scenario111 } from "./entity-4.24-bumper-holdable-dash-smuggle.js";
import { scenario as scenario112 } from "./entity-4.25-throwable-backboost.js";
import { scenario as scenario113 } from "./entity-4.26-theovator.js";
import { scenario as scenario114 } from "./entity-4.27-waterboost.js";
import { scenario as scenario115 } from "./entity-4.20-jelly-regrab.js";
import { scenario as scenario116 } from "./entity-4.22.1-holdable-stall.js";
import { scenario as scenario117 } from "./entity-4.22.3-jelly-neutral-jump.js";
import { scenario as scenario118 } from "./entity-4.22.4-holdable-laddering.js";
import { scenario as scenario119 } from "./entity-4.23-jelly-ultra.js";
import { scenario as scenario120 } from "./entity-4.26-jellyvator.js";
import { scenario as scenario121 } from "./entity-4.29-springboost-cancel.js";
import { scenario as scenario122 } from "./entity-4.11-holdable-core-hyper.js";
import { scenario as scenario123 } from "./entity-4.14-heart-ultra.js";
import { scenario as scenario124 } from "./entity-4.15-jumpthrough-clip.js";
import { scenario as scenario125 } from "./entity-4.18.2-reform-boost-cassette-boost.js";
import { scenario as scenario126 } from "./entity-4.18.2.1-cassoosted-fuper.js";
import { scenario as scenario127 } from "./other-5.3-cassette-raise.js";
import { scenario as scenario128 } from "./other-5.9-screen-transition-cassette-offset.js";
import { scenario as scenario129 } from "./other-5.10-spinner-stunning.js";
import { scenario as scenario130 } from "./other-5.11-spinner-freeze.js";
import { scenario as scenario131 } from "./entity-4.21-holdable-slash.js";
import { scenario as scenario132 } from "./entity-4.19-seeker-bounce.js";
import { scenario as scenario133 } from "./entity-4.28-koral-clip.js";
import { scenario as scenario134 } from "./entity-4.16-lava-neutral.js";
import { scenario as scenario135 } from "./other-5.13-undemo-omed-dashing.js";
import { scenario as scenario136 } from "./dashless-3.7.11-disappearing-block-cornerboost.js";
import { scenario as scenario137 } from "./other-5.8-roboboost.js";
import { scenario as scenario138 } from "./mechanics-dash-aim-sampling.js";
import { scenario as scenario139 } from "./other-5.1-bino-tech.js";
import { scenario as scenario140 } from "./other-5.1.1-bino-clip.js";
import { scenario as scenario141 } from "./other-5.1.2-bino-control-storage.js";
import { scenario as scenario142 } from "./other-5.1.3-bino-interaction-storage.js";
import { scenario as scenario143 } from "./other-5.1.4-bino-extensions.js";
import { scenario as scenario144 } from "./other-5.2-bubsdrop.js";
import { scenario as scenario145 } from "./mechanics-corner-correction-down.js";
import { scenario as scenario146 } from "./delayed-wallbounce.js";
import { scenario as scenario147 } from "./mechanics-move-block-side-grab.js";
import { scenario as scenario148 } from "./mechanics-bounce-block-side-grab.js";

export const scenarios: readonly ScenarioDefinition[] = Object.freeze([
  scenario0,
  scenario1,
  scenario2,
  scenario3,
  scenario4,
  scenario5,
  scenario6,
  scenario7,
  scenario8,
  scenario9,
  scenario10,
  scenario11,
  scenario12,
  scenario13,
  scenario14,
  scenario15,
  scenario16,
  scenario17,
  scenario18,
  scenario19,
  scenario20,
  scenario21,
  scenario22,
  scenario23,
  scenario24,
  scenario25,
  scenario26,
  scenario27,
  scenario28,
  scenario29,
  scenario30,
  scenario31,
  scenario32,
  scenario33,
  scenario34,
  scenario35,
  scenario36,
  scenario37,
  scenario38,
  scenario39,
  scenario40,
  scenario41,
  scenario42,
  scenario43,
  scenario44,
  scenario45,
  scenario46,
  scenario47,
  scenario48,
  scenario49,
  scenario50,
  scenario51,
  scenario52,
  scenario53,
  scenario54,
  scenario55,
  scenario56,
  scenario57,
  scenario58,
  scenario59,
  scenario60,
  scenario61,
  scenario62,
  scenario63,
  scenario64,
  scenario65,
  scenario66,
  scenario67,
  scenario68,
  scenario69,
  scenario70,
  scenario71,
  scenario72,
  scenario73,
  scenario74,
  scenario75,
  scenario76,
  scenario77,
  scenario78,
  scenario79,
  scenario80,
  scenario81,
  scenario82,
  scenario83,
  scenario84,
  scenario85,
  scenario86,
  scenario87,
  scenario88,
  scenario89,
  scenario90,
  scenario91,
  scenario92,
  scenario93,
  scenario94,
  scenario95,
  scenario96,
  scenario97,
  scenario98,
  scenario99,
  scenario100,
  scenario101,
  scenario102,
  scenario103,
  scenario104,
  scenario105,
  scenario106,
  scenario107,
  scenario108,
  scenario109,
  scenario110,
  scenario111,
  scenario112,
  scenario113,
  scenario114,
  scenario115,
  scenario116,
  scenario117,
  scenario118,
  scenario119,
  scenario120,
  scenario121,
  scenario122,
  scenario123,
  scenario124,
  scenario125,
  scenario126,
  scenario127,
  scenario128,
  scenario129,
  scenario130,
  scenario131,
  scenario132,
  scenario133,
  scenario134,
  scenario135,
  scenario136,
  scenario137,
  scenario138,
  scenario139,
  scenario140,
  scenario141,
  scenario142,
  scenario143,
  scenario144,
  scenario145,
  scenario146,
  scenario147,
  scenario148,
]);
