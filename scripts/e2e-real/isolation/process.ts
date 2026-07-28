import type { ChildProcess } from 'node:child_process'
import { spawnSync } from 'node:child_process'
import { lstatSync, realpathSync } from 'node:fs'

import type { ProcessIdentity } from '../types.js'
import { comparablePath } from './game-install.js'

export interface OwnershipDecision {
  readonly owned: boolean
  readonly reason?: string
}

export interface CleanupLogger { warn(message: string): void }

export function validateProcessIdentity(
  expected: ProcessIdentity,
  actual: ProcessIdentity | null,
): OwnershipDecision {
  if (!actual) return { owned: false, reason: 'process no longer exists or identity is unavailable' }
  if (actual.processId !== expected.processId) {
    return { owned: false, reason: `PID mismatch: expected ${expected.processId}, got ${actual.processId}` }
  }
  if (comparablePath(actual.executablePath) !== comparablePath(expected.executablePath)) {
    return { owned: false, reason: `executable mismatch: expected ${expected.executablePath}, got ${actual.executablePath}` }
  }
  if (actual.creationTimeUtc !== expected.creationTimeUtc) {
    return { owned: false, reason: `creation time mismatch: expected ${expected.creationTimeUtc}, got ${actual.creationTimeUtc}` }
  }
  return { owned: true }
}

export function queryProcessIdentity(processId: number): ProcessIdentity | null {
  if (!Number.isSafeInteger(processId) || processId <= 0) throw new Error(`invalid process id: ${processId}`)
  if (process.platform === 'win32') {
    const script = `
$targetProcess = Get-CimInstance Win32_Process -Filter 'ProcessId = ${processId}'
if ($null -eq $targetProcess) { exit 3 }
[pscustomobject]@{
  processId = [int]$targetProcess.ProcessId
  executablePath = [string]$targetProcess.ExecutablePath
  creationTimeUtc = $targetProcess.CreationDate.ToUniversalTime().ToString('o')
} | ConvertTo-Json -Compress
`
    const result = spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', script], {
      encoding: 'utf8', windowsHide: true,
    })
    if (result.status === 3) return null
    if (result.error) throw result.error
    if (result.status !== 0) throw new Error(`process identity query failed: ${result.stderr.trim()}`)
    return JSON.parse(result.stdout.trim()) as ProcessIdentity
  }
  try {
    return {
      processId,
      executablePath: realpathSync(`/proc/${processId}/exe`),
      creationTimeUtc: lstatSync(`/proc/${processId}`).birthtime.toISOString(),
    }
  } catch {
    return null
  }
}

export async function waitForProcessIdentity(
  processId: number,
  executablePath: string,
  timeoutMs = 5_000,
): Promise<ProcessIdentity> {
  const deadline = Date.now() + timeoutMs
  do {
    const identity = queryProcessIdentity(processId)
    if (identity) {
      if (comparablePath(identity.executablePath) !== comparablePath(executablePath)) {
        throw new Error(`spawned PID ${processId} executable mismatch: expected ${executablePath}, got ${identity.executablePath}`)
      }
      return identity
    }
    await new Promise<void>((resolveWait) => setTimeout(resolveWait, 50))
  } while (Date.now() < deadline)
  throw new Error(`spawned PID ${processId} identity was unavailable within ${timeoutMs} ms`)
}

export async function terminateOwnedProcess(options: {
  readonly child: Pick<ChildProcess, 'pid' | 'exitCode' | 'kill'> | undefined
  readonly expectedIdentity: ProcessIdentity
  readonly logger?: CleanupLogger
  readonly queryIdentity?: (processId: number) => ProcessIdentity | null
  readonly terminate?: (child: Pick<ChildProcess, 'pid' | 'exitCode' | 'kill'>) => void
  readonly exitTimeoutMs?: number
  readonly exitPollMs?: number
}): Promise<boolean> {
  const { child } = options
  if (!child?.pid || child.exitCode !== null) return false
  let actualIdentity: ProcessIdentity | null
  try {
    actualIdentity = (options.queryIdentity ?? queryProcessIdentity)(child.pid)
  } catch (error) {
    ;(options.logger ?? console).warn(`refusing to terminate PID ${child.pid}: identity query failed: ${String(error)}`)
    return false
  }
  const decision = validateProcessIdentity(options.expectedIdentity, actualIdentity)
  if (!decision.owned) {
    ;(options.logger ?? console).warn(`refusing to terminate PID ${child.pid}: ${decision.reason}`)
    return false
  }
  ;(options.terminate ?? defaultTerminate)(child)
  await waitForOwnedProcessExit(
    options.expectedIdentity,
    options.queryIdentity ?? queryProcessIdentity,
    options.exitTimeoutMs ?? 5_000,
    options.exitPollMs ?? 50,
  )
  return true
}

async function waitForOwnedProcessExit(
  expected: ProcessIdentity,
  queryIdentity: (processId: number) => ProcessIdentity | null,
  timeoutMs: number,
  pollMs: number,
): Promise<void> {
  const deadline = Date.now() + timeoutMs
  do {
    const actual = queryIdentity(expected.processId)
    if (!actual || !validateProcessIdentity(expected, actual).owned) return
    await new Promise<void>((resolveWait) => setTimeout(resolveWait, pollMs))
  } while (Date.now() < deadline)
  throw new Error(`owned PID ${expected.processId} did not exit within ${timeoutMs} ms`)
}

function defaultTerminate(child: Pick<ChildProcess, 'pid' | 'kill'>): void {
  if (process.platform === 'win32') {
    const result = spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], {
      stdio: 'ignore', windowsHide: true,
    })
    if (result.error) throw result.error
    if (result.status !== 0 && result.status !== 128) throw new Error(`taskkill exited with ${result.status}`)
  } else {
    child.kill('SIGKILL')
  }
}
