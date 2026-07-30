import { fireEvent, render, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeAll, describe, expect, it, vi } from "vitest";
import type { GameViewViewport } from "../camera";
import { createInitialState } from "../model";
import { createBlankGymMap } from "../training/editorProject";
import { VISUAL_THEMES } from "../visualThemes";

vi.mock("./GameView", () => ({
  GameView: ({
    children,
    cameraPosition,
  }: {
    children?: ReactNode | ((viewport: GameViewViewport) => ReactNode);
    cameraPosition?: { x: number; y: number };
  }) => {
    const viewport = {
      width: 320,
      height: 180,
      camera: {
        x: cameraPosition?.x ?? 0,
        y: cameraPosition?.y ?? 0,
        width: 320,
        height: 180,
      },
    };
    return (
      <div
        className="game-screen"
        data-camera-x={viewport.camera.x}
        data-camera-y={viewport.camera.y}
      >
        {typeof children === "function" ? children(viewport) : children}
      </div>
    );
  },
}));

import { MapEditor } from "./MapEditor";

beforeAll(() => {
  Object.defineProperty(Element.prototype, "setPointerCapture", {
    configurable: true,
    value: vi.fn(),
  });
});

function editor(map = createBlankGymMap(), onChange = vi.fn()) {
  return {
    onChange,
    ...render(
      <MapEditor
        map={map}
        state={createInitialState(map)}
        frame={0}
        theme={VISUAL_THEMES[0]}
        experiencing={false}
        ready
        onChange={onChange}
        onExperienceChange={() => {}}
        onResetExperience={() => {}}
      />,
    ),
  };
}

describe("MapEditor interactions", () => {
  it("shows four resize handles and deletes the selected object with Delete", () => {
    const { container, onChange } = editor();
    fireEvent.pointerDown(container.querySelector(".editor-object.solid")!, {
      clientX: 10,
      clientY: 10,
      pointerId: 1,
    });
    const handles = [
      ...container.querySelectorAll(".editor-resize-handles rect"),
    ];
    expect(handles).toHaveLength(4);
    expect(
      handles.every(
        (handle) =>
          handle.getAttribute("width") === "4" &&
          handle.getAttribute("height") === "4",
      ),
    ).toBe(true);
    fireEvent.keyDown(window, { key: "Delete" });
    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ solids: [] }),
    );
  });

  it("keeps the entity palette text-only", () => {
    const { container } = editor();
    expect(
      container.querySelector(
        ".editor-entity-atlas, .editor-entity-fallback, .editor-entity-material",
      ),
    ).toBeNull();
    expect(
      within(container).getByRole("button", { name: "Zip Mover" }),
    ).toBeInTheDocument();
  });

  it("pans a fixed 320x180 editor camera across wide maps", () => {
    const map = createBlankGymMap();
    map.bounds.width = 960;
    const { container } = editor(map);
    expect(container.querySelector(".map-editor-overlay")).toHaveAttribute(
      "viewBox",
      "0 0 320 180",
    );

    fireEvent.click(within(container).getByRole("button", { name: "相机向右" }));

    expect(container.querySelector(".game-screen")).toHaveAttribute(
      "data-camera-x",
      "160",
    );
    expect(container.querySelector(".map-editor-overlay")).toHaveAttribute(
      "viewBox",
      "160 0 320 180",
    );
  });

  it("edits room and entity properties from the inspector", () => {
    const map = createBlankGymMap();
    map.entities.push({
      kind: "zip_mover",
      bounds: { x: 40, y: 80, width: 32, height: 16 },
      direction: { x: 0, y: 0 },
      nodes: [{ x: 104, y: 80 }],
      name: "zipMover",
    });
    const first = editor(map);
    fireEvent.change(within(first.container).getByLabelText("ROOM NAME"), {
      target: { value: "短图" },
    });
    expect(first.onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ name: "短图" }),
    );
    fireEvent.pointerDown(
      first.container.querySelector(".editor-object.entity")!,
      { clientX: 10, clientY: 10, pointerId: 1 },
    );
    fireEvent.change(within(first.container).getByLabelText("NAME"), {
      target: { value: "movingPlatform" },
    });
    expect(first.onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({
        entities: [expect.objectContaining({ name: "movingPlatform" })],
      }),
    );
    expect(
      within(first.container).getByText("ZIP MOVER 终点"),
    ).toBeInTheDocument();
  });
});
