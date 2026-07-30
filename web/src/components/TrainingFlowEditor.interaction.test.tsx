import { fireEvent, render, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeAll, describe, expect, it, vi } from "vitest";
import type { GameViewViewport } from "../camera";
import { PLAYGROUND, createInitialState } from "../model";
import { createTrainingProject } from "../training/editorProject";
import { VISUAL_THEMES } from "../visualThemes";

vi.mock("./GameView", () => ({
  GameView: ({
    children,
    cameraPosition,
    cameraViewport,
  }: {
    children?: ReactNode | ((viewport: GameViewViewport) => ReactNode);
    cameraPosition?: { x: number; y: number };
    cameraViewport?: GameViewViewport["camera"];
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
      >
        {typeof children === "function" ? children(viewport) : children}
      </div>
    );
  },
}));
vi.mock("./TrainingGround", () => ({
  TrainingGround: () => <div data-testid="training-preview" />,
}));
vi.mock("../simulator/wasmClient", () => ({
  WasmClient: class {
    ready = async () => {};
    dispose = () => {};
    fuzzSearch = async () => ({ candidates: [], evaluations: [] });
  },
}));

import { TrainingFlowEditor } from "./TrainingFlowEditor";

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
      right: 960,
      bottom: 544,
      width: 960,
      height: 544,
      toJSON: () => ({}),
    }),
  });
});

function cameraScreenPoint(x: number, y: number) {
  return { clientX: x * 3, clientY: (y - 364) * 3 + 2 };
}

