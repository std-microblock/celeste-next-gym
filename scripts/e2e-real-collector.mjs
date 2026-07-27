import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { connect } from 'node:net'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawn, spawnSync } from 'node:child_process'

import {
  createRunContext,
  reserveLoopbackPort,
  terminateOwnedProcess,
  updateRunManifest,
  validateGameInstall,
  waitForOwnedEverest,
  waitForProcessIdentity,
} from './e2e-isolation.mjs'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const gameRoot = resolve(root, 'vendor', 'celeste-game')
const modRoot = resolve(root, 'mods', 'CelesteGymCollector')
const playgroundModRoot = resolve(root, 'mods', 'CelesteGymPlayground')
const serviceRoot = resolve(root, 'services', 'collector')
const showGameWindow = process.env.E2E_SHOW_WINDOWS === '1'
const skipTransitions = process.env.E2E_SKIP_TRANSITIONS === '1' || showGameWindow
const collectOnly = process.env.E2E_COLLECT_ONLY === '1'
const requestedScenarios = new Set(
  (process.env.E2E_SCENARIOS ?? '').split(',').map((name) => name.trim()).filter(Boolean),
)
const includePlaygroundSwim = process.env.E2E_PLAYGROUND_SWIM !== '0'
const includePlaygroundBooster = process.env.E2E_PLAYGROUND_BOOSTER !== '0'
const includePlaygroundWind = process.env.E2E_PLAYGROUND_WIND !== '0'
const includePlaygroundStarFly = process.env.E2E_PLAYGROUND_STAR_FLY !== '0'
const includePlaygroundLaunch = process.env.E2E_PLAYGROUND_LAUNCH !== '0'
const includePlaygroundBumper = process.env.E2E_PLAYGROUND_BUMPER !== '0'
const includePlaygroundBadelineBoost = process.env.E2E_PLAYGROUND_BADELINE_BOOST !== '0'
const includePlaygroundMiscStates = process.env.E2E_PLAYGROUND_MISC_STATES !== '0'
const areaId = Number.parseInt(process.env.E2E_AREA_ID ?? '1', 10)
if (!Number.isSafeInteger(areaId) || areaId < 0) throw new Error('E2E_AREA_ID must be a non-negative integer')
const areaSid = process.env.E2E_AREA_SID?.trim() || undefined
const defaultMapFiles = new Map([
  [1, '1-ForsakenCity.bin'],
  [2, '2-OldSite.bin'],
  [4, '4-GoldenRidge.bin'],
])
const mapFile = process.env.E2E_MAP_FILE ?? defaultMapFiles.get(areaId) ?? `${areaId}-Unknown.bin`
const room = process.env.E2E_ROOM || undefined

function run(command, args, cwd = root) {
  const result = spawnSync(command, args, { cwd, stdio: 'inherit', shell: false })
  if (result.error) throw result.error
  if (result.status !== 0) throw new Error(`${command} exited with ${result.status}`)
}

function capture(command, args, cwd = root) {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8', shell: false })
  if (result.error) throw result.error
  if (result.status !== 0) throw new Error(`${command} exited with ${result.status}: ${result.stderr.trim()}`)
  return result.stdout.trim()
}

function waitForPort(port, timeoutMs) {
  const deadline = Date.now() + timeoutMs
  return new Promise((resolveWait, reject) => {
    const attempt = () => {
      const socket = connect(port, '127.0.0.1')
      socket.once('connect', () => { socket.destroy(); resolveWait() })
      socket.once('error', () => {
        socket.destroy()
        if (Date.now() >= deadline) reject(new Error(`port ${port} was not ready within ${timeoutMs}ms`))
        else setTimeout(attempt, 200)
      })
    }
    attempt()
  })
}

const steamRoots = (process.env.E2E_STEAM_CELESTE_ROOTS ?? '')
  .split(process.platform === 'win32' ? ';' : ':')
  .map((path) => path.trim())
  .filter(Boolean)
const gameInstall = validateGameInstall({ repoRoot: root, gameRoot, steamRoots })
const git = {
  branch: capture('git', ['branch', '--show-current']),
  head: capture('git', ['rev-parse', 'HEAD']),
}

