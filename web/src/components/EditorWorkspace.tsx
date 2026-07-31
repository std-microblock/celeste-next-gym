import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import {
  FiArchive,
  FiCheck,
  FiChevronDown,
  FiDownload,
  FiEdit3,
  FiFolder,
  FiGitBranch,
  FiMap,
  FiPlus,
  FiSave,
  FiTrash2,
  FiUpload,
  FiX,
} from "react-icons/fi";
import type { KeyBindings, GymMap, SimState } from "../model";
import type { WasmClient } from "../simulator/wasmClient";
import {
  createBlankGymMap,
  createTrainingCatalogWorkspace,
  createTrainingDocument,
  createTrainingProject,
  createTrainingTechniqueWorkspace,
  openTrainingCatalogWorkspace,
  removeTrainingProjectFiles,
  removeTrainingTechniqueFiles,
  saveTrainingCatalogWorkspace,
  trainingSlug,
  type TrainingCatalogWorkspace,
  type TrainingProject,
  type TrainingTechniqueWorkspace,
} from "../training/editorProject";
import type { VisualTheme } from "../visualThemes";
import type { LiveRenderRefs } from "./GameView";
import { MapEditor } from "./MapEditor";
import { TrainingFlowEditor } from "./TrainingFlowEditor";
import {
  TrainingRecorder,
  type TrainingRecordingScope,
} from "./TrainingRecorder";

function downloadJson(name: string, value: unknown): void {
  const url = URL.createObjectURL(
    new Blob([`${JSON.stringify(value, null, 2)}\n`], {
      type: "application/json",
    }),
  );
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = name;
  anchor.click();
  URL.revokeObjectURL(url);
}

function uniqueId(label: string, existing: Iterable<string>, fallback: string) {
  const used = new Set(existing);
  const base = trainingSlug(label) === "training-map"
    ? fallback
    : trainingSlug(label);
  let candidate = base;
  let suffix = 2;
  while (used.has(candidate)) candidate = `${base}-${suffix++}`;
  return candidate;
}

const EDITABLE_ID_PATTERN = /^[a-z0-9][a-z0-9_-]*$/;

function editableIdError(
  label: string,
  value: string,
  existing: Iterable<string>,
): string | null {
  if (!value) return `${label}不能为空`;
  if (!EDITABLE_ID_PATTERN.test(value))
    return `${label}只能使用小写字母、数字、连字符和下划线`;
  if (new Set(existing).has(value)) return `${label}“${value}”已被使用`;
  return null;
}

function ButtonIcon({ children }: { children: ReactNode }) {
  return <span className="editor-button-icon" aria-hidden="true">{children}</span>;
}

interface BinImportState {
  fileName: string;
  bytes: ArrayBuffer;
  rooms: string[];
  room: string;
}

interface ManagedDropdownItem {
  id: string;
  label: string;
  description?: string;
}

