import type { GymMap, SimInput, SimState } from '../model'

interface ResponseMessage {
  id: number
  ok: boolean
  value?: unknown
  error?: string
}

export interface SimulationRunner {
  simulate(state: SimState, inputs: SimInput[], map: GymMap): Promise<SimState[]>
}

export class WasmClient implements SimulationRunner {
  private readonly worker = new Worker(new URL('./wasm.worker.ts', import.meta.url), { type: 'module' })
  private readonly pending = new Map<number, { resolve(value: unknown): void; reject(error: Error): void }>()
  private nextId = 1

  constructor() {
    this.worker.onmessage = (event: MessageEvent<ResponseMessage>) => {
      const response = event.data
      const request = this.pending.get(response.id)
      if (!request) return
      this.pending.delete(response.id)
      if (response.ok) request.resolve(response.value)
      else request.reject(new Error(response.error ?? 'WASM Worker 失败'))
    }
    this.worker.onerror = (event) => {
      const error = new Error(event.message || 'WASM Worker 无法启动')
      for (const request of this.pending.values()) request.reject(error)
      this.pending.clear()
    }
  }

  private request<T>(message: Record<string, unknown>): Promise<T> {
    const id = this.nextId++
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve: resolve as (value: unknown) => void, reject })
      this.worker.postMessage({ ...message, id })
    })
  }

  ready(): Promise<void> {
    return this.request({ type: 'ready' })
  }

  async loadMap(url: string, room: string, name: string): Promise<GymMap> {
    const response = await fetch(url)
    if (!response.ok) throw new Error(`地图文件加载失败：HTTP ${response.status}`)
    return this.request({ type: 'loadMap', bytes: await response.arrayBuffer(), room, name })
  }

  simulate(state: SimState, inputs: SimInput[], map: GymMap): Promise<SimState[]> {
    return this.request({ type: 'simulate', state, inputs, map })
  }

  dispose(): void {
    this.worker.terminate()
    this.pending.clear()
  }
}