function cleanupOwned(label, child, identity) {
  if (!identity) {
    if (child?.pid && child.exitCode === null) {
      console.warn(`refusing to terminate ${label} PID ${child.pid}: no recorded process identity`)
    }
    return false
  }
  try {
    return terminateOwnedProcess({ child, expectedIdentity: identity })
  } catch (error) {
    console.warn(`failed to terminate owned ${label} PID ${child?.pid}: ${String(error)}`)
    return false
  }
}
const modPortReservation = await reserveLoopbackPort()
const httpPortReservation = await reserveLoopbackPort()
const runContext = createRunContext({
  repoRoot: root,
  gameInstall,
  modPort: modPortReservation.port,
  httpPort: httpPortReservation.port,
  git,
})
let game
let service
let gameIdentity
let serviceIdentity
let runError
let encode
let decode
try {
  const requireFromService = createRequire(resolve(serviceRoot, 'package.json'))
  const codec = requireFromService('@msgpack/msgpack')
  encode = codec.encode
  decode = codec.decode
  run('dotnet', ['build', resolve(modRoot, 'Source', 'CelesteGymCollector.csproj'), '-c', 'Release'])
  const installedMod = resolve(gameInstall.gameRoot, 'Mods', 'CelesteGymCollector')
  mkdirSync(resolve(installedMod, 'Code'), { recursive: true })
  copyFileSync(resolve(modRoot, 'everest.yaml'), resolve(installedMod, 'everest.yaml'))
  copyFileSync(
    resolve(modRoot, 'Source', 'bin', 'Release', 'net8.0', 'CelesteGymCollector.dll'),
    resolve(installedMod, 'Code', 'CelesteGymCollector.dll'),
  )
  const gameModsRoot = resolve(gameInstall.gameRoot, 'Mods')
  const installedPlaygroundMod = resolve(gameModsRoot, 'CelesteGymPlayground')
  const installedPlaygroundZip = resolve(gameModsRoot, 'CelesteGymPlayground.zip')
  if (dirname(installedPlaygroundMod) !== gameModsRoot || dirname(installedPlaygroundZip) !== gameModsRoot) {
    throw new Error('refusing to replace a playground mod outside the game Mods directory')
  }
  rmSync(installedPlaygroundMod, { recursive: true, force: true })
  rmSync(installedPlaygroundZip, { force: true })
  run('7z', ['a', '-tzip', '-mx=0', installedPlaygroundZip, 'everest.yaml', 'Maps'], playgroundModRoot)
  run(process.execPath, [resolve(serviceRoot, 'node_modules', 'typescript', 'bin', 'tsc'), '-p', 'tsconfig.json'], serviceRoot)

  await modPortReservation.release()
  updateRunManifest(runContext, { status: 'starting-game' })
  game = spawn(gameInstall.executable, ['--disable-splash', '--loglevel', 'info'], {
    cwd: gameInstall.gameRoot,
    windowsHide: !showGameWindow,
    stdio: 'ignore',
    shell: false,
    env: {
      ...process.env,
      CELESTE_GYM_COLLECTOR_PORT: String(modPortReservation.port),
      CELESTE_GYM_RUN_NONCE: runContext.runNonce,
      EVEREST_SAVEPATH: runContext.saveRoot,
      EVEREST_TMPDIR: runContext.tempRoot,
    },
  })
  if (!game.pid) throw new Error('Celeste child did not expose a process id')
  gameIdentity = await waitForProcessIdentity(game.pid, gameInstall.executable)
  updateRunManifest(runContext, {
    status: 'waiting-for-everest',
    game_process: gameIdentity,
  })
  const everestPing = await waitForOwnedEverest(modPortReservation.port, {
    runNonce: runContext.runNonce,
    processId: game.pid,
    port: modPortReservation.port,
  }, 30_000)
  updateRunManifest(runContext, { status: 'game-authenticated', everest_ping: everestPing })

  await httpPortReservation.release()
  service = spawn(process.execPath, [resolve(serviceRoot, 'dist', 'src', 'index.js')], {
    cwd: serviceRoot,
    windowsHide: true,
    stdio: 'ignore',
    shell: false,
    env: {
      ...process.env,
      COLLECTOR_BACKEND: 'everest',
      COLLECTOR_PORT: String(httpPortReservation.port),
      COLLECTOR_TIMEOUT_MS: '60000',
      EVEREST_COLLECTOR_PORT: String(modPortReservation.port),
      EVEREST_AREA_ID: String(areaId),
      ...(areaSid ? { EVEREST_AREA_SID: areaSid } : {}),
    },
  })
  if (!service.pid) throw new Error('collector service child did not expose a process id')
  serviceIdentity = await waitForProcessIdentity(service.pid, process.execPath)
  updateRunManifest(runContext, {
    status: 'waiting-for-service',
    service_process: serviceIdentity,
  })
  await waitForPort(httpPortReservation.port, 10_000)

  let health
  const healthDeadline = Date.now() + 30_000
  do {
    health = await fetch(`http://127.0.0.1:${httpPortReservation.port}/health`).then((response) => response.json())
    if (health.ready) break
    await new Promise((resolveWait) => setTimeout(resolveWait, 250))
  } while (Date.now() < healthDeadline)
  if (!health?.ready) throw new Error(`HTTP collector is not ready: ${JSON.stringify(health)}`)

  const mapPath = process.env.E2E_MAP_PATH
    ? resolve(root, process.env.E2E_MAP_PATH)
    : areaSid === 'CelesteGymPlayground/Playground'
      ? resolve(playgroundModRoot, 'Maps', 'CelesteGymPlayground', 'Playground.bin')
      : resolve(gameRoot, 'Content', 'Maps', mapFile)
  const map = readFileSync(mapPath)
  const scenarios = areaSid === 'CelesteGymPlayground/Playground' ? [
    {
      name: 'playground-load',
      initial: { pos: [64, 496], speed: [0, 0] },
      inputs: Array.from({ length: 30 }, () => input({ move_x: 1 })),
    },
    {
      name: 'mechanics-corner-correction-up',
      initial: { pos: [477, 275], speed: [0, -105] },
      inputs: Array.from({ length: 8 }, () => input()),
    },
    {
      name: 'mechanics-corner-correction-horizontal',
      initial: { pos: [392, 82], speed: [0, 0] },
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        dash_pressed: frame === 0,
      })),
    },
    {
      name: 'mechanics-directional-spikes-away',
      initial: { pos: [360, 496], speed: [0, -60] },
      inputs: Array.from({ length: 4 }, () => input()),
      verify: verifyDirectionalSpikesAway,
    },
    {
      name: 'mechanics-directional-spikes-into',
      initial: { pos: [360, 496], speed: [0, 60] },
      inputs: Array.from({ length: 4 }, () => input()),
      verify: verifyDirectionalSpikesInto,
    },
    {
      name: 'mechanics-berry-train',
      initial: { pos: [160, 468], speed: [0, 0] },
      inputs: Array.from({ length: 64 }, () => input()),
      verify: verifyBerryTrain,
    },
    {
      name: 'mechanics-screen-transition-up',
      initial: { pos: [640, 4], speed: [80, -160], dashes: 0, stamina: 20 },
      inputs: Array.from({ length: 42 }, () => input()),
      verify: verifyUpwardScreenTransition,
    },
    {
      name: 'mechanics-liftboost-zip-jump',
      initial: { pos: [64, 440], speed: [0, 0] },
      inputs: Array.from({ length: 24 }, (_, frame) => input({
        jump_pressed: frame === 10,
        jump_held: frame >= 10 && frame < 16,
      })),
      verify: verifyZipMoverLiftboost,
    },
    {
      name: 'dash-spring-cancel',
      initial: { pos: [80, 488], speed: [0, 100], dashes: 0 },
      inputs: Array.from({ length: 16 }, (_, frame) => input({
        dash_pressed: frame === 0,
      })),
      verify: verifySpringCancel,
    },
    {
      name: 'dash-spiked-wallbounce',
      initial: { pos: [396, 207], speed: [0, 0] },
      inputs: Array.from({ length: 14 }, (_, frame) => input({
        move_y: -1,
        jump_pressed: frame === 5,
        jump_held: frame >= 5 && frame < 12,
        dash_pressed: frame === 0,
      })),
      verify: verifySpikedWallbounce,
    },
    {
      name: 'dash-spiked-wallbounce-late',
      initial: { pos: [396, 207], speed: [0, 0] },
      inputs: Array.from({ length: 7 }, (_, frame) => input({
        move_y: -1,
        jump_pressed: frame === 6,
        jump_held: frame === 6,
        dash_pressed: frame === 0,
      })),
      verify: verifyLateSpikedWallbounce,
    },
    {
      name: 'dash-superwave',
      initial: { pos: [240, 496], speed: [0, 0] },
      inputs: Array.from({ length: 30 }, (_, frame) => input({
        move_x: frame <= 10 ? 1 : -1,
        move_y: frame >= 11 ? 1 : 0,
        jump_pressed: frame === 10 || frame === 26,
        jump_held: frame === 10 || frame === 26,
        dash_pressed: frame === 0 || frame === 11,
      })),
      verify: verifySuperwave,
    },
    {
      name: 'dash-demodash-gap',
      initial: { pos: [712, 320], speed: [0, 0] },
      inputs: Array.from({ length: 30 }, (_, frame) => input({
        move_x: 1,
        crouch_dash_pressed: frame === 0,
      })),
      verify: verifyDemodashGap,
    },
    {
      name: 'dash-ultra',
      initial: { pos: [200, 480], speed: [0, 0] },
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        move_y: 1,
        dash_pressed: frame === 0,
      })),
      verify: verifyUltra,
    },
    {
      name: 'dash-grounded-ultra',
      initial: { pos: [820, 496], speed: [300, 0] },
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        move_y: 1,
        dash_pressed: frame === 0,
      })),
      verify: verifyGroundedUltra,
    },
    {
      name: 'dash-delayed-ultra',
      initial: { pos: [200, 420], speed: [0, 0] },
      inputs: Array.from({ length: 36 }, (_, frame) => input({
        move_x: 1,
        move_y: 1,
        dash_pressed: frame === 0,
      })),
      verify: verifyDelayedUltra,
    },
    {
      name: 'dash-chained-ultras',
      initial: { pos: [200, 461], speed: [0, 0] },
      inputs: Array.from({ length: 40 }, (_, frame) => input({
        move_x: 1,
        move_y: 1,
        dash_pressed: frame === 0 || frame === 16,
      })),
      verify: verifyChainedUltras,
    },
    ...(includePlaygroundSwim ? [
      {
        name: 'playground-swim-idle',
        initial: { pos: [504, 456], speed: [0, 0] },
        inputs: Array.from({ length: 30 }, () => input()),
      },
      {
        name: 'playground-swim-right',
        initial: { pos: [504, 456], speed: [0, 0], state: 'Swim' },
        inputs: Array.from({ length: 30 }, () => input({ move_x: 1 })),
      },
      {
        name: 'playground-swim-surface-idle',
        initial: { pos: [504, 428], speed: [0, 0], state: 'Swim' },
        inputs: Array.from({ length: 20 }, () => input()),
      },
      {
        name: 'playground-swim-up',
        initial: { pos: [504, 456], speed: [0, 0], state: 'Swim' },
        inputs: Array.from({ length: 20 }, () => input({ move_y: -1 })),
      },
      {
        name: 'playground-swim-down',
        initial: { pos: [504, 456], speed: [0, 0], state: 'Swim' },
        inputs: Array.from({ length: 30 }, () => input({ move_y: 1 })),
      },
      {
        name: 'playground-swim-diagonal',
        initial: { pos: [504, 456], speed: [0, 0], state: 'Swim' },
        inputs: Array.from({ length: 20 }, () => input({ move_x: 1, move_y: 1 })),
      },
      {
        name: 'playground-swim-exit-right',
        initial: { pos: [552, 456], speed: [0, 0], state: 'Swim' },
        inputs: Array.from({ length: 20 }, () => input({ move_x: 1 })),
      },
      {
        name: 'playground-swim-jump',
        initial: { pos: [504, 428], speed: [0, 0], state: 'Swim' },
        inputs: Array.from({ length: 20 }, (_, frame) => input({
          jump_pressed: frame === 0,
          jump_held: frame < 8,
        })),
      },
      {
        name: 'playground-swim-dash',
        initial: { pos: [504, 456], speed: [0, 0], state: 'Swim' },
        inputs: Array.from({ length: 16 }, (_, frame) => input({
          move_x: 1,
          dash_pressed: frame === 0,
        })),
      },
    ] : []),
    ...(includePlaygroundBooster ? [
      {
        name: 'entity-4.1-archie',
        initial: { pos: [245, 400], speed: [0, 0] },
        inputs: Array.from({ length: 36 }, (_, frame) => input({
          move_x: 1,
          crouch_dash_pressed: frame === 0,
        })),
      },
      {
        name: 'playground-green-booster-auto',
        initial: { pos: [760, 440], speed: [0, 0] },
        inputs: Array.from({ length: 30 }, () => input()),
      },
      {
        name: 'playground-green-booster-right',
        initial: { pos: [760, 440], speed: [0, 0] },
        inputs: Array.from({ length: 24 }, (_, frame) => input({
          move_x: 1,
          dash_pressed: frame === 1,
        })),
      },
      {
        name: 'playground-green-booster-up',
        initial: { pos: [760, 440], speed: [0, 0] },
        inputs: Array.from({ length: 24 }, (_, frame) => input({
          move_y: -1,
          dash_pressed: frame === 1,
        })),
      },
      {
        name: 'playground-red-booster-auto',
        initial: { pos: [824, 440], speed: [0, 0] },
        inputs: Array.from({ length: 30 }, () => input()),
      },
      {
        name: 'playground-red-booster-right',
        initial: { pos: [824, 440], speed: [0, 0] },
        inputs: Array.from({ length: 30 }, (_, frame) => input({
          move_x: 1,
          dash_pressed: frame === 1,
        })),
      },
      {
        name: 'playground-red-booster-up',
        initial: { pos: [824, 440], speed: [0, 0] },
        inputs: Array.from({ length: 30 }, (_, frame) => input({
          move_y: -1,
          dash_pressed: frame === 1,
        })),
      },
    ] : []),
    ...(includePlaygroundWind ? [
      {
        name: 'playground-wind-idle',
        initial: { pos: [680, 200], speed: [0, 0] },
        inputs: Array.from({ length: 90 }, () => input()),
      },
      {
        name: 'playground-wind-dash-left',
        initial: { pos: [760, 200], speed: [0, 0] },
        inputs: Array.from({ length: 45 }, (_, frame) => input({
          move_x: -1,
          dash_pressed: frame === 8,
        })),
      },
      {
        name: 'playground-wind-ground-standing',
        initial: { pos: [820, 248], speed: [0, 0] },
        inputs: Array.from({ length: 45 }, () => input()),
      },
      {
        name: 'playground-wind-ground-ducking',
        initial: { pos: [820, 248], speed: [0, 0], ducking: true },
        inputs: Array.from({ length: 45 }, () => input({ move_y: 1 })),
      },
      {
        name: 'playground-wind-wall-shield',
        initial: { pos: [892, 248], speed: [0, 0] },
        inputs: Array.from({ length: 45 }, () => input()),
      },
    ] : []),
    ...(includePlaygroundStarFly ? [
      {
        name: 'mechanics-dash-attack-late-shield',
        initial: { pos: [55, 120], speed: [0, 0] },
        inputs: Array.from({ length: 40 }, (_, frame) => input({
          move_x: 1,
          dash_pressed: frame === 0,
        })),
      },
      {
        name: 'entity-4.12-featherboost',
        initial: { pos: [120, 200], speed: [0, 0] },
        inputs: Array.from({ length: 45 }, (_, frame) => input(
          frame >= 27 ? { move_x: 1, move_y: -1 } : {},
        )),
      },
      {
        name: 'entity-4.13-feather-super',
        initial: { pos: [900, 496], speed: [0, 0] },
        inputs: Array.from({ length: 50 }, (_, frame) => input({
          move_x: 1,
          jump_pressed: frame === 28,
          jump_held: frame >= 28 && frame < 40,
        })),
      },
      {
        name: 'entity-4.15.1-feather-clip',
        initial: { pos: [120, 320], speed: [0, 0] },
        inputs: Array.from({ length: 180 }, () => input({ move_y: 1 })),
      },
      {
        name: 'playground-starfly-right',
        initial: { pos: [120, 200], speed: [0, 0] },
        inputs: Array.from({ length: 180 }, () => input({ move_x: 1 })),
      },
      {
        name: 'playground-starfly-idle-timeout',
        initial: { pos: [120, 200], speed: [0, 0] },
        inputs: Array.from({ length: 180 }, () => input()),
      },
      {
        name: 'playground-starfly-up-exit',
        initial: { pos: [360, 400], speed: [0, 0] },
        inputs: Array.from({ length: 180 }, () => input({ move_y: -1 })),
      },
      {
        name: 'playground-starfly-turn-up',
        initial: { pos: [120, 200], speed: [0, 0] },
        inputs: Array.from({ length: 180 }, (_, frame) => input(
          frame < 55 ? { move_x: 1 } : { move_y: -1 },
        )),
      },
      {
        name: 'playground-starfly-turn-back',
        initial: { pos: [120, 200], speed: [0, 0] },
        inputs: Array.from({ length: 180 }, (_, frame) => input({
          move_x: frame < 55 ? 1 : -1,
        })),
      },
      {
        name: 'playground-starfly-dash-cancel',
        initial: { pos: [120, 200], speed: [0, 0] },
        inputs: Array.from({ length: 80 }, (_, frame) => input({
          move_x: 1,
          dash_pressed: frame === 50,
        })),
      },
      {
        name: 'playground-starfly-wall-jump-cancel',
        initial: { pos: [120, 200], speed: [0, 0] },
        inputs: Array.from({ length: 150 }, (_, frame) => input({
          move_x: 1,
          jump_pressed: frame === 120,
          jump_held: frame >= 120 && frame < 128,
        })),
      },
      {
        name: 'playground-starfly-wall-grab-cancel',
        initial: { pos: [120, 200], speed: [0, 0] },
        inputs: Array.from({ length: 150 }, (_, frame) => input({
          move_x: 1,
          grab_held: frame >= 120,
        })),
      },
      {
        name: 'playground-starfly-shield-bounce',
        initial: { pos: [120, 120], speed: [0, 0] },
        inputs: Array.from({ length: 30 }, () => input()),
      },
      {
        name: 'playground-starfly-shield-dash',
        initial: { pos: [80, 120], speed: [0, 0] },
        inputs: Array.from({ length: 80 }, (_, frame) => input({
          move_x: 1,
          dash_pressed: frame === 0,
        })),
      },
      {
        name: 'playground-starfly-renew',
        initial: { pos: [120, 320], speed: [0, 0] },
        inputs: Array.from({ length: 220 }, () => input({ move_x: 1 })),
      },
    ] : []),
    ...(includePlaygroundLaunch ? [
      {
        name: 'playground-launch-up',
        initial: { pos: [500, 400], speed: [0, -280], state: 'Launch' },
        inputs: Array.from({ length: 80 }, () => input()),
      },
      {
        name: 'playground-launch-side',
        initial: { pos: [500, 300], speed: [280, -150], state: 'Launch' },
        inputs: Array.from({ length: 80 }, () => input({ move_x: 1 })),
      },
      {
        name: 'playground-launch-dash-cancel',
        initial: { pos: [500, 300], speed: [280, -150], state: 'Launch' },
        inputs: Array.from({ length: 30 }, (_, frame) => input({
          move_x: 1,
          dash_pressed: frame === 0,
        })),
      },
      {
        name: 'playground-summit-launch',
        initial: { pos: [500, 400], speed: [0, 0], state: 'SummitLaunch' },
        inputs: Array.from({ length: 60 }, () => input()),
      },
    ] : []),
    ...(includePlaygroundBumper ? [
      {
        name: 'entity-4.3-bumper-clip',
        initial: { pos: [589, 206], speed: [0, 0] },
        inputs: Array.from({ length: 50 }, (_, frame) => input({
          move_x: 1,
          dash_pressed: frame === 4,
        })),
      },
      {
        name: 'entity-4.4-explosion-boost',
        initial: { pos: [589, 206], speed: [0, 0] },
        inputs: Array.from({ length: 30 }, () => input({ move_x: -1 })),
      },
      {
        name: 'playground-bumper-left-idle',
        initial: { pos: [589, 206], speed: [0, 0] },
        inputs: Array.from({ length: 80 }, () => input()),
      },
      {
        name: 'playground-bumper-left-hold',
        initial: { pos: [589, 206], speed: [0, 0] },
        inputs: Array.from({ length: 80 }, () => input({ move_x: -1 })),
      },
      {
        name: 'playground-bumper-right-hold',
        initial: { pos: [611, 206], speed: [0, 0] },
        inputs: Array.from({ length: 80 }, () => input({ move_x: 1 })),
      },
    ] : []),
    ...(includePlaygroundBadelineBoost ? [
      {
        name: 'playground-badeline-boost-launch',
        initial: { pos: [320, 400], speed: [0, 0] },
        inputs: Array.from({ length: 120 }, () => input()),
      },
      {
        name: 'playground-badeline-boost-summit',
        initial: { pos: [448, 400], speed: [0, 0] },
        inputs: Array.from({ length: 80 }, () => input()),
      },
    ] : []),
    ...(includePlaygroundMiscStates ? [
      {
        name: 'playground-dummy-state',
        initial: { pos: [500, 300], speed: [200, -100], state: 'Dummy' },
        inputs: Array.from({ length: 60 }, () => input()),
      },
      {
        name: 'playground-frozen-state',
        initial: { pos: [600, 300], speed: [60, 30], state: 'Frozen' },
        inputs: Array.from({ length: 20 }, () => input()),
      },
      {
        name: 'playground-temple-fall-state',
        initial: { pos: [200, 300], speed: [0, 0], state: 'TempleFall' },
        inputs: Array.from({ length: 140 }, () => input()),
      },
      {
        name: 'playground-reflection-fall-state',
        initial: { pos: [504, 300], speed: [0, 0], state: 'ReflectionFall' },
        inputs: Array.from({ length: 260 }, () => input()),
      },
    ] : []),
  ] : areaId === 2 ? [
    {
      name: 'dreamdash',
      initial: { pos: [776, -50], speed: [0, 0] },
      inputs: Array.from({ length: 40 }, (_, frame) => input({
        move_x: 1,
        dash_pressed: frame === 0,
      })),
    },
  ] : areaId === 4 ? [
    {
      name: 'swim-idle',
      initial: { pos: [1512, -248], speed: [0, 0] },
      inputs: Array.from({ length: 30 }, () => input()),
    },
  ] : [
    { name: 'run', inputs: Array.from({ length: 30 }, () => input({ move_x: 1 })) },
    {
      name: 'jump',
      inputs: Array.from({ length: 45 }, (_, frame) => input({
        jump_pressed: frame === 0,
        jump_held: frame < 12,
      })),
    },
    {
      name: 'dash',
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        dash_pressed: frame === 0,
      })),
    },
    {
      name: 'super',
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        jump_pressed: frame === 4,
        jump_held: frame >= 4 && frame < 10,
        dash_pressed: frame === 0,
      })),
    },
    {
      name: 'hyper',
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        move_y: frame <= 4 ? 1 : 0,
        jump_pressed: frame === 4,
        jump_held: frame >= 4 && frame < 10,
        dash_pressed: frame === 0,
      })),
    },
    {
      name: 'extended-super',
      inputs: Array.from({ length: 16 }, (_, frame) => input({
        move_x: 1,
        jump_pressed: frame === 10,
        jump_held: frame >= 10 && frame < 16,
        dash_pressed: frame === 0,
      })),
    },
    {
      name: 'superwave',
      inputs: Array.from({ length: 30 }, (_, frame) => input({
        move_x: frame <= 10 ? 1 : -1,
        move_y: frame >= 11 ? 1 : 0,
        jump_pressed: frame === 10 || frame === 26,
        jump_held: frame === 10 || frame === 26,
        dash_pressed: frame === 0 || frame === 11,
      })),
    },
    {
      name: 'reverse-super',
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: frame < 4 ? 1 : -1,
        jump_pressed: frame === 4,
        jump_held: frame >= 4 && frame < 10,
        dash_pressed: frame === 0,
      })),
    },
    {
      name: 'demodash',
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        crouch_dash_pressed: frame === 0,
      })),
    },
    {
      name: 'demohyper',
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        jump_pressed: frame === 4,
        jump_held: frame >= 4 && frame < 10,
        crouch_dash_pressed: frame === 0,
      })),
    },
    {
      name: 'up-diagonal-demo',
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        move_y: -1,
        crouch_dash_pressed: frame === 0,
      })),
    },
    {
      name: 'wavedash',
      initial: { pos: [70, 112], speed: [0, 0] },
      inputs: Array.from({ length: 18 }, (_, frame) => input({
        move_x: 1,
        move_y: frame <= 10 ? 1 : 0,
        jump_pressed: frame === 10,
        jump_held: frame >= 10 && frame < 16,
        dash_pressed: frame === 0,
      })),
    },
    {
      name: 'ultra',
      initial: { pos: [150, 45], speed: [0, 0] },
      inputs: Array.from({ length: 24 }, (_, frame) => input({
        move_x: 1,
        move_y: frame <= 4 ? 1 : 0,
        dash_pressed: frame === 0,
      })),
    },
    {
      name: 'wallbounce',
      initial: { pos: [140, 112], speed: [0, 0] },
      inputs: Array.from({ length: 14 }, (_, frame) => input({
        move_y: -1,
        jump_pressed: frame === 5,
        jump_held: frame >= 5 && frame < 12,
        dash_pressed: frame === 0,
      })),
    },
    {
      name: 'coyote-jump',
      initial: { pos: [42, 144], speed: [0, 0] },
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        jump_pressed: frame === 3,
        jump_held: frame >= 3 && frame < 9,
      })),
    },
    {
      name: 'buffered-jump',
      initial: { pos: [19, 135], speed: [0, 100] },
      inputs: Array.from({ length: 14 }, (_, frame) => input({
        jump_pressed: frame === 3,
        jump_held: frame >= 3 && frame < 11,
      })),
    },
    {
      name: 'fastfall',
      initial: { pos: [120, 60], speed: [0, 160] },
      inputs: Array.from({ length: 24 }, () => input({ move_y: 1 })),
    },
    {
      name: 'wall-slide',
      initial: { pos: [140, 96], speed: [0, 60] },
      inputs: Array.from({ length: 20 }, () => input({ move_x: 1 })),
    },
    {
      name: 'wall-jump',
      initial: { pos: [140, 112], speed: [0, 30] },
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: 1,
        jump_pressed: frame === 0,
        jump_held: frame < 6,
      })),
    },
    {
      name: 'climb',
      initial: { pos: [140, 112], speed: [0, 30] },
      inputs: Array.from({ length: 20 }, () => input({
        move_x: 1,
        move_y: -1,
        grab_held: true,
      })),
    },
    {
      name: 'mechanics-climbhop',
      initial: { pos: [140, 112], speed: [0, 30] },
      inputs: Array.from({ length: 90 }, () => input({
        move_x: 1,
        move_y: -1,
        grab_held: true,
      })),
    },
    {
      name: 'climb-jump',
      initial: { pos: [140, 112], speed: [0, 30] },
      inputs: Array.from({ length: 16 }, (_, frame) => input({
        move_x: 1,
        grab_held: frame <= 8,
        jump_pressed: frame === 8,
        jump_held: frame >= 8 && frame < 14,
      })),
    },
    {
      name: 'wallboost',
      initial: { pos: [140, 112], speed: [0, 30] },
      inputs: Array.from({ length: 12 }, (_, frame) => input({
        move_x: frame === 4 ? -1 : 0,
        grab_held: frame <= 3,
        jump_pressed: frame === 3,
        jump_held: frame >= 3 && frame < 10,
      })),
    },
    {
      name: 'stamina-cancel',
      initial: { pos: [140, 112], speed: [0, 30] },
      inputs: Array.from({ length: 30 }, (_, frame) => input({
        move_y: -1,
        grab_held: frame < 8 || frame >= 11,
      })),
    },
    {
      name: 'spike-death-respawn',
      initial: { pos: [60, 150], speed: [0, 90] },
      inputs: Array.from({ length: 120 }, () => input()),
    },
  ]
  const selectedScenarios = requestedScenarios.size === 0
    ? scenarios
    : scenarios.filter((scenario) => requestedScenarios.has(scenario.name))
  if (requestedScenarios.size > 0 && selectedScenarios.length !== requestedScenarios.size) {
    const found = new Set(selectedScenarios.map((scenario) => scenario.name))
    const missing = [...requestedScenarios].filter((name) => !found.has(name))
    throw new Error(`unknown E2E_SCENARIOS: ${missing.join(', ')}`)
  }
  const summaries = []
  for (const scenario of selectedScenarios) {
    const request = {
      map,
      ...(room ? { room } : {}),
      dream_dash: areaId === 2,
      inputs: scenario.inputs,
      initial_snapshot: {
        pos: [19, 144], speed: [0, 0], state: 'Normal', facing: 'Right',
        dashes: 1, stamina: 110, on_ground: false, ducking: false,
        can_dream_dash: areaId === 2,
        ...scenario.initial,
      },
      frames: scenario.inputs.length,
      skip_transitions: skipTransitions,
    }
    const response = await fetch(`http://127.0.0.1:${httpPortReservation.port}/api/simulate`, {
      method: 'POST',
      headers: { 'content-type': 'application/octet-stream' },
      body: encode(request),
    })
    const body = decode(new Uint8Array(await response.arrayBuffer()))
    if (!response.ok || !body.success) throw new Error(`${scenario.name} failed: ${JSON.stringify(body)}`)
    if (body.states.length !== request.frames + 1) throw new Error(`${scenario.name}: invalid state count`)
    if (!body.states[0]._everest_fields || Object.keys(body.states[0]._everest_fields).length < 100) {
      throw new Error(`${scenario.name}: real reflected Everest fields are missing`)
    }
    scenario.verify?.(body.states)
    const tracePath = resolve(root, '.tmp', `e2e-${scenario.name}-trace.json`)
    writeFileSync(tracePath, JSON.stringify({ inputs: request.inputs, states: body.states }))
    if (!collectOnly) {
      const compareArgs = [
        'run', '-q', '-p', 'celeste-physics', '--example', 'compare_real_trace', '--',
        tracePath, mapPath,
      ]
      if (room) compareArgs.push(room)
      run('cargo', compareArgs)
    }
    summaries.push({
      name: scenario.name,
      frames: body.states.length,
      first: pickCore(body.states[0]),
      last: pickCore(body.states.at(-1)),
      reflectedFieldCount: Object.keys(body.states[0]._everest_fields).length,
      tracePath,
    })
  }
  updateRunManifest(runContext, { status: 'completed' })
  console.log(JSON.stringify({ health, scenarios: summaries }, null, 2))
} catch (error) {
  runError = error
  updateRunManifest(runContext, { status: 'failed', error: String(error) })
  throw error
} finally {
  await Promise.allSettled([modPortReservation.release(), httpPortReservation.release()])
  const cleanup = {
    service_terminated: cleanupOwned('collector', service, serviceIdentity),
    game_terminated: cleanupOwned('Celeste', game, gameIdentity),
  }
  updateRunManifest(runContext, {
    status: runError ? 'failed-cleanup-finished' : 'cleanup-finished',
    cleanup,
  })
}

