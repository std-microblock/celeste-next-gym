import assert from "node:assert/strict";
import { test } from "node:test";
import { resolve } from "node:path";

import { compileCatalog } from "./build-training-mod.mjs";

const catalog = resolve(import.meta.dirname, "../training");

test("compiles the root catalog into deterministic Mod training rooms", () => {
  const result = compileCatalog(catalog);
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
  assert.equal(result.catalog.sourceWorkspace, "training");
  assert.equal(result.catalog.projects[0].technique.id, "hyper");
  assert.equal(
    result.catalog.projects[0].technique.section.title,
    "冲刺技巧",
  );
});
