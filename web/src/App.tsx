import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { GameView } from './components/GameView'
import { InputTimeline } from './components/InputTimeline'
import { KeyBindings } from './components/KeyBindings'
import { StateInspector } from './components/StateInspector'
import { StartSettings, type StartConfiguration } from './components/StartSettings'
import {
  ACTIONS,
  ACTION_GLYPHS,
  ACTION_LABELS,
  DEFAULT_BINDINGS,
  PLAYGROUND,
  bindingLabel,
  buttonsToInput,
  createInitialState,
  makeEmptyButtons,
  type Action,
  type FrameButtons,
  type GymMap,
  type KeyBindings as Bindings,
  type SimState,
} from './model'
import { FrameCache } from './simulator/frameCache'
import { WasmClient } from './simulator/wasmClient'
import { compareTraces, createWebTrace, initialStateFromTrace, parseTrace } from './recording/trace'

interface RunDocument {
  version: 2
  map: GymMap
  initial_state: SimState
  inputs: FrameButtons[]
  bindings: Bindings
}

const PLAYGROUND_MAP_URL = '/assets/original/maps/CelesteGymPlayground-Playground.bin'
const DEFAULT_ROOM = 'playground'

function loadBindings(): Bindings {
  try {
    const saved = JSON.parse(localStorage.getItem('celeste-gym-bindings') ?? '') as Partial<Bindings>
    return { ...DEFAULT_BINDINGS, ...saved }
  } catch {
    return { ...DEFAULT_BINDINGS }
  }
}

function download(name: string, contents: string): void {
  const url = URL.createObjectURL(new Blob([contents], { type: 'application/json' }))
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = name
  anchor.click()
  URL.revokeObjectURL(url)
}

function buttonsFromKeys(keys: ReadonlySet<string>, bindings: Bindings, latched?: ReadonlySet<string>): FrameButtons {
  const buttons = makeEmptyButtons()
  for (const action of ACTIONS) {
    const code = bindings[action]
    buttons[action] = keys.has(code) || Boolean(latched?.has(code))
  }
  return buttons
}