function pickCore(state) {
  return {
    frame: state._frame,
    pos: state.pos,
    speed: state.speed,
    state: state.state,
    facing: state.facing,
    dashes: state.dashes,
    stamina: state.stamina,
    on_ground: state.on_ground,
    ducking: state.ducking,
  }
}

function input(overrides = {}) {
  return {
    move_x: 0,
    move_y: 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: false,
    crouch_dash_pressed: false,
    grab_held: false,
    ...overrides,
  }
}

function semanticAssert(condition, scenario, message) {
  if (!condition) throw new Error(`${scenario}: semantic verification failed: ${message}`)
}

function field(state, name) {
  return state?._everest_fields?.[name]
}

function near(actual, expected, tolerance = 0.01) {
  return Math.abs(actual - expected) <= tolerance
}

function verifyDirectionalSpikesAway(states) {
  semanticAssert(states.every((state) => !state.dead), 'mechanics-directional-spikes-away', 'moving away from upward spikes must remain alive')
}

function verifyDirectionalSpikesInto(states) {
  semanticAssert(states[1]?.dead === true, 'mechanics-directional-spikes-into', 'moving into upward spikes must die on frame 1')
}

function verifyBerryTrain(states) {
  const first = states.findIndex((state) => Number(field(state, 'StrawberryCollectIndex')) >= 1)
  const second = states.findIndex((state) => Number(field(state, 'StrawberryCollectIndex')) >= 2)
  semanticAssert(first >= 27, 'mechanics-berry-train', `first berry collected too early at frame ${first}; follower delay plus nine safe-ground frames were not observed`)
  semanticAssert(second > first, 'mechanics-berry-train', 'second berry never collected')
  semanticAssert(second - first === 17, 'mechanics-berry-train', `later berry queue offset was ${second - first} frames instead of 17`)
}

