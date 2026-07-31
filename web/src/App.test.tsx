import { cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { GymMap, SimState } from "./model";
import { PLAYGROUND } from "./model";
import { createBlankGymMap } from "./training/editorProject";

const wasm = vi.hoisted(() => ({
  loadMap: vi.fn(),
}));

vi.mock("./simulator/wasmClient", () => ({
  WasmClient: class {
    ready() {
      return Promise.resolve();
    }

    loadMap(...args: unknown[]) {
      return wasm.loadMap(...args);
    }

    simulate(state: SimState) {
      return Promise.resolve([state]);
    }

    dispose() {}
  },
}));

vi.mock("./components/GameplaySprite", () => ({
  GameplayStrawberry: () => null,
}));
vi.mock("./components/GameView", () => ({
  GameView: () => <div data-testid="game-view" />,
}));
vi.mock("./components/InputTimeline", () => ({
  InputTimeline: () => null,
}));
vi.mock("./components/KeyBindings", () => ({
  KeyBindings: () => null,
}));
vi.mock("./components/StateInspector", () => ({
  StateInspector: () => null,
}));
vi.mock("./components/StartSettings", () => ({
  StartSettings: () => null,
}));
vi.mock("./components/TrainingGround", () => ({
  TrainingGround: () => <div data-testid="training-ground" />,
}));
vi.mock("./components/EditorWorkspace", () => ({
  EditorWorkspace: ({
    map,
    state,
    ready,
    experiencing,
    onMapChange,
    onExperienceChange,
  }: {
    map: GymMap;
    state: SimState;
    ready: boolean;
    experiencing: boolean;
    onMapChange: (map: GymMap) => void;
    onExperienceChange: (experiencing: boolean, map: GymMap) => void;
  }) => (
    <div>
      <span data-testid="simulation-map">{map.name}</span>
      <span data-testid="live-position-x">{state.pos.x}</span>
      <span data-testid="editor-experiencing">{String(experiencing)}</span>
      <span data-testid="wasm-ready">{String(ready)}</span>
      <button
        type="button"
        onClick={() =>
          onMapChange({
            ...createBlankGymMap("editor-room"),
            name: "Editor Room",
          })
        }
      >
        选择编辑地图
      </button>
      <button type="button" onClick={() => onExperienceChange(true, map)}>
        开始实时体验
      </button>
      <button type="button" onClick={() => onExperienceChange(false, map)}>
        停止实时体验
      </button>
      <button
        type="button"
        onClick={() => {
          const second = {
            ...createBlankGymMap("second-room"),
            name: "Second Room",
            spawn: { x: 100, y: 152 },
          };
          onMapChange(second);
          onExperienceChange(true, second);
        }}
      >
        切换并体验第二张地图
      </button>
    </div>
  ),
}));

import App from "./App";

afterEach(() => {
  cleanup();
  wasm.loadMap.mockClear();
  vi.restoreAllMocks();
});

describe("App startup map loading", () => {
  it("readies WASM without loading Playground.bin and retains the editor map", async () => {
    const view = render(<App />);

    await waitFor(() =>
      expect(view.getByTestId("training-ground")).toBeInTheDocument(),
    );
    fireEvent.change(view.getByLabelText("页面模式"), {
      target: { value: "editor" },
    });
    fireEvent.click(view.getByRole("button", { name: "选择编辑地图" }));
    expect(view.getByTestId("simulation-map")).toHaveTextContent("Editor Room");

    await waitFor(() =>
      expect(view.getByTestId("wasm-ready")).toHaveTextContent("true"),
    );
    expect(view.getByTestId("simulation-map")).toHaveTextContent("Editor Room");
    expect(wasm.loadMap).not.toHaveBeenCalled();
  });

  it("uses the bundled in-memory playground without binary decoding", async () => {
    const view = render(<App />);

    fireEvent.change(view.getByLabelText("页面模式"), {
      target: { value: "play" },
    });

    await waitFor(() =>
      expect(view.getByText(PLAYGROUND.name)).toBeInTheDocument(),
    );
    expect(wasm.loadMap).not.toHaveBeenCalled();
  });

  it("starts the selected editor map after stopping the previous experience", async () => {
    const view = render(<App />);
    fireEvent.change(view.getByLabelText("页面模式"), {
      target: { value: "editor" },
    });
    await waitFor(() =>
      expect(view.getByTestId("wasm-ready")).toHaveTextContent("true"),
    );

    fireEvent.click(view.getByRole("button", { name: "开始实时体验" }));
    fireEvent.click(view.getByRole("button", { name: "停止实时体验" }));
    fireEvent.click(
      view.getByRole("button", { name: "切换并体验第二张地图" }),
    );

    expect(view.getByTestId("simulation-map")).toHaveTextContent("Second Room");
    expect(view.getByTestId("live-position-x")).toHaveTextContent("100");
    expect(view.getByTestId("editor-experiencing")).toHaveTextContent("true");
  });
});