export default function App() {
  const client = useMemo(() => new WasmClient(), [])
  const [map, setMap] = useState<GymMap>(() => structuredClone(PLAYGROUND))
  const cache = useMemo(() => new FrameCache(client, map, createInitialState(map), 360), [client])
  const [mode, setMode] = useState<'play' | 'advanced'>('play')
  const [liveState, setLiveState] = useState<SimState>(() => createInitialState(map))
  const liveStateRef = useRef(liveState)
  const [liveFrame, setLiveFrame] = useState(0)
  const liveFrameRef = useRef(0)
  const livePreviousButtons = useRef<FrameButtons>(makeEmptyButtons())
  const [, redraw] = useState(0)
  const [frame, setFrame] = useState(0)
  const frameRef = useRef(frame)
  const [playing, setPlaying] = useState(false)
  const [recording, setRecording] = useState(false)
  const [speed, setSpeed] = useState(1)
  const [wasmStatus, setWasmStatus] = useState<'loading' | 'ready' | 'error'>('loading')
  const [notice, setNotice] = useState('WASM 核心启动中…')
  const [bindings, setBindings] = useState<Bindings>(loadBindings)
  const [liveButtons, setLiveButtons] = useState<FrameButtons>(makeEmptyButtons)
  const [bindingsOpen, setBindingsOpen] = useState(false)
  const [startSettingsOpen, setStartSettingsOpen] = useState(false)
  const [startSettingsBusy, setStartSettingsBusy] = useState(false)
  const keys = useRef(new Set<string>())
  const latched = useRef(new Set<string>())
  const advancing = useRef(false)
  const calculationRevision = useRef(0)

  const replaceLiveSession = useCallback((initial: SimState) => {
    liveStateRef.current = initial
    setLiveState(initial)
    liveFrameRef.current = 0
    setLiveFrame(0)
    livePreviousButtons.current = makeEmptyButtons()
  }, [])

  useEffect(() => cache.subscribe(() => redraw((value) => value + 1)), [cache])
  useEffect(() => { frameRef.current = frame }, [frame])
  useEffect(() => () => client.dispose(), [client])

  useEffect(() => {
    let active = true
    client.ready().then(async () => {
      if (!active) return
      const decodedMap = await client.loadMap(
        PLAYGROUND_MAP_URL,
        DEFAULT_ROOM,
        'CelesteGymPlayground / playground',
      )
      if (!active) return
      const loadedMap = { ...decodedMap, room: DEFAULT_ROOM }
      setMap(loadedMap)
      const initial = createInitialState(loadedMap)
      cache.replace(loadedMap, initial, cache.getInputs().map((input) => ({ ...input })))
      replaceLiveSession(initial)
      frameRef.current = 0
      setFrame(0)
      setWasmStatus('ready')
      setNotice('WASM 已从 CelesteGymPlayground/Playground.bin 解码测试房间')
    }).catch((error: Error) => {
      if (!active) return
      setWasmStatus('error')
      setNotice(`WASM 加载失败：${error.message}`)
    })
    return () => { active = false }
  }, [cache, client, replaceLiveSession])

  useEffect(() => {
    if (mode !== 'play' || wasmStatus !== 'ready') return
    let active = true
    let animation = 0
    let last = performance.now()
    let carry = 0
    let simulating = false

    const tick = (now: number) => {
      carry += (now - last) * 60 / 1000
      last = now
      const steps = Math.min(6, Math.floor(carry))
      if (steps > 0 && !simulating) {
        carry -= steps
        let previous = livePreviousButtons.current
        const inputs = Array.from({ length: steps }, (_, offset) => {
          const current = buttonsFromKeys(keys.current, bindings, offset === 0 ? latched.current : undefined)
          const input = buttonsToInput(current, previous)
          previous = current
          return input
        })
        latched.current.clear()
        livePreviousButtons.current = previous
        simulating = true
        void client.simulate(liveStateRef.current, inputs, map).then((trace) => {
          if (!active) return
          const current = trace.at(-1)
          if (!current) throw new Error('WASM 未返回游玩状态')
          liveStateRef.current = current
          setLiveState(current)
          liveFrameRef.current += steps
          setLiveFrame(liveFrameRef.current)
        }).catch((error: Error) => {
          if (active) setNotice(error.message)
        }).finally(() => { simulating = false })
      }
      if (active) animation = requestAnimationFrame(tick)
    }
    animation = requestAnimationFrame(tick)
    return () => {
      active = false
      cancelAnimationFrame(animation)
    }
  }, [bindings, client, map, mode, wasmStatus])

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLElement && event.target.matches('input, textarea, select, button')) return
      if (!keys.current.has(event.code)) {
        latched.current.add(event.code)
        keys.current.add(event.code)
        setLiveButtons(buttonsFromKeys(keys.current, bindings))
      }
      keys.current.add(event.code)
      if (Object.values(bindings).includes(event.code)) event.preventDefault()
    }
    const up = (event: KeyboardEvent) => {
      if (keys.current.delete(event.code)) setLiveButtons(buttonsFromKeys(keys.current, bindings))
    }
    const blur = () => {
      keys.current.clear()
      latched.current.clear()
      setLiveButtons(makeEmptyButtons())
    }
    window.addEventListener('keydown', down)
    window.addEventListener('keyup', up)
    window.addEventListener('blur', blur)
    return () => {
      window.removeEventListener('keydown', down)
      window.removeEventListener('keyup', up)
      window.removeEventListener('blur', blur)
    }
  }, [bindings])

  const seek = useCallback((target: number, pause = true) => {
    const requested = Math.max(0, Math.round(target))
    cache.ensureCapacity(requested)
    const next = Math.min(cache.frameCount, requested)
    if (pause) {
      setPlaying(false)
      setRecording(false)
    }
    frameRef.current = next
    setFrame(next)
    if (!cache.getState(next)) {
      const requestRevision = ++calculationRevision.current
      setNotice(`从最近检查点计算到 F${next}…`)
      void cache.ensureFrame(next).then((state) => {
        if (state && requestRevision === calculationRevision.current && frameRef.current === next) setNotice(`F${next} 已由 WASM 计算并缓存`)
      }).catch((error: Error) => {
        if (requestRevision === calculationRevision.current) setNotice(error.message)
      })
    }
  }, [cache])

  useEffect(() => {
    if ((!playing && !recording) || wasmStatus !== 'ready') return
    let active = true
    let animation = 0
    let last = performance.now()
    let carry = 0

    const sampleButtons = (includeLatched: boolean): FrameButtons => {
      return buttonsFromKeys(keys.current, bindings, includeLatched ? latched.current : undefined)
    }

    const tick = (now: number) => {
      carry += (now - last) * 60 / 1000 * speed
      last = now
      const steps = Math.min(6, Math.floor(carry))
      if (steps > 0 && !advancing.current) {
        carry -= steps
        const start = frameRef.current
        const previousCapacity = cache.frameCount
        const target = start + steps
        cache.ensureCapacity(target)
        if (cache.frameCount !== previousCapacity) setNotice(`时间线容量自动扩展到 ${cache.frameCount} 帧`)
        if (recording) {
          cache.setButtonsRange(start, Array.from({ length: steps }, (_, offset) => sampleButtons(offset === 0)))
          latched.current.clear()
        }
        advancing.current = true
        void cache.ensureFrame(target).then((state) => {
          if (!active || !state) return
          frameRef.current = target
          setFrame(target)
        }).catch((error: Error) => {
          setPlaying(false)
          setRecording(false)
          setNotice(error.message)
        }).finally(() => { advancing.current = false })
      }
      if (active) animation = requestAnimationFrame(tick)
    }
    animation = requestAnimationFrame(tick)
    return () => {
      active = false
      cancelAnimationFrame(animation)
    }
  }, [bindings, cache, playing, recording, speed, wasmStatus])

  const inputs = cache.getInputs()
  const states = cache.getStates()
  const exactState = cache.getState(frame)
  const visible = exactState ? { frame, state: exactState } : cache.getNearestState(frame)

  const changeBinding = useCallback((action: Action, code: string) => {
    setBindings((current) => {
      const next = { ...current }
      const collision = ACTIONS.find((candidate) => candidate !== action && current[candidate] === code)
      if (collision) next[collision] = current[action]
      next[action] = code
      localStorage.setItem('celeste-gym-bindings', JSON.stringify(next))
      return next
    })
  }, [])

  const reset = () => {
    setPlaying(false)
    setRecording(false)
    cache.reset(createInitialState(map))
    seek(0)
    setNotice('输入和逐帧缓存已清空')
  }

  const applyStartConfiguration = async ({ room, position }: StartConfiguration) => {
    setPlaying(false)
    setRecording(false)
    setStartSettingsBusy(true)
    setNotice(`正在加载房间 ${room}…`)
    try {
      const decodedMap = await client.loadMap(
        PLAYGROUND_MAP_URL,
        room,
        `CelesteGymPlayground / ${room}`,
      )
      const configuredMap: GymMap = {
        ...decodedMap,
        room,
        spawn: { ...position },
      }
      calculationRevision.current += 1
      setMap(configuredMap)
      const initial = createInitialState(configuredMap)
      cache.replace(configuredMap, initial, cache.getInputs().map((input) => ({ ...input })))
      replaceLiveSession(initial)
      frameRef.current = 0
      setFrame(0)
      setStartSettingsOpen(false)
      setNotice(`已从房间 ${room} 的 (${position.x}, ${position.y}) 开始，时间线输入已保留`)
      void cache.ensureFrame(1)
    } catch (error) {
      setNotice(error instanceof Error ? `起点设置失败：${error.message}` : '起点设置失败')
    } finally {
      setStartSettingsBusy(false)
    }
  }

  const exportRun = () => {
    const document: RunDocument = {
      version: 2,
      map,
      initial_state: cache.getState(0) ?? createInitialState(map),
      inputs: inputs.map((input) => ({ ...input })),
      bindings,
    }
    download('celeste-gym-timeline.json', JSON.stringify(document, null, 2))
    setNotice('时间线已导出')
  }

  const exportTrace = async () => {
    try {
      const endFrame = frameRef.current
      await cache.ensureFrame(endFrame)
      const trace = createWebTrace(map, cache.getInputs(), cache.getStates(), endFrame, undefined, cache.getSimulationInputs(endFrame))
      download(`celeste-gym-web-${endFrame}-frames.trace.json`, JSON.stringify(trace, null, 2))
      setNotice(`已导出 F0–F${endFrame} 的输入和 ${trace.states.length} 个逐帧状态`)
    } catch (error) {
      setNotice(error instanceof Error ? error.message : '逐帧数据导出失败')
    }
  }

  const compareTrace = async (file: File) => {
    try {
      const expected = parseTrace(JSON.parse(await file.text()))
      const endFrame = expected.inputs.length
      const comparisonMap = expected.map.data ?? map
      setPlaying(false)
      setRecording(false)
      const traceMap = { ...comparisonMap, room: expected.map.room }
      setMap(traceMap)
      const initial = initialStateFromTrace(expected.states[0], traceMap)
      cache.replaceSimulationInputs(traceMap, initial, expected.inputs)
      replaceLiveSession(initial)
      await cache.ensureFrame(endFrame)
      frameRef.current = endFrame
      setFrame(endFrame)
      const actual = createWebTrace(traceMap, cache.getInputs(), cache.getStates(), endFrame, undefined, cache.getSimulationInputs(endFrame))
      const result = compareTraces(actual, expected)
      setNotice(result.matched
        ? `对比通过：${result.compared_frames} 帧，位置 ${result.max_position_error.toFixed(6)}，速度 ${result.max_speed_error.toFixed(6)}`
        : `对比失败：${result.reason}；位置 ${result.max_position_error.toFixed(6)}，速度 ${result.max_speed_error.toFixed(6)}`)
    } catch (error) {
      setNotice(error instanceof Error ? error.message : '逐帧数据对比失败')
    }
  }

  const importRun = async (file: File) => {
    try {
      const document = JSON.parse(await file.text()) as RunDocument
      if (document.version !== 2 || !document.map || !document.initial_state || !Array.isArray(document.inputs)) throw new Error('不是 Celeste Next Gym v2 时间线')
      setPlaying(false)
      setRecording(false)
      setMap(document.map)
      cache.replace(document.map, document.initial_state, document.inputs)
      replaceLiveSession(document.initial_state)
      if (document.bindings) setBindings({ ...DEFAULT_BINDINGS, ...document.bindings })
      seek(0)
      setNotice(`已导入 ${file.name} · 后续 state 将按需重算`)
    } catch (error) {
      setNotice(error instanceof Error ? error.message : '导入失败')
    }
  }

  const selectMode = (nextMode: 'play' | 'advanced') => {
    if (nextMode === 'play') {
      setPlaying(false)
      setRecording(false)
      setBindingsOpen(false)
      setStartSettingsOpen(false)
    }
    setMode(nextMode)
  }

  return <div className={`app-shell ${mode === 'play' ? 'play-mode' : 'advanced-mode'}`}>
    <nav className="mode-tabs" role="tablist" aria-label="页面模式">
      <button role="tab" aria-selected={mode === 'play'} onClick={() => selectMode('play')}>游玩</button>
      <button role="tab" aria-selected={mode === 'advanced'} onClick={() => selectMode('advanced')}>高级</button>
    </nav>

    {mode === 'play' ? <main className="play-workspace">
      <GameView map={map} state={liveState} states={[]} frame={liveFrame} stale={false} />
    </main> : <>
      <div className="mountain-backdrop" />
      <header className="topbar">
      <div className="brand-mark"><span className="wing">◇</span><div><strong>CELESTE</strong><em>NEXT GYM</em></div></div>
      <div className={`wasm-status ${wasmStatus}`} title="celeste-wasm 0.2.0 · rebuilt from current Rust source"><i />WASM 0.2.0 <span>{wasmStatus === 'ready' ? 'ONLINE' : wasmStatus === 'error' ? 'FAILED' : 'BOOTING'}</span></div>
      <div className="top-actions">
        <button disabled={wasmStatus !== 'ready'} onClick={() => setStartSettingsOpen(true)}>起点</button>
        <button onClick={() => setBindingsOpen(true)}>键位</button>
        <label className="file-button">导入时间线<input type="file" accept="application/json,.json" onChange={(event) => event.target.files?.[0] && void importRun(event.target.files[0])} /></label>
        <button onClick={exportRun}>导出时间线</button>
        <button onClick={() => void exportTrace()}>导出逐帧</button>
        <label className="file-button">对比逐帧<input type="file" accept="application/json,.json" onChange={(event) => event.target.files?.[0] && void compareTrace(event.target.files[0])} /></label>
      </div>
      </header>

      <main className="workspace">
      <section className="stage panel-frame">
        <div className="stage-header">
          <div><small>CELESTE 1.4.0.0-FNA · ROOM {map.room ?? DEFAULT_ROOM} · START {map.spawn.x}, {map.spawn.y}</small><h1>{map.name}</h1></div>
          <div className="cache-meter"><span>VALID THROUGH</span><strong>F{String(cache.computedThrough).padStart(4, '0')}</strong></div>
        </div>
        <GameView map={map} state={visible.state} states={states} frame={visible.frame} stale={!exactState || visible.frame !== frame} />
        <div className="transport">
          <button aria-label="回到第一帧" onClick={() => seek(0)}>│◀</button>
          <button aria-label="上一帧" onClick={() => seek(frame - 1)}>◀</button>
          <button className="play-button" disabled={wasmStatus !== 'ready'} aria-label={playing ? '暂停' : '播放'} onClick={() => { setRecording(false); setPlaying((value) => !value) }}>{playing ? 'Ⅱ' : '▶'}</button>
          <button aria-label="下一帧" onClick={() => seek(frame + 1)}>▶</button>
          <button className={recording ? 'record-button active' : 'record-button'} disabled={wasmStatus !== 'ready'} onClick={() => { setPlaying(false); setRecording((value) => !value) }}><i />{recording ? '录制中' : '录制输入'}</button>
          <select aria-label="播放速度" value={speed} onChange={(event) => setSpeed(Number(event.target.value))}>
            <option value={0.25}>0.25×</option><option value={0.5}>0.5×</option><option value={1}>1×</option><option value={2}>2×</option>
          </select>
          <button onClick={reset}>重置</button>
        </div>
      </section>

      <aside className="right-rail">
        <StateInspector frame={frame} state={visible.state} exact={Boolean(exactState)} />
        <section className="current-input panel-frame">
          <div className="panel-heading"><div><small>FRAME {String(Math.min(frame, inputs.length - 1)).padStart(4, '0')}</small><h2>当前输入</h2></div></div>
          <div className="input-buttons">
            {ACTIONS.map((action) => {
              const inputFrame = Math.min(frame, inputs.length - 1)
              const active = recording ? liveButtons[action] : inputs[inputFrame]?.[action] ?? false
              return <button key={action} className={`${action} ${active ? 'active' : ''}`} onClick={() => {
                cache.setFrame(inputFrame, action, !active)
                seek(inputFrame, false)
              }}><span>{ACTION_GLYPHS[action]}</span><strong>{ACTION_LABELS[action]}</strong><kbd>{bindingLabel(bindings[action])}</kbd></button>
            })}
          </div>
        </section>
        <div className="notice" role="status"><i className={wasmStatus} />{notice}</div>
      </aside>

      <InputTimeline
        frame={frame}
        inputs={inputs}
        states={states}
        onSeek={seek}
        onPaint={(action, from, to, value) => cache.paint(action, from, to, value)}
        onMove={(action, targetAction, start, end, delta) => cache.moveRun(action, targetAction, start, end, delta)}
        onEditComplete={() => {
          const current = frameRef.current
          if (!cache.getState(current)) seek(current, false)
        }}
        onResize={(frames) => {
          cache.resize(frames)
          if (frameRef.current > cache.frameCount) seek(cache.frameCount)
        }}
      />
      </main>
      <footer>celeste-wasm 0.2.0 rebuilt · Celeste 1.4.0.0-fna Gameplay atlas · CelesteGymPlayground/Playground.bin · 60 FPS</footer>
      {bindingsOpen && <KeyBindings bindings={bindings} onChange={changeBinding} onClose={() => setBindingsOpen(false)} />}
      {startSettingsOpen && <StartSettings
        room={map.room ?? DEFAULT_ROOM}
        position={map.spawn}
        busy={startSettingsBusy}
        onApply={(configuration) => { void applyStartConfiguration(configuration) }}
        onClose={() => setStartSettingsOpen(false)}
      />}
    </>}
  </div>
}