function verifyUpwardScreenTransition(states) {
  const entered = states.findIndex((state, frame) => frame > 0
    && near(state.speed[0], 0)
    && near(state.speed[1], -105)
    && state.dashes === 0
    && near(state.stamina, 20))
  semanticAssert(entered > 0, 'mechanics-screen-transition-up', `BeforeUpTransition did not apply 0/-105 with delayed resource refill: ${JSON.stringify(states.slice(0, 6).map(pickCore))}`)
  const completed = states.findIndex((state, frame) => frame > entered && state.dashes >= 1 && near(state.stamina, 110))
  semanticAssert(completed - entered === 40, 'mechanics-screen-transition-up', `0.65 second transition plus the final coroutine resume took ${completed - entered} frames instead of 40`)
  semanticAssert(completed > 0 && near(states[completed].pos[1], -5), 'mechanics-screen-transition-up', `upward transition ended at y=${states[completed]?.pos[1]} instead of the source-derived target y=-5`)
}

function verifyZipMoverLiftboost(states) {
  const jumped = states.find((state) => state.speed[1] < -105.01)
  semanticAssert(jumped, 'mechanics-liftboost-zip-jump', 'jump never inherited the upward ZipMover lift speed')
  const retained = field(jumped, 'lastLiftSpeed')
  semanticAssert(Array.isArray(retained) && retained[1] < 0, 'mechanics-liftboost-zip-jump', `jump frame did not retain an upward lastLiftSpeed: ${JSON.stringify(retained)}`)
  semanticAssert(jumped.on_ground === false && jumped.dead === false, 'mechanics-liftboost-zip-jump', 'liftboost jump did not leave the moving platform alive')
}

