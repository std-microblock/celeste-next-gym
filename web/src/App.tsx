import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { GameView } from './components/GameView'
import { InputTimeline } from './components/InputTimeline'
import { KeyBindings } from './components/KeyBindings'
import { StateInspector } from './components/StateInspector'
import {
  ACTIONS,
  ACTION_GLYPHS,
  ACTION_LABELS,
  DEFAULT_BINDINGS,
  PLAYGROUND,
  bindingLabel,
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
  const keys = useRef(new Set<string>())
  const latched = useRef(new Set<string>())
  const advancing = useRef(false)
  const calculationRevision = useRef(0)

  useEffect(() => cache.subscribe(() => redraw((value) => value + 1)), [cache])
  useEffect(() => { frameRef.current = frame }, [frame])
  useEffect(() => () => client.dispose(), [client])

  useEffect(() => {
    let active = true
    client.ready().then(async () => {
      if (!active) return
      const decodedMap = await client.loadMap(
        '/assets/original/maps/CelesteGymPlayground-Playground.bin',
        'playground',
        'CelesteGymPlayground / playground',
      )
      if (!active) return
      setMap(decodedMap)
      cache.replace(decodedMap, createInitialState(decodedMap), cache.getInputs().map((input) => ({ ...input })))
      frameRef.current = 0
      setFrame(0)
      setWasmStatus('ready')
      setNotice('WASM 已从 CelesteGymPlayground/Playground.bin 解码测试房间')
      void cache.ensureFrame(1)
    }).catch((error: Error) => {
      if (!active) return
      setWasmStatus('error')
      setNotice(`WASM 加载失败：${error.message}`)
    })
    return () => { active = false }
  }, [cache, client])

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if ((event.target as HTMLElement | null)?.matches('input, textarea, select, button')) return
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
      setMap(comparisonMap)
      cache.replaceSimulationInputs(comparisonMap, initialStateFromTrace(expected.states[0], comparisonMap), expected.inputs)
      await cache.ensureFrame(endFrame)
      frameRef.current = endFrame
      setFrame(endFrame)
      const actual = createWebTrace(comparisonMap, cache.getInputs(), cache.getStates(), endFrame, undefined, cache.getSimulationInputs(endFrame))
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
      if (document.bindings) setBindings({ ...DEFAULT_BINDINGS, ...document.bindings })
      seek(0)
      setNotice(`已导入 ${file.name} · 后续 state 将按需重算`)
    } catch (error) {
      setNotice(error instanceof Error ? error.message : '导入失败')
    }
  }

  return <div className="app-shell">
    <div className="mountain-backdrop" />
    <header className="topbar">
      <div className="brand-mark"><span className="wing">◇</span><div><strong>CELESTE</strong><em>NEXT GYM</em></div></div>
      <div className={`wasm-status ${wasmStatus}`} title="celeste-wasm 0.2.0 · rebuilt from current Rust source"><i />WASM 0.2.0 <span>{wasmStatus === 'ready' ? 'ONLINE' : wasmStatus === 'error' ? 'FAILED' : 'BOOTING'}</span></div>
      <div className="top-actions">
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
          <div><small>CELESTE 1.4.0.0-FNA · BINARYPACKER TEST ROOM</small><h1>{map.name}</h1></div>
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
  </div>
}
