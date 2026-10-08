import { strict as assert } from "node:assert";
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  checkMap,
  inputsToTas,
  openMap,
  openMapSource,
  parseTasLines,
  renderTrace,
  tasFramesToInputs,
} from "../src/index.ts";
import { defaultWasmDir } from "../src/wasm.ts";

const here = fileURLToPath(new URL(".", import.meta.url));
const hasWasm = existsSync(resolve(defaultWasmDir(), "celeste_wasm_bg.wasm"));

test("TAS lines: press edges per binding and TakeNewer axes", async () => {
  const inputs = tasFramesToInputs(await parseTasLines("2,R,J;1,R,K,J;1,L,R;1,L,R;1,L;1,X;1,X"));
  assert.equal(inputs.length, 8);
  assert.deepEqual(inputs.map((input) => input.jump_pressed), [true, false, true, false, false, false, false, false]);
  assert.equal(inputs[2].jump_held, true);
  // R held, then L+R: the newer (left) wins and stays while both are held.
  assert.deepEqual(inputs.map((input) => input.move_x), [1, 1, 1, -1, -1, -1, 0, 0]);
  // A held dash only presses once.
  assert.deepEqual(inputs.map((input) => input.dash_pressed), [false, false, false, false, false, false, true, false]);
});

test("inputsToTas round-trips simulator inputs", async () => {
  const source = tasFramesToInputs(await parseTasLines("3,R;1,R,D,X;3,R;12,R,J;1,R,K;4,G,U;2"));
  const back = tasFramesToInputs(await parseTasLines(inputsToTas(source)));
  assert.deepEqual(back, source);
});

test("simulate, fuzz, check and render the bundled playground", { skip: !hasWasm && "WASM bundle not built" }, async () => {
  const map = await openMap("playground");
  const trace = await map.simulate("20,R");
  assert.equal(trace.states.length, 21);
  assert.ok(trace.final.pos.x > trace.states[0].pos.x);

  const spec = JSON.parse(readFileSync(join(here, "..", "examples", "playground-hyper.fuzz.json"), "utf8"));
  const fuzz = await map.fuzz(spec);
  assert.ok(fuzz.result.best, "fuzz found a candidate");
  assert.equal(fuzz.best_inputs?.length, (fuzz.result.best!.bindings.jump_at ?? 0) + 20);
  const replay = await map.simulate(fuzz.best_inputs!);
  assert.deepEqual(replay.final.pos, (fuzz.result.best!.final_state as { pos: unknown }).pos);

  const report = await checkMap(openMapSource("playground"), { frames: 5 });
  assert.equal(report.status, "ok");

  const dir = mkdtempSync(join(tmpdir(), "celeste-gym-"));
  try {
    const png = await renderTrace(trace, join(dir, "frame.png"), { hud: true, hitboxes: true });
    assert.equal(png.width, 640);
    const gif = await renderTrace(trace, join(dir, "run.gif"), { every: 5 });
    assert.ok(statSync(gif.path).size > 1000);
    assert.equal(readFileSync(gif.path).subarray(0, 3).toString(), "GIF");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