function verifySpringCancel(states) {
  const dash = states.find((state) => state.state === 2)
  const beforeDashSpeed = field(dash, 'beforeDashSpeed')
  semanticAssert(Array.isArray(beforeDashSpeed), 'dash-spring-cancel', 'Dash did not expose beforeDashSpeed')
  semanticAssert(near(beforeDashSpeed[0], 0) && near(beforeDashSpeed[1], -185), 'dash-spring-cancel', `Dash replaced ${JSON.stringify(beforeDashSpeed)} instead of the floor spring 0/-185 velocity`)
  semanticAssert(dash.dashes === 0, 'dash-spring-cancel', 'buffered Dash did not spend the spring-refilled dash')
}

function verifySpikedWallbounce(states) {
  const launch = states[6]
  semanticAssert(states.every((state) => !state.dead), 'dash-spiked-wallbounce', 'on-time wallbounce touched the directional spikes lethally')
  semanticAssert(launch?.state === 0 && near(launch.speed[0], -170) && near(launch.speed[1], -160), 'dash-spiked-wallbounce', `entry-frame launch was ${JSON.stringify(launch?.speed)}`)
}

function verifyLateSpikedWallbounce(states) {
  semanticAssert(states.some((state) => state.dead), 'dash-spiked-wallbounce-late', 'one-frame-late input unexpectedly survived')
}

