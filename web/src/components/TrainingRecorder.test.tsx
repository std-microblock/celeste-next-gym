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
  it("starts on an action and writes the module when the end region is reached", async () => {
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
        scope={{ type: "module", index: 0 }}
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
    expect(onChange.mock.calls[0][0].training.modules[0].tutorial.fuzz.inputs).toEqual([
      { id: "dash", keys: ["dash"], at: 0, verify: true },
    ]);
    expect(view.getByText("录制完成")).toBeInTheDocument();
  });
});
