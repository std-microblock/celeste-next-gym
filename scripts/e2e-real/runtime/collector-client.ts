import { createRequire } from 'node:module'
import { resolve } from 'node:path'

import { SERVICE_READY_TIMEOUT_MS } from '../constants.js'
import type { E2EState, SimulateRequest } from '../types.js'

interface Codec {
  encode(value: unknown): Uint8Array
  decode(value: Uint8Array): unknown
}

interface SimulationBody {
  readonly success?: boolean
  readonly error?: unknown
  readonly states?: unknown
}

export interface CollectorClient {
  health(): Promise<Record<string, unknown>>
  waitUntilReady(timeoutMs?: number): Promise<Record<string, unknown>>
  simulate(request: SimulateRequest): Promise<unknown>
}

export function createCollectorClient(serviceRoot: string, port: number): CollectorClient {
  const requireFromService = createRequire(resolve(serviceRoot, 'package.json'))
  const codec = requireFromService('@msgpack/msgpack') as Codec
  return {
    async health(): Promise<Record<string, unknown>> {
      const value = await fetch(`http://127.0.0.1:${port}/health`).then(async (response) => await response.json())
      if (!value || typeof value !== 'object') throw new Error('collector health response is not an object')
      return value as Record<string, unknown>
    },
    async waitUntilReady(timeoutMs = SERVICE_READY_TIMEOUT_MS): Promise<Record<string, unknown>> {
      const deadline = Date.now() + timeoutMs
      let health: Record<string, unknown> = {}
      do {
        health = await this.health()
        if (health.ready === true) return health
        await new Promise<void>((resolveWait) => setTimeout(resolveWait, 250))
      } while (Date.now() < deadline)
      throw new Error(`HTTP collector is not ready: ${JSON.stringify(health)}`)
    },
    async simulate(request: SimulateRequest): Promise<unknown> {
      const response = await fetch(`http://127.0.0.1:${port}/api/simulate`, {
        method: 'POST',
        headers: { 'content-type': 'application/octet-stream' },
        body: Buffer.from(codec.encode(request)),
      })
      const body = codec.decode(new Uint8Array(await response.arrayBuffer())) as SimulationBody
      if (!response.ok || body.success !== true) throw new Error(`simulation failed: ${JSON.stringify(body)}`)
      return body.states
    },
  }
}

export function asStates(value: unknown): readonly E2EState[] {
  if (!Array.isArray(value)) throw new Error('collector states are not an array')
  return value as readonly E2EState[]
}
