import { act, fireEvent, render, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import {
  createBlankGymMap,
  createTrainingProject,
} from "../training/editorProject";
import { VISUAL_THEMES } from "../visualThemes";

const wasmBehavior = vi.hoisted(() => ({
  entryCheckPassed: true,
  entryChecks: 0,
}));

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
    entryCheck = async () => {
      wasmBehavior.entryChecks += 1;
      return wasmBehavior.entryCheckPassed;
    };
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
    state,
  }: {
    children?: ReactNode | ((viewport: unknown) => ReactNode);
    state: { pos: { x: number; y: number } };
  }) => (
    <div
      className="game-screen"
      data-player-x={state.pos.x}
      data-player-y={state.pos.y}
    >
      {typeof children === "function"
        ? children({ width: 320, height: 180 })
        : children}
    </div>
  ),
}));
vi.mock("./TrainingPrompt", () => ({
  mapPointTargetPercent: (
    map: { bounds: { x: number; y: number; width: number; height: number } },
    point: { x: number; y: number },
  ) => ({
    x: ((point.x - map.bounds.x) / map.bounds.width) * 100,
    y: ((point.y - map.bounds.y) / map.bounds.height) * 100,
  }),
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
  it("starts free, opens one guided segment, and keeps R in free practice", async () => {
    const project = createTrainingProject(createBlankGymMap());
    project.training.modules[0].end_trigger.bounds = {
      x: 0,
      y: 0,
      width: 320,
      height: 180,
    };
    for (let index = 1; index < 3; index += 1) {
      const module = structuredClone(project.training.modules[0]);
      module.id = `lesson-${index + 1}`;
      module.trigger.id = `${module.id}-start`;
      module.trigger.bounds.x = 190 + index * 48;
      module.end_trigger.id = `${module.id}-end`;
      module.tutorial.id = `${module.id}-tutorial`;
      module.tutorial.title = `教程 ${index + 1}`;
      module.tutorial.summary = `第 ${index + 1} 个训练的原始说明。`;
      module.validation.initial_state.pos.x = 200 + index * 40;
      project.training.modules.push(module);
    }
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
      ).toHaveTextContent("0/3 模块完成"),
    );
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
    expect(view.queryByTestId("training-prompt")).not.toBeInTheDocument();
    expect(
      view.container.querySelector(".training-lesson-stages"),
    ).not.toBeInTheDocument();
    expect(view.getByTestId("training-timeline")).toBeInTheDocument();
    const tutorialCards = view.getAllByRole("button", { name: /^教学：/ });
    expect(tutorialCards).toHaveLength(3);
    expect(tutorialCards[1]).toHaveTextContent("教程 2");
    expect(tutorialCards[1]).toHaveTextContent("第 2 个训练的原始说明");
    const tutorialButton = view.getByRole("button", {
      name: /教学：教程 1/,
    });
    expect(tutorialButton.parentElement).toHaveClass("game-screen");
    expect(
      tutorialButton.style.getPropertyValue("--tutorial-x"),
    ).toMatch(/%$/);
    expect(
      tutorialButton.style.getPropertyValue("--tutorial-y"),
    ).toMatch(/%$/);
    expect(tutorialButton).toHaveTextContent("触发区已激活");

    wasmBehavior.entryCheckPassed = false;
    wasmBehavior.entryChecks = 0;
    fireEvent.keyDown(window, { code: "Semicolon" });
    for (let frame = 0; frame < 4; frame += 1) await advance();
    fireEvent.keyUp(window, { code: "Semicolon" });
    wasmBehavior.entryCheckPassed = true;
    expect(wasmBehavior.entryChecks).toBeGreaterThan(0);
    expect(
      view.container.querySelector(".training-failure"),
    ).toBeInTheDocument();
    expect(view.getByRole("dialog", { name: "训练失败" })).toBeInTheDocument();
    expect(view.getByRole("button", { name: "R 重试" })).toBeInTheDocument();
    expect(
      view.container.querySelectorAll(".training-success-toast"),
    ).toHaveLength(0);

    fireEvent.click(view.getByRole("button", { name: "R 重试" }));
    expect(
      view.container.querySelector(".training-failure"),
    ).not.toBeInTheDocument();

    fireEvent.click(
      view.getByRole("button", {
        name: /教学：教程 1/,
      }),
    );
    expect(view.getByTestId("training-prompt")).toHaveTextContent("正在准备");
    for (let frame = 0; frame < 4; frame += 1) {
      await advance();
      if (view.queryByRole("button", { name: "下一步" })) break;
    }
    expect(view.getByTestId("training-prompt")).toHaveTextContent("演示 1/1");
    expect(view.getByRole("button", { name: "完成演示" })).toBeEnabled();
    expect(view.getByRole("button", { name: "退出教学" })).toBeInTheDocument();
    fireEvent.click(view.getByRole("button", { name: "退出教学" }));
    expect(
      view.container.querySelector(".training-lesson-stages"),
    ).not.toBeInTheDocument();
    expect(view.queryByTestId("training-prompt")).not.toBeInTheDocument();
    expect(view.container.querySelector(".play-button")).toHaveTextContent("▶");

    const playerXBeforeCompletion = view
      .container.querySelector(".game-screen")
      ?.getAttribute("data-player-x");
    fireEvent.keyUp(window, { code: "Semicolon" });
    await advance();
    fireEvent.keyDown(window, { code: "Semicolon" });
    for (let frame = 0; frame < 12; frame += 1) {
      await advance();
      if (
        view.container
          .querySelector(".stage-header h1")
          ?.textContent?.includes("1/3 模块完成")
      )
        break;
    }
    expect(
      view.container.querySelectorAll(".training-success-toast"),
    ).toHaveLength(1);
    await waitFor(() =>
      expect(
        view.container.querySelector(".stage-header h1"),
      ).toHaveTextContent("1/3 模块完成"),
    );
    expect(view.queryByTestId("training-prompt")).not.toBeInTheDocument();
    expect(view.container).not.toHaveTextContent("NaN");
    expect(view.container).toHaveTextContent("已通过当前步骤条件");
    expect(view.container.querySelector(".play-button")).toHaveTextContent("Ⅱ");
    expect(view.container).toHaveTextContent("地图进度");
    expect(view.getAllByRole("button", { name: /^教学：/ })).toHaveLength(3);
    expect(
      view.container
        .querySelector(".game-screen")
        ?.getAttribute("data-player-x"),
    ).toBe(playerXBeforeCompletion);

    const resetPosition = view.getByRole("combobox", { name: "R 重置位置" });
    expect(resetPosition).toHaveValue("current");
    fireEvent.change(resetPosition, { target: { value: "start" } });
    expect(resetPosition).toHaveValue("start");
    fireEvent.keyDown(window, { code: "KeyR" });
    await waitFor(() =>
      expect(
        view.container.querySelector(".stage-header h1"),
      ).toHaveTextContent("0/3 模块完成"),
    );
    expect(view.queryByTestId("training-prompt")).not.toBeInTheDocument();
    expect(
      view.container.querySelector(".training-lesson-stages"),
    ).not.toBeInTheDocument();
    expect(
      view.container.querySelectorAll(".training-success-toast"),
    ).toHaveLength(0);
    const reopenedTutorial = view.getByRole("button", {
      name: /教学：教程 1/,
    });
    fireEvent.click(reopenedTutorial);
    expect(view.getByTestId("training-prompt")).toHaveTextContent("演示 1/1");
    expect(view.getByRole("button", { name: "完成演示" })).toBeEnabled();
    expect(view.container).not.toHaveTextContent("观察这一步产生的结果");
  });
});
