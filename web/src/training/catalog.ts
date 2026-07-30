import { createInitialState, type GymMap, type SimState } from "../model";
import type { TrainingDefinition, TrainingObjective } from "./session";

export interface TrainingDocument extends TrainingDefinition {
  version: 2;
  id: string;
  title: string;
  summary: string;
  entry: TrainingDefinition["entry"] & {
    check: string[];
    failure: { title: string; body: string };
  };
  teaching: {
    steps: Array<{
      prompt: string;
      order_error: { title: string; body: string };
      window_error: { title: string; body: string };
    }>;
  };
  assist: {
    result_sample_after_input_frames: number;
    auto_slowdown: {
      enabled_by_default: boolean;
      radius_frames: number;
      minimum_multiplier: number;
    };
  };
  fuzz: TrainingDefinition["fuzz"] & {
    version: 1;
    variables: Array<{
      name: string;
      range: { from: number | string; to: number | string; step?: number };
    }>;
    observe_until: number | string;
    success: string[];
    checkpoints?: NonNullable<TrainingDefinition["fuzz"]["checkpoints"]>;
    objectives: TrainingObjective[];
    search: { bindings: Record<string, number>; output: string[] };
    limits?: {
      max_candidates?: number;
      max_input_frames?: number;
      max_trie_nodes?: number;
      max_cache_bytes?: number;
      max_expression_operations?: number;
    };
  };
}

export interface TrainingTrigger {
  id: string;
  bounds: { x: number; y: number; width: number; height: number };
}

export interface TrainingModule {
  id: string;
  /** Entering this region arms the tutorial. */
  trigger: TrainingTrigger;
  /** Reaching this region completes an editor recording for this tutorial. */
  end_trigger: TrainingTrigger;
  tutorial: TrainingDocument;
  validation: {
    initial_state: SimState;
    fuzz?: TrainingDocument["fuzz"];
  };
}

/** Map-owned training script. Tutorials are modules activated by its triggers. */
export interface TrainingMapDocument {
  version: 2;
  id: string;
  title: string;
  summary: string;
  modules: TrainingModule[];
  finish: {
    trigger: TrainingTrigger;
    require_all_modules: boolean;
  };
}

export interface TrainingVariant {
  id: string;
  title: string;
  summary: string;
  training: TrainingMapDocument;
  map: GymMap;
  initial: SimState;
}

export interface TrainingTechnique {
  id: string;
  title: string;
  summary: string;
  related: string[];
  variants: TrainingVariant[];
}

export interface TrainingTechniqueDocument {
  id: string;
  title: string;
  summary: string;
  related: string[];
  section: { id: string; title: string; badge: string };
}

interface BundledWorkspaceManifest {
  version: 1;
  projects: Array<{
    id: string;
    map: string;
    training: string;
    initial_module?: string;
  }>;
}

interface BundledCatalogManifest {
  version: 1;
  sections: TrainingTechniqueDocument["section"][];
  techniques: Array<{ path: string }>;
}

const bundledJson = import.meta.glob("../../../training/**/*.json", {
  eager: true,
  import: "default",
}) as Record<string, unknown>;
const catalogPath = "../../../training/celeste-gym.catalog.json";

function bundledDocument<T>(path: string): T {
  const document = bundledJson[path];
  if (document === undefined) throw new Error(`缺少训练工作区文件：${path}`);
  return document as T;
}

function workspaceBase(techniquePath: string): string {
  return techniquePath.slice(0, -"technique.json".length);
}

function loadBundledTechnique(
  techniquePath: string,
  document: TrainingTechniqueDocument,
): TrainingTechnique {
  const base = workspaceBase(techniquePath);
  const manifest = bundledDocument<BundledWorkspaceManifest>(
    `${base}celeste-gym.workspace.json`,
  );
  if (manifest.version !== 1)
    throw new Error(`${document.id} 的训练工作区版本必须为 1`);
  return {
    id: document.id,
    title: document.title,
    summary: document.summary,
    related: document.related,
    variants: manifest.projects.map((entry) => {
      const map = bundledDocument<GymMap>(`${base}${entry.map}`);
      const training = bundledDocument<TrainingMapDocument>(
        `${base}${entry.training}`,
      );
      const initialModule = entry.initial_module
        ? training.modules.find((module) => module.id === entry.initial_module)
        : undefined;
      if (entry.initial_module && !initialModule)
        throw new Error(
          `${document.id}/${entry.id} 找不到初始模块 ${entry.initial_module}`,
        );
      return {
        id: entry.id,
        title: training.title,
        summary: training.summary,
        map,
        training,
        initial: initialModule
          ? structuredClone(initialModule.validation.initial_state)
          : createInitialState(map),
      };
    }),
  };
}

const bundledCatalog = bundledDocument<BundledCatalogManifest>(catalogPath);
if (bundledCatalog.version !== 1)
  throw new Error("训练目录 celeste-gym.catalog.json 版本必须为 1");

const bundledTechniques = bundledCatalog.techniques.map(({ path }) => {
  const techniquePath = `../../../training/${path}/technique.json`;
  const document = bundledDocument<TrainingTechniqueDocument>(techniquePath);
  return {
    document,
    technique: loadBundledTechnique(
      techniquePath,
      document,
    ),
  };
});

export const trainingCatalog = bundledTechniques.map(
  ({ technique }) => technique,
);

export const trainingCatalogSections = bundledCatalog.sections.map(
  (section) => ({
    ...section,
    techniques: bundledTechniques
      .filter(({ document }) => document.section.id === section.id)
      .map(({ technique }) => technique),
  }),
);

export function findTrainingVariant(
  techniqueId: string,
  variantId: string,
): TrainingVariant | undefined {
  return trainingCatalog
    .find((technique) => technique.id === techniqueId)
    ?.variants.find((variant) => variant.id === variantId);
}
