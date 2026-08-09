import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { parseActorOptions } from "../../gym-actors.js";

describe("persistent gym actor launcher", () => {
  it("defaults to one hidden repository-owned actor", () => {
    assert.deepEqual(parseActorOptions([]), {
      count: 1,
      areaId: 1,
      showWindows: false,
      smoke: false,
    });
  });

  it("accepts bounded parallel actors and an explicit Mod SID", () => {
    assert.deepEqual(
      parseActorOptions([
        "--actors",
        "4",
        "--area-id",
        "7",
        "--area-sid",
        "Example/TrainingMap",
        "--show-windows",
        "--smoke",
      ]),
      {
        count: 4,
        areaId: 7,
        areaSid: "Example/TrainingMap",
        showWindows: true,
        smoke: true,
      },
    );
  });

  it("rejects unsafe actor counts and unknown flags", () => {
    assert.throws(() => parseActorOptions(["--actors", "0"]), /1 through 32/);
    assert.throws(() => parseActorOptions(["--actors", "33"]), /1 through 32/);
    assert.throws(() => parseActorOptions(["--unknown"]), /unknown gym actor/);
  });
});
