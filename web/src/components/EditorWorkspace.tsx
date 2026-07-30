import { useEffect, useMemo, useRef, useState } from "react";
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

interface BinImportState {
  fileName: string;
  bytes: ArrayBuffer;
  rooms: string[];
  room: string;
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
  onMapChange: (map: GymMap) => void;
  onExperienceChange: (experiencing: boolean) => void;
  onResetExperience: () => void;
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

  const deleteSection = async () => {
    if (!currentSection) return;
    const owned = catalog.techniques.filter(
      (technique) => technique.metadata.section.id === currentSection.id,
    );
    if (
      !window.confirm(
        `删除大分类“${currentSection.title}”？其中 ${owned.length} 个二级分类及其房间文件也会删除。`,
      )
    )
      return;
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

  const deleteTechnique = async () => {
    if (!currentTechnique) return;
    if (
      !window.confirm(
        `删除二级分类“${currentTechnique.metadata.title}”及其 ${currentTechnique.projects.length} 个房间？`,
      )
    )
      return;
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
    if (!window.confirm(`删除房间“${current.training.title}”？`)) return;
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
              地图
            </button>
            <button
              className={section === "training" ? "active" : ""}
              onClick={() => setSection("training")}
            >
              训练流程
            </button>
          </div>
          <select
            aria-label="房间项目"
            disabled={!currentTechnique?.projects.length}
            value={current ? projectIndex : ""}
            onChange={(event) => {
              const index = Number(event.target.value);
              setProjectIndex(index);
              if (currentTechnique?.projects[index])
                props.onMapChange(currentTechnique.projects[index].map);
            }}
          >
            {!current && <option value="">暂无房间</option>}
            {currentTechnique?.projects.map((project, index) => (
              <option value={index} key={`${project.id}-${index}`}>
                {project.training.title}
              </option>
            ))}
          </select>
          <button onClick={addProject}>新建房间</button>
          <button disabled={!current} onClick={() => void deleteProject()}>
            删除房间
          </button>
          <button onClick={() => void openFolder()}>打开目录</button>
          <button disabled={!current} onClick={() => importRef.current?.click()}>
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
            导出
          </button>
          <button
            disabled={!directory || saveState === "saving"}
            onClick={() => void saveNow()}
          >
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
          <label>
            <small>大分类</small>
            <select
              aria-label="大分类"
              value={sectionId}
              onChange={(event) => {
                const nextSectionId = event.target.value;
                setSectionId(nextSectionId);
                selectTechnique(
                  catalog.techniques.find(
                    (technique) =>
                      technique.metadata.section.id === nextSectionId,
                  ),
                );
              }}
            >
              {catalog.sections.map((candidate) => (
                <option key={candidate.id} value={candidate.id}>
                  {candidate.title}
                </option>
              ))}
            </select>
          </label>
          <label>
            <small>大分类名称</small>
            <input
              aria-label="大分类名称"
              disabled={!currentSection}
              value={currentSection?.title ?? ""}
              onChange={(event) => updateSection("title", event.target.value)}
            />
          </label>
          <label className="editor-badge-field">
            <small>徽标</small>
            <input
              aria-label="大分类徽标"
              disabled={!currentSection}
              value={currentSection?.badge ?? ""}
              onChange={(event) => updateSection("badge", event.target.value)}
            />
          </label>
          <button onClick={addSection}>新建大分类</button>
          <button disabled={!currentSection} onClick={() => void deleteSection()}>
            删除大分类
          </button>
          <span className="editor-catalog-divider" />
          <label>
            <small>二级分类</small>
            <select
              aria-label="二级分类"
              value={currentTechnique?.metadata.id ?? ""}
              onChange={(event) =>
                selectTechnique(
                  sectionTechniques.find(
                    (technique) => technique.metadata.id === event.target.value,
                  ),
                )
              }
            >
              {!sectionTechniques.length && <option value="">暂无二级分类</option>}
              {sectionTechniques.map((technique) => (
                <option key={technique.metadata.id} value={technique.metadata.id}>
                  {technique.metadata.title}
                </option>
              ))}
            </select>
          </label>
          <label>
            <small>二级分类名称</small>
            <input
              aria-label="二级分类名称"
              disabled={!currentTechnique}
              value={currentTechnique?.metadata.title ?? ""}
              onChange={(event) => updateTechnique("title", event.target.value)}
            />
          </label>
          <label className="editor-technique-summary">
            <small>简介</small>
            <input
              aria-label="二级分类简介"
              disabled={!currentTechnique}
              value={currentTechnique?.metadata.summary ?? ""}
              onChange={(event) => updateTechnique("summary", event.target.value)}
            />
          </label>
          <button disabled={!currentSection} onClick={addTechnique}>
            新建二级分类
          </button>
          <button disabled={!currentTechnique} onClick={() => void deleteTechnique()}>
            删除二级分类
          </button>
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
              <button onClick={() => setBinImport(null)}>取消</button>
              <button className="primary" onClick={() => void importBinRoom()}>
                导入所选房间
              </button>
            </div>
          </section>
        </div>
      )}
    </div>
  );
}
