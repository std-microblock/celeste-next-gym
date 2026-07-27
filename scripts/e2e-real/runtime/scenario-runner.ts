import { resolve } from 'node:path'

import { createRequest } from '../request.js'
import type { E2EState, ScenarioDefinition, SimulateRequest } from '../types.js'
import { validateCollectedStates } from '../validation.js'
import { createVerifyContext, pickCore } from '../verify.js'

export interface ScenarioExecutionDependencies {
  simulate(request: SimulateRequest): Promise<unknown>
  writeTrace(tracePath: string, inputs: SimulateRequest['inputs'], states: readonly E2EState[]): void
  compare(options: { tracePath: string; mapPath: string; room?: string }): void | Promise<void>
}

export interface ScenarioSummary {
  readonly name: string
  readonly frames: number
  readonly first: ReturnType<typeof pickCore>
  readonly last: ReturnType<typeof pickCore>
  readonly reflectedFieldCount: number
  readonly tracePath: string
  readonly dead: boolean
}

export async function executeScenario(options: {
  readonly scenario: ScenarioDefinition
  readonly map: Uint8Array
  readonly mapPath: string
  readonly repoRoot: string
  readonly room?: string
  readonly skipTransitions: boolean
  readonly collectOnly: boolean
  readonly dependencies: ScenarioExecutionDependencies
}): Promise<ScenarioSummary> {
  const request = createRequest({
    scenario: options.scenario,
    map: options.map,
    ...(options.room ? { room: options.room } : {}),
    skipTransitions: options.skipTransitions,
  })
  const collected = await options.dependencies.simulate(request)
  validateCollectedStates(collected, request)
  const states = collected
  const tracePath = resolve(options.repoRoot, '.tmp', `e2e-${options.scenario.name}-trace.json`)

  // A failed semantic guard must still leave the authoritative Everest trace behind.
  options.dependencies.writeTrace(tracePath, request.inputs, states)
  await options.scenario.verify?.(states, createVerifyContext({
    scenario: options.scenario,
    inputs: request.inputs,
    initialSnapshot: request.initial_snapshot,
    room: options.room,
    mapPath: options.mapPath,
    tracePath,
  }))
  if (!options.collectOnly) {
    await options.dependencies.compare({
      tracePath,
      mapPath: options.mapPath,
      ...(options.room ? { room: options.room } : {}),
    })
  }
  const first = states[0]
  return {
    name: options.scenario.name,
    frames: states.length,
    first: pickCore(first),
    last: pickCore(states.at(-1)),
    reflectedFieldCount: Object.keys(first?._everest_fields ?? {}).length,
    tracePath,
    dead: states.some((state) => state.dead),
  }
}
