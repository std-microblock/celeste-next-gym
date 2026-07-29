import { fireEvent, render, waitFor } from "@testing-library/react";
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
  TrainingPrompt: ({ text }: { text: string }) => (
    <div data-testid="training-prompt">{text}</div>
  ),
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
  it("restores the completed module, counter and entry prompt", async () => {
    const project = createTrainingProject(createBlankGymMap());
    const variant = {
      id: project.id,
      title: project.training.title,
      summary: project.training.summary,
      map: project.map,
      training: project.training,
      initial: project.training.modules[0].validation.initial_state,
    };
    const callbacks: Array<(time: number) => void> = [];
    vi.stubGlobal(
      "requestAnimationFrame",
      (callback: (time: number) => void) => {
        callbacks.push(callback);
        return callbacks.length;
      },
    );
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
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
    expect(view.getByTestId("training-prompt")).toHaveTextContent(
      project.training.modules[0].tutorial.entry.hint,
    );
    expect(view.getByTestId("training-timeline")).toBeInTheDocument();
    fireEvent.keyDown(window, { code: "Semicolon" });
    callbacks.shift()?.(1_000_000);
    await waitFor(() =>
      expect(
        view.container.querySelector(".stage-header h1"),
      ).toHaveTextContent("1/1 模块完成"),
    );
    expect(view.queryByTestId("training-prompt")).not.toBeInTheDocument();
    expect(view.container).not.toHaveTextContent("NaN");
    expect(view.container).toHaveTextContent("已通过当前步骤条件");

    fireEvent.keyDown(window, { code: "KeyR" });
    await waitFor(() =>
      expect(
        view.container.querySelector(".stage-header h1"),
      ).toHaveTextContent("0/1 模块完成"),
    );
    expect(view.getByTestId("training-prompt")).toHaveTextContent(
      project.training.modules[0].tutorial.entry.hint,
    );
    expect(
      view.container.querySelectorAll(".training-success-toast"),
    ).toHaveLength(0);
  });
});
