import net from 'node:net'

import { LOOPBACK } from './ports.js'

export interface EverestPing {
  readonly success?: boolean
  readonly error?: string
  readonly run_nonce?: string
  readonly process_id?: number
  readonly collector_port?: number
  readonly [key: string]: unknown
}

export interface ExpectedCollectorOwnership {
  readonly runNonce: string
  readonly processId: number
  readonly port: number
}

export async function pingEverest(port: number, timeoutMs = 2_000): Promise<EverestPing> {
  return await new Promise<EverestPing>((resolvePing, reject) => {
    const socket = net.createConnection({ host: LOOPBACK, port })
    let data = ''
    let settled = false
    const finish = (callback: () => void): void => {
      if (settled) return
      settled = true
      clearTimeout(timer)
      socket.destroy()
      callback()
    }
    const timer = setTimeout(() => finish(() => reject(new Error(`Everest ping timed out on ${port}`))), timeoutMs)
    timer.unref()
    socket.setEncoding('utf8')
    socket.once('connect', () => socket.write('{"command":"ping"}\n'))
    socket.on('data', (chunk) => {
      data += chunk
      const newline = data.indexOf('\n')
      if (newline < 0) return
      finish(() => {
        try {
          resolvePing(JSON.parse(data.slice(0, newline)) as EverestPing)
        } catch (error) {
          reject(new Error(`invalid Everest ping response: ${String(error)}`))
        }
      })
    })
    socket.once('error', (error) => finish(() => reject(error)))
    socket.once('end', () => finish(() => reject(new Error('Everest closed ping without a response'))))
  })
}

export function validateCollectorOwnership(
  response: EverestPing,
  expected: ExpectedCollectorOwnership,
): EverestPing {
  if (response.run_nonce !== expected.runNonce) {
    throw new Error(`Everest run nonce mismatch: expected ${expected.runNonce}, got ${response.run_nonce ?? '(missing)'}`)
  }
  if (response.process_id !== expected.processId) {
    throw new Error(`Everest process mismatch: expected ${expected.processId}, got ${response.process_id ?? '(missing)'}`)
  }
  if (response.collector_port !== expected.port) {
    throw new Error(`Everest collector port mismatch: expected ${expected.port}, got ${response.collector_port ?? '(missing)'}`)
  }
  if (response.success !== true) throw new Error(`Everest ping failed: ${response.error ?? 'unknown error'}`)
  return response
}

export async function waitForOwnedEverest(
  port: number,
  expected: ExpectedCollectorOwnership,
  timeoutMs = 30_000,
): Promise<EverestPing> {
  const deadline = Date.now() + timeoutMs
  let lastError = 'collector did not answer'
  do {
    try {
      const response = await pingEverest(port)
      if (response.run_nonce !== expected.runNonce) {
        throw new Error(`Everest run nonce mismatch: expected ${expected.runNonce}, got ${response.run_nonce ?? '(missing)'}`)
      }
      if (response.process_id !== expected.processId) {
        throw new Error(`Everest process mismatch: expected ${expected.processId}, got ${response.process_id ?? '(missing)'}`)
      }
      if (response.collector_port !== expected.port) {
        throw new Error(`Everest collector port mismatch: expected ${expected.port}, got ${response.collector_port ?? '(missing)'}`)
      }
      if (response.success === true) return response
      lastError = response.error ?? 'game content is still loading'
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      if (/nonce mismatch|process mismatch|collector port mismatch/.test(message)) throw error
      lastError = message
    }
    await new Promise<void>((resolveWait) => setTimeout(resolveWait, 200))
  } while (Date.now() < deadline)
  throw new Error(`owned Everest collector was not ready within ${timeoutMs} ms: ${lastError}`)
}
