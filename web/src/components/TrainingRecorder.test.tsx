import { fireEvent, render, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { PLAYGROUND } from "../model";
import {
  createTrainingModule,
  createTrainingProject,
} from "../training/editorProject";
import { VISUAL_THEMES } from "../visualThemes";

vi.mock("./GameView", () => ({
  GameView: ({
    children,
  }: {
    children?:
      | ReactNode
      | ((viewport: { width: number; height: number }) => ReactNode);
  }) => (
    <div className="game-screen">
      {typeof children === "function"
        ? children({ width: 960, height: 544 })
        : children}
    </div>
  ),
}));

vi.mock("../simulator/wasmClient", () => ({
  WasmClient: class {
    calls = 0;
    ready = async () => {};
    dispose = () => {};
    simulate = async (state: Record<string, unknown>) => {
      this.calls += 1;
      return [
        {
          ...state,
          pos: this.calls >= 4 ? { x: 250, y: 152 } : { x: 150, y: 152 },
          dead: false,
        },
      ];
    };
  },
}));

import { TrainingRecorder } from "./TrainingRecorder";

describe("training recorder runtime", () => {
  it("records every region before opening record-all editing", async () => {
    const project = createTrainingProject(PLAYGROUND);
    project.training.modules = Array.from({ length: 3 }, (_, index) => {
      const module = createTrainingModule(project.map, index);
      module.trigger.bounds = { x: 0, y: 0, width: 200, height: 544 };
      module.end_trigger.bounds = { x: 140, y: 140, width: 40, height: 30 };
      return module;
    });
    project.training.finish.trigger.bounds = {
      x: 240,
      y: 140,
      width: 40,
      height: 30,
    };
    const onChange = vi.fn();
    let nextAnimationId = 0;
    const animations = new Map<number, (time: number) => void>();
    vi.stubGlobal(
      "requestAnimationFrame",
      (callback: (time: number) => void) => {
        const id = ++nextAnimationId;
        animations.set(id, callback);
        return id;
      },
    );
    vi.stubGlobal("cancelAnimationFrame", (id: number) => {
      animations.delete(id);
    });
    const view = render(
      <TrainingRecorder
        project={project}
        scope={{ type: "all" }}
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
        onChange={onChange}
        onExit={vi.fn()}
      />,
    );
    await waitFor(() =>
      expect(view.getByText("已暂停待命")).toBeInTheDocument(),
    );

    const recordModule = async (code: string, completed: number) => {
      fireEvent.keyDown(window, { code });
      const animation = animations.values().next().value;
      if (!animation) throw new Error("录制器没有请求动画帧");
      animations.clear();
      animation(performance.now() + 1_000);
      fireEvent.keyUp(window, { code });
      await waitFor(() =>
        expect(
          view.getByText(`录制全部 · ${completed}/3`),
        ).toBeInTheDocument(),
      );
    };

    await recordModule("Semicolon", 1);
    expect(
      view.container.querySelector(".training-record-objective-window"),
    ).not.toBeInTheDocument();
    await recordModule("KeyL", 2);
    expect(
      view.container.querySelector(".training-record-objective-window"),
    ).not.toBeInTheDocument();
    await recordModule("KeyK", 3);
    expect(
      view.container.querySelector(".training-record-objective-window"),
    ).not.toBeInTheDocument();
    expect(view.container.querySelector(".training-recorder-bar > span"))
      .toHaveTextContent("全部 3 个区域已录制；继续前往绿色终点区。");
    const finishAnimation = animations.values().next().value;
    if (!finishAnimation) throw new Error("录制器在结束区后停止了游玩");
    animations.clear();
    finishAnimation(performance.now() + 1_000);
    await waitFor(() =>
      expect(
        view.container.querySelector(".training-record-objective-window"),
      ).toBeInTheDocument(),
    );
    expect(view.container.querySelector(".training-recorder-bar > span"))
      .toHaveTextContent(
        "已到达终点；全部 3 个区域录制完成，现在开始编辑关键节点。",
      );

    for (let count = 1; count <= 3; count += 1) {
      fireEvent.click(view.getByRole("button", { name: "删除关键点" }));
      await waitFor(() => expect(onChange).toHaveBeenCalledTimes(count));
    }
    expect(view.getByText("录制完成")).toBeInTheDocument();
    view.unmount();
  });

  it("writes a module and resets the full record-all session with R", async () => {
    const project = createTrainingProject(PLAYGROUND);
    project.training.modules[0].end_trigger.bounds = {
      x: 140,
      y: 140,
      width: 40,
      height: 30,
    };
    project.training.finish.trigger.bounds = {
      x: 140,
      y: 140,
      width: 40,
      height: 30,
    };
    const onChange = vi.fn();
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
      <TrainingRecorder
        project={project}
        scope={{ type: "all" }}
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
        onChange={onChange}
        onExit={vi.fn()}
      />,
    );
    await waitFor(() =>
      expect(view.getByText("已暂停待命")).toBeInTheDocument(),
    );
    fireEvent.keyDown(window, { code: "Semicolon" });
    callbacks.shift()?.(1_000_000);
    await waitFor(() =>
      expect(view.getByText("目标节点 1/1")).toBeInTheDocument(),
    );
    expect(onChange).not.toHaveBeenCalled();
    expect(view.getByText("速度")).toBeInTheDocument();
    expect(view.getByText("资源")).toBeInTheDocument();
    expect(view.getByText("位置")).toBeInTheDocument();
    const speedTarget = view.getByRole("button", { name: /X 速度/ });
    const ySpeedTarget = view.getByRole("button", { name: /Y 速度/ });
    expect(speedTarget).toHaveTextContent("0 px/s");
    expect(view.getByRole("button", { name: "生成教程" })).toBeEnabled();
    fireEvent.click(speedTarget);
    expect(speedTarget).toHaveAttribute("aria-pressed", "true");
    expect(ySpeedTarget).toHaveAttribute("aria-pressed", "false");
    const speedMode = view.getByRole("combobox", {
      name: "X 速度目标方式",
    });
    expect(speedMode).toHaveValue("match_and_maximize");
    fireEvent.change(speedMode, { target: { value: "maximize" } });
    expect(speedMode).toHaveValue("maximize");
    fireEvent.click(view.getByRole("button", { name: /冲刺/ }));
    expect(
      view.getByRole("combobox", { name: "冲刺目标方式" }),
    ).toHaveValue("maximize");
    expect(
      view.container.querySelector(".training-record-objective-window"),
    ).toHaveStyle({ left: "198px" });
    fireEvent.click(view.getByRole("button", { name: "删除关键点" }));
    await waitFor(() => expect(onChange).toHaveBeenCalledTimes(1));
    expect(
      onChange.mock.calls[0][0].training.modules[0].tutorial.fuzz.inputs,
    ).toEqual([{ id: "dash", keys: ["dash"], at: 0, verify: true }]);
    expect(
      onChange.mock.calls[0][0].training.modules[0].tutorial.fuzz.checkpoints,
    ).toEqual([]);
    expect(view.getByText("录制完成")).toBeInTheDocument();
    fireEvent.keyDown(window, { code: "KeyR" });
    await waitFor(() => expect(onChange).toHaveBeenCalledTimes(2));
    expect(view.getByText("已暂停待命")).toBeInTheDocument();
    expect(view.getByText("录制全部 · 0/1")).toBeInTheDocument();
    expect(
      view.container.querySelector(".training-recorder-bar > span"),
    ).toHaveTextContent("教程 1 已待命；首个非 WASD 动作记为 F0。");
    expect(view.getByRole("button", { name: "导出教程 JSON" })).toBeDisabled();
    expect(
      onChange.mock.calls[1][0].training.modules[0].tutorial.fuzz.inputs[0].id,
    ).toBe("dash-entry");
  });
});
