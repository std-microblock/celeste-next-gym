import { describe, expect, it } from "vitest";
import { PLAYGROUND } from "../model";
import {
  createEditorBrushEntity,
  createEditorEntity,
  deleteSelections,
  editorEntityHitBounds,
  entityBrushPoints,
  objectsInRegion,
  resizeEditorBounds,
  selectionBoundsUnion,
  selectionInList,
  selectionKey,
  setEditorSpikeDirection,
  snapCoordinate,
  snapToGrid,
} from "./MapEditor";

describe("map editor helpers", () => {
  it("snaps coordinates relative to the room origin", () => {
    expect(snapToGrid(18)).toBe(16);
    expect(snapToGrid(18, 2)).toBe(18);
    expect(snapToGrid(-5, -16)).toBe(-8);
  });

  it("creates simulator-ready entities from palette templates", () => {
    expect(createEditorEntity("spikes", 40, 80)).toEqual({
      kind: "spikes",
      bounds: { x: 40, y: 77, width: 32, height: 3 },
      direction: { x: 0, y: -1 },
      name: "spikesUp",
    });
    expect(createEditorEntity("unknown", 0, 0)).toBeNull();
    expect(createEditorEntity("refill", 40, 80)).toEqual({
      kind: "refill",
      bounds: { x: 40, y: 80, width: 16, height: 16 },
      direction: { x: 0, y: 0 },
      name: "refill",
    });
    expect(createEditorEntity("falling-block", 40, 80)).toEqual({
      kind: "falling_block",
      bounds: { x: 40, y: 80, width: 32, height: 16 },
      direction: { x: 1, y: 0 },
      name: "fallingBlock",
    });
  });

  it("exposes all four spike directions and the crystal spinner", () => {
    expect(createEditorEntity("spikes-down", 8, 16)).toEqual(
      expect.objectContaining({
        name: "spikesDown",
        direction: { x: 0, y: 1 },
        bounds: { x: 8, y: 16, width: 32, height: 3 },
      }),
    );
    expect(createEditorEntity("spikes-left", 8, 16)).toEqual(
      expect.objectContaining({
        name: "spikesLeft",
        direction: { x: -1, y: 0 },
        bounds: { x: 5, y: 16, width: 3, height: 32 },
      }),
    );
    expect(createEditorEntity("crystal-spinner", 24, 32)).toEqual({
      kind: "crystal_static_spinner",
      bounds: { x: 24, y: 32, width: 16, height: 12 },
      direction: { x: 0, y: 0 },
      name: "spinner",
    });
  });

  it("rotates spike bounds while preserving their length and gives thin spikes a usable hit area", () => {
    const spikes = createEditorEntity("spikes-up", 40, 80)!;
    const left = setEditorSpikeDirection(spikes, { x: -1, y: 0 });
    expect(left).toEqual(
      expect.objectContaining({
        name: "spikesLeft",
        direction: { x: -1, y: 0 },
        bounds: { x: 37, y: 80, width: 3, height: 32 },
      }),
    );
    expect(editorEntityHitBounds(spikes)).toEqual({
      x: 40,
      y: 74,
      width: 32,
      height: 9,
    });
    expect(editorEntityHitBounds(left)).toEqual({
      x: 34,
      y: 80,
      width: 9,
      height: 32,
    });
  });

  it("anchors upward and leftward spike colliders on the painted surface", () => {
    const up = createEditorEntity("spikes-up", 40, 80)!;
    const left = createEditorEntity("spikes-left", 40, 80)!;
    expect(up.bounds.y + up.bounds.height).toBe(80);
    expect(left.bounds.x + left.bounds.width).toBe(40);
  });

  it("fills every crossed editor grid cell with one-tile brush entities", () => {
    expect(
      entityBrushPoints({ x: 8, y: 16 }, { x: 40, y: 16 }, PLAYGROUND),
    ).toEqual([
      { x: 8, y: 16 },
      { x: 16, y: 16 },
      { x: 24, y: 16 },
      { x: 32, y: 16 },
      { x: 40, y: 16 },
    ]);
    expect(createEditorBrushEntity("spikes-up", 24, 80)).toEqual(
      expect.objectContaining({
        bounds: { x: 24, y: 77, width: 8, height: 3 },
      }),
    );
    expect(createEditorBrushEntity("crystal-spinner", 24, 32, "red")).toEqual(
      expect.objectContaining({ variant: "red" }),
    );
  });

  it("creates zip movers with a movable destination node", () => {
    expect(createEditorEntity("zip_mover", 40, 80)).toEqual({
      kind: "zip_mover",
      bounds: { x: 40, y: 80, width: 32, height: 16 },
      direction: { x: 0, y: 0 },
      nodes: [{ x: 104, y: 80 }],
      name: "zipMover",
    });
  });

  it("resizes from every corner on the editor grid", () => {
    const bounds = { x: 40, y: 40, width: 32, height: 24 };
    expect(
      resizeEditorBounds(bounds, "nw", { x: 17, y: 25 }, PLAYGROUND),
    ).toEqual({ x: 16, y: 24, width: 56, height: 40 });
    expect(
      resizeEditorBounds(bounds, "se", { x: 91, y: 83 }, PLAYGROUND),
    ).toEqual({ x: 40, y: 40, width: 48, height: 40 });
  });

  it("snaps to whole pixels by default and to the 8px grid on demand", () => {
    expect(snapCoordinate(18, 0, false)).toBe(18);
    expect(snapCoordinate(18.6, 0, false)).toBe(19);
    expect(snapCoordinate(18, 0, true)).toBe(16);
    expect(snapCoordinate(18, 2, true)).toBe(18);
    expect(snapCoordinate(-5, -16, true)).toBe(-8);
  });

  it("tracks multi-selection membership and the union of selected bounds", () => {
    const a = { type: "solid" as const, index: 0 };
    const b = { type: "entity" as const, index: 1 };
    expect(selectionKey(a)).toBe("solid:0");
    expect(selectionInList([a, b], a)).toBe(true);
    expect(selectionInList([a], b)).toBe(false);
    expect(selectionBoundsUnion(PLAYGROUND, [])).toBeNull();
    expect(selectionBoundsUnion(PLAYGROUND, [a])).toEqual(
      PLAYGROUND.solids[0],
    );
    expect(
      selectionBoundsUnion(PLAYGROUND, [
        a,
        { type: "entity", index: 0 },
      ]),
    ).toEqual({
      x: 0,
      y: 400,
      width: 960,
      height: 144,
    });
  });

  it("finds every solid and entity intersecting a region", () => {
    const inside = objectsInRegion(PLAYGROUND, {
      x: 0,
      y: 240,
      width: 300,
      height: 400,
    });
    expect(inside).toEqual(
      expect.arrayContaining([
        { type: "solid", index: 0 },
        { type: "entity", index: 0 },
      ]),
    );
    const none = objectsInRegion(PLAYGROUND, {
      x: 400,
      y: 400,
      width: 8,
      height: 8,
    });
    expect(none).toEqual([]);
  });

  it("deletes all selected solids and entities at once", () => {
    const map = deleteSelections(PLAYGROUND, [
      { type: "solid", index: 0 },
      { type: "entity", index: 0 },
      { type: "entity", index: 1 },
    ]);
    expect(map.solids.length).toBe(PLAYGROUND.solids.length - 1);
    expect(map.entities.length).toBe(PLAYGROUND.entities.length - 2);
    expect(map.entities.some((entity) => entity.name === "jumpThru")).toBe(
      false,
    );
  });

});
