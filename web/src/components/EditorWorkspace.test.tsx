import {
  cleanup,
  fireEvent,
  render,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createInitialState,
  DEFAULT_BINDINGS,
  PLAYGROUND,
} from "../model";
import {
  createTrainingCatalogWorkspace,
  createTrainingProject,
  createTrainingTechniqueWorkspace,
  openTrainingCatalogWorkspace,
  saveTrainingCatalogWorkspace,
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

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function memoryDirectory() {
  const makeDirectory = (name: string): FileSystemDirectoryHandle => {
    const files = new Map<string, string>();
    const directories = new Map<string, FileSystemDirectoryHandle>();
    return {
      name,
      kind: "directory",
      async getFileHandle(fileName: string, options?: { create?: boolean }) {
        if (!files.has(fileName) && !options?.create)
          throw new DOMException("Not found", "NotFoundError");
        return {
          name: fileName,
          kind: "file",
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
        } as FileSystemFileHandle;
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
        return child;
      },
      async removeEntry(entryName: string) {
        if (!files.delete(entryName) && !directories.delete(entryName))
          throw new DOMException("Not found", "NotFoundError");
      },
      async *entries() {
        for (const [fileName] of files)
          yield [fileName, await this.getFileHandle(fileName)] as const;
        for (const entry of directories) yield entry;
      },
    } as FileSystemDirectoryHandle;
  };
  return makeDirectory("shared-training-folder");
}

describe("editor folder workspace", () => {
  it("opens the paired training document and switches to the training flow", async () => {
    const directory = memoryDirectory();
    const project = createTrainingProject(PLAYGROUND);
    project.training.title = "共享文件夹训练";
    const catalog = createTrainingCatalogWorkspace();
    catalog.techniques[0].projects = [project];
    await saveTrainingCatalogWorkspace(directory, catalog);
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
        wasmClient={{
          listMapRooms: vi.fn(),
          loadMapBytes: vi.fn(),
        }}
        experiencing={false}
        ready
        onMapChange={onMapChange}
        onExperienceChange={vi.fn()}
        onResetExperience={vi.fn()}
      />,
    );

    fireEvent.click(view.getByRole("button", { name: "打开目录" }));

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

  it("creates, renames, selects, and deletes catalog levels", async () => {
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
        wasmClient={{ listMapRooms: vi.fn(), loadMapBytes: vi.fn() }}
        experiencing={false}
        ready
        onMapChange={vi.fn()}
        onExperienceChange={vi.fn()}
        onResetExperience={vi.fn()}
      />,
    );

    fireEvent.click(view.getByRole("button", { name: "大分类菜单" }));
    fireEvent.click(
      within(view.getByRole("menu", { name: "大分类" })).getByRole("button", {
        name: "新建大分类",
      }),
    );
    fireEvent.change(view.getByLabelText("大分类名称"), {
      target: { value: "移动技巧" },
    });
    fireEvent.click(view.getByRole("button", { name: "完成" }));
    expect(view.getByRole("button", { name: "大分类菜单" })).toHaveTextContent(
      "移动技巧",
    );

    fireEvent.click(view.getByRole("button", { name: "二级分类菜单" }));
    fireEvent.click(
      within(view.getByRole("menu", { name: "二级分类" })).getByRole(
        "button",
        { name: "新建二级分类" },
      ),
    );
    fireEvent.change(view.getByLabelText("二级分类名称"), {
      target: { value: "超冲" },
    });
    fireEvent.click(view.getByRole("button", { name: "完成" }));
    expect(
      view.getByRole("button", { name: "二级分类菜单" }),
    ).toHaveTextContent("超冲");

    fireEvent.click(view.getByRole("button", { name: "二级分类菜单" }));
    fireEvent.click(
      within(view.getByRole("menu", { name: "二级分类" })).getByRole(
        "button",
        { name: "删除二级分类" },
      ),
    );
    fireEvent.click(view.getByRole("button", { name: "确认删除" }));
    await waitFor(() =>
      expect(
        view.getByRole("button", { name: "二级分类菜单" }),
      ).toHaveTextContent("暂无二级分类"),
    );

    fireEvent.click(view.getByRole("button", { name: "大分类菜单" }));
    expect(view.getByRole("menu", { name: "大分类" })).toBeInTheDocument();
    fireEvent.keyDown(document, { key: "Escape" });
    expect(
      view.queryByRole("menu", { name: "大分类" }),
    ).not.toBeInTheDocument();
  });

  it("renames catalog and room IDs, migrates paths, and updates references", async () => {
    const directory = memoryDirectory();
    const catalog = createTrainingCatalogWorkspace();
    const section = catalog.sections[0];
    const related = createTrainingTechniqueWorkspace(
      section,
      "related-technique",
      "关联技巧",
    );
    related.metadata.related = [catalog.techniques[0].metadata.id];
    catalog.techniques.push(related);
    await saveTrainingCatalogWorkspace(directory, catalog);
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
        wasmClient={{ listMapRooms: vi.fn(), loadMapBytes: vi.fn() }}
        experiencing={false}
        ready
        onMapChange={onMapChange}
        onExperienceChange={vi.fn()}
        onResetExperience={vi.fn()}
      />,
    );

    expect(view.getByRole("button", { name: "打开目录" }).querySelector("svg"))
      .not.toBeNull();
    fireEvent.click(view.getByRole("button", { name: "打开目录" }));
    await view.findByTestId("training-flow");

    fireEvent.click(view.getByRole("button", { name: "大分类菜单" }));
    let menu = view.getByRole("menu", { name: "大分类" });
    fireEvent.click(
      within(menu).getByRole("button", { name: "重命名大分类" }),
    );
    fireEvent.change(view.getByLabelText("大分类 ID"), {
      target: { value: "movement" },
    });
    fireEvent.click(within(menu).getByRole("button", { name: "应用 ID" }));
    await waitFor(() =>
      expect(view.getByText("大分类 ID 已改为 movement")).toBeInTheDocument(),
    );
    fireEvent.click(within(menu).getByRole("button", { name: "完成" }));

    fireEvent.click(view.getByRole("button", { name: "二级分类菜单" }));
    menu = view.getByRole("menu", { name: "二级分类" });
    fireEvent.click(
      within(menu).getByRole("button", { name: "重命名二级分类" }),
    );
    fireEvent.change(view.getByLabelText("二级分类 ID"), {
      target: { value: "hyper" },
    });
    fireEvent.click(within(menu).getByRole("button", { name: "应用 ID" }));
    await waitFor(() =>
      expect(view.getByText("二级分类 ID 已改为 hyper")).toBeInTheDocument(),
    );
    fireEvent.click(within(menu).getByRole("button", { name: "完成" }));

    fireEvent.click(view.getByRole("button", { name: "房间菜单" }));
    menu = view.getByRole("menu", { name: "房间" });
    fireEvent.click(within(menu).getByRole("button", { name: "重命名房间" }));
    fireEvent.change(view.getByLabelText("房间 ID"), {
      target: { value: "hyper-room" },
    });
    fireEvent.click(within(menu).getByRole("button", { name: "应用 ID" }));
    expect(onMapChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ room: "hyper-room" }),
    );
    fireEvent.click(within(menu).getByRole("button", { name: "完成" }));
    fireEvent.click(view.getByRole("button", { name: "保存" }));
    await waitFor(() =>
      expect(view.getByText(/已保存到 shared-training-folder/)).toBeInTheDocument(),
    );

    const loaded = await openTrainingCatalogWorkspace(directory, PLAYGROUND);
    expect(loaded.sections[0].id).toBe("movement");
    expect(loaded.techniques.map((technique) => technique.path)).toEqual([
      "movement/hyper",
      "movement/related-technique",
    ]);
    expect(loaded.techniques[0].metadata.section.id).toBe("movement");
    expect(loaded.techniques[0].metadata.id).toBe("hyper");
    expect(loaded.techniques[0].projects[0].map.room).toBe("hyper-room");
    expect(loaded.techniques[1].metadata.related).toEqual(["hyper"]);

    const oldSection = await directory.getDirectoryHandle("uncategorized");
    await expect(
      oldSection.getDirectoryHandle("new-technique"),
    ).rejects.toMatchObject({ name: "NotFoundError" });
    await expect(
      oldSection.getDirectoryHandle("related-technique"),
    ).rejects.toMatchObject({ name: "NotFoundError" });
    const movement = await directory.getDirectoryHandle("movement");
    await expect(movement.getDirectoryHandle("hyper")).resolves.toBeDefined();
  });

  it("rejects unsafe and duplicate IDs", async () => {
    const directory = memoryDirectory();
    const catalog = createTrainingCatalogWorkspace();
    const second = createTrainingProject(PLAYGROUND);
    second.map.room = "occupied-room";
    second.id = "second-project";
    catalog.techniques[0].projects.push(second);
    await saveTrainingCatalogWorkspace(directory, catalog);
    window.showDirectoryPicker = vi.fn(async () => directory);
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
        wasmClient={{ listMapRooms: vi.fn(), loadMapBytes: vi.fn() }}
        experiencing={false}
        ready
        onMapChange={vi.fn()}
        onExperienceChange={vi.fn()}
        onResetExperience={vi.fn()}
      />,
    );

    fireEvent.click(view.getByRole("button", { name: "打开目录" }));
    await view.findByTestId("training-flow");
    fireEvent.click(view.getByRole("button", { name: "房间菜单" }));
    const menu = view.getByRole("menu", { name: "房间" });
    fireEvent.click(within(menu).getByRole("button", { name: "重命名房间" }));
    fireEvent.change(view.getByLabelText("房间 ID"), {
      target: { value: "Bad Room" },
    });
    fireEvent.click(within(menu).getByRole("button", { name: "应用 ID" }));
    expect(view.getByRole("alert")).toHaveTextContent(
      "只能使用小写字母、数字、连字符和下划线",
    );
    fireEvent.change(view.getByLabelText("房间 ID"), {
      target: { value: "occupied-room" },
    });
    fireEvent.click(within(menu).getByRole("button", { name: "应用 ID" }));
    expect(view.getByRole("alert")).toHaveTextContent("已被使用");
  });

  it("lists rooms from a BIN and imports only the selected room", async () => {
    const initial = createInitialState(PLAYGROUND);
    const listMapRooms = vi.fn(async () => ["a-00", "b-01"]);
    const loadMapBytes = vi.fn(
      async (_bytes: ArrayBuffer, room: string, name: string) => ({
        ...PLAYGROUND,
        room,
        name,
      }),
    );
    const onMapChange = vi.fn();
    const view = render(
      <EditorWorkspace
        map={PLAYGROUND}
        state={initial}
        frame={0}
        states={[initial]}
        stateFrameOffset={0}
        theme={VISUAL_THEMES[0]}
        bindings={DEFAULT_BINDINGS}
        wasmClient={{ listMapRooms, loadMapBytes }}
        experiencing={false}
        ready
        onMapChange={onMapChange}
        onExperienceChange={vi.fn()}
        onResetExperience={vi.fn()}
      />,
    );
    const file = new File([new Uint8Array([1, 2, 3])], "chapter.bin");
    Object.defineProperty(file, "arrayBuffer", {
      value: async () => new ArrayBuffer(3),
    });
    const input = view.container.querySelector<HTMLInputElement>(
      'input[type="file"][accept^=".bin"]',
    );
    expect(input).not.toBeNull();
    fireEvent.change(input!, { target: { files: [file] } });

    const roomSelect = await view.findByLabelText("BIN 房间");
    fireEvent.change(roomSelect, { target: { value: "b-01" } });
    fireEvent.click(view.getByRole("button", { name: "导入所选房间" }));

    await waitFor(() => expect(loadMapBytes).toHaveBeenCalled());
    expect(loadMapBytes.mock.calls[0][1]).toBe("b-01");
    expect(view.getByRole("button", { name: "房间菜单" })).toHaveTextContent(
      "chapter.bin / b-01",
    );
    expect(onMapChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ room: "b-01" }),
    );

    fireEvent.click(view.getByRole("button", { name: "房间菜单" }));
    fireEvent.click(
      within(view.getByRole("menu", { name: "房间" })).getByRole("button", {
        name: "重命名房间",
      }),
    );
    fireEvent.change(view.getByLabelText("房间名称"), {
      target: { value: "泡泡出口练习" },
    });
    fireEvent.click(view.getByRole("button", { name: "完成" }));
    expect(view.getByRole("button", { name: "房间菜单" })).toHaveTextContent(
      "泡泡出口练习",
    );
    expect(onMapChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ name: "泡泡出口练习", room: "b-01" }),
    );
    fireEvent.click(view.getByRole("button", { name: "训练流程" }));
    expect(view.getByTestId("training-flow")).toHaveTextContent(
      "泡泡出口练习",
    );
    fireEvent.click(view.getByRole("button", { name: "房间菜单" }));
    fireEvent.click(
      within(view.getByRole("menu", { name: "房间" })).getByRole("button", {
        name: "删除房间",
      }),
    );
    fireEvent.click(view.getByRole("button", { name: "确认删除" }));
    await waitFor(() =>
      expect(view.getByRole("button", { name: "房间菜单" })).toHaveTextContent(
        "未命名房间",
      ),
    );
  });
});
