import assert from 'node:assert/strict'
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs'
import net from 'node:net'
import { tmpdir } from 'node:os'
import { dirname, resolve } from 'node:path'
import { afterEach, describe, it } from 'node:test'
import { fileURLToPath } from 'node:url'

import {
  createRunContext,
  pingEverest,
  reserveLoopbackPort,
  terminateOwnedProcess,
  updateRunManifest,
  validateCollectorOwnership,
  validateGameInstall,
  validateProcessIdentity,
  waitForOwnedEverest,
} from './e2e-isolation.mjs'

const temporaryRoots = []
const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')

afterEach(() => {
  for (const root of temporaryRoots.splice(0)) rmSync(root, { recursive: true, force: true })
})

describe('E2E isolation', () => {
  it('reserves dynamic loopback ports without launching a game', async () => {
    const reservation = await reserveLoopbackPort()
    assert.ok(reservation.port > 0)
    await assert.rejects(
      () => new Promise((resolveConnect, reject) => {
        const server = net.createServer()
        server.once('error', reject)
        server.listen(reservation.port, '127.0.0.1', resolveConnect)
      }),
      /EADDRINUSE/,
    )
    await reservation.release()
  })

  it('writes a per-run manifest and isolated save/temp roots', () => {
    const repoRoot = temporaryRepo()
    const gameInstall = validateGameInstall({ repoRoot, gameRoot: resolve(repoRoot, 'vendor', 'celeste-game') })
    const context = createRunContext({
      repoRoot,
      gameInstall,
      modPort: 32001,
      httpPort: 43001,
      git: { branch: 'test', head: 'abc' },
    })
    updateRunManifest(context, { status: 'authenticated', game_process: { process_id: 42 } })
    assert.match(context.runRoot, /\.tmp[\\/]e2e-runs/)
    assert.equal(context.manifest.status, 'authenticated')
    assert.equal(context.manifest.ports.mod, 32001)
    assert.ok(context.runNonce.length > 20)
    const stored = JSON.parse(readFileSync(context.manifestPath, 'utf8'))
    assert.equal(stored.run_nonce, context.runNonce)
    assert.equal(stored.isolation.save_root, context.saveRoot)
  })

  it('rejects a game install outside vendor/celeste-game or configured as Steam', () => {
    const repoRoot = temporaryRepo()
    const gameRoot = resolve(repoRoot, 'vendor', 'celeste-game')
    assert.throws(() => validateGameInstall({ repoRoot, gameRoot: resolve(repoRoot, 'other') }), /must be/)
    assert.throws(() => validateGameInstall({ repoRoot, gameRoot, steamRoots: [gameRoot] }), /Steam/)
  })

  it('rejects a reparse or symlinked game install', { skip: process.platform !== 'win32' }, () => {
    const repoRoot = mkdtempSync(resolve(tmpdir(), 'celeste-e2e-link-'))
    temporaryRoots.push(repoRoot)
    const realGame = resolve(repoRoot, 'real-game')
    mkdirSync(realGame, { recursive: true })
    writeFileSync(resolve(realGame, 'Celeste.exe'), '')
    mkdirSync(resolve(repoRoot, 'vendor'), { recursive: true })
    symlinkSync(realGame, resolve(repoRoot, 'vendor', 'celeste-game'), 'junction')
    assert.throws(
      () => validateGameInstall({ repoRoot, gameRoot: resolve(repoRoot, 'vendor', 'celeste-game') }),
      /reparse\/symlink/,
    )
  })

  it('authenticates ping by nonce, child PID, and selected port', async () => {
    let pingCount = 0
    const server = net.createServer((socket) => {
      socket.setEncoding('utf8')
      socket.once('data', () => {
        pingCount++
        socket.end(`${JSON.stringify({
          success: pingCount >= 2,
          error: pingCount >= 2 ? undefined : 'game content is still loading',
          run_nonce: 'run-123',
          process_id: 456,
          collector_port: 32123,
        })}\n`)
      })
    })
    await new Promise((resolveListen) => server.listen(0, '127.0.0.1', resolveListen))
    const address = server.address()
    assert.ok(address && typeof address !== 'string')
    const firstResponse = await pingEverest(address.port)
    assert.equal(firstResponse.success, false)
    const response = await waitForOwnedEverest(
      address.port,
      { runNonce: 'run-123', processId: 456, port: 32123 },
      2_000,
    )
    validateCollectorOwnership(response, { runNonce: 'run-123', processId: 456, port: 32123 })
    assert.ok(pingCount >= 2)
    assert.throws(
      () => validateCollectorOwnership(response, { runNonce: 'wrong', processId: 456, port: 32123 }),
      /nonce mismatch/,
    )
    await new Promise((resolveClose) => server.close(resolveClose))
  })

  it('keeps the Mod port and ownership handshake wired to environment-backed protocol fields', () => {
    const moduleSource = readFileSync(
      resolve(repoRoot, 'mods', 'CelesteGymCollector', 'Source', 'CelesteGymCollectorModule.cs'),
      'utf8',
    )
    const protocolSource = readFileSync(
      resolve(repoRoot, 'mods', 'CelesteGymCollector', 'Source', 'Protocol.cs'),
      'utf8',
    )
    assert.match(moduleSource, /CELESTE_GYM_COLLECTOR_PORT/)
    assert.match(moduleSource, /CELESTE_GYM_RUN_NONCE/)
    assert.match(moduleSource, /Environment\.ProcessId/)
    assert.match(protocolSource, /JsonPropertyName\("run_nonce"\)/)
    assert.match(protocolSource, /JsonPropertyName\("process_id"\)/)
    assert.match(protocolSource, /JsonPropertyName\("collector_port"\)/)
  })

  it('refuses cleanup ownership when PID identity changes', () => {
    const expected = {
      processId: 91,
      executablePath: 'D:\\test\\Celeste.exe',
      creationTimeUtc: '2026-07-27T00:00:00.0000000Z',
    }
    assert.equal(validateProcessIdentity(expected, { ...expected }).owned, true)
    assert.equal(validateProcessIdentity(expected, { ...expected, processId: 92 }).owned, false)
    assert.equal(validateProcessIdentity(expected, { ...expected, executablePath: 'C:\\Steam\\Celeste.exe' }).owned, false)
    assert.equal(validateProcessIdentity(expected, { ...expected, creationTimeUtc: 'later' }).owned, false)
    assert.equal(validateProcessIdentity(expected, null).owned, false)

    let terminated = false
    const warnings = []
    const child = { pid: 91, exitCode: null }
    const refused = terminateOwnedProcess({
      child,
      expectedIdentity: expected,
      queryIdentity: () => ({ ...expected, executablePath: 'C:\\Steam\\Celeste.exe' }),
      terminate: () => { terminated = true },
      logger: { warn: (message) => warnings.push(message) },
    })
    assert.equal(refused, false)
    assert.equal(terminated, false)
    assert.match(warnings[0], /refusing to terminate/)

    const accepted = terminateOwnedProcess({
      child,
      expectedIdentity: expected,
      queryIdentity: () => ({ ...expected }),
      terminate: () => { terminated = true },
    })
    assert.equal(accepted, true)
    assert.equal(terminated, true)
  })
})

function temporaryRepo() {
  const repoRoot = mkdtempSync(resolve(tmpdir(), 'celeste-e2e-isolation-'))
  temporaryRoots.push(repoRoot)
  const gameRoot = resolve(repoRoot, 'vendor', 'celeste-game')
  mkdirSync(gameRoot, { recursive: true })
  writeFileSync(resolve(gameRoot, 'Celeste.exe'), '')
  return repoRoot
}
