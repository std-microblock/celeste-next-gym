/**
 * Regression tests for the headless render backend.
 *
 * A "frozen" canvas must be drawable in the same tick. @napi-rs/canvas decodes
 * `Image.src = <Buffer>` on a worker thread, so the old `freeze` implementation
 * (Image built from `canvas.toBuffer("image/png")`) silently produced empty
 * bitmaps: the merged gameplay atlas and every cached tile layer drew nothing
 * and `--render` output became background + HUD only.
 */
import { strict as assert } from "node:assert";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { createRenderCanvas, freezeRenderCanvas } from "../../../web/src/render/canvasBackend.ts";
import { SceneRenderer, installNodeRenderBackend } from "../src/render.ts";
import { openMap } from "../src/index.ts";
import { defaultWasmDir } from "../src/wasm.ts";

const hasWasm = existsSync(resolve(defaultWasmDir(), "celeste_wasm_bg.wasm"));

test("a frozen render canvas is drawn in full by the very next drawImage", () => {
  installNodeRenderBackend();
  const source = createRenderCanvas(64, 64);
  const sourceContext = source.getContext("2d");
  assert.ok(sourceContext);
  sourceContext.fillStyle = "#36f";
  sourceContext.fillRect(0, 0, 64, 64);

  // Same tick: no await, no setImmediate — exactly how gameRenderer.ts uses it.
  const target = createRenderCanvas(64, 64);
  const targetContext = target.getContext("2d");
  assert.ok(targetContext);
  targetContext.drawImage(freezeRenderCanvas(source), 0, 0);

  const pixels = targetContext.getImageData(0, 0, 64, 64).data;
  let opaque = 0;
  for (let i = 3; i < pixels.length; i += 4) if (pixels[i] !== 0) opaque += 1;
  assert.equal(opaque, 64 * 64, "frozen canvas drew an empty bitmap (async decode leaked into freeze)");
});

test("a rendered frame contains tiles and sprites, not a flat background", { skip: !hasWasm && "WASM bundle not built" }, async () => {
  const map = await openMap("playground");
  const trace = await map.simulate("20,R;1,R,J;12,R,J;20,R");
  const renderer = await SceneRenderer.create(undefined, { camera: "room", scale: 1 });
  const view = await map.viewFor(trace.final);
  const index = trace.states.length - 1;
  const canvas = renderer.drawFrame(view, trace.states, index, trace.inputs);
  assert.ok(canvas.width > 0 && canvas.height > 0);

  const data = canvas.getContext("2d").getImageData(0, 0, canvas.width, canvas.height).data;
  const buckets = new Map<number, number>();
  for (let i = 0; i < data.length; i += 4) {
    const key = ((data[i] >> 4) << 8) | ((data[i + 1] >> 4) << 4) | (data[i + 2] >> 4);
    buckets.set(key, (buckets.get(key) ?? 0) + 1);
  }
  const total = canvas.width * canvas.height;
  const modalShare = Math.max(...buckets.values()) / total;
  // Blank-atlas frames measure ~62 colours / 0.98 modal share; real frames are
  // ~230 colours / 0.39, so both thresholds have wide margins.
  assert.ok(buckets.size > 100, `frame has too few colours (${buckets.size}): atlases or tile layer did not draw`);
  assert.ok(modalShare < 0.6, `frame is ${(modalShare * 100).toFixed(0)}% one colour: tiles/sprites missing`);
});
