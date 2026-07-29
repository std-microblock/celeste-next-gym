import type { GymMap, SimState } from "../model";
import type { TrainingDefinition } from "./session";
import { hyper } from "./techniques/hyper.ts";

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
    objectives: Array<
      | { type: "maximize" | "minimize"; expression: string }
      | { type: "approach"; expression: string; target: number }
    >;
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
  /** Stable ID of a training_trigger entity in GymMap.entities. */
  id: string;
}

export interface TrainingModule {
  id: string;
  trigger: TrainingTrigger;
  tutorial: TrainingDocument;
  validation: {
    initial_state: SimState;
    fuzz?: TrainingDocument["fuzz"];
  };
}

/** Map-owned training script. Tutorials are modules activated by its triggers. */
export interface TrainingMapDocument {
  version: 3;
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

export const trainingCatalog = [hyper];

export const trainingCatalogSections = [
  {
    id: "dash-tech",
    title: "冲刺技巧",
    badge: "DASH TECH",
    techniques: [hyper],
  },
];

export function findTrainingVariant(
  techniqueId: string,
  variantId: string,
): TrainingVariant | undefined {
  return trainingCatalog
    .find((technique) => technique.id === techniqueId)
    ?.variants.find((variant) => variant.id === variantId);
}
