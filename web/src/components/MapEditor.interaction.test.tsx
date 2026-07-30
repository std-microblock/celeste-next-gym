import { fireEvent, render, waitFor, within } from "@testing-library/react";
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
    cameraViewport,
    state,
  }: {
    children?: ReactNode | ((viewport: GameViewViewport) => ReactNode);
    cameraPosition?: { x: number; y: number };
    cameraViewport?: GameViewViewport["camera"];
    state: { pos: { x: number; y: number } };
  }) => {
    const viewport: GameViewViewport = {
      width: 320,
      height: 180,
      camera: cameraViewport ?? {
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
        data-camera-width={viewport.camera.width}
        data-camera-height={viewport.camera.height}
        data-state-x={state.pos.x}
        data-state-y={state.pos.y}
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
  Object.defineProperty(SVGSVGElement.prototype, "getBoundingClientRect", {
    configurable: true,
    value: () => ({
      x: 0,
      y: 0,
      left: 0,
      top: 0,
      right: 640,
      bottom: 360,
      width: 640,
      height: 360,
      toJSON: () => ({}),
    }),
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

  it("drags, zooms, and fits the editor camera to the whole map", () => {
    const map = createBlankGymMap();
    map.bounds.width = 960;
    map.bounds.height = 270;
    const { container } = editor(map);
    const overlay = container.querySelector(".map-editor-overlay")!;
    const background = container.querySelector(".editor-map-hitarea")!;

    fireEvent.pointerDown(background, {
      clientX: 240,
      clientY: 120,
      pointerId: 4,
    });
    fireEvent.pointerMove(overlay, {
      clientX: 140,
      clientY: 120,
      pointerId: 4,
    });
    fireEvent.pointerUp(overlay, { pointerId: 4 });
    expect(container.querySelector(".game-screen")).toHaveAttribute(
      "data-camera-x",
      "50",
    );

    fireEvent.click(within(container).getByRole("button", { name: "放大地图" }));
    expect(container.querySelector(".game-screen")).toHaveAttribute(
      "data-camera-width",
      "256",
    );

    fireEvent.click(
      within(container).getByRole("button", { name: "地图适配屏幕" }),
    );
    expect(container.querySelector(".game-screen")).toHaveAttribute(
      "data-camera-width",
      "960",
    );
    expect(container.querySelector(".game-screen")).toHaveAttribute(
      "data-camera-height",
      "540",
    );
    expect(overlay).toHaveAttribute("viewBox", "0 -135 960 540");
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

  it("paints a continuous row of lethal, surface-aligned spikes while dragging", () => {
    const { container, onChange } = editor();
    fireEvent.click(within(container).getByRole("button", { name: "上刺" }));
    const overlay = container.querySelector(".map-editor-overlay")!;
    const background = container.querySelector(".editor-map-hitarea")!;
    fireEvent.pointerDown(background, {
      clientX: 80,
      clientY: 304,
      pointerId: 7,
    });
    fireEvent.pointerMove(overlay, {
      clientX: 144,
      clientY: 304,
      pointerId: 7,
    });
    fireEvent.pointerUp(overlay, { pointerId: 7 });

    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({
        entities: [40, 48, 56, 64, 72].map((x) =>
          expect.objectContaining({
            kind: "spikes",
            bounds: { x, y: 149, width: 8, height: 3 },
            direction: { x: 0, y: -1 },
          }),
        ),
      }),
    );
    expect(
      within(container).getByText("按住左键拖动可连续铺设，松开后仍保留当前画笔。"),
    ).toBeInTheDocument();
  });

  it("resets a dead real-time editor run immediately with R", () => {
    const map = createBlankGymMap();
    const onResetExperience = vi.fn();
    const view = render(
      <MapEditor
        map={map}
        state={{ ...createInitialState(map), dead: true, respawn_frames: 80 }}
        frame={12}
        theme={VISUAL_THEMES[0]}
        experiencing
        ready
        onChange={vi.fn()}
        onExperienceChange={vi.fn()}
        onResetExperience={onResetExperience}
      />,
    );
    fireEvent.keyDown(window, { code: "KeyR", key: "r" });
    expect(onResetExperience).toHaveBeenCalledTimes(1);
    view.unmount();
  });

  it("records, scrubs, and expands the complete player trajectory", async () => {
    const map = createBlankGymMap();
    const first = createInitialState(map);
    const second = {
      ...structuredClone(first),
      pos: { x: 52, y: 100 },
      ducking: true,
      state: "StDuck",
    };
    const third = {
      ...structuredClone(first),
      pos: { x: 76, y: 92 },
      state: "StDash",
    };
    const onExperienceChange = vi.fn();
    const renderEditor = (
      experiencing: boolean,
      frame: number,
      state: typeof first,
      states: typeof first[],
    ) => (
      <MapEditor
        map={map}
        state={state}
        frame={frame}
        states={states}
        stateFrameOffset={0}
        theme={VISUAL_THEMES[0]}
        experiencing={experiencing}
        ready
        onChange={vi.fn()}
        onExperienceChange={onExperienceChange}
        onResetExperience={vi.fn()}
      />
    );
    const view = render(renderEditor(true, 0, first, [first]));
    const screen = within(view.container);

    fireEvent.click(screen.getByRole("button", { name: "录制轨迹" }));
    view.rerender(renderEditor(true, 2, third, [first, second, third]));
    await waitFor(() =>
      expect(screen.getByRole("slider", { name: "轨迹进度" })).toHaveAttribute(
        "max",
        "2",
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "结束录制" }));
    view.rerender(renderEditor(false, 2, third, [first, second, third]));

    const progress = screen.getByRole("slider", { name: "轨迹进度" });
    expect(progress).toHaveAttribute("max", "2");
    fireEvent.change(progress, { target: { value: "1" } });
    expect(view.container.querySelector(".game-screen")).toHaveAttribute(
      "data-state-x",
      "52",
    );
    const currentCollider = view.container.querySelector(
      '.editor-trajectory-overlay rect.current[data-frame="1"]',
    );
    expect(currentCollider).toHaveAttribute("x", "48");
    expect(currentCollider).toHaveAttribute("y", "94");
    expect(currentCollider).toHaveAttribute("width", "8");
    expect(currentCollider).toHaveAttribute("height", "6");

    fireEvent.click(screen.getByRole("button", { name: "显示全部轨迹" }));
    expect(
      view.container.querySelectorAll(".editor-trajectory-overlay rect"),
    ).toHaveLength(3);
    expect(
      view.container.querySelector(".editor-trajectory-line"),
    ).toHaveAttribute("d", expect.stringContaining("L 76 92"));
  });
});
