import type { GymMap, SimInput, SimState } from '../model'
import type { TrainingCandidate } from '../training/session'

interface ResponseMessage {
  id: number
  ok: boolean
  value?: unknown
  error?: string
}

export interface SimulationRunner {
  simulate(state: SimState, inputs: SimInput[], map: GymMap): Promise<SimState[]>
}

export interface FuzzSearchResult { candidates: TrainingCandidate[] }

export class WasmClient implements SimulationRunner {
  private readonly worker = new Worker(new URL('./wasm.worker.ts', import.meta.url), { type: 'module' })
  private readonly pending = new Map<number, { resolve(value: unknown): void; reject(error: Error): void }>()
  private readonly mapDownloads = new Map<string, Promise<ArrayBuffer>>()
  private nextId = 1
  private cachedMap: GymMap | undefined
  private mapVersion = 0

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
    let download = this.mapDownloads.get(url)
    if (!download) {
      download = fetch(url).then((response) => {
        if (!response.ok) throw new Error(`地图文件加载失败：HTTP ${response.status}`)
        return response.arrayBuffer()
      })
      this.mapDownloads.set(url, download)
    }
    return this.request({ type: 'loadMap', bytes: await download, room, name })
  }

  simulate(state: SimState, inputs: SimInput[], map: GymMap): Promise<SimState[]> {
    let mapPayload: GymMap | undefined
    if (map !== this.cachedMap) {
      this.cachedMap = map
      this.mapVersion += 1
      mapPayload = map
    }
    return this.request({ type: 'simulate', state, inputs, map: mapPayload, mapVersion: this.mapVersion })
  }

  fuzzSearch(state: SimState, fuzz: string, map: GymMap): Promise<FuzzSearchResult> {
    let mapPayload: GymMap | undefined
    if (map !== this.cachedMap) {
      this.cachedMap = map
      this.mapVersion += 1
      mapPayload = map
    }
    return this.request({ type: 'fuzzSearch', state, fuzz, map: mapPayload, mapVersion: this.mapVersion })
  }

  entryCheck(state: SimState, checks: string[]): Promise<boolean> {
    return this.request({ type: 'entryCheck', state, checks })
  }

  dispose(): void {
    this.worker.terminate()
    this.pending.clear()
  }
}
