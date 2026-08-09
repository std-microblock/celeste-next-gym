import assert from "node:assert/strict";
import { resolve } from "node:path";
import { describe, it } from "node:test";

import {
  removeValidatedTarget,
  resolveRandomizerArchive,
} from "../runtime/prepare-mods.js";

describe("validated Mod replacement", () => {
  it("bounds transient Windows handle retries to the exact validated target", () => {
    const parent = resolve("D:\\repo", "vendor", "celeste-game", "Mods");
    const target = resolve(parent, "CelesteGymPlayground.zip");
    let attempts = 0;
    let waits = 0;
    removeValidatedTarget(target, parent, {
      remove: () => {
        attempts++;
        if (attempts < 3)
          throw Object.assign(new Error("busy"), { code: "EPERM" });
      },
      wait: (milliseconds) => {
        waits += milliseconds;
      },
    });
    assert.equal(attempts, 3);
    assert.equal(waits, 200);
  });
});

describe("Randomizer mod discovery", () => {
  it("prefers the explicit Randomizer archive without mutating other mods", () => {
    const configured = resolve("D:\\mods\\Randomizer.zip");
    const lookedUp: string[] = [];
    const result = resolveRandomizerArchive(
      { CELESTE_GYM_RANDOMIZER_ZIP: configured },
      (path) => {
        lookedUp.push(path);
        return path === configured;
      },
    );
    assert.equal(result, configured);
    assert.deepEqual(lookedUp, [configured]);
  });

  it("returns null when no known Randomizer archive exists", () => {
    assert.equal(resolveRandomizerArchive({}, () => false), null);
  });
});
