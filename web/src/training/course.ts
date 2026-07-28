import type { SimState } from '../model.ts'
import type { TrainingMapDocument, TrainingModule, TrainingTrigger } from './catalog.ts'

export interface TrainingCompletion {
  moduleId: string
  title: string
  triggerFrame: number
  startedFrame: number
  completedFrame: number
  targetFrame?: number
  actualInputFrame?: number
  accuracy: number
  reactionFrames: number
  objectiveValues: number[]
  windows: Array<{ from: number; to: number }>
  actualInputs: Array<{ frame: number; keys: string[] }>
}

const PLAYER_HALF_WIDTH = 4
const STANDING_HEIGHT = 11
const DUCKING_HEIGHT = 6

/** Training triggers use the same bottom-centred player position as SimState. */
export function triggerContainsPlayer(trigger: TrainingTrigger, state: SimState): boolean {
  const player = {
    left: state.pos.x - PLAYER_HALF_WIDTH,
    right: state.pos.x + PLAYER_HALF_WIDTH,
    top: state.pos.y - (state.ducking ? DUCKING_HEIGHT : STANDING_HEIGHT),
    bottom: state.pos.y,
  }
  const bounds = trigger.bounds
  return player.right >= bounds.x
    && player.left <= bounds.x + bounds.width
    && player.bottom >= bounds.y
    && player.top <= bounds.y + bounds.height
}

export function moduleAtPlayer(document: TrainingMapDocument, state: SimState, completedIds: ReadonlySet<string>): TrainingModule | undefined {
  return document.modules.find((module) => !completedIds.has(module.id) && triggerContainsPlayer(module.trigger, state))
}

export function allModulesCompleted(document: TrainingMapDocument, completedIds: ReadonlySet<string>): boolean {
  return document.modules.every((module) => completedIds.has(module.id))
}

/** 100 at the Fuzzer best point, minus eight percentage points per frame. */
export function timingAccuracy(actualFrame: number | undefined, targetFrame: number | undefined): number {
  if (actualFrame === undefined || targetFrame === undefined) return 0
  return Math.max(0, 100 - Math.abs(actualFrame - targetFrame) * 8)
}

export function average(values: readonly number[]): number {
  return values.length === 0 ? 0 : values.reduce((sum, value) => sum + value, 0) / values.length
}