function EditorManagedDropdown({
  id,
  label,
  selectedId,
  selectedLabel,
  items,
  open,
  disabled = false,
  onOpenChange,
  onSelect,
  onCreate,
  onDelete,
  renameContent,
  deleteDescription,
}: {
  id: string;
  label: string;
  selectedId?: string;
  selectedLabel: string;
  items: ManagedDropdownItem[];
  open: boolean;
  disabled?: boolean;
  onOpenChange(open: boolean): void;
  onSelect(id: string): void;
  onCreate(): void;
  onDelete(): void | Promise<void>;
  renameContent: ReactNode;
  deleteDescription: string;
}) {
  const [mode, setMode] = useState<"list" | "rename" | "delete">("list");
  const rootRef = useRef<HTMLDivElement>(null);
  const close = () => {
    setMode("list");
    onOpenChange(false);
  };

  useEffect(() => {
    if (!open) return;
    const dismiss = () => {
      setMode("list");
      onOpenChange(false);
    };
    const pointerDown = (event: PointerEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) dismiss();
    };
    const keyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") dismiss();
    };
    document.addEventListener("pointerdown", pointerDown);
    document.addEventListener("keydown", keyDown);
    return () => {
      document.removeEventListener("pointerdown", pointerDown);
      document.removeEventListener("keydown", keyDown);
    };
  }, [onOpenChange, open]);

  return (
    <div className="editor-managed-dropdown" ref={rootRef}>
      <button
        type="button"
        className="editor-managed-trigger"
        aria-label={`${label}菜单`}
        aria-expanded={open}
        aria-controls={`editor-managed-${id}`}
        disabled={disabled}
        onClick={() => {
          setMode("list");
          onOpenChange(!open);
        }}
      >
        <span>
          <small>{label}</small>
          <strong>{selectedLabel}</strong>
        </span>
        <FiChevronDown aria-hidden="true" />
      </button>
      {open && (
        <div
          id={`editor-managed-${id}`}
          className="editor-managed-menu"
          role="menu"
          aria-label={label}
        >
          {mode === "list" ? (
            <>
              <div className="editor-managed-list">
                {items.length ? (
                  items.map((item) => (
                    <button
                      type="button"
                      role="menuitemradio"
                      aria-checked={item.id === selectedId}
                      className={item.id === selectedId ? "active" : ""}
                      key={item.id}
                      onClick={() => {
                        onSelect(item.id);
                        close();
                      }}
                    >
                      <span>{item.label}</span>
                      {item.description && <small>{item.description}</small>}
                    </button>
                  ))
                ) : (
                  <p>暂无{label}</p>
                )}
              </div>
              <div className="editor-managed-actions">
                <button
                  type="button"
                  onClick={() => {
                    onCreate();
                    setMode("rename");
                  }}
                >
                  <ButtonIcon><FiPlus /></ButtonIcon>
                  新建{label}
                </button>
                <button
                  type="button"
                  disabled={!selectedId}
                  onClick={() => setMode("rename")}
                >
                  <ButtonIcon><FiEdit3 /></ButtonIcon>
                  重命名{label}
                </button>
                <button
                  type="button"
                  className="danger"
                  disabled={!selectedId}
                  onClick={() => setMode("delete")}
                >
                  <ButtonIcon><FiTrash2 /></ButtonIcon>
                  删除{label}
                </button>
              </div>
            </>
          ) : mode === "rename" ? (
            <div className="editor-managed-editor">
              <header>
                <strong>重命名{label}</strong>
                <small>名称和 ID 可分别修改；ID 需要明确应用</small>
              </header>
              {renameContent}
              <div className="editor-managed-confirm-actions">
                <button type="button" onClick={() => setMode("list")}>
                  <ButtonIcon><FiX /></ButtonIcon>
                  返回
                </button>
                <button type="button" className="primary" onClick={close}>
                  <ButtonIcon><FiCheck /></ButtonIcon>
                  完成
                </button>
              </div>
            </div>
          ) : (
            <div className="editor-managed-delete">
              <strong>确认删除{label}？</strong>
              <p>{deleteDescription}</p>
              <div className="editor-managed-confirm-actions">
                <button type="button" onClick={() => setMode("list")}>
                  <ButtonIcon><FiX /></ButtonIcon>
                  取消
                </button>
                <button
                  type="button"
                  className="danger"
                  onClick={() => void Promise.resolve(onDelete()).then(close)}
                >
                  <ButtonIcon><FiTrash2 /></ButtonIcon>
                  确认删除
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export interface EditorWorkspaceProps {
  map: GymMap;
  state: SimState;
  frame: number;
  states: readonly SimState[];
  stateFrameOffset: number;
  theme: VisualTheme;
  bindings: KeyBindings;
  wasmClient: Pick<WasmClient, "listMapRooms" | "loadMapBytes">;
  experiencing: boolean;
  ready: boolean;
  /** Live-render refs forwarded to the embedded GameView (editor experience). */
  liveRefs?: LiveRenderRefs;
  onMapChange: (map: GymMap) => void;
  onExperienceChange: (experiencing: boolean, map: GymMap) => void;
  onResetExperience: (map: GymMap) => void;
}

export function EditorWorkspace(props: EditorWorkspaceProps) {
  const [section, setSection] = useState<"map" | "training">("map");
  const [catalog, setCatalog] = useState<TrainingCatalogWorkspace>(() =>
    createTrainingCatalogWorkspace(),
  );
  const [sectionId, setSectionId] = useState("uncategorized");
  const [techniqueId, setTechniqueId] = useState("new-technique");
  const [projectIndex, setProjectIndex] = useState(0);
  const [directory, setDirectory] = useState<FileSystemDirectoryHandle | null>(
    null,
  );
  const [saveState, setSaveState] = useState<
    "memory" | "dirty" | "saving" | "saved" | "error"
  >("memory");
  const [notice, setNotice] = useState("当前为浏览器内存目录");
  const [recordingScope, setRecordingScope] =
    useState<TrainingRecordingScope | null>(null);
  const [binImport, setBinImport] = useState<BinImportState | null>(null);
  const [managedDropdown, setManagedDropdown] = useState<string | null>(null);
  const [sectionIdDraft, setSectionIdDraft] = useState("");
  const [techniqueIdDraft, setTechniqueIdDraft] = useState("");
  const [roomIdDraft, setRoomIdDraft] = useState("");
  const [sectionIdError, setSectionIdError] = useState<string | null>(null);
  const [techniqueIdError, setTechniqueIdError] = useState<string | null>(null);
  const [roomIdError, setRoomIdError] = useState<string | null>(null);
  const importRef = useRef<HTMLInputElement>(null);
  const binImportRef = useRef<HTMLInputElement>(null);
  const revision = useRef(0);

  const currentSection = catalog.sections.find(
    (candidate) => candidate.id === sectionId,
  );
  const sectionTechniques = useMemo(
    () =>
      catalog.techniques.filter(
        (candidate) => candidate.metadata.section.id === sectionId,
      ),
    [catalog.techniques, sectionId],
  );
  const currentTechnique = catalog.techniques.find(
    (candidate) => candidate.metadata.id === techniqueId,
  );
  const current = currentTechnique?.projects[projectIndex];
  const allProjects = catalog.techniques.flatMap(
    (technique) => technique.projects,
  );

  useEffect(() => {
    setSectionIdDraft(currentSection?.id ?? "");
    setSectionIdError(null);
  }, [currentSection?.id]);

  useEffect(() => {
    setTechniqueIdDraft(currentTechnique?.metadata.id ?? "");
    setTechniqueIdError(null);
  }, [currentTechnique?.metadata.id]);

  useEffect(() => {
    setRoomIdDraft(current?.map.room ?? current?.id ?? "");
    setRoomIdError(null);
  }, [current?.id, current?.map.room]);

  useEffect(() => {
    const first = catalog.techniques[0]?.projects[0];
    if (first) props.onMapChange(first.map);
    // The editor deliberately starts from a small blank room instead of the playground.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const markChanged = (next: TrainingCatalogWorkspace) => {
    revision.current += 1;
    setCatalog(next);
    if (directory) setSaveState("dirty");
  };

  const selectTechnique = (
    technique: TrainingTechniqueWorkspace | undefined,
    nextProjectIndex = 0,
  ) => {
    setTechniqueId(technique?.metadata.id ?? "");
    setProjectIndex(nextProjectIndex);
    const project = technique?.projects[nextProjectIndex];
    if (project) props.onMapChange(project.map);
  };

  const replaceTechnique = (technique: TrainingTechniqueWorkspace) => {
    markChanged({
      ...catalog,
      techniques: catalog.techniques.map((candidate) =>
        candidate.metadata.id === technique.metadata.id
          ? technique
          : candidate,
      ),
    });
  };

  const changeCurrent = (project: TrainingProject) => {
    if (!currentTechnique) return;
    replaceTechnique({
      ...currentTechnique,
      projects: currentTechnique.projects.map((candidate, index) =>
        index === projectIndex ? project : candidate,
      ),
    });
    props.onMapChange(project.map);
  };

  useEffect(() => {
    if (!directory || saveState !== "dirty") return;
    const savingRevision = revision.current;
    const timer = window.setTimeout(() => {
      setSaveState("saving");
      void saveTrainingCatalogWorkspace(directory, catalog)
        .then(() => {
          if (revision.current === savingRevision) {
            setSaveState("saved");
            setNotice(`已自动保存到 ${directory.name}`);
          } else {
            setSaveState("dirty");
          }
        })
        .catch((error: Error) => {
          setSaveState("error");
          setNotice(`自动保存失败：${error.message}`);
        });
    }, 500);
    return () => window.clearTimeout(timer);
  }, [catalog, directory, saveState]);

  const openFolder = async () => {
    if (!window.showDirectoryPicker) {
      setNotice("当前浏览器不支持 File System Access；可继续用 JSON 导入/导出");
      return;
    }
    try {
      const handle = await window.showDirectoryPicker({
        id: "celeste-gym-training-workspace",
        mode: "readwrite",
      });
      const loaded = await openTrainingCatalogWorkspace(handle, props.map);
      const firstSection = loaded.sections[0];
      const firstTechnique = loaded.techniques.find(
        (candidate) => candidate.metadata.section.id === firstSection?.id,
      );
      setDirectory(handle);
      revision.current += 1;
      setCatalog(loaded);
      setSectionId(firstSection?.id ?? "");
      selectTechnique(firstTechnique);
      setSection("training");
      setSaveState("saved");
      setNotice(
        `已打开 ${handle.name} · ${loaded.sections.length} 个大分类 · ${loaded.techniques.length} 个二级分类`,
      );
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") return;
      setSaveState("error");
      setNotice(error instanceof Error ? error.message : "文件夹打开失败");
    }
  };

  const saveNow = async () => {
    if (!directory) {
      setNotice("请先打开一个文件夹以启用自动保存");
      return;
    }
    try {
      const savingRevision = revision.current;
      setSaveState("saving");
      await saveTrainingCatalogWorkspace(directory, catalog);
      if (revision.current === savingRevision) {
        setSaveState("saved");
        setNotice(`已保存到 ${directory.name}`);
      } else {
        setSaveState("dirty");
      }
    } catch (error) {
      setSaveState("error");
      setNotice(error instanceof Error ? error.message : "保存失败");
    }
  };

  const importFiles = async (files: FileList) => {
    if (!current) {
      setNotice("请先创建并选择一个二级分类");
      return;
    }
    try {
      let map: GymMap | undefined;
      let training: TrainingProject["training"] | undefined;
      for (const file of Array.from(files)) {
        const value = JSON.parse(await file.text()) as Partial<GymMap> &
          Partial<TrainingProject["training"]>;
        if (
          Array.isArray(value.solids) &&
          Array.isArray(value.entities) &&
          value.bounds &&
          value.spawn
        )
          map = value as GymMap;
        if (value.version === 2 && Array.isArray(value.modules) && value.finish)
          training = value as TrainingProject["training"];
      }
      changeCurrent(
        createTrainingProject(
          map ?? current.map,
          training ?? current.training,
        ),
      );
      setNotice(`已导入 ${files.length} 个 JSON 文件`);
    } catch (error) {
      setNotice(
        error instanceof Error ? `导入失败：${error.message}` : "导入失败",
      );
    } finally {
      if (importRef.current) importRef.current.value = "";
    }
  };

  const inspectBin = async (file: File) => {
    try {
      setNotice(`正在读取 ${file.name} 的房间列表…`);
      const bytes = await file.arrayBuffer();
      const rooms = await props.wasmClient.listMapRooms(bytes);
      if (!rooms.length) throw new Error("地图中没有可导入的房间");
      setBinImport({ fileName: file.name, bytes, rooms, room: rooms[0] });
      setNotice(`${file.name} 包含 ${rooms.length} 个房间`);
    } catch (error) {
      setNotice(
        error instanceof Error ? `BIN 导入失败：${error.message}` : "BIN 导入失败",
      );
    } finally {
      if (binImportRef.current) binImportRef.current.value = "";
    }
  };

  const importBinRoom = async () => {
    if (!binImport || !currentTechnique) return;
    try {
      const room = uniqueId(
        binImport.room,
        allProjects.map((project) => project.map.room ?? project.id),
        `imported-room-${allProjects.length + 1}`,
      );
      const map = {
        ...(await props.wasmClient.loadMapBytes(
          binImport.bytes.slice(0),
          binImport.room,
          `${binImport.fileName} / ${binImport.room}`,
        )),
        room,
      };
      const id = uniqueId(
        room,
        allProjects.map((project) => project.id),
        `imported-room-${allProjects.length + 1}`,
      );
      const project = createTrainingProject(
        map,
        createTrainingDocument(map, id),
      );
      project.id = id;
      project.mapFileName = `${id}.map.json`;
      project.trainingFileName = `${id}.training.json`;
      const nextTechnique = {
        ...currentTechnique,
        projects: [...currentTechnique.projects, project],
      };
      replaceTechnique(nextTechnique);
      setProjectIndex(nextTechnique.projects.length - 1);
      props.onMapChange(map);
      setBinImport(null);
      setNotice(`已从 ${binImport.fileName} 导入房间 ${binImport.room}`);
    } catch (error) {
      setNotice(
        error instanceof Error ? `房间导入失败：${error.message}` : "房间导入失败",
      );
    }
  };

  const addSection = () => {
    const id = uniqueId(
      "category",
      catalog.sections.map((candidate) => candidate.id),
      `category-${catalog.sections.length + 1}`,
    );
    const nextSection = {
      id,
      title: `新大分类 ${catalog.sections.length + 1}`,
      badge: "CATEGORY",
    };
    markChanged({ ...catalog, sections: [...catalog.sections, nextSection] });
    setSectionId(id);
    selectTechnique(undefined);
    setNotice(`已创建大分类 ${nextSection.title}`);
  };

  const updateSection = (field: "title" | "badge", value: string) => {
    if (!currentSection) return;
    const updated = { ...currentSection, [field]: value };
    markChanged({
      sections: catalog.sections.map((candidate) =>
        candidate.id === sectionId ? updated : candidate,
      ),
      techniques: catalog.techniques.map((technique) =>
        technique.metadata.section.id === sectionId
          ? {
              ...technique,
              metadata: { ...technique.metadata, section: { ...updated } },
            }
          : technique,
      ),
    });
  };

  const persistCatalogIdChange = async (
    next: TrainingCatalogWorkspace,
    previousTechniques: readonly TrainingTechniqueWorkspace[],
    successNotice: string,
  ): Promise<boolean> => {
    if (directory) {
      try {
        setSaveState("saving");
        await saveTrainingCatalogWorkspace(directory, next);
      } catch (error) {
        setSaveState("error");
        setNotice(
          error instanceof Error
            ? `ID 修改失败：${error.message}`
            : "ID 修改失败",
        );
        return false;
      }
    }

    const stalePaths = previousTechniques.filter(
      (previous) =>
        !next.techniques.some((technique) => technique.path === previous.path),
    );
    const cleanupFailures: string[] = [];
    if (directory) {
      for (const previous of stalePaths) {
        try {
          await removeTrainingTechniqueFiles(directory, previous);
        } catch {
          cleanupFailures.push(previous.path);
        }
      }
    }

    markChanged(next);
    if (directory) setSaveState("saved");
    setNotice(
      cleanupFailures.length
        ? `${successNotice}；旧目录未能清理：${cleanupFailures.join("、")}`
        : successNotice,
    );
    return true;
  };

  const applySectionId = async () => {
    if (!currentSection) return;
    const nextId = sectionIdDraft.trim();
    const error = editableIdError(
      "大分类 ID",
      nextId,
      catalog.sections
        .filter((candidate) => candidate.id !== currentSection.id)
        .map((candidate) => candidate.id),
    );
    setSectionIdError(error);
    if (error) return;
    if (nextId === currentSection.id) {
      setNotice("大分类 ID 没有变化");
      return;
    }

    const previousTechniques = catalog.techniques.filter(
      (technique) => technique.metadata.section.id === currentSection.id,
    );
    const updatedSection = { ...currentSection, id: nextId };
    const next: TrainingCatalogWorkspace = {
      sections: catalog.sections.map((candidate) =>
        candidate.id === currentSection.id ? updatedSection : candidate,
      ),
      techniques: catalog.techniques.map((technique) =>
        technique.metadata.section.id === currentSection.id
          ? {
              ...technique,
              path: `${nextId}/${technique.metadata.id}`,
              metadata: {
                ...technique.metadata,
                section: { ...updatedSection },
              },
            }
          : technique,
      ),
    };
    if (
      await persistCatalogIdChange(
        next,
        previousTechniques,
        `大分类 ID 已改为 ${nextId}`,
      )
    ) {
      setSectionId(nextId);
      setSectionIdDraft(nextId);
      setSectionIdError(null);
    }
  };

  const deleteSection = async () => {
    if (!currentSection) return;
    const owned = catalog.techniques.filter(
      (technique) => technique.metadata.section.id === currentSection.id,
    );
    try {
      if (directory)
        for (const technique of owned)
          await removeTrainingTechniqueFiles(directory, technique);
      const next: TrainingCatalogWorkspace = {
        sections: catalog.sections.filter(
          (candidate) => candidate.id !== currentSection.id,
        ),
        techniques: catalog.techniques.filter(
          (technique) => technique.metadata.section.id !== currentSection.id,
        ),
      };
      markChanged(next);
      const fallbackSection = next.sections[0];
      setSectionId(fallbackSection?.id ?? "");
      selectTechnique(
        next.techniques.find(
          (technique) =>
            technique.metadata.section.id === fallbackSection?.id,
        ),
      );
      setNotice(`已删除大分类 ${currentSection.title}`);
    } catch (error) {
      setNotice(error instanceof Error ? `删除失败：${error.message}` : "删除失败");
    }
  };

  const addTechnique = () => {
    if (!currentSection) return;
    const id = uniqueId(
      "technique",
      catalog.techniques.map((candidate) => candidate.metadata.id),
      `technique-${catalog.techniques.length + 1}`,
    );
    const technique = createTrainingTechniqueWorkspace(
      currentSection,
      id,
      `新二级分类 ${sectionTechniques.length + 1}`,
      `${currentSection.id}/${id}`,
    );
    markChanged({
      ...catalog,
      techniques: [...catalog.techniques, technique],
    });
    selectTechnique(technique);
    setNotice(`已创建二级分类 ${technique.metadata.title}`);
  };

  const updateTechnique = (field: "title" | "summary", value: string) => {
    if (!currentTechnique) return;
    replaceTechnique({
      ...currentTechnique,
      metadata: { ...currentTechnique.metadata, [field]: value },
    });
  };

  const applyTechniqueId = async () => {
    if (!currentTechnique) return;
    const previousId = currentTechnique.metadata.id;
    const nextId = techniqueIdDraft.trim();
    const error = editableIdError(
      "二级分类 ID",
      nextId,
      catalog.techniques
        .filter((candidate) => candidate.metadata.id !== previousId)
        .map((candidate) => candidate.metadata.id),
    );
    setTechniqueIdError(error);
    if (error) return;
    if (nextId === previousId) {
      setNotice("二级分类 ID 没有变化");
      return;
    }

    const next: TrainingCatalogWorkspace = {
      ...catalog,
      techniques: catalog.techniques.map((technique) => {
        const related = technique.metadata.related.map((id) =>
          id === previousId ? nextId : id,
        );
        if (technique.metadata.id !== previousId)
          return {
            ...technique,
            metadata: { ...technique.metadata, related },
          };
        return {
          ...technique,
          path: `${technique.metadata.section.id}/${nextId}`,
          metadata: { ...technique.metadata, id: nextId, related },
        };
      }),
    };
    if (
      await persistCatalogIdChange(
        next,
        [currentTechnique],
        `二级分类 ID 已改为 ${nextId}`,
      )
    ) {
      setTechniqueId(nextId);
      setTechniqueIdDraft(nextId);
      setTechniqueIdError(null);
    }
  };

  const deleteTechnique = async () => {
    if (!currentTechnique) return;
    try {
      if (directory)
        await removeTrainingTechniqueFiles(directory, currentTechnique);
      const next = {
        ...catalog,
        techniques: catalog.techniques.filter(
          (candidate) => candidate.metadata.id !== currentTechnique.metadata.id,
        ),
      };
      markChanged(next);
      selectTechnique(
        next.techniques.find(
          (candidate) => candidate.metadata.section.id === sectionId,
        ),
      );
      setNotice(`已删除二级分类 ${currentTechnique.metadata.title}`);
    } catch (error) {
      setNotice(error instanceof Error ? `删除失败：${error.message}` : "删除失败");
    }
  };

  const addProject = () => {
    if (!currentTechnique) {
      setNotice("请先创建一个二级分类");
      return;
    }
    const id = uniqueId(
      "training-map",
      allProjects.map((project) => project.id),
      `training-map-${allProjects.length + 1}`,
    );
    const project = createTrainingProject(createBlankGymMap(`${id}-room`));
    project.id = id;
    project.training.id = id;
    project.mapFileName = `${id}.map.json`;
    project.trainingFileName = `${id}.training.json`;
    const nextTechnique = {
      ...currentTechnique,
      projects: [...currentTechnique.projects, project],
    };
    replaceTechnique(nextTechnique);
    setProjectIndex(nextTechnique.projects.length - 1);
    props.onMapChange(project.map);
    setNotice(`已创建房间项目 ${id}`);
  };

  const deleteProject = async () => {
    if (!currentTechnique || !current) return;
    try {
      if (directory)
        await removeTrainingProjectFiles(directory, currentTechnique, current);
      const projects = currentTechnique.projects.filter(
        (_, index) => index !== projectIndex,
      );
      const nextTechnique = { ...currentTechnique, projects };
      replaceTechnique(nextTechnique);
      const nextIndex = Math.min(projectIndex, Math.max(0, projects.length - 1));
      setProjectIndex(nextIndex);
      if (projects[nextIndex]) props.onMapChange(projects[nextIndex].map);
      setNotice(`已删除房间 ${current.training.title}`);
    } catch (error) {
      setNotice(error instanceof Error ? `删除失败：${error.message}` : "删除失败");
    }
  };

  const renameCurrentProject = (name: string) => {
    if (!current) return;
    changeCurrent({
      ...current,
      map: { ...current.map, name },
      training: { ...current.training, title: name },
    });
  };

  const applyRoomId = () => {
    if (!current) return;
    const nextId = roomIdDraft.trim();
    const error = editableIdError(
      "房间 ID",
      nextId,
      allProjects
        .filter((project) => project !== current)
        .map((project) => project.map.room ?? project.id),
    );
    setRoomIdError(error);
    if (error) return;
    if (nextId === (current.map.room ?? current.id)) {
      setNotice("房间 ID 没有变化");
      return;
    }
    changeCurrent({ ...current, map: { ...current.map, room: nextId } });
    setRoomIdDraft(nextId);
    setRoomIdError(null);
    setNotice(`房间 ID 已改为 ${nextId}`);
  };

  if (recordingScope && current) {
    return (
      <TrainingRecorder
        project={current}
        scope={recordingScope}
        bindings={props.bindings}
        theme={props.theme}
        onChange={(next) => {
          changeCurrent(next);
          setNotice("教程录制数据已更新；可继续录制其它区域或导出 JSON");
        }}
        onExit={() => setRecordingScope(null)}
      />
    );
  }

  return (
    <div className="editor-workspace-shell">
      <header className="editor-workspace-header">
        <nav className="editor-workspace-nav" aria-label="编辑器工作区">
          <div className="editor-section-tabs">
            <button
              className={section === "map" ? "active" : ""}
              onClick={() => setSection("map")}
            >
              <ButtonIcon><FiMap /></ButtonIcon>
              地图
            </button>
            <button
              className={section === "training" ? "active" : ""}
              onClick={() => setSection("training")}
            >
              <ButtonIcon><FiGitBranch /></ButtonIcon>
              训练流程
            </button>
          </div>
          <button onClick={() => void openFolder()}>
            <ButtonIcon><FiFolder /></ButtonIcon>
            打开目录
          </button>
          <button disabled={!current} onClick={() => importRef.current?.click()}>
            <ButtonIcon><FiUpload /></ButtonIcon>
            导入 JSON
          </button>
          <input
            ref={importRef}
            hidden
            multiple
            type="file"
            accept="application/json,.json"
            onChange={(event) =>
              event.target.files && void importFiles(event.target.files)
            }
          />
          <button
            disabled={!currentTechnique}
            onClick={() => binImportRef.current?.click()}
          >
            <ButtonIcon><FiArchive /></ButtonIcon>
            从 BIN 导入房间
          </button>
          <input
            ref={binImportRef}
            hidden
            type="file"
            accept=".bin,application/octet-stream"
            onChange={(event) =>
              event.target.files?.[0] && void inspectBin(event.target.files[0])
            }
          />
          <button
            disabled={!current}
            onClick={() => {
              if (!current) return;
              downloadJson(current.mapFileName, current.map);
              downloadJson(current.trainingFileName, current.training);
              setNotice("地图与训练脚本已导出");
            }}
          >
            <ButtonIcon><FiDownload /></ButtonIcon>
            导出
          </button>
          <button
            disabled={!directory || saveState === "saving"}
            onClick={() => void saveNow()}
          >
            <ButtonIcon><FiSave /></ButtonIcon>
            保存
          </button>
          <span className={`editor-save-state ${saveState}`}>
            <i />
            {saveState === "saving"
              ? "保存中"
              : saveState === "saved"
                ? "已自动保存"
                : saveState === "dirty"
                  ? "等待保存"
                  : saveState === "error"
                    ? "保存错误"
                    : "内存目录"}
          </span>
          <output title={notice}>{notice}</output>
        </nav>
        <nav className="editor-catalog-nav" aria-label="训练分类目录">
          <EditorManagedDropdown
            id="section"
            label="大分类"
            selectedId={currentSection?.id}
            selectedLabel={currentSection?.title ?? "暂无大分类"}
            items={catalog.sections.map((candidate) => ({
              id: candidate.id,
              label: candidate.title,
              description: candidate.badge,
            }))}
            open={managedDropdown === "section"}
            onOpenChange={(open) =>
              setManagedDropdown(open ? "section" : null)
            }
            onSelect={(nextSectionId) => {
              setSectionId(nextSectionId);
              selectTechnique(
                catalog.techniques.find(
                  (technique) =>
                    technique.metadata.section.id === nextSectionId,
                ),
              );
            }}
            onCreate={addSection}
            onDelete={deleteSection}
            deleteDescription={
              currentSection
                ? `“${currentSection.title}”下的 ${sectionTechniques.length} 个二级分类及其房间也会删除。`
                : "当前没有可删除的大分类。"
            }
            renameContent={
              <div className="editor-managed-fields">
                <label>
                  <span>ID</span>
                  <div className="editor-managed-id-row">
                    <input
                      aria-label="大分类 ID"
                      value={sectionIdDraft}
                      spellCheck={false}
                      onChange={(event) => {
                        setSectionIdDraft(event.target.value);
                        setSectionIdError(null);
                      }}
                    />
                    <button
                      type="button"
                      className="primary"
                      disabled={sectionIdDraft.trim() === currentSection?.id}
                      onClick={() => void applySectionId()}
                    >
                      <ButtonIcon><FiCheck /></ButtonIcon>
                      应用 ID
                    </button>
                  </div>
                  {sectionIdError && <em role="alert">{sectionIdError}</em>}
                </label>
                <label>
                  <span>名称</span>
                  <input
                    aria-label="大分类名称"
                    value={currentSection?.title ?? ""}
                    onChange={(event) =>
                      updateSection("title", event.target.value)
                    }
                  />
                </label>
                <label>
                  <span>徽标</span>
                  <input
                    aria-label="大分类徽标"
                    value={currentSection?.badge ?? ""}
                    onChange={(event) =>
                      updateSection("badge", event.target.value)
                    }
                  />
                </label>
              </div>
            }
          />
          <span className="editor-catalog-path">›</span>
          <EditorManagedDropdown
            id="technique"
            label="二级分类"
            selectedId={currentTechnique?.metadata.id}
            selectedLabel={currentTechnique?.metadata.title ?? "暂无二级分类"}
            items={sectionTechniques.map((technique) => ({
              id: technique.metadata.id,
              label: technique.metadata.title,
              description: `${technique.projects.length} 个房间`,
            }))}
            open={managedDropdown === "technique"}
            disabled={!currentSection}
            onOpenChange={(open) =>
              setManagedDropdown(open ? "technique" : null)
            }
            onSelect={(nextTechniqueId) =>
              selectTechnique(
                sectionTechniques.find(
                  (technique) => technique.metadata.id === nextTechniqueId,
                ),
              )
            }
            onCreate={addTechnique}
            onDelete={deleteTechnique}
            deleteDescription={
              currentTechnique
                ? `“${currentTechnique.metadata.title}”及其 ${currentTechnique.projects.length} 个房间文件会被删除。`
                : "当前没有可删除的二级分类。"
            }
            renameContent={
              <div className="editor-managed-fields">
                <label>
                  <span>ID</span>
                  <div className="editor-managed-id-row">
                    <input
                      aria-label="二级分类 ID"
                      value={techniqueIdDraft}
                      spellCheck={false}
                      onChange={(event) => {
                        setTechniqueIdDraft(event.target.value);
                        setTechniqueIdError(null);
                      }}
                    />
                    <button
                      type="button"
                      className="primary"
                      disabled={
                        techniqueIdDraft.trim() ===
                        currentTechnique?.metadata.id
                      }
                      onClick={() => void applyTechniqueId()}
                    >
                      <ButtonIcon><FiCheck /></ButtonIcon>
                      应用 ID
                    </button>
                  </div>
                  {techniqueIdError && <em role="alert">{techniqueIdError}</em>}
                </label>
                <label>
                  <span>名称</span>
                  <input
                    aria-label="二级分类名称"
                    value={currentTechnique?.metadata.title ?? ""}
                    onChange={(event) =>
                      updateTechnique("title", event.target.value)
                    }
                  />
                </label>
                <label>
                  <span>简介</span>
                  <textarea
                    aria-label="二级分类简介"
                    value={currentTechnique?.metadata.summary ?? ""}
                    onChange={(event) =>
                      updateTechnique("summary", event.target.value)
                    }
                  />
                </label>
              </div>
            }
          />
          <span className="editor-catalog-path">›</span>
          <EditorManagedDropdown
            id="room"
            label="房间"
            selectedId={current ? String(projectIndex) : undefined}
            selectedLabel={current?.training.title ?? "暂无房间"}
            items={(currentTechnique?.projects ?? []).map((project, index) => ({
              id: String(index),
              label: project.training.title,
              description: project.map.room ?? project.id,
            }))}
            open={managedDropdown === "room"}
            disabled={!currentTechnique}
            onOpenChange={(open) =>
              setManagedDropdown(open ? "room" : null)
            }
            onSelect={(indexText) => {
              const index = Number(indexText);
              setProjectIndex(index);
              if (currentTechnique?.projects[index])
                props.onMapChange(currentTechnique.projects[index].map);
            }}
            onCreate={addProject}
            onDelete={deleteProject}
            deleteDescription={
              current
                ? `“${current.training.title}”对应的地图和训练脚本文件会被删除。`
                : "当前没有可删除的房间。"
            }
            renameContent={
              <div className="editor-managed-fields">
                <label>
                  <span>房间名称</span>
                  <input
                    aria-label="房间名称"
                    value={current?.training.title ?? ""}
                    onChange={(event) =>
                      renameCurrentProject(event.target.value)
                    }
                  />
                </label>
                <label>
                  <span>房间 ID</span>
                  <div className="editor-managed-id-row">
                    <input
                      aria-label="房间 ID"
                      value={roomIdDraft}
                      spellCheck={false}
                      onChange={(event) => {
                        setRoomIdDraft(event.target.value);
                        setRoomIdError(null);
                      }}
                    />
                    <button
                      type="button"
                      className="primary"
                      disabled={
                        roomIdDraft.trim() ===
                        (current?.map.room ?? current?.id ?? "")
                      }
                      onClick={applyRoomId}
                    >
                      <ButtonIcon><FiCheck /></ButtonIcon>
                      应用 ID
                    </button>
                  </div>
                  {roomIdError && <em role="alert">{roomIdError}</em>}
                  <small>项目文件名保持不变。</small>
                </label>
              </div>
            }
          />
        </nav>
      </header>

      {!current ? (
        <main className="editor-empty-state">
          <strong>
            {currentTechnique
              ? "这个二级分类还没有房间"
              : "这个大分类还没有二级分类"}
          </strong>
          <span>
            {currentTechnique
              ? "点击“新建房间”，或从 Celeste .bin 导入一个房间。"
              : "点击“新建二级分类”，然后即可创建或导入房间。"}
          </span>
          <button
            disabled={!currentSection}
            onClick={currentTechnique ? addProject : addTechnique}
          >
            <ButtonIcon><FiPlus /></ButtonIcon>
            {currentTechnique ? "新建房间" : "新建二级分类"}
          </button>
        </main>
      ) : section === "map" ? (
        <MapEditor
          map={current.map}
          state={props.state}
          frame={props.frame}
          states={props.states}
          stateFrameOffset={props.stateFrameOffset}
          theme={props.theme}
          experiencing={props.experiencing}
          ready={props.ready}
          liveRefs={props.liveRefs}
          onChange={(map) => changeCurrent({ ...current, map })}
          onExperienceChange={props.onExperienceChange}
          onResetExperience={props.onResetExperience}
        />
      ) : (
        <TrainingFlowEditor
          project={current}
          theme={props.theme}
          bindings={props.bindings}
          ready={props.ready}
          onChange={changeCurrent}
          onStartRecording={(scope) => setRecordingScope(scope)}
        />
      )}

      {binImport && (
        <div className="editor-bin-dialog-backdrop" role="presentation">
          <section
            className="editor-bin-dialog"
            role="dialog"
            aria-modal="true"
            aria-label="从 BIN 导入房间"
          >
            <small>CELESTE BINARY MAP</small>
            <h2>选择要导入的房间</h2>
            <p>{binImport.fileName} · {binImport.rooms.length} 个房间</p>
            <label>
              <span>房间</span>
              <select
                aria-label="BIN 房间"
                value={binImport.room}
                onChange={(event) =>
                  setBinImport({ ...binImport, room: event.target.value })
                }
              >
                {binImport.rooms.map((room) => (
                  <option key={room} value={room}>{room}</option>
                ))}
              </select>
            </label>
            <div>
              <button onClick={() => setBinImport(null)}>
                <ButtonIcon><FiX /></ButtonIcon>
                取消
              </button>
              <button className="primary" onClick={() => void importBinRoom()}>
                <ButtonIcon><FiUpload /></ButtonIcon>
                导入所选房间
              </button>
            </div>
          </section>
        </div>
      )}
    </div>
  );
}
