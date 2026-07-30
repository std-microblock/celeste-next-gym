import {
  ACTIONS,
  createInitialState,
  type GymMap,
  type SimState,
} from "../model";
import type {
  TrainingDocument,
  TrainingMapDocument,
  TrainingModule,
  TrainingTechniqueDocument,
} from "./catalog";
import { trainingEntryInput, verifiedInputs } from "./session";

export interface TrainingProject {
  id: string;
  mapFileName: string;
  trainingFileName: string;
  initialModuleId?: string;
  map: GymMap;
  training: TrainingMapDocument;
}

export interface TrainingWorkspaceManifest {
  version: 1;
  projects: Array<{
    id: string;
    map: string;
    training: string;
    initial_module?: string;
  }>;
}

export interface TrainingCatalogSection {
  id: string;
  title: string;
  badge: string;
}

export interface TrainingTechniqueWorkspace {
  path: string;
  metadata: TrainingTechniqueDocument;
  projects: TrainingProject[];
}

export interface TrainingCatalogWorkspace {
  sections: TrainingCatalogSection[];
  techniques: TrainingTechniqueWorkspace[];
}

interface TrainingCatalogManifest {
  version: 1;
  sections: TrainingCatalogSection[];
  techniques: Array<{ path: string }>;
}

export interface ProjectValidationIssue {
  path: string;
  message: string;
  severity: "error" | "warning";
}

const WORKSPACE_MANIFEST = "celeste-gym.workspace.json";
const CATALOG_MANIFEST = "celeste-gym.catalog.json";
const TECHNIQUE_DOCUMENT = "technique.json";
const DIRECTION_KEYS = new Set(["up", "down", "left", "right"]);

export function createBlankGymMap(id = "untitled-room"): GymMap {
  return {
    name: "未命名房间",
    room: id,
    bounds: { x: 0, y: 0, width: 320, height: 180 },
    spawn: { x: 32, y: 152 },
    solids: [{ x: 0, y: 152, width: 320, height: 28 }],
    entities: [],
    source_package: null,
  };
}

export function trainingSlug(value: string): string {
  return (
    value
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9_-]+/g, "-")
      .replace(/^-+|-+$/g, "") || "training-map"
  );
}

function defaultFuzz(): TrainingDocument["fuzz"] {
  return {
    version: 1,
    inputs: [{ id: "dash-entry", keys: ["dash"], at: 0, verify: true }],
    variables: [],
    observe_until: 8,
    success: ["!final.dead"],
    objectives: [{ type: "maximize", expression: "final.speed.x" }],
    search: { bindings: {}, output: ["best", "windows", "coverage"] },
  };
}

export function createTrainingModule(map: GymMap, index = 0): TrainingModule {
  const id = `lesson-${index + 1}`;
  const initial = createInitialState(map);
  initial.on_ground = true;
  const startX = map.spawn.x - 24;
  const endX = Math.min(
    map.bounds.x + Math.max(0, map.bounds.width - 64),
    startX + 128,
  );
  return {
    id,
    trigger: {
      id: `${id}-start`,
      bounds: {
        x: startX,
        y: map.spawn.y - 40,
        width: 64,
        height: 48,
      },
    },
    end_trigger: {
      id: `${id}-end`,
      bounds: {
        x: endX,
        y: map.spawn.y - 40,
        width: 64,
        height: 48,
      },
    },
    tutorial: {
      version: 2,
      id: `${id}-tutorial`,
      title: `教程 ${index + 1}`,
      summary: "说明这个模块要教授的技巧。",
      entry: {
        input_id: "dash-entry",
        hint: "触发区已激活：下一个动作请按 Dash。",
        check: ["!current.dead"],
        failure: { title: "入口动作不正确", body: "请按提示完成第一个动作。" },
      },
      teaching: {
        steps: [
          {
            prompt: "按 Dash。",
            order_error: {
              title: "动作顺序不正确",
              body: "请按当前提示输入。",
            },
            window_error: {
              title: "错过输入窗口",
              body: "请在绿色可行区间内输入。",
            },
          },
        ],
      },
      assist: {
        result_sample_after_input_frames: 0,
        auto_slowdown: {
          enabled_by_default: true,
          radius_frames: 12,
          minimum_multiplier: 0.85,
        },
      },
      fuzz: defaultFuzz(),
    },
    validation: { initial_state: initial },
  };
}

