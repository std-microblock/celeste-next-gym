import { readFile } from "node:fs/promises";
import { decode, encode } from "@msgpack/msgpack";
import { beforeAll, describe, expect, it } from "vitest";
import init, {
  cache_simulation_map_msgpack,
  fuzz_search_cached_map_msgpack,
} from "../src/wasm/celeste_wasm.js";
import { makeEmptyButtons } from "../src/model";
import {
  createBlankGymMap,
  createTrainingProject,
} from "../src/training/editorProject";
import { applyTutorialRecording } from "../src/training/recording";

beforeAll(async () => {
  await init({
    module_or_path: await readFile("src/wasm/celeste_wasm_bg.wasm"),
  });
});

describe("recorded tutorial Fuzz smoke", () => {
  it("executes automatically generated JSON in celeste-wasm", () => {
    const project = createTrainingProject(createBlankGymMap());
    const module = project.training.modules[0];
    module.end_trigger.bounds = { ...project.map.bounds };
    const frame = makeEmptyButtons();
    frame.right = true;
    frame.dash = true;
    const recorded = applyTutorialRecording(
      project,
      0,
      module.validation.initial_state,
      [frame],
    );
    cache_simulation_map_msgpack(encode(recorded.map));
    const result = decode(
      fuzz_search_cached_map_msgpack(
        encode(recorded.training.modules[0].validation.initial_state),
        JSON.stringify(recorded.training.modules[0].tutorial.fuzz),
      ),
    ) as { candidates?: unknown[]; error?: string };
    expect(result.error).toBeUndefined();
    expect(result.candidates?.length).toBeGreaterThan(0);
  });
});
