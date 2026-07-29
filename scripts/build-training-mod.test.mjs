import assert from "node:assert/strict";
import { test } from "node:test";
import { resolve } from "node:path";

import { compileWorkspace } from "./build-training-mod.mjs";

const example = resolve(import.meta.dirname, "../.tmp/example-workfolder-tutorial");

test("compiles the example workspace into one deterministic training room", () => {
  const result = compileWorkspace(example);
  assert.equal(result.fixture.sid, "CelesteGymTraining/Training");
  assert.equal(result.fixture.rooms.length, 1);
  assert.equal(result.fixture.rooms[0].name, "untitled-room");
  assert.deepEqual(result.fixture.rooms[0].bounds, [0, 0, 504, 184]);
  assert.deepEqual(result.fixture.rooms[0].solids.at(-1), [416, 104, 88, 80]);
  assert.equal(result.catalog.skin, "strawberry-jam-2021-beginner-gym");
  assert.equal(result.catalog.projects[0].training.modules[0].tutorial.title, "教程 1");
});