describe("training recording controls", () => {
  it("shows a start and end region for every module", () => {
    const project = createTrainingProject(PLAYGROUND);
    const { container } = render(
      <TrainingFlowEditor
        project={project}
        theme={VISUAL_THEMES[0]}
        bindings={{
          up: "KeyW",
          down: "KeyS",
          left: "KeyA",
          right: "KeyD",
          jump: "KeyL",
          dash: "Semicolon",
          crouch_dash: "KeyK",
          grab: "Quote",
        }}
        ready
        onChange={vi.fn()}
        onStartRecording={vi.fn()}
      />,
    );
    expect(container.querySelectorAll("rect.module-start")).toHaveLength(1);
    expect(container.querySelectorAll("rect.module-end")).toHaveLength(1);
    expect(
      container.querySelectorAll(".training-trigger-resize-handles rect"),
    ).toHaveLength(4);
    expect(within(container).getByText("END TRIGGER ID")).toBeInTheDocument();
    expect(container.querySelector(".training-trigger-overlay")).toHaveAttribute(
      "viewBox",
      "0 364 320 180",
    );
    fireEvent.click(
      within(container).getByRole("button", { name: "训练相机向右" }),
    );
    expect(container.querySelector(".training-trigger-overlay")).toHaveAttribute(
      "viewBox",
      "160 364 320 180",
    );
    fireEvent.click(
      within(container).getByRole("button", { name: "训练地图适配屏幕" }),
    );
    expect(container.querySelector(".game-screen")).toHaveAttribute(
      "data-camera-height",
      "544",
    );
  });

  it("pans the training trigger canvas by dragging empty map space", () => {
    const project = createTrainingProject(PLAYGROUND);
    const view = render(
      <TrainingFlowEditor
        project={project}
        theme={VISUAL_THEMES[0]}
        bindings={{
          up: "KeyW",
          down: "KeyS",
          left: "KeyA",
          right: "KeyD",
          jump: "KeyL",
          dash: "Semicolon",
          crouch_dash: "KeyK",
          grab: "Quote",
        }}
        ready
        onChange={vi.fn()}
        onStartRecording={vi.fn()}
      />,
    );
    const overlay = view.container.querySelector(
      "svg.training-trigger-overlay",
    )!;
    fireEvent.pointerDown(overlay, {
      pointerId: 9,
      clientX: 300,
      clientY: 180,
    });
    fireEvent.pointerMove(overlay, {
      pointerId: 9,
      clientX: 150,
      clientY: 180,
    });
    fireEvent.pointerUp(overlay, { pointerId: 9 });
    expect(overlay).toHaveAttribute("viewBox", "50 364 320 180");
  });

  it("edits recorded checkpoint conditions and objectives in form mode", () => {
    const project = createTrainingProject(PLAYGROUND);
    project.training.modules[0].tutorial.fuzz.checkpoints = [
      {
        id: "recorded-node-3",
        at: "jump_frame",
        description: "F3 · 跳跃后最大化 X 速度",
        success: ["after.pos.x >= 120"],
        objectives: [{ type: "maximize", expression: "after.speed.x" }],
      },
    ];
    const onChange = vi.fn();
    const view = render(
      <TrainingFlowEditor
        project={project}
        theme={VISUAL_THEMES[0]}
        bindings={{
          up: "KeyW",
          down: "KeyS",
          left: "KeyA",
          right: "KeyD",
          jump: "KeyL",
          dash: "Semicolon",
          crouch_dash: "KeyK",
          grab: "Quote",
        }}
        ready
        onChange={onChange}
        onStartRecording={vi.fn()}
      />,
    );
    const checkpoint = view.container.querySelector<HTMLElement>(
      ".fuzz-checkpoint-editor",
    )!;
    expect(within(checkpoint).getByText("CHECKPOINT 1")).toBeInTheDocument();
    expect(within(checkpoint).getByLabelText("目标 1 表达式")).toHaveValue(
      "after.speed.x",
    );

    fireEvent.change(
      within(checkpoint).getByLabelText(
        "CHECKPOINT SUCCESS · 每行一个 Rhai 条件",
      ),
      { target: { value: "after.pos.x >= 140" } },
    );
    expect(
      onChange.mock.calls.at(-1)?.[0].training.modules[0].tutorial.fuzz
        .checkpoints[0].success,
    ).toEqual(["after.pos.x >= 140"]);
  });

  it("starts either the selected or all-region recording session", () => {
    const project = createTrainingProject(PLAYGROUND);
    const onStartRecording = vi.fn();
    const view = render(
      <TrainingFlowEditor
        project={project}
        theme={VISUAL_THEMES[0]}
        bindings={{
          up: "KeyW",
          down: "KeyS",
          left: "KeyA",
          right: "KeyD",
          jump: "KeyL",
          dash: "Semicolon",
          crouch_dash: "KeyK",
          grab: "Quote",
        }}
        ready
        onChange={vi.fn()}
        onStartRecording={onStartRecording}
      />,
    );
    fireEvent.click(
      within(view.container).getByRole("button", { name: /录制当前区域/ }),
    );
    fireEvent.click(
      within(view.container).getByRole("button", { name: /录制全部区域/ }),
    );
    expect(onStartRecording).toHaveBeenNthCalledWith(1, {
      type: "module",
      index: 0,
    });
    expect(onStartRecording).toHaveBeenNthCalledWith(2, { type: "all" });
    expect(createInitialState(project.map).pos).toEqual(project.map.spawn);
  });

  it("moves a tutorial end region independently on the map", () => {
    const project = createTrainingProject(PLAYGROUND);
    const onChange = vi.fn();
    const view = render(
      <TrainingFlowEditor
        project={project}
        theme={VISUAL_THEMES[0]}
        bindings={{
          up: "KeyW",
          down: "KeyS",
          left: "KeyA",
          right: "KeyD",
          jump: "KeyL",
          dash: "Semicolon",
          crouch_dash: "KeyK",
          grab: "Quote",
        }}
        ready
        onChange={onChange}
        onStartRecording={vi.fn()}
      />,
    );
    const end = view.container.querySelector("rect.module-end")!;
    const overlay = view.container.querySelector(
      "svg.training-trigger-overlay",
    )!;
    const originalX = project.training.modules[0].end_trigger.bounds.x;
    fireEvent.pointerDown(end, {
      pointerId: 1,
      ...cameraScreenPoint(
        originalX,
        project.training.modules[0].end_trigger.bounds.y,
      ),
    });
    fireEvent.pointerMove(overlay, {
      pointerId: 1,
      ...cameraScreenPoint(
        originalX + 16,
        project.training.modules[0].end_trigger.bounds.y,
      ),
    });
    expect(
      onChange.mock.calls.at(-1)?.[0].training.modules[0].end_trigger.bounds.x,
    ).toBe(originalX + 16);
    expect(
      onChange.mock.calls.at(-1)?.[0].training.modules[0].trigger.bounds.x,
    ).toBe(project.training.modules[0].trigger.bounds.x);
  });

  it("resizes the selected start, end, and finish trigger with corner handles", () => {
    const project = createTrainingProject(PLAYGROUND);
    const onChange = vi.fn();
    const view = render(
      <TrainingFlowEditor
        project={project}
        theme={VISUAL_THEMES[0]}
        bindings={{
          up: "KeyW",
          down: "KeyS",
          left: "KeyA",
          right: "KeyD",
          jump: "KeyL",
          dash: "Semicolon",
          crouch_dash: "KeyK",
          grab: "Quote",
        }}
        ready
        onChange={onChange}
        onStartRecording={vi.fn()}
      />,
    );
    const overlay = view.container.querySelector(
      "svg.training-trigger-overlay",
    )!;
    const end = view.container.querySelector("rect.module-end")!;
    const original = project.training.modules[0].end_trigger.bounds;
    fireEvent.pointerDown(end, {
      pointerId: 2,
      ...cameraScreenPoint(original.x, original.y),
    });
    fireEvent.pointerUp(overlay, { pointerId: 2 });
    const southeast = view.container.querySelector(
      '.training-trigger-resize-handles rect[data-corner="se"]',
    )!;
    fireEvent.pointerDown(southeast, {
      pointerId: 3,
      ...cameraScreenPoint(
        original.x + original.width,
        original.y + original.height,
      ),
    });
    fireEvent.pointerMove(overlay, {
      pointerId: 3,
      ...cameraScreenPoint(
        original.x + original.width + 16,
        original.y + original.height + 8,
      ),
    });
    const resized =
      onChange.mock.calls.at(-1)?.[0].training.modules[0].end_trigger.bounds;
    expect(resized.width).toBe(original.width + 16);
    expect(resized.height).toBe(original.height + 8);

    fireEvent.click(
      within(view.container).getByRole("button", { name: /终点 Trigger/ }),
    );
    expect(
      view.container.querySelectorAll(".training-trigger-resize-handles rect"),
    ).toHaveLength(4);
  });
});