export function createTrainingDocument(
  map: GymMap,
  id = trainingSlug(map.room ?? map.name),
): TrainingMapDocument {
  return {
    version: 2,
    id,
    title: map.name,
    summary: "一张可实时测试的训练地图。",
    modules: [createTrainingModule(map)],
    finish: {
      trigger: {
        id: `${id}-finish`,
        bounds: {
          x: map.bounds.x + Math.max(0, map.bounds.width - 96),
          y: map.bounds.y,
          width: 96,
          height: map.bounds.height,
        },
      },
      require_all_modules: true,
    },
  };
}

export function createTrainingProject(
  map: GymMap,
  training = createTrainingDocument(map),
): TrainingProject {
  const id = trainingSlug(training.id);
  const normalized = normalizeTrainingDocument(map, training);
  return {
    id,
    mapFileName: `${id}.map.json`,
    trainingFileName: `${id}.training.json`,
    map: structuredClone(map),
    training: normalized,
  };
}

export function createTrainingTechniqueWorkspace(
  section: TrainingCatalogSection = {
    id: "uncategorized",
    title: "未分类",
    badge: "GENERAL",
  },
  id = "new-technique",
  title = "新二级分类",
  path = `${section.id}/${id}`,
): TrainingTechniqueWorkspace {
  return {
    path,
    metadata: {
      id,
      title,
      summary: "说明这个技巧分类包含的训练内容。",
      related: [],
      section: { ...section },
    },
    projects: [
      createTrainingProject(createBlankGymMap(`${id}-room-1`)),
    ],
  };
}

export function createTrainingCatalogWorkspace(): TrainingCatalogWorkspace {
  const section: TrainingCatalogSection = {
    id: "uncategorized",
    title: "未分类",
    badge: "GENERAL",
  };
  return {
    sections: [section],
    techniques: [createTrainingTechniqueWorkspace(section)],
  };
}

/** Adds editor recording regions to training files created before end_trigger existed. */
export function normalizeTrainingDocument(
  map: GymMap,
  training: TrainingMapDocument,
): TrainingMapDocument {
  const normalized = structuredClone(training);
  normalized.modules.forEach((module, index) => {
    if (module.end_trigger) return;
    const nextStart = normalized.modules[index + 1]?.trigger.bounds;
    const fallback = normalized.finish.trigger.bounds;
    const bounds = nextStart ?? fallback;
    module.end_trigger = {
      id: `${module.id}-end`,
      bounds: { ...bounds },
    };
  });
  // Keep the parameter in the migration signature so later schema migrations
  // can use map bounds without changing every workspace loading call.
  void map;
  return normalized;
}

function issue(
  path: string,
  message: string,
  severity: ProjectValidationIssue["severity"] = "error",
): ProjectValidationIssue {
  return { path, message, severity };
}

