import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  decode,
  encode,
} from "../../web/node_modules/@msgpack/msgpack/dist.esm/index.mjs";
import init, {
  decode_celeste_map_msgpack,
  simulate_msgpack,
} from "../../web/src/wasm/celeste_wasm.js";

const recorderRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = resolve(recorderRoot, "..");
await init({
  module_or_path: await readFile(
    resolve(repoRoot, "web", "src", "wasm", "celeste_wasm_bg.wasm"),
  ),
});

const binary = resolve(
  recorderRoot,
  "maps",
  "CelesteGymPlayground",
  "Playground.bin",
);
const mapResponse = decode(
  decode_celeste_map_msgpack(await readFile(binary), "playground"),
);
if (!mapResponse.success || !mapResponse.map)
  throw new Error(mapResponse.error ?? "failed to decode playground map");

const empty = () => ({
  move_x: 0,
  move_y: 0,
  jump_pressed: false,
  jump_held: false,
  dash_pressed: false,
  crouch_dash_pressed: false,
  grab_held: false,
});
const samples = [
  {
    name: "web-run-right.trace.json",
    inputs: Array.from({ length: 30 }, () => ({ ...empty(), move_x: 1 })),
  },
  {
    name: "web-jump-right.trace.json",
    inputs: Array.from({ length: 45 }, (_, frame) => ({
      ...empty(),
      move_x: 1,
      jump_pressed: frame === 0,
      jump_held: frame < 10,
    })),
  },
];

for (const sample of samples) {
  const initial = {
    pos: mapResponse.map.spawn,
    speed: { x: 0, y: 0 },
    state: "Normal",
    facing: true,
    dashes: 1,
    stamina: 110,
    on_ground: false,
    ducking: false,
    can_dream_dash: true,
    dead: false,
    death_freeze_pending: false,
    respawn_frames: 0,
    dash_dir: { x: 0, y: 0 },
  };
  const response = decode(
    simulate_msgpack(
      encode(initial),
      encode(sample.inputs),
      encode(mapResponse.map),
      sample.inputs.length,
    ),
  );
  if (!response.states || response.states.length !== sample.inputs.length + 1) {
    throw new Error(response.error ?? `${sample.name} simulation failed`);
  }
  const trace = {
    format: "celeste-next-gym-trace",
    version: 1,
    source: "web",
    recorded_at: "2026-07-28T00:00:00.000Z",
    map: {
      sid: "CelesteGymPlayground/Playground",
      room: "playground",
      binary: "maps/CelesteGymPlayground/Playground.bin",
      data: mapResponse.map,
    },
    inputs: sample.inputs,
    states: response.states.map((state, frame) => ({
      frame,
      pos: [state.pos.x, state.pos.y],
      speed: [state.speed.x, state.speed.y],
      state: state.state,
      facing: state.facing,
      dashes: state.dashes,
      stamina: state.stamina,
      on_ground: state.on_ground,
      ducking: state.ducking,
      dead: state.dead,
      snapshot: state,
    })),
  };
  const output = resolve(recorderRoot, "examples", sample.name);
  await mkdir(dirname(output), { recursive: true });
  await writeFile(output, `${JSON.stringify(trace, null, 2)}\n`, "utf8");
  console.log(`wrote ${output}`);
}
