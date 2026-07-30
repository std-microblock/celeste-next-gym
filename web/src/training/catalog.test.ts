import { describe, expect, it } from "vitest";
import { trainingCatalog, trainingCatalogSections } from "./catalog";

describe("bundled JSON training workspaces", () => {
  it("builds the complete Hyper catalog from workspace JSON", () => {
    const hyper = trainingCatalog.find((technique) => technique.id === "hyper");
    expect(hyper?.variants.map((variant) => variant.id)).toEqual([
      "route",
      "spike-gap",
      "bubble-exit",
    ]);
    expect(hyper?.variants[0].map.room).toBe("hyper-route");
    expect(hyper?.variants[0].training.id).toBe("hyper-route");
    expect(hyper?.variants[2].initial.state).toBe("Boost");
    expect(trainingCatalogSections[0]).toMatchObject({
      id: "dash-tech",
      title: "冲刺技巧",
      badge: "DASH TECH",
    });
  });
});
