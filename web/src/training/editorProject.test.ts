import { describe, expect, it } from "vitest";
import { PLAYGROUND } from "../model";
import {
  createBlankGymMap,
  createTrainingCatalogWorkspace,
  createTrainingProject,
  openTrainingCatalogWorkspace,
  openTrainingWorkspace,
  saveTrainingCatalogWorkspace,
  saveTrainingWorkspace,
  validateTrainingProject,
} from "./editorProject";

interface MemoryDirectoryNode {
  files: Map<string, string>;
  directories: Map<string, MemoryDirectoryNode>;
  directory: FileSystemDirectoryHandle;
}

function memoryDirectory() {
  const makeDirectory = (name: string): MemoryDirectoryNode => {
    const files = new Map<string, string>();
    const directories = new Map<string, MemoryDirectoryNode>();
    const directory = {
      name,
      kind: "directory" as const,
      async getFileHandle(fileName: string, options?: { create?: boolean }) {
        if (!files.has(fileName) && !options?.create)
          throw new DOMException("Not found", "NotFoundError");
        return {
          name: fileName,
          kind: "file" as const,
          async getFile() {
            return new File([files.get(fileName) ?? ""], fileName, {
              type: "application/json",
            });
          },
          async createWritable() {
            return {
              async write(value: string) {
                files.set(fileName, value);
              },
              async close() {},
            };
          },
        };
      },
      async getDirectoryHandle(
        directoryName: string,
        options?: { create?: boolean },
      ) {
        let child = directories.get(directoryName);
        if (!child && options?.create) {
          child = makeDirectory(directoryName);
          directories.set(directoryName, child);
        }
        if (!child) throw new DOMException("Not found", "NotFoundError");
        return child.directory;
      },
      async removeEntry(entryName: string) {
        if (!files.delete(entryName) && !directories.delete(entryName))
          throw new DOMException("Not found", "NotFoundError");
      },
      async *entries() {
        for (const fileName of files.keys())
          yield [fileName, await this.getFileHandle(fileName)] as const;
        for (const [directoryName, child] of directories)
          yield [directoryName, child.directory] as const;
      },
    };
    return {
      files,
      directories,
      directory: directory as unknown as FileSystemDirectoryHandle,
    };
  };
  const root = makeDirectory("training-workspace");
  return {
    files: root.files,
    directory: root.directory as unknown as FileSystemDirectoryHandle,
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

  it("migrates older modules to a per-tutorial end region", () => {
    const project = createTrainingProject(PLAYGROUND);
    const legacy = structuredClone(project.training);
    delete (legacy.modules[0] as Partial<typeof legacy.modules[0]>).end_trigger;
    const migrated = createTrainingProject(PLAYGROUND, legacy);
    expect(migrated.training.modules[0].end_trigger).toEqual({
      id: "lesson-1-end",
      bounds: migrated.training.finish.trigger.bounds,
    });
    expect(validateTrainingProject(migrated)).toEqual([]);
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
    project.initialModuleId = project.training.modules[0].id;
    await saveTrainingWorkspace(memory.directory, [project]);
    expect(memory.files.has("celeste-gym.workspace.json")).toBe(true);
    expect(memory.files.has(project.mapFileName)).toBe(true);
    expect(memory.files.has(project.trainingFileName)).toBe(true);
    const loaded = await openTrainingWorkspace(memory.directory, PLAYGROUND);
    expect(loaded).toHaveLength(1);
    expect(loaded[0].training).toEqual(project.training);
    expect(loaded[0].map).toEqual(project.map);
    expect(loaded[0].initialModuleId).toBe(project.initialModuleId);
  });

  it("round-trips the required root catalog and nested technique workspace", async () => {
    const memory = memoryDirectory();
    const catalog = createTrainingCatalogWorkspace();
    catalog.sections[0].title = "测试大分类";
    catalog.techniques[0].metadata.section.title = "测试大分类";
    catalog.techniques[0].metadata.title = "测试二级分类";
    await saveTrainingCatalogWorkspace(memory.directory, catalog);
    const loaded = await openTrainingCatalogWorkspace(
      memory.directory,
      PLAYGROUND,
    );
    expect(loaded.sections[0].title).toBe("测试大分类");
    expect(loaded.techniques[0].metadata.title).toBe("测试二级分类");
    expect(loaded.techniques[0].path).toBe("uncategorized/new-technique");
  });

  it("rejects a non-empty single-technique folder without a root catalog", async () => {
    const memory = memoryDirectory();
    await saveTrainingWorkspace(memory.directory, [
      createTrainingProject(PLAYGROUND),
    ]);
    await expect(
      openTrainingCatalogWorkspace(memory.directory, PLAYGROUND),
    ).rejects.toThrow("请选择完整训练目录");
  });
});
