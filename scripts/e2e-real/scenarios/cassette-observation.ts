import type { E2EState } from '../types.js'

export interface CassetteObservation {
  readonly index: number
  readonly position: readonly [number, number]
  readonly activated: boolean
  readonly collidable: boolean
}

export function cassetteBlocks(state: E2EState | undefined): readonly CassetteObservation[] {
  const raw = state?._everest_fields.cassetteBlocks
  if (!Array.isArray(raw)) return []
  return raw.flatMap((value) => {
    if (!value || typeof value !== 'object') return []
    const record = value as Record<string, unknown>
    const position = record.position
    if (typeof record.index !== 'number' || !Array.isArray(position)
      || typeof position[0] !== 'number' || typeof position[1] !== 'number'
      || typeof record.activated !== 'boolean' || typeof record.collidable !== 'boolean') return []
    return [{
      index: record.index,
      position: [position[0], position[1]],
      activated: record.activated,
      collidable: record.collidable,
    }]
  })
}

export function cassetteBlock(state: E2EState | undefined, index: number): CassetteObservation | undefined {
  return cassetteBlocks(state).find((block) => block.index === index)
}
