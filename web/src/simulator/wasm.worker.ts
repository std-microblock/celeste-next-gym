import { decode, encode } from '@msgpack/msgpack'
import type { GymMap, SimInput, SimState } from '../model'
import init, { decode_celeste_map_msgpack, simulate_msgpack } from '../wasm/celeste_wasm.js'

type Request =
  | { id: number; type: 'ready' }
  | { id: number; type: 'loadMap'; bytes: ArrayBuffer; room: string; name: string }
  | { id: number; type: 'simulate'; state: SimState; inputs: SimInput[]; map: GymMap }

const ready = init()

self.onmessage = async (event: MessageEvent<Request>) => {
  const request = event.data
  try {
    await ready
    if (request.type === 'ready') {
      self.postMessage({ id: request.id, ok: true, value: 'ready' })
      return
    }
    if (request.type === 'loadMap') {
      const response = decode(new Uint8Array(decode_celeste_map_msgpack(new Uint8Array(request.bytes), request.room))) as { map?: Omit<GymMap, 'name'>; error?: string }
      if (!response.map) throw new Error(response.error ?? 'WASM 无法解码 Celeste 地图')
      self.postMessage({ id: request.id, ok: true, value: { ...response.map, name: request.name } })
      return
    }
    const bytes = simulate_msgpack(
      encode(request.state),
      encode(request.inputs),
      encode({
        bounds: request.map.bounds,
        spawn: request.map.spawn,
        solids: request.map.solids,
        entities: request.map.entities,
        source_package: request.map.source_package,
      }),
      request.inputs.length,
    )
    const response = decode(bytes) as { states?: SimState[]; error?: string }
    if (!response.states) throw new Error(response.error ?? 'WASM 返回了无效轨迹')
    self.postMessage({ id: request.id, ok: true, value: response.states })
  } catch (error) {
    self.postMessage({ id: request.id, ok: false, error: error instanceof Error ? error.message : String(error) })
  }
}
