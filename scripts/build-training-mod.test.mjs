import assert from "node:assert/strict";
import { test } from "node:test";
import { resolve } from "node:path";

import { compileWorkspace } from "./build-training-mod.mjs";

const example = resolve(import.meta.dirname, "../web/src/training/maps");

test("compiles the committed JSON workspace into deterministic training rooms", () => {
  const result = compileWorkspace(example);
  assert.equal(result.fixture.sid, "CelesteGymTraining/Training");
  assert.deepEqual(
    result.fixture.rooms.map((room) => room.name),
    ["hyper-route", "hyper-spikes", "hyper-bubble"],
  );
  assert.equal(result.catalog.skin, "strawberry-jam-2021-beginner-gym");
  assert.deepEqual(
    result.catalog.projects.map((project) => project.id),
    ["route", "spike-gap", "bubble-exit"],
  );
  assert.equal(
    result.catalog.projects[0].training.modules[0].tutorial.title,
    "跨越断层",
  );
});
