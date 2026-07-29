import { act, fireEvent, render, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import {
  createBlankGymMap,
  createTrainingProject,
} from "../training/editorProject";
import { VISUAL_THEMES } from "../visualThemes";

vi.mock("../simulator/wasmClient", () => ({
  WasmClient: class {
    ready = async () => {};
    dispose = () => {};
    fuzzSearch = async () => {
      const candidate = {
        bindings: {},
        verified_inputs: [{ input_index: 0, frame: 0, keys: ["dash"] }],
        objective_values: [Number.NaN],
        successful: true,
        final_state: { speed: { x: 240, y: 0 } },
      };
      return { candidates: [candidate], evaluations: [candidate] };
    };
    entryCheck = async () => true;
    simulate = async (state: Record<string, unknown>) => [
      {
        ...state,
        state: "Dash",
        speed: { x: 240, y: 0 },
        dead: false,
      },
    ];
  },
}));
vi.mock("./GameView", () => ({
  GameView: ({
    children,
  }: {
    children?: ReactNode | ((viewport: unknown) => ReactNode);
  }) => (
    <div className="game-screen">
      {typeof children === "function" ? children({}) : children}
    </div>
  ),
}));
vi.mock("./TrainingPrompt", () => ({
  TrainingPrompt: ({ text, hidden }: { text: string; hidden?: boolean }) =>
    hidden ? null : <div data-testid="training-prompt">{text}</div>,
}));
vi.mock("./TrainingCatalogSidebar", () => ({
  TrainingCatalogSidebar: () => null,
  TrainingVariantThumbnail: () => null,
}));
vi.mock("./TrainingTimeline", () => ({
  TrainingResultTimeline: () => null,
  TrainingTimeline: () => <div data-testid="training-timeline" />,
}));
vi.mock("./GameplaySprite", () => ({
  GameplayStrawberry: () => null,
}));

import { TrainingGround } from "./TrainingGround";

describe("training R reset", () => {
  it("keeps stage transitions playing and can restart the whole map", async () => {
    const project = createTrainingProject(createBlankGymMap());
    const variant = {
      id: project.id,
      title: project.training.title,
      summary: project.training.summary,
      map: project.map,
      training: project.training,
      initial: project.training.modules[0].validation.initial_state,
    };
    const callbacks = new Map<number, (time: number) => void>();
    let nextAnimation = 0;
    vi.stubGlobal(
      "requestAnimationFrame",
      (callback: (time: number) => void) => {
        nextAnimation += 1;
        callbacks.set(nextAnimation, callback);
        return nextAnimation;
      },
    );
    vi.stubGlobal("cancelAnimationFrame", (id: number) => callbacks.delete(id));
    const view = render(
      <TrainingGround
        techniqueId="hyper"
        variantId={project.id}
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
        theme={VISUAL_THEMES[0]}
        onSelectTraining={vi.fn()}
        variantOverride={variant}
        editorPreview
      />,
    );
    await waitFor(() =>
      expect(
        view.container.querySelector(".stage-header h1"),
      ).toHaveTextContent("0/1 模块完成"),
    );
    expect(view.getByTestId("training-prompt")).toHaveTextContent("演示");
    expect(view.getByTestId("training-timeline")).toBeInTheDocument();

    let time = performance.now();
    const advance = async () => {
      time += 17;
      await act(async () => {
        const next = callbacks.entries().next().value;
        if (next) {
          callbacks.delete(next[0]);
          next[1](time);
        }
        await Promise.resolve();
        await Promise.resolve();
      });
    };
    for (let frame = 0; frame < 32; frame += 1) await advance();
    await waitFor(() =>
      expect(
        view.container.querySelector(".training-lesson-stages .active"),
      ).toHaveTextContent("辅助实操"),
    );
    expect(view.getByRole("button", { name: "Ⅱ" })).toBeInTheDocument();

    fireEvent.keyDown(window, { code: "Semicolon" });
    for (let frame = 0; frame < 12; frame += 1) await advance();
    await waitFor(() =>
      expect(
        view.container.querySelector(".training-lesson-stages .active"),
      ).toHaveTextContent("自由练习"),
    );
    expect(view.queryByTestId("training-prompt")).not.toBeInTheDocument();
    expect(view.getByRole("button", { name: "Ⅱ" })).toBeInTheDocument();

    fireEvent.keyUp(window, { code: "Semicolon" });
    await advance();
    fireEvent.keyDown(window, { code: "Semicolon" });
    for (let frame = 0; frame < 12; frame += 1) await advance();
    await waitFor(() =>
      expect(
        view.container.querySelector(".stage-header h1"),
      ).toHaveTextContent("1/1 模块完成"),
    );
    expect(view.queryByTestId("training-prompt")).not.toBeInTheDocument();
    expect(view.container).not.toHaveTextContent("NaN");
    expect(view.container).toHaveTextContent("已通过当前步骤条件");

    fireEvent.change(view.getByRole("combobox", { name: "R 重试位置" }), {
      target: { value: "start" },
    });
    fireEvent.keyDown(window, { code: "KeyR" });
    await waitFor(() =>
      expect(
        view.container.querySelector(".stage-header h1"),
      ).toHaveTextContent("0/1 模块完成"),
    );
    expect(view.getByTestId("training-prompt")).toHaveTextContent("演示");
    expect(
      view.container.querySelectorAll(".training-success-toast"),
    ).toHaveLength(0);
  });
});
