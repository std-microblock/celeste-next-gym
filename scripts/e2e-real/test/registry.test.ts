import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { buildRegistry, selectScenarios } from "../registry.js";
import { testScenario } from "./helpers.js";

describe("scenario registry", () => {
  it("derives active/candidate counts and excludes candidates by default", () => {
    const registry = buildRegistry([
      testScenario("active-one"),
      testScenario("candidate-one", { status: "candidate" }),
    ]);
    assert.deepEqual(registry.counts, { active: 1, candidate: 1 });
    assert.deepEqual(
      selectScenarios(registry, {}).map((scenario) => scenario.name),
      ["active-one"],
    );
    assert.equal(
      selectScenarios(registry, { includeCandidates: true }).length,
      2,
    );
  });

  it("rejects duplicate and unknown names before selection", () => {
    assert.throws(
      () => buildRegistry([testScenario("same"), testScenario("same")]),
      /duplicate/,
    );
    const registry = buildRegistry([testScenario("known")]);
    assert.throws(
      () => selectScenarios(registry, { names: new Set(["missing"]) }),
      /unknown/,
    );
  });

  it("rejects requested candidates and target conflicts unless explicitly enabled", () => {
    const registry = buildRegistry([
      testScenario("candidate", { status: "candidate" }),
    ]);
    assert.throws(
      () => selectScenarios(registry, { names: new Set(["candidate"]) }),
      /excluded/,
    );
    assert.throws(
      () =>
        selectScenarios(registry, {
          names: new Set(["candidate"]),
          target: "area-2",
          includeCandidates: true,
        }),
      /excluded/,
    );
  });

  it("validates recording primary ownership for implemented techniques", () => {
    const first = testScenario("first-primary", {
      techniqueIds: ["1.2"],
      recording: { primaryFor: ["1.2"], startFrame: 2, endFrame: 8 },
    });
    const second = testScenario("second-primary", {
      techniqueIds: ["1.2"],
      recording: { primaryFor: ["1.2"], preRollFrames: 3, postRollFrames: 4 },
    });
    assert.throws(
      () => buildRegistry([first, second]),
      /multiple primary recordings/,
    );
    assert.throws(
      () =>
        buildRegistry(
          [
            testScenario("candidate-primary", {
              status: "candidate",
              techniqueIds: ["1.2"],
              recording: { primaryFor: ["1.2"], startFrame: 0, endFrame: 1 },
            }),
          ],
          { implementedTechniqueIds: new Set(["1.2"]) },
        ),
      /candidate cannot be primary/,
    );
    assert.throws(
      () =>
        buildRegistry([
          testScenario("invalid-window", {
            techniqueIds: ["1.2"],
            recording: { primaryFor: ["1.2"], startFrame: 4, endFrame: 2 },
          }),
        ]),
      /invalid absolute recording/,
    );
  });
});
