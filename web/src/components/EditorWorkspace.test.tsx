import { fireEvent, render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  createInitialState,
  DEFAULT_BINDINGS,
  PLAYGROUND,
} from "../model";
import {
  createTrainingProject,
  saveTrainingWorkspace,
} from "../training/editorProject";
import { VISUAL_THEMES } from "../visualThemes";

vi.mock("./MapEditor", () => ({
  MapEditor: () => <div data-testid="map-editor" />,
}));
vi.mock("./TrainingFlowEditor", () => ({
  TrainingFlowEditor: ({ project }: { project: { training: { title: string } } }) => (
    <div data-testid="training-flow">{project.training.title}</div>
  ),
}));
vi.mock("./TrainingRecorder", () => ({
  TrainingRecorder: () => <div data-testid="training-recorder" />,
}));

import { EditorWorkspace } from "./EditorWorkspace";

function memoryDirectory() {
  const files = new Map<string, string>();
  const directory = {
    name: "shared-training-folder",
    kind: "directory" as const,
    async getFileHandle(name: string, options?: { create?: boolean }) {
      if (!files.has(name) && !options?.create)
        throw new DOMException("Not found", "NotFoundError");
      return {
        name,
        kind: "file" as const,
        async getFile() {
          return new File([files.get(name) ?? ""], name, {
            type: "application/json",
          });
        },
        async createWritable() {
          return {
            async write(value: string) {
              files.set(name, value);
            },
            async close() {},
          };
        },
      };
    },
    async *entries() {
      for (const name of files.keys())
        yield [name, await this.getFileHandle(name)] as const;
    },
  };
  return directory as unknown as FileSystemDirectoryHandle;
}

describe("editor folder workspace", () => {
  it("opens the paired training document and switches to the training flow", async () => {
    const directory = memoryDirectory();
    const project = createTrainingProject(PLAYGROUND);
    project.training.title = "共享文件夹训练";
    await saveTrainingWorkspace(directory, [project]);
    window.showDirectoryPicker = vi.fn(async () => directory);
    const onMapChange = vi.fn();
    const initial = createInitialState(PLAYGROUND);
    const view = render(
      <EditorWorkspace
        map={PLAYGROUND}
        state={initial}
        frame={0}
        states={[initial]}
        stateFrameOffset={0}
        theme={VISUAL_THEMES[0]}
        bindings={DEFAULT_BINDINGS}
        experiencing={false}
        ready
        onMapChange={onMapChange}
        onExperienceChange={vi.fn()}
        onResetExperience={vi.fn()}
      />,
    );

    fireEvent.click(view.getByRole("button", { name: "打开文件夹" }));

    await waitFor(() =>
      expect(view.getByTestId("training-flow")).toHaveTextContent(
        "共享文件夹训练",
      ),
    );
    expect(view.getByRole("button", { name: "训练流程" })).toHaveClass(
      "active",
    );
    expect(view.queryByTestId("map-editor")).not.toBeInTheDocument();
    expect(onMapChange).toHaveBeenLastCalledWith(project.map);
  });
});
