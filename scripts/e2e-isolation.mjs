import { randomUUID } from 'node:crypto'
import {
  lstatSync,
  mkdirSync,
  realpathSync,
  renameSync,
  writeFileSync,
} from 'node:fs'
import net from 'node:net'
import { dirname, relative, resolve, sep } from 'node:path'
import { spawnSync } from 'node:child_process'

const LOOPBACK = '127.0.0.1'

function comparablePath(path) {
  const normalized = resolve(path).replaceAll('\\', '/').replace(/\/$/, '')
  return process.platform === 'win32' ? normalized.toLowerCase() : normalized
}

function assertNoLinkedSegments(base, target) {
  const rel = relative(base, target)
  if (rel.startsWith('..') || resolve(base, rel) !== resolve(target)) {
    throw new Error(`path escapes repository root: ${target}`)
  }
  let current = resolve(base)
  for (const segment of rel.split(sep).filter(Boolean)) {
    current = resolve(current, segment)
    if (lstatSync(current).isSymbolicLink()) {
      throw new Error(`reparse/symlink path is not allowed for the E2E game install: ${current}`)
    }
  }
}

export function validateGameInstall({ repoRoot, gameRoot, steamRoots = [] }) {
  const expectedRoot = resolve(repoRoot, 'vendor', 'celeste-game')
  if (comparablePath(gameRoot) !== comparablePath(expectedRoot)) {
    throw new Error(`E2E game root must be ${expectedRoot}, got ${resolve(gameRoot)}`)
  }
  assertNoLinkedSegments(repoRoot, gameRoot)

  const realRepoRoot = realpathSync(repoRoot)
  const realGameRoot = realpathSync(gameRoot)
  const expectedRealRoot = resolve(realRepoRoot, 'vendor', 'celeste-game')
  if (comparablePath(realGameRoot) !== comparablePath(expectedRealRoot)) {
    throw new Error(`E2E game root resolves outside the repository vendor install: ${realGameRoot}`)
  }

  const executable = resolve(gameRoot, 'Celeste.exe')
  assertNoLinkedSegments(repoRoot, executable)
  const realExecutable = realpathSync(executable)
  if (comparablePath(dirname(realExecutable)) !== comparablePath(realGameRoot)) {
    throw new Error(`Celeste executable resolves outside the isolated game root: ${realExecutable}`)
  }

  const normalizedGameRoot = comparablePath(realGameRoot)
  if (/\/steamapps\/common\/celeste(?:\/|$)/i.test(normalizedGameRoot)) {
    throw new Error(`refusing to use a Steam Celeste install for E2E: ${realGameRoot}`)
  }
  for (const steamRoot of steamRoots) {
    let resolvedSteamRoot
    try {
      resolvedSteamRoot = realpathSync(steamRoot)
    } catch {
      resolvedSteamRoot = resolve(steamRoot)
    }
    if (normalizedGameRoot === comparablePath(resolvedSteamRoot)) {
      throw new Error(`refusing to use configured Steam Celeste install for E2E: ${realGameRoot}`)
    }
  }

  return { gameRoot: realGameRoot, executable: realExecutable }
}

export async function reserveLoopbackPort() {
  const server = net.createServer()
  await new Promise((resolveListen, reject) => {
    server.once('error', reject)
    server.listen({ host: LOOPBACK, port: 0, exclusive: true }, resolveListen)
  })
  const address = server.address()
  if (address === null || typeof address === 'string') {
    server.close()
    throw new Error('failed to reserve a loopback TCP port')
  }
  let released = false
  return {
    port: address.port,
    async release() {
      if (released) return
      released = true
      await new Promise((resolveClose, reject) => {
        server.close((error) => error ? reject(error) : resolveClose())
      })
    },
  }
}

export function createRunContext({ repoRoot, gameInstall, modPort, httpPort, git }) {
  const runId = `${new Date().toISOString().replaceAll(':', '-')}-${process.pid}-${randomUUID()}`
  const runNonce = randomUUID()
  const runRoot = resolve(repoRoot, '.tmp', 'e2e-runs', runId)
  const saveRoot = resolve(runRoot, 'userdata')
  const tempRoot = resolve(runRoot, 'tmp')
  mkdirSync(saveRoot, { recursive: true })
  mkdirSync(tempRoot, { recursive: true })
  const context = {
    runId,
    runNonce,
    runRoot,
    saveRoot,
    tempRoot,
    manifestPath: resolve(runRoot, 'manifest.json'),
    manifest: {
      version: 1,
      run_id: runId,
      run_nonce: runNonce,
      status: 'preparing',
      created_at: new Date().toISOString(),
      launcher_pid: process.pid,
      git,
      game: {
        root: gameInstall.gameRoot,
        executable: gameInstall.executable,
      },
      ports: { mod: modPort, http: httpPort },
      isolation: { save_root: saveRoot, temp_root: tempRoot },
    },
  }
  updateRunManifest(context)
  return context
}

