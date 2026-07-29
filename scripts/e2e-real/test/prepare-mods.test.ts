import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { describe, it } from "node:test";

import {
  removeValidatedTarget,
  stagePlaygroundAssembly,
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

  it("stages the Playground custom entity assembly into Code", () => {
    const root = mkdtempSync(join(tmpdir(), "celeste-gym-trigger-mod-"));
    try {
      const source = resolve(root, "source");
      const built = resolve(
        source,
        "Source",
        "bin",
        "Release",
        "net8.0",
      );
      mkdirSync(built, { recursive: true });
      writeFileSync(resolve(built, "CelesteGymPlayground.dll"), "assembly");
      const staged = resolve(root, "staged");
      stagePlaygroundAssembly(source, staged);
      assert.equal(
        readFileSync(resolve(staged, "Code", "CelesteGymPlayground.dll"), "utf8"),
        "assembly",
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
