import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, it } from "node:test";

import { scenarios } from "../scenarios/index.js";
import {
  assertNoPlaygroundMapOverride,
  groupScenariosByLifecycle,
  materializePlaygroundMap,
  scenarioFixture,
  scenarioLifecycleKey,
} from "../runtime/playground-map.js";
import type { ScenarioDefinition } from "../types.js";

const transition = requireScenario("mechanics-screen-transition-up");
const zipJump = requireScenario("mechanics-liftboost-zip-jump");
const bubbleSuper = requireScenario("entity-4.2-bubble-super");
const bubbleDemohyper = requireScenario("entity-4.2-bubble-demohyper");
const binoClip = requireScenario("other-5.1.1-bino-clip");
const binoInteractionStorage = requireScenario(
  "other-5.1.3-bino-interaction-storage",
);
const binoExtensions = requireScenario("other-5.1.4-bino-extensions");

describe("per-scenario Playground maps", () => {
  it("limits screen-transition fixture contents to its declared closure", () => {
    const generated = scenarioFixture(transition);
    assert.deepEqual(
      generated.fixture.rooms.map((room) => room.name),
      ["playground", "transition_0"],
    );
    assert.equal(
      generated.fixture.rooms.flatMap((room) => room.entities).length,
      0,
    );
    assert.equal(
      generated.fixture.rooms.some((room) =>
        room.entities.some((entity) => entity.kind === "theo_crystal"),
      ),
      false,
    );
  });

  it("hashes canonical fixtures and only groups identical maps into one lifecycle", () => {
    assert.equal(
      scenarioFixture(transition).hash,
      scenarioFixture(transition).hash,
    );
    assert.notEqual(
      scenarioLifecycleKey(transition),
      scenarioLifecycleKey(zipJump),
    );
    const duplicate = Object.freeze({
      ...transition,
      name: `${transition.name}-duplicate`,
    });
    const groups = groupScenariosByLifecycle([transition, zipJump, duplicate]);
    assert.equal(groups.length, 2);
    assert.deepEqual(
      groups
        .find((group) => group.key === scenarioLifecycleKey(transition))
        ?.scenarios.map((item) => item.name),
      [transition.name, duplicate.name],
    );
  });

  it("gives bubble coyote scenarios their source jumpthrough without an unrelated zip mover", () => {
    for (const scenario of [bubbleSuper, bubbleDemohyper]) {
      const entities = scenarioFixture(scenario).fixture.rooms.flatMap(
        (room) => room.entities,
      );
      assert.deepEqual(
        entities.find((entity) => entity.kind === "jump_thru")?.bounds,
        [112, 400, 112, 8],
      );
      assert.equal(
        entities.some(
          (entity) =>
            entity.kind === "booster" &&
            entity.bounds[0] === 230 &&
            entity.bounds[1] === 384,
        ),
        true,
      );
      assert.equal(
        entities.some((entity) => entity.kind === "zip_mover"),
        false,
      );
    }
  });

  it("keeps the Bino interaction-storage candidate in its native Booster window", () => {
    const fixture = scenarioFixture(binoInteractionStorage).fixture;
    const playground = fixture.rooms.find((room) => room.name === "playground");
    const lookout = playground?.entities.find(
      (entity) => entity.id === "tech-5.1.3-lookout",
    );
    const booster = playground?.entities.find(
      (entity) => entity.id === "tech-5.1.3-interrupting-booster",
    );

    assert.deepEqual(lookout?.bounds, [938, 493, 4, 4]);
    assert.deepEqual(booster?.bounds, [924, 491, 16, 16]);
    assert.equal(
      playground?.solids.some(
        ([x, y, width, height]) =>
          x === 936 && y === 0 && width === 24 && height === 496,
      ),
      false,
    );
    assert.equal(binoInteractionStorage.inputs.length, 300);
    assert.equal(binoInteractionStorage.inputs[120]?.move_x, 1);
  });

  it("aims the Bino Clip candidate left past the spinner view boundary", () => {
    const fixture = scenarioFixture(binoClip).fixture;
    const playground = fixture.rooms.find((room) => room.name === "playground");
    const spinner = playground?.entities.find(
      (entity) => entity.id === "tech-5.1.1-spinner",
    );

    assert.deepEqual(spinner?.bounds, [628, 484, 16, 12]);
    assert.equal(binoClip.inputs.length, 230);
    assert.equal(binoClip.inputs[54]?.move_x, 0);
    assert.equal(binoClip.inputs[55]?.move_x, -1);
    assert.equal(binoClip.inputs[204]?.move_x, -1);
    assert.equal(binoClip.inputs[205]?.jump_pressed, true);
  });

  it("exits the terminal Bino summit node with MenuCancel rather than arrival", () => {
    const fixture = scenarioFixture(binoExtensions).fixture;
    const playground = fixture.rooms.find((room) => room.name === "playground");
    const lookout = playground?.entities.find(
      (entity) => entity.id === "tech-5.1.4-lookout",
    );

    assert.deepEqual(lookout?.nodes, [
      [896, 400],
      [896, 72],
      [24, 24],
    ]);
    assert.equal(binoExtensions.inputs.length, 640);
    assert.equal(binoExtensions.inputs[519]?.jump_pressed, false);
    assert.equal(binoExtensions.inputs[520]?.jump_pressed, true);
  });

  it("rejects a map override instead of silently bypassing scenario parts", () => {
    assert.throws(
      () => assertNoPlaygroundMapOverride(transition, "custom.bin"),
      /cannot override/,
    );
  });

  it("stages the exact compiled scenario bin that Rust comparison reads", () => {
    const root = mkdtempSync(join(tmpdir(), "celeste-gym-playground-map-"));
    try {
      const playgroundModRoot = resolve(root, "source-mod");
      mkdirSync(playgroundModRoot, { recursive: true });
      writeFileSync(
        resolve(playgroundModRoot, "everest.yaml"),
        "- Name: CelesteGymPlayground\n",
        "utf8",
      );
      const compiled = Buffer.from("scenario-specific-bin");
      const result = materializePlaygroundMap({
        repoRoot: root,
        runRoot: resolve(root, "run"),
        paths: {
          repoRoot: root,
          gameRoot: resolve(root, "game"),
          modRoot: resolve(root, "collector"),
          playgroundModRoot,
          serviceRoot: resolve(root, "service"),
        },
        scenario: transition,
        compile: (_fixturePath, mapPath) => writeFileSync(mapPath, compiled),
      });
      const installedMap = resolve(
        result.modRoot,
        "Maps",
        "CelesteGymPlayground",
        "Playground.bin",
      );
      assert.deepEqual(readFileSync(result.mapPath), compiled);
      assert.deepEqual(readFileSync(installedMap), compiled);
      assert.deepEqual(
        JSON.parse(readFileSync(result.fixturePath, "utf8")),
        result.fixture,
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

function requireScenario(name: string): ScenarioDefinition {
  const scenario = scenarios.find((candidate) => candidate.name === name);
  if (!scenario) throw new Error(`missing scenario ${name}`);
  return scenario;
}
