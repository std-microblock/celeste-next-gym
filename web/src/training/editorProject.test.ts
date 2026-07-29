import { describe, expect, it } from "vitest";
import { PLAYGROUND } from "../model";
import {
  createBlankGymMap,
  createTrainingProject,
  openTrainingWorkspace,
  saveTrainingWorkspace,
  validateTrainingProject,
} from "./editorProject";

function memoryDirectory() {
  const files = new Map<string, string>();
  const directory = {
    name: "training-workspace",
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
  return {
    files,
    directory: directory as unknown as FileSystemDirectoryHandle,
  };
}

describe("training editor projects", () => {
  it("starts new projects with only a floor and spawn point", () => {
    const map = createBlankGymMap();
    expect(map.bounds).toEqual({ x: 0, y: 0, width: 320, height: 180 });
    expect(map.solids).toEqual([{ x: 0, y: 152, width: 320, height: 28 }]);
    expect(map.entities).toEqual([]);
    expect(map.spawn).toEqual({ x: 32, y: 152 });
  });

  it("creates a valid map-owned training project", () => {
    const project = createTrainingProject(PLAYGROUND);
    expect(project.training.version).toBe(2);
    expect(project.training.modules).toHaveLength(1);
    expect(validateTrainingProject(project)).toEqual([]);
  });

  it("reports trigger and entry mistakes before invoking WASM", () => {
    const project = createTrainingProject(PLAYGROUND);
    project.training.finish.trigger.id = project.training.modules[0].trigger.id;
    project.training.modules[0].tutorial.fuzz.inputs[0].at = 2;
    const issues = validateTrainingProject(project);
    expect(issues.some((item) => item.path === "finish.trigger.id")).toBe(true);
    expect(issues.some((item) => item.path.endsWith("entry.input_id"))).toBe(
      true,
    );
  });

  it("round-trips a complete workspace folder", async () => {
    const memory = memoryDirectory();
    const project = createTrainingProject(PLAYGROUND);
    project.training.title = "文件夹训练图";
    await saveTrainingWorkspace(memory.directory, [project]);
    expect(memory.files.has("celeste-gym.workspace.json")).toBe(true);
    expect(memory.files.has(project.mapFileName)).toBe(true);
    expect(memory.files.has(project.trainingFileName)).toBe(true);
    const loaded = await openTrainingWorkspace(memory.directory, PLAYGROUND);
    expect(loaded).toHaveLength(1);
    expect(loaded[0].training).toEqual(project.training);
    expect(loaded[0].map).toEqual(project.map);
  });
});
