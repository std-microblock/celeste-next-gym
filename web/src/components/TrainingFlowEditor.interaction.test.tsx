import { fireEvent, render, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeAll, describe, expect, it, vi } from "vitest";
import { PLAYGROUND, createInitialState } from "../model";
import { createTrainingProject } from "../training/editorProject";
import { VISUAL_THEMES } from "../visualThemes";

vi.mock("./GameView", () => ({
  GameView: ({ children }: { children?: ReactNode }) => (
    <div className="game-screen">{children}</div>
  ),
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
    expect(within(container).getByText("END TRIGGER ID")).toBeInTheDocument();
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
    fireEvent.click(within(view.container).getByRole("button", { name: /录制当前区域/ }));
    fireEvent.click(within(view.container).getByRole("button", { name: /录制全部区域/ }));
    expect(onStartRecording).toHaveBeenNthCalledWith(1, { type: "module", index: 0 });
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
    const overlay = view.container.querySelector("svg.training-trigger-overlay")!;
    const originalX = project.training.modules[0].end_trigger.bounds.x;
    fireEvent.pointerDown(end, {
      pointerId: 1,
      clientX: originalX,
      clientY: project.training.modules[0].end_trigger.bounds.y,
    });
    fireEvent.pointerMove(overlay, {
      pointerId: 1,
      clientX: originalX + 16,
      clientY: project.training.modules[0].end_trigger.bounds.y,
    });
    expect(
      onChange.mock.calls.at(-1)?.[0].training.modules[0].end_trigger.bounds.x,
    ).toBe(originalX + 16);
    expect(
      onChange.mock.calls.at(-1)?.[0].training.modules[0].trigger.bounds.x,
    ).toBe(project.training.modules[0].trigger.bounds.x);
  });
});