function verifySuperwave(states) {
  semanticAssert(near(states[11].speed[0], 260) && near(states[11].speed[1], -105) && states[11].dashes >= 1, 'dash-superwave', 'extended Super keyframe is missing')
  semanticAssert(states[22].on_ground && states[22].ducking && states[22].speed[0] < -200, 'dash-superwave', 'reverse down-diagonal landing keyframe is missing')
  semanticAssert(near(states[27].speed[0], -325) && near(states[27].speed[1], -52.5) && states[27].dashes >= 1, 'dash-superwave', 'reverse Hyper keyframe is missing')
}

function verifyDemodashGap(states) {
  semanticAssert(states.some((state) => state.ducking && state.pos[0] > 720), 'dash-demodash-gap', 'crouched dash never entered the six-pixel tunnel')
  semanticAssert(states.at(-1).pos[0] > 760 && !states.at(-1).dead, 'dash-demodash-gap', 'demo did not progress through the low tunnel')
}

function ultraLandingFrames(states) {
  const frames = []
  for (let frame = 1; frame < states.length; frame++) {
    const before = states[frame - 1]
    const after = states[frame]
    if (!before.ducking && after.ducking && after.speed[0] > before.speed[0] * 1.19) frames.push(frame)
  }
  return frames
}

function verifyUltra(states) {
  const landing = ultraLandingFrames(states).find((frame) => states[frame].state === 2)
  semanticAssert(landing !== undefined, 'dash-ultra', 'no in-Dash landing applied the 1.2 multiplier')
  const landed = states[landing]
  const expected = 240 * Math.SQRT1_2 * 1.2
  semanticAssert(near(landed.speed[0], expected) && near(landed.speed[1], 0), 'dash-ultra', `landing speed was ${JSON.stringify(landed.speed)} instead of ${expected}/0`)
  const dashDir = field(landed, 'DashDir')
  semanticAssert(Array.isArray(dashDir) && near(dashDir[0], 1) && near(dashDir[1], 0), 'dash-ultra', `landing did not flatten DashDir to 1/0: ${JSON.stringify(dashDir)}`)
}

function verifyGroundedUltra(states) {
  semanticAssert(states.some((state) => state.ducking && state.speed[0] >= 359.99), 'dash-grounded-ultra', 'grounded landing never preserved 300 entry speed and applied 1.2')
}

function verifyDelayedUltra(states) {
  const landing = ultraLandingFrames(states).find((frame) => states[frame].state !== 2)
  semanticAssert(landing !== undefined, 'dash-delayed-ultra', 'no post-Dash landing applied the delayed 1.2 multiplier')
}

function verifyChainedUltras(states) {
  const landings = ultraLandingFrames(states)
  semanticAssert(landings.length >= 2, 'dash-chained-ultras', `only ${landings.length} multiplicative landings were observed`)
  semanticAssert(states[landings[1]].speed[0] > states[landings[0]].speed[0], 'dash-chained-ultras', 'second Ultra did not compound the first')
}