export function validateTrainingProject(
  project: Pick<TrainingProject, "map" | "training">,
): ProjectValidationIssue[] {
  const result: ProjectValidationIssue[] = [];
  const { map, training } = project;
  if (training.version !== 2)
    result.push(issue("version", "训练地图版本必须为 2"));
  if (!training.id.trim()) result.push(issue("id", "地图脚本 ID 不能为空"));
  if (!training.modules.length)
    result.push(issue("modules", "至少需要一个教程模块"));
  if (map.bounds.width <= 0 || map.bounds.height <= 0)
    result.push(issue("map.bounds", "地图边界尺寸必须为正数"));

  const moduleIds = new Set<string>();
  const triggerIds = new Set<string>();
  for (const [moduleIndex, module] of training.modules.entries()) {
    const base = `modules[${moduleIndex}]`;
    if (!module.id.trim()) result.push(issue(`${base}.id`, "模块 ID 不能为空"));
    else if (moduleIds.has(module.id))
      result.push(issue(`${base}.id`, `模块 ID ${module.id} 重复`));
    moduleIds.add(module.id);
    if (!module.trigger.id.trim())
      result.push(issue(`${base}.trigger.id`, "Trigger ID 不能为空"));
    else if (triggerIds.has(module.trigger.id))
      result.push(
        issue(`${base}.trigger.id`, `Trigger ID ${module.trigger.id} 重复`),
      );
    triggerIds.add(module.trigger.id);
    if (module.trigger.bounds.width <= 0 || module.trigger.bounds.height <= 0)
      result.push(issue(`${base}.trigger.bounds`, "Trigger 尺寸必须为正数"));

    if (!module.end_trigger?.id.trim())
      result.push(issue(`${base}.end_trigger.id`, "结束 Trigger ID 不能为空"));
    else if (triggerIds.has(module.end_trigger.id))
      result.push(
        issue(
          `${base}.end_trigger.id`,
          `Trigger ID ${module.end_trigger.id} 重复`,
        ),
      );
    if (module.end_trigger) {
      triggerIds.add(module.end_trigger.id);
      if (
        module.end_trigger.bounds.width <= 0 ||
        module.end_trigger.bounds.height <= 0
      )
        result.push(
          issue(`${base}.end_trigger.bounds`, "结束 Trigger 尺寸必须为正数"),
        );
    }

    const entry = trainingEntryInput(module.tutorial);
    if (!entry)
      result.push(
        issue(
          `${base}.tutorial.entry.input_id`,
          "入口必须指向一个 verify 不为 false 的 Fuzz 输入",
        ),
      );
    else if (entry.at !== 0)
      result.push(
        issue(`${base}.tutorial.entry.input_id`, "入口输入必须位于本地 F0"),
      );
    const verified = verifiedInputs(module.tutorial);
    const entryIndex = Math.max(
      0,
      verified.findIndex(
        (input) => input.id === module.tutorial.entry.input_id,
      ),
    );
    const expectedSteps = entry
      ? verified.length - entryIndex
      : verified.length;
    if (module.tutorial.teaching.steps.length !== expectedSteps)
      result.push(
        issue(
          `${base}.tutorial.teaching.steps`,
          `教学步骤应有 ${expectedSteps} 项，当前为 ${module.tutorial.teaching.steps.length}`,
        ),
      );

    const inputIds = new Set<string>();
    for (const [inputIndex, input] of module.tutorial.fuzz.inputs.entries()) {
      const inputPath = `${base}.tutorial.fuzz.inputs[${inputIndex}]`;
      if (!input.id?.trim())
        result.push(issue(`${inputPath}.id`, "输入 ID 不能为空"));
      else if (inputIds.has(input.id))
        result.push(issue(`${inputPath}.id`, `输入 ID ${input.id} 重复`));
      inputIds.add(input.id);
      if (!input.keys.length)
        result.push(issue(`${inputPath}.keys`, "按键组合不能为空"));
      if (new Set(input.keys).size !== input.keys.length)
        result.push(issue(`${inputPath}.keys`, "按键不能重复"));
      for (const key of input.keys)
        if (!ACTIONS.includes(key as (typeof ACTIONS)[number]))
          result.push(issue(`${inputPath}.keys`, `未知按键 ${key}`));
      const hasDirection = input.keys.some((key) => DIRECTION_KEYS.has(key));
      if (hasDirection && input.held_time === undefined)
        result.push(
          issue(`${inputPath}.held_time`, "方向输入必须设置 held_time"),
        );
      if (
        !hasDirection &&
        input.held_time !== undefined &&
        !input.keys.some((key) => key === "jump" || key === "grab")
      )
        result.push(
          issue(
            `${inputPath}.held_time`,
            "纯动作输入不应设置 held_time",
            "warning",
          ),
        );
    }
    for (const [name, value] of Object.entries(
      module.tutorial.fuzz.limits ?? {},
    )) {
      if (value !== undefined && value <= 0)
        result.push(
          issue(`${base}.tutorial.fuzz.limits.${name}`, "资源上限必须大于 0"),
        );
    }
  }

  const finish = training.finish.trigger;
  if (!finish.id.trim())
    result.push(issue("finish.trigger.id", "终点 Trigger ID 不能为空"));
  if (triggerIds.has(finish.id))
    result.push(
      issue("finish.trigger.id", "终点 Trigger ID 与模块 Trigger 重复"),
    );
  if (finish.bounds.width <= 0 || finish.bounds.height <= 0)
    result.push(issue("finish.trigger.bounds", "终点 Trigger 尺寸必须为正数"));
  return result;
}

