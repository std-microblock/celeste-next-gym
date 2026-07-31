import { cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { GymMap } from "./model";
import { PLAYGROUND } from "./model";
import { createBlankGymMap } from "./training/editorProject";

const wasm = vi.hoisted(() => ({
  loadRequests: [] as Array<{
    room: string;
    resolve: (map: GymMap) => void;
  }>,
}));

vi.mock("./simulator/wasmClient", () => ({
  WasmClient: class {
    ready() {
      return Promise.resolve();
    }

    loadMap(_url: string, room: string) {
      return new Promise<GymMap>((resolve) => {
        wasm.loadRequests.push({ room, resolve });
      });
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

function resolvePlaygroundLoads(): void {
  for (const request of wasm.loadRequests.splice(0)) {
    request.resolve({
      ...structuredClone(PLAYGROUND),
      room: request.room,
      name: `Playground / ${request.room}`,
    });
  }
}

afterEach(() => {
  cleanup();
  wasm.loadRequests.length = 0;
  vi.restoreAllMocks();
});

describe("App startup map loading", () => {
  it("does not replace an editor-selected map when Playground finishes loading later", async () => {
    const view = render(<App />);

    fireEvent.change(view.getByLabelText("页面模式"), {
      target: { value: "editor" },
    });
    fireEvent.click(view.getByRole("button", { name: "选择编辑地图" }));
    expect(view.getByTestId("simulation-map")).toHaveTextContent("Editor Room");

    await waitFor(() => expect(wasm.loadRequests).toHaveLength(2));
    resolvePlaygroundLoads();

    await waitFor(() =>
      expect(view.getByTestId("wasm-ready")).toHaveTextContent("true"),
    );
    expect(view.getByTestId("simulation-map")).toHaveTextContent("Editor Room");
  });

  it("uses the decoded Playground when no map has claimed the simulator", async () => {
    const view = render(<App />);

    await waitFor(() => expect(wasm.loadRequests).toHaveLength(2));
    resolvePlaygroundLoads();
    fireEvent.change(view.getByLabelText("页面模式"), {
      target: { value: "play" },
    });

    await waitFor(() =>
      expect(view.getByText("Playground / playground")).toBeInTheDocument(),
    );
  });
});
