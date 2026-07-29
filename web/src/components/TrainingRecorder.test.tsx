import { fireEvent, render, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { PLAYGROUND } from "../model";
import { createTrainingProject } from "../training/editorProject";
import { VISUAL_THEMES } from "../visualThemes";

vi.mock("./GameView", () => ({
  GameView: ({ children }: { children?: ReactNode }) => (
    <div className="game-screen">{children}</div>
  ),
}));

vi.mock("../simulator/wasmClient", () => ({
  WasmClient: class {
    ready = async () => {};
    dispose = () => {};
    simulate = async (state: Record<string, unknown>) => [
      {
        ...state,
        pos: { x: 150, y: 152 },
        dead: false,
      },
    ];
  },
}));

import { TrainingRecorder } from "./TrainingRecorder";

describe("training recorder runtime", () => {
  it("writes a module and resets the full record-all session with R", async () => {
    const project = createTrainingProject(PLAYGROUND);
    project.training.modules[0].end_trigger.bounds = {
      x: 140,
      y: 140,
      width: 40,
      height: 30,
    };
    const onChange = vi.fn();
    const callbacks: Array<(time: number) => void> = [];
    vi.stubGlobal("requestAnimationFrame", (callback: (time: number) => void) => {
      callbacks.push(callback);
      return callbacks.length;
    });
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
    await waitFor(() => expect(view.getByText("已暂停待命")).toBeInTheDocument());
    fireEvent.keyDown(window, { code: "Semicolon" });
    callbacks.shift()?.(1_000_000);
    await waitFor(() => expect(onChange).toHaveBeenCalledTimes(1));
    expect(
      onChange.mock.calls[0][0].training.modules[0].tutorial.fuzz.inputs,
    ).toEqual([{ id: "dash", keys: ["dash"], at: 0, verify: true }]);
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