async function readJson<T>(
  directory: FileSystemDirectoryHandle,
  name: string,
): Promise<T> {
  const handle = await directory.getFileHandle(name);
  return JSON.parse(await (await handle.getFile()).text()) as T;
}

async function writeJson(
  directory: FileSystemDirectoryHandle,
  name: string,
  value: unknown,
): Promise<void> {
  const handle = await directory.getFileHandle(name, { create: true });
  const writable = await handle.createWritable();
  await writable.write(`${JSON.stringify(value, null, 2)}\n`);
  await writable.close();
}

export async function openTrainingWorkspace(
  directory: FileSystemDirectoryHandle,
  fallbackMap: GymMap,
): Promise<TrainingProject[]> {
  let manifest: TrainingWorkspaceManifest | null = null;
  try {
    manifest = await readJson<TrainingWorkspaceManifest>(
      directory,
      WORKSPACE_MANIFEST,
    );
  } catch {
    manifest = null;
  }
  if (manifest?.version === 1) {
    return Promise.all(
      manifest.projects.map(async (entry) => {
        const map = await readJson<GymMap>(directory, entry.map);
        const training = await readJson<TrainingMapDocument>(
          directory,
          entry.training,
        );
        return {
          id: entry.id,
          mapFileName: entry.map,
          trainingFileName: entry.training,
          initialModuleId: entry.initial_module,
          map,
          training: normalizeTrainingDocument(map, training),
        };
      }),
    );
  }

  const trainingNames: string[] = [];
  for await (const [name, handle] of directory.entries()) {
    if (handle.kind === "file" && name.endsWith(".training.json"))
      trainingNames.push(name);
  }
  if (!trainingNames.length) return [createTrainingProject(fallbackMap)];
  return Promise.all(
    trainingNames.sort().map(async (trainingFileName) => {
      const base = trainingFileName.slice(0, -".training.json".length);
      const mapFileName = `${base}.map.json`;
      let map = structuredClone(fallbackMap);
      try {
        map = await readJson<GymMap>(directory, mapFileName);
      } catch {
        /* Existing catalog folders keep maps in TypeScript. */
      }
      const training = await readJson<TrainingMapDocument>(
        directory,
        trainingFileName,
      );
      return {
        id: trainingSlug(training.id || base),
        mapFileName,
        trainingFileName,
        map,
        training: normalizeTrainingDocument(map, training),
      };
    }),
  );
}

export async function saveTrainingWorkspace(
  directory: FileSystemDirectoryHandle,
  projects: readonly TrainingProject[],
): Promise<void> {
  const manifest: TrainingWorkspaceManifest = {
    version: 1,
    projects: projects.map((project) => ({
      id: project.id,
      map: project.mapFileName,
      training: project.trainingFileName,
      ...(project.initialModuleId
        ? { initial_module: project.initialModuleId }
        : {}),
    })),
  };
  for (const project of projects) {
    await writeJson(directory, project.mapFileName, project.map);
    await writeJson(directory, project.trainingFileName, project.training);
  }
  await writeJson(directory, WORKSPACE_MANIFEST, manifest);
}