export function updateRunManifest(context, patch = {}) {
  context.manifest = { ...context.manifest, ...patch, updated_at: new Date().toISOString() }
  const temporaryPath = `${context.manifestPath}.tmp`
  writeFileSync(temporaryPath, `${JSON.stringify(context.manifest, null, 2)}\n`, 'utf8')
  renameSync(temporaryPath, context.manifestPath)
}

export async function pingEverest(port, timeoutMs = 2_000) {
  return await new Promise((resolvePing, reject) => {
    const socket = net.createConnection({ host: LOOPBACK, port })
    let data = ''
    let settled = false
    const finish = (callback) => {
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
          resolvePing(JSON.parse(data.slice(0, newline)))
        } catch (error) {
          reject(new Error(`invalid Everest ping response: ${String(error)}`))
        }
      })
    })
    socket.once('error', (error) => finish(() => reject(error)))
    socket.once('end', () => finish(() => reject(new Error('Everest closed ping without a response'))))
  })
}

export function validateCollectorOwnership(response, expected) {
  if (response?.run_nonce !== expected.runNonce) {
    throw new Error(`Everest run nonce mismatch: expected ${expected.runNonce}, got ${response?.run_nonce ?? '(missing)'}`)
  }
  if (response?.process_id !== expected.processId) {
    throw new Error(`Everest process mismatch: expected ${expected.processId}, got ${response?.process_id ?? '(missing)'}`)
  }
  if (response?.collector_port !== expected.port) {
    throw new Error(`Everest collector port mismatch: expected ${expected.port}, got ${response?.collector_port ?? '(missing)'}`)
  }
  if (response?.success !== true) throw new Error(`Everest ping failed: ${response?.error ?? 'unknown error'}`)
  return response
}

export async function waitForOwnedEverest(port, expected, timeoutMs = 30_000) {
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
    await new Promise((resolveWait) => setTimeout(resolveWait, 200))
  } while (Date.now() < deadline)
  throw new Error(`owned Everest collector was not ready within ${timeoutMs} ms: ${lastError}`)
}

export function validateProcessIdentity(expected, actual) {
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

export function queryProcessIdentity(processId) {
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
      encoding: 'utf8',
      windowsHide: true,
    })
    if (result.status === 3) return null
    if (result.error) throw result.error
    if (result.status !== 0) throw new Error(`process identity query failed: ${result.stderr.trim()}`)
    return JSON.parse(result.stdout.trim())
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

export async function waitForProcessIdentity(processId, executablePath, timeoutMs = 5_000) {
  const deadline = Date.now() + timeoutMs
  do {
    const identity = queryProcessIdentity(processId)
    if (identity) {
      if (comparablePath(identity.executablePath) !== comparablePath(executablePath)) {
        throw new Error(`spawned PID ${processId} executable mismatch: expected ${executablePath}, got ${identity.executablePath}`)
      }
      return identity
    }
    await new Promise((resolveWait) => setTimeout(resolveWait, 50))
  } while (Date.now() < deadline)
  throw new Error(`spawned PID ${processId} identity was unavailable within ${timeoutMs} ms`)
}

export function terminateOwnedProcess({
  child,
  expectedIdentity,
  logger = console,
  queryIdentity = queryProcessIdentity,
  terminate = defaultTerminate,
}) {
  if (!child?.pid || child.exitCode !== null) return false
  let actualIdentity
  try {
    actualIdentity = queryIdentity(child.pid)
  } catch (error) {
    logger.warn(`refusing to terminate PID ${child.pid}: identity query failed: ${String(error)}`)
    return false
  }
  const decision = validateProcessIdentity(expectedIdentity, actualIdentity)
  if (!decision.owned) {
    logger.warn(`refusing to terminate PID ${child.pid}: ${decision.reason}`)
    return false
  }
  terminate(child)
  return true
}

function defaultTerminate(child) {
  if (process.platform === 'win32') {
    const result = spawnSync('taskkill.exe', ['/PID', String(child.pid), '/T', '/F'], {
      stdio: 'ignore',
      windowsHide: true,
    })
    if (result.error) throw result.error
    if (result.status !== 0 && result.status !== 128) {
      throw new Error(`taskkill exited with ${result.status}`)
    }
  } else {
    child.kill('SIGKILL')
  }
}
