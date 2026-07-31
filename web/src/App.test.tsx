import { cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { GymMap } from "./model";
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

    simulate() {
      return Promise.resolve([]);
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
    ready,
    onMapChange,
  }: {
    map: GymMap;
    ready: boolean;
    onMapChange: (map: GymMap) => void;
  }) => (
    <div>
      <span data-testid="simulation-map">{map.name}</span>
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
});
