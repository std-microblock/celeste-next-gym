import { readFile } from 'node:fs/promises'
import { decode, encode } from '@msgpack/msgpack'
import init, {
  cache_simulation_map_msgpack,
  decode_celeste_map_msgpack,
  simulate_cached_map_msgpack,
  simulate_msgpack,
} from '../src/wasm/celeste_wasm.js'

await init({ module_or_path: await readFile(new URL('../src/wasm/celeste_wasm_bg.wasm', import.meta.url)) })

const state = {
  pos: { x: 64, y: 496 },
  speed: { x: 0, y: 0 },
  state: 'Normal',
  facing: true,
  dashes: 1,
  stamina: 110,
  on_ground: false,
  ducking: false,
  can_dream_dash: true,
  dead: false,
  death_freeze_pending: false,
  respawn_frames: 0,
  dash_dir: { x: 0, y: 0 },
}

const input = {
  move_x: 1,
  move_y: 0,
  jump_pressed: false,
  jump_held: false,
  dash_pressed: false,
  crouch_dash_pressed: false,
  grab_held: false,
  talk_pressed: false,
}

const map = {
  bounds: { x: 0, y: 0, width: 960, height: 544 },
  spawn: { x: 64, y: 496 },
  solids: [{ x: 0, y: 496, width: 960, height: 48 }],
  entities: [],
  source_package: 'CelesteGymPlayground',
}

const response = decode(simulate_msgpack(encode(state), encode([input]), encode(map), 1))
if (!response.states || response.states.length !== 2) throw new Error(response.error ?? 'WASM smoke test failed')
const delayedLeftInputs = Array.from({ length: 6 }, (_, frame) => ({
  ...input,
  move_x: frame >= 2 ? -1 : 0,
  dash_pressed: frame === 1,
}))
const delayedLeftTrace = decode(simulate_msgpack(encode(state), encode(delayedLeftInputs), encode(map), delayedLeftInputs.length))
const dashBegin = delayedLeftTrace.states?.[2]
const dashLaunch = delayedLeftTrace.states?.[6]
if (!dashBegin || dashBegin.speed.x !== 0 || dashBegin.speed.y !== 0 || dashBegin.dash_dir.x !== 0 || dashBegin.dash_dir.y !== 0) {
  throw new Error(delayedLeftTrace.error ?? 'DashBegin locked aim before DashCoroutine resumed')
}
if (!dashLaunch || dashLaunch.speed.x !== -240 || dashLaunch.speed.y !== 0 || dashLaunch.dash_dir.x !== -1 || dashLaunch.dash_dir.y !== 0 || dashLaunch.facing !== false) {
  throw new Error(delayedLeftTrace.error ?? 'DashCoroutine did not sample frame-3 left input')
}
for (const facing of [false, true]) {
  for (const [move_y, speed] of [[-1, { x: 123, y: -80 }], [1, { x: -123, y: 80 }]]) {
    const verticalEntryTrace = decode(simulate_msgpack(encode({
      ...state,
      pos: { x: 64, y: 300 },
      speed,
      facing,
    }), encode(Array.from({ length: 5 }, (_, frame) => ({
      ...input,
      move_x: 0,
      move_y,
      dash_pressed: frame === 0,
    }))), encode(map), 5))
    const verticalEntry = verticalEntryTrace.states?.[1]
    const verticalLaunch = verticalEntryTrace.states?.[5]
    if (!verticalEntry
      || verticalEntry.speed.x !== 0
      || verticalEntry.speed.y !== 0
      || verticalEntry.dash_dir.x !== 0
      || verticalEntry.dash_dir.y !== 0
      || verticalEntry.pos.x !== 64
      || verticalEntry.pos.y !== 300
      || verticalEntry.facing !== facing) {
      throw new Error(verticalEntryTrace.error ?? `Vertical DashBegin retained velocity while facing ${facing ? 'right' : 'left'}`)
    }
    if (!verticalLaunch
      || verticalLaunch.speed.x !== 0
      || verticalLaunch.speed.y !== move_y * 240
      || verticalLaunch.dash_dir.x !== 0
      || verticalLaunch.dash_dir.y !== move_y) {
      throw new Error(verticalEntryTrace.error ?? `Vertical dash launch retained rightward velocity while facing ${facing ? 'right' : 'left'}`)
    }
  }
}
const mapBytes = await readFile(new URL('../public/assets/original/maps/CelesteGymPlayground-Playground.bin', import.meta.url))
const decodedMap = decode(decode_celeste_map_msgpack(mapBytes, 'playground'))
if (!decodedMap.map || decodedMap.map.source_package !== 'CelesteGymPlayground') throw new Error(decodedMap.error ?? 'WASM map decode failed')
const decodedKinds = new Set(decodedMap.map.entities.map((entity) => entity.kind))
for (const required of ['water', 'dream_block', 'booster', 'red_booster', 'fly_feather', 'bumper', 'ice_ball', 'badeline_boost', 'wind']) {
  if (!decodedKinds.has(required)) throw new Error(`Decoded playground is missing ${required}`)
}
const playgroundState = { ...state, pos: decodedMap.map.spawn }
cache_simulation_map_msgpack(encode(decodedMap.map))
const playgroundTrace = decode(simulate_cached_map_msgpack(encode(playgroundState), encode([input]), 1))
if (!playgroundTrace.states || playgroundTrace.states.length !== 2) throw new Error(playgroundTrace.error ?? 'Decoded playground simulation failed')
const runTrace = decode(simulate_msgpack(encode(playgroundState), encode(Array.from({ length: 30 }, () => input)), encode(decodedMap.map), 30))
if (!runTrace.states || runTrace.states.at(-1).pos.x <= decodedMap.map.spawn.x) throw new Error(runTrace.error ?? 'Playground movement smoke test failed')
const longMap = {
  ...map,
  bounds: { x: 0, y: 0, width: 4096, height: 544 },
  solids: [{ x: 0, y: 496, width: 4096, height: 48 }],
}
const longTrace = decode(simulate_msgpack(encode(state), encode(Array.from({ length: 800 }, () => input)), encode(longMap), 800))
const longFinal = longTrace.states?.at(-1)
if (!longFinal || longFinal.pos.x < 1200 || Math.abs(longFinal.speed.x - 90) > .01) throw new Error(longTrace.error ?? 'Long-running movement stopped unexpectedly')
console.log(`WASM smoke test passed: decoded ${decodedMap.map.source_package}/playground; run x=${runTrace.states.at(-1).pos.x}; long-run x=${longFinal.pos.x}`)
