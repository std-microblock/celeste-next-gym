// Example TypeScript fuzz spec: find the longest-distance jump timing from the
// spawn of Forsaken City room 1 that stays alive.
//
//   celeste-gym fuzz tools/gym-cli/examples/longest-jump.fuzz.ts --top 3 --render out.gif --hud
//
// A `.ts` spec may export `map`, `room`, `bin`, `state` (merged over the
// start state) and a default spec — or a (ctx) => spec function, which gets
// `{ api, map }` (`map` = opened MapSession when `map` is exported).
import { defineFuzz } from "../src/fuzzTypes.ts";

export const map = "vanilla:1-ForsakenCity";
export const room = "1";

export default defineFuzz({
  version: 1,
  variables: [
    { name: "run", range: { from: 0, to: 20, step: 2 } },
    { name: "hold", range: { from: 1, to: 12 } },
  ],
  inputs: [
    { keys: ["right"], at: 0, held_time: "hold::inf" },
    { keys: ["jump"], at: "run", held_time: "hold" },
  ],
  observe_until: "run + 40",
  success: ["!final.dead"],
  objectives: [{ type: "maximize", expression: "final.pos.x - initial.pos.x" }],
  search: { output: ["best", "windows"] },
});