async function tryReadJson<T>(
  directory: FileSystemDirectoryHandle,
  name: string,
): Promise<T | null> {
  try {
    return await readJson<T>(directory, name);
  } catch {
    return null;
  }
}

async function directoryAt(
  root: FileSystemDirectoryHandle,
  path: string,
  create = false,
): Promise<FileSystemDirectoryHandle> {
  let directory = root;
  if (path === "." || !path) return directory;
  for (const segment of path.split("/").filter(Boolean))
    directory = await directory.getDirectoryHandle(segment, { create });
  return directory;
}

export async function openTrainingCatalogWorkspace(
  root: FileSystemDirectoryHandle,
  fallbackMap: GymMap,
): Promise<TrainingCatalogWorkspace> {
  const manifest = await tryReadJson<TrainingCatalogManifest>(
    root,
    CATALOG_MANIFEST,
  );
  if (!manifest) {
    for await (const _entry of root.entries())
      throw new Error(`目录缺少 ${CATALOG_MANIFEST}，请选择完整训练目录`);
    return createTrainingCatalogWorkspace();
  }
  if (manifest.version !== 1)
    throw new Error(`${CATALOG_MANIFEST} 版本必须为 1`);
  const paths = manifest.techniques.map((entry) => entry.path);

  const techniques = await Promise.all(
    [...new Set(paths)].map(async (path) => {
      const directory = await directoryAt(root, path);
      const metadata = await readJson<TrainingTechniqueDocument>(
        directory,
        TECHNIQUE_DOCUMENT,
      );
      return {
        path,
        metadata,
        projects: await openTrainingWorkspace(directory, fallbackMap),
      };
    }),
  );
  const sections = [...manifest.sections];
  for (const technique of techniques) {
    const section = technique.metadata.section;
    if (!sections.some((candidate) => candidate.id === section.id))
      throw new Error(
        `${technique.path}/${TECHNIQUE_DOCUMENT} 引用了目录中不存在的大分类 ${section.id}`,
      );
  }
  return { sections, techniques };
}

export async function saveTrainingCatalogWorkspace(
  root: FileSystemDirectoryHandle,
  catalog: TrainingCatalogWorkspace,
): Promise<void> {
  for (const technique of catalog.techniques) {
    const directory = await directoryAt(root, technique.path, true);
    await writeJson(directory, TECHNIQUE_DOCUMENT, technique.metadata);
    await saveTrainingWorkspace(directory, technique.projects);
  }
  const manifest: TrainingCatalogManifest = {
    version: 1,
    sections: catalog.sections,
    techniques: catalog.techniques.map((technique) => ({
      path: technique.path,
    })),
  };
  await writeJson(root, CATALOG_MANIFEST, manifest);
}

async function removeFileIfPresent(
  directory: FileSystemDirectoryHandle,
  name: string,
): Promise<void> {
  try {
    await directory.removeEntry(name);
  } catch (error) {
    if (!(error instanceof DOMException) || error.name !== "NotFoundError")
      throw error;
  }
}

export async function removeTrainingProjectFiles(
  root: FileSystemDirectoryHandle,
  technique: TrainingTechniqueWorkspace,
  project: TrainingProject,
): Promise<void> {
  const directory = await directoryAt(root, technique.path);
  await removeFileIfPresent(directory, project.mapFileName);
  await removeFileIfPresent(directory, project.trainingFileName);
}

export async function removeTrainingTechniqueFiles(
  root: FileSystemDirectoryHandle,
  technique: TrainingTechniqueWorkspace,
): Promise<void> {
  const parts = technique.path.split("/").filter(Boolean);
  const name = parts.pop();
  const parent = await directoryAt(root, parts.join("/"));
  if (name) await parent.removeEntry(name, { recursive: true });
}

export function coreValidationState(state: SimState): SimState {
  return structuredClone(state);
}
