/**
 * Regression tests for the headless render backend.
 *
 * Two traps live in `freeze`: @napi-rs/canvas decodes `Image.src = <Buffer>` on
 * a worker thread, so a frozen Image drawn in the same tick is blank (the
 * original bug: background + HUD only), and drawing from a *canvas* instead
 * makes Skia copy the whole source surface on every `drawImage` (~18 MB for an
 * 8x8 tile out of the 1024x4600 gameplay atlas, which OOMs a small machine).
 * The backend therefore freezes asynchronously, and the atlas is never wrapped
 * in a canvas at all.
 */
import { strict as assert } from "node:assert";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import {
  createRenderCanvas,
  freezeRenderCanvas,
  freezeRenderCanvasAsync,
} from "../../../web/src/render/canvasBackend.ts";
import { loadAssets, prepareGameAssets } from "../../../web/src/render/gameRenderer.ts";
import { SceneRenderer, installNodeRenderBackend } from "../src/render.ts";
import { openMap } from "../src/index.ts";
import { defaultWasmDir } from "../src/wasm.ts";

const hasWasm = existsSync(resolve(defaultWasmDir(), "celeste_wasm_bg.wasm"));

function opaquePixels(draw: (canvas: HTMLCanvasElement) => void): number {
  const target = createRenderCanvas(64, 64);
  draw(target);
  const context = target.getContext("2d");
  assert.ok(context);
  const data = context.getImageData(0, 0, 64, 64).data;
  let opaque = 0;
  for (let i = 3; i < data.length; i += 4) if (data[i] !== 0) opaque += 1;
  return opaque;
}

function filledSquare(): HTMLCanvasElement {
  const canvas = createRenderCanvas(64, 64);
  const context = canvas.getContext("2d");
  assert.ok(context);
  context.fillStyle = "#36f";
  context.fillRect(0, 0, 64, 64);
  return canvas;
}

test("a sync-frozen canvas is drawn in full by the very next drawImage", () => {
  installNodeRenderBackend();
  const frozen = freezeRenderCanvas(filledSquare());
  // Same tick: no await, no setImmediate — how gameRenderer's draw loops use it.
  assert.equal(opaquePixels((target) => target.getContext("2d")?.drawImage(frozen, 0, 0)), 64 * 64);
});

test("an async-frozen canvas is a decoded image, drawn in full on the first frame", async () => {
  installNodeRenderBackend();
  const frozen = await freezeRenderCanvasAsync(filledSquare());
  assert.equal(
    typeof (frozen as unknown as { getContext?: unknown }).getContext,
    "undefined",
    "the baked source must be a decoded image; a canvas copies its whole surface per draw",
  );
  assert.equal(opaquePixels((target) => target.getContext("2d")?.drawImage(frozen, 0, 0)), 64 * 64);
});

test("the gameplay atlas stays a decoded image, never an offscreen canvas", async () => {
  installNodeRenderBackend();
  const assets = await loadAssets();
  assert.ok(assets.image.width > 0 && assets.image.height > 0);
  assert.equal(
    typeof (assets.image as unknown as { getContext?: unknown }).getContext,
    "undefined",
    "an atlas wrapped in a canvas copies the whole surface on every sprite draw",
  );
  // Already decoded: preparing it again must not re-bake it through a PNG.
  assert.equal(await prepareGameAssets(assets), assets);
  const opaque = opaquePixels((target) =>
    target
      .getContext("2d")
      ?.drawImage(assets.image as unknown as CanvasImageSource, 0, 0, assets.image.width, assets.image.height, 0, 0, 64, 64),
  );
  assert.ok(opaque > 0, "the atlas drew nothing");
});

test("a rendered frame contains tiles and sprites, not a flat background", { skip: !hasWasm && "WASM bundle not built" }, async () => {
  const map = await openMap("playground");
  const trace = await map.simulate("20,R;1,R,J;12,R,J;20,R");
  const renderer = await SceneRenderer.create(undefined, { camera: "room", scale: 1 });
  const view = await map.viewFor(trace.final);
  await renderer.prepare(view);
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
