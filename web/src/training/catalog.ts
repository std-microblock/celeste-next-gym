import type { GymMap, SimState } from '../model'
import type { TrainingDefinition } from './session'
import { hyper } from './techniques/hyper.ts'

export interface TrainingDocument extends TrainingDefinition {
  version: 1
  technique_id: string
  variant_id: string
  variant_title: string
  summary: string
  entry: TrainingDefinition['entry'] & { check: string[]; failure: { title: string; body: string } }
  teaching: { steps: Array<{ prompt: string; order_error: { title: string; body: string }; window_error: { title: string; body: string } }> }
  assist: { result_sample_after_input_frames: number; auto_slowdown: { enabled_by_default: boolean; radius_frames: number; minimum_multiplier: number } }
  fuzz: TrainingDefinition['fuzz'] & {
    version: 1
    variables: Array<{ name: string; range: { from: number; to: number } }>
    observe_until: number | string
    success: string[]
    objectives: Array<{ type: 'maximize' | 'minimize'; expression: string }>
    search: { bindings: Record<string, number>; output: string[] }
  }
}

export interface TrainingVariant {
  id: string
  title: string
  summary: string
  document: TrainingDocument
  map: GymMap
  initial: SimState
  validationFuzz?: TrainingDocument['fuzz']
}

export interface TrainingTechnique {
  id: string
  title: string
  summary: string
  related: string[]
  variants: TrainingVariant[]
}

export const trainingCatalog = [hyper]

export function findTrainingVariant(techniqueId: string, variantId: string): TrainingVariant | undefined {
  return trainingCatalog.find((technique) => technique.id === techniqueId)?.variants.find((variant) => variant.id === variantId)
}
