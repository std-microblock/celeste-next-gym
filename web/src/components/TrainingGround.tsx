import { useEffect, useMemo, useRef, useState } from 'react'
import { DEFAULT_BINDINGS, buttonsToInput, makeEmptyButtons, type FrameButtons, type GymMap, type KeyBindings, type SimState } from '../model'
import { WasmClient } from '../simulator/wasmClient'
import { assistedRate, candidateWindow, createTrainingSession, keySemantics, nextTargetFrame, rebuildTrainingSession, verifiedInputs, verifyTrainingInput, type TrainingCandidate, type TrainingDefinition, type TrainingSession } from '../training/session'
import { GameView } from './GameView'
import { TrainingTimeline } from './TrainingTimeline'

interface TrainingDocument extends TrainingDefinition {
  scene: { map_fixture: string; initial_snapshot: string }
  entry: TrainingDefinition['entry'] & { check: string[]; failure: { title: string; body: string } }
  teaching: { steps: Array<{ prompt: string; order_error: { title: string; body: string }; window_error: { title: string; body: string } }> }
  assist: { auto_slowdown: { enabled_by_default: boolean; radius_frames: number; minimum_multiplier: number } }
}

interface Attempt { frame: number; keys: string[]; entryCheckPassed?: boolean }
interface VerificationLog { frame: number; actual: string[]; expected: string[]; outcome: 'accepted' | 'ignored' | 'entry_check_failed' | 'input_order_mismatch' | 'timing_window_miss' }

function buttonsFromKeyboard(keys: ReadonlySet<string>, bindings: KeyBindings): FrameButtons {
  return {
    up: keys.has(bindings.up), down: keys.has(bindings.down), left: keys.has(bindings.left), right: keys.has(bindings.right),
    jump: keys.has(bindings.jump), dash: keys.has(bindings.dash), grab: keys.has(bindings.grab),
  }
}

function pressedVerification(current: FrameButtons, previous: FrameButtons): boolean {
  return (current.dash && !previous.dash) || (current.jump && !previous.jump) || (current.grab && !previous.grab)
}

function verificationKeys(buttons: FrameButtons, document: TrainingDefinition, step: number): string[] {
  const expected = verifiedInputs(document)[step]?.keys ?? []
  const actual = keySemantics(buttons)
  // With verify:false direction holds, only the new action belongs to the
  // teaching input. A definition that includes a direction in verify:true
  // deliberately opts into strict directional matching.
  return expected.some((key) => ['up', 'down', 'left', 'right'].includes(key))
    ? actual
    : actual.filter((key) => !['up', 'down', 'left', 'right'].includes(key))
}

/** The lesson runner owns simulation and review state, while its timeline is read-only. */
export function TrainingGround() {
  const client = useMemo(() => new WasmClient(), [])
  const [document, setDocument] = useState<TrainingDocument | null>(null)
  const [map, setMap] = useState<GymMap | null>(null)
  const [initial, setInitial] = useState<SimState | null>(null)
  const [candidates, setCandidates] = useState<TrainingCandidate[]>([])
  const [session, setSession] = useState<TrainingSession>(() => createTrainingSession([]))
  const sessionRef = useRef(session)
  const [snapshots, setSnapshots] = useState<SimState[]>([])
  const snapshotsRef = useRef<SimState[]>([])
  const [frame, setFrame] = useState(0)
  const frameRef = useRef(0)
  const [playing, setPlaying] = useState(false)
  const [baseRate, setBaseRate] = useState(1)
  const [autoSlowdown, setAutoSlowdown] = useState(true)
  const [resetFrame, setResetFrame] = useState(0)
  const [fuzzStartFrame, setFuzzStartFrame] = useState<number | null>(null)
  const fuzzStartRef = useRef<number | null>(null)
  const [notice, setNotice] = useState('正在加载训练定义…')
  const keys = useRef(new Set<string>())
  const previousButtons = useRef<FrameButtons>(makeEmptyButtons())
  const attempts = useRef<Attempt[]>([])
  const [attemptRevision, setAttemptRevision] = useState(0)
  const [verificationLog, setVerificationLog] = useState<VerificationLog[]>([])
  const simulating = useRef(false)

  const applySession = (next: TrainingSession) => { sessionRef.current = next; setSession(next) }

  useEffect(() => () => client.dispose(), [client])
  useEffect(() => {
    let active = true
    void (async () => {
      try {
        await client.ready()
        const root = '/training/hyper-basic.training.json'
        const training = await fetch(root).then(async (response) => {
          if (!response.ok) throw new Error(`训练定义加载失败：HTTP ${response.status}`)
          return response.json() as Promise<TrainingDocument>
        })
        const [loadedMap, loadedInitial] = await Promise.all([
          fetch(`/${training.scene.map_fixture}`).then((response) => response.json() as Promise<GymMap>),
          fetch(`/${training.scene.initial_snapshot}`).then((response) => response.json() as Promise<SimState>),
        ])
        const result = await client.fuzzSearch(loadedInitial, JSON.stringify(training.fuzz), loadedMap)
        if (!active) return
        if (result.candidates.length === 0) throw new Error('该训练定义没有成功候选，无法生成教学窗口')
        setDocument(training)
        setMap(loadedMap)
        setInitial(loadedInitial)
        setCandidates(result.candidates)
        applySession(createTrainingSession(result.candidates))
        snapshotsRef.current = [loadedInitial]
        setSnapshots([loadedInitial])
        setAutoSlowdown(training.assist.auto_slowdown.enabled_by_default)
        setNotice(`已加载 ${training.title} · ${result.candidates.length} 个可行候选`)
      } catch (error) {
        if (active) setNotice(error instanceof Error ? error.message : '训练加载失败')
      }
    })()
    return () => { active = false }
  }, [client])

  const seek = (requested: number) => {
    const next = Math.max(0, Math.min(snapshotsRef.current.length - 1, Math.round(requested)))
    setPlaying(false)
    frameRef.current = next
    setFrame(next)
    if (!document) return
    const start = attempts.current.find((attempt) => attempt.entryCheckPassed === true && attempt.keys.includes('dash'))?.frame ?? null
    if (start === null || next < start) {
      fuzzStartRef.current = null
      setFuzzStartFrame(null)
      applySession(createTrainingSession(candidates))
      setAttemptRevision((value) => value + 1)
      return
    }
    fuzzStartRef.current = start
    const replay = start === null ? [] : attempts.current
      .filter((attempt) => attempt.frame >= start && attempt.frame <= next)
      .map((attempt) => ({ ...attempt, frame: attempt.frame - start }))
    applySession(rebuildTrainingSession(document, candidates, replay))
    setFuzzStartFrame(start)
    setAttemptRevision((value) => value + 1)
  }

  const resetTo = (target = resetFrame) => {
    const snapshot = snapshotsRef.current[target] ?? initial
    if (!snapshot) return
    snapshotsRef.current = snapshotsRef.current.slice(0, target + 1)
    setSnapshots([...snapshotsRef.current])
    attempts.current = attempts.current.filter((attempt) => attempt.frame < target)
    if (fuzzStartRef.current !== null && target <= fuzzStartRef.current) {
      fuzzStartRef.current = null
      setFuzzStartFrame(null)
    }
    previousButtons.current = makeEmptyButtons()
    seek(target)
    setNotice(`已回到 R 点 F${target}`)
  }

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLElement && event.target.matches('input, select, button')) return
      keys.current.add(event.code)
      const gameInput = Object.values(DEFAULT_BINDINGS).includes(event.code)
      if (gameInput || event.code === 'KeyR') event.preventDefault()
      if (gameInput) setPlaying(true)
      if (event.code === 'KeyR' && !event.repeat) resetTo()
    }
    const up = (event: KeyboardEvent) => { keys.current.delete(event.code) }
    window.addEventListener('keydown', down)
    window.addEventListener('keyup', up)
    return () => { window.removeEventListener('keydown', down); window.removeEventListener('keyup', up) }
  // resetTo intentionally reads current state through React's render closure.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [resetFrame, initial, document, candidates])

  useEffect(() => {
    if (!playing || !document || !map || !initial || session.phase === 'failed') return
    let active = true
    let animation = 0
    let last = performance.now()
    let carry = 0
    const tick = (now: number) => {
      const fuzzInputIndex = verifiedInputs(document)[sessionRef.current.nextVerifiedInput]?.fuzzInputIndex
      const target = fuzzInputIndex === undefined ? undefined : nextTargetFrame(sessionRef.current.candidates, fuzzInputIndex)
      const fuzzFrame = fuzzStartRef.current === null ? 0 : Math.max(0, frameRef.current - fuzzStartRef.current)
      const rate = autoSlowdown ? assistedRate(baseRate, fuzzFrame, target, document.assist.auto_slowdown.radius_frames, document.assist.auto_slowdown.minimum_multiplier) : baseRate
      carry += Math.min(250, now - last) * 60 / 1000 * rate
      last = now
      if (carry >= 1 && !simulating.current) {
        carry -= 1
        const currentFrame = frameRef.current
        const current = buttonsFromKeyboard(keys.current, DEFAULT_BINDINGS)
        const input = buttonsToInput(current, previousButtons.current)
        const beforeSession = sessionRef.current
        const triggers = pressedVerification(current, previousButtons.current)
        const shouldVerify = beforeSession.phase === 'pre_fuzz' ? current.dash && !previousButtons.current.dash : triggers
        previousButtons.current = current
        simulating.current = true
        void client.simulate(snapshotsRef.current[currentFrame]!, [input], map).then(async (trace) => {
          if (!active) return
          const after = trace.at(-1)
          if (!after) throw new Error('训练模拟未返回状态')
          const nextFrame = currentFrame + 1
          snapshotsRef.current = [...snapshotsRef.current.slice(0, nextFrame), after]
          attempts.current = attempts.current.filter((attempt) => attempt.frame <= currentFrame)
          setSnapshots(snapshotsRef.current)
          frameRef.current = nextFrame
          setFrame(nextFrame)
          if (shouldVerify) {
            const localFrame = beforeSession.phase === 'pre_fuzz' ? 0 : currentFrame - (fuzzStartRef.current ?? currentFrame)
            const semanticKeys = verificationKeys(current, document, beforeSession.nextVerifiedInput)
            const entryPassed = beforeSession.phase === 'pre_fuzz' ? await client.entryCheck(after, document.entry.check) : true
            const nextSession = verifyTrainingInput(beforeSession, document, localFrame, semanticKeys, entryPassed)
            attempts.current = [...attempts.current, { frame: currentFrame, keys: semanticKeys, entryCheckPassed: entryPassed }]
            const expected = verifiedInputs(document)[beforeSession.nextVerifiedInput]?.keys ?? []
            const outcome: VerificationLog['outcome'] = nextSession.phase === 'failed'
              ? nextSession.failure?.kind ?? 'input_order_mismatch'
              : 'accepted'
            setVerificationLog((entries) => [{
              frame: currentFrame,
              actual: semanticKeys,
              expected,
              outcome,
            }, ...entries].slice(0, 6))
            if (beforeSession.phase === 'pre_fuzz' && nextSession.phase === 'fuzz') {
              fuzzStartRef.current = currentFrame
              setFuzzStartFrame(currentFrame)
            }
            applySession(nextSession)
            setAttemptRevision((value) => value + 1)
            if (nextSession.phase === 'failed') {
              setPlaying(false)
              setNotice(nextSession.failure?.kind === 'input_order_mismatch' ? document.teaching.steps[nextSession.nextVerifiedInput]?.order_error.body ?? '输入顺序不正确。' : nextSession.failure?.kind === 'timing_window_miss' ? document.teaching.steps[nextSession.nextVerifiedInput]?.window_error.body ?? '错过输入窗口。' : document.entry.failure.body)
            } else if (nextSession.phase === 'success') {
              setNotice('成功：已命中可行候选。')
              setPlaying(false)
            }
          }
        }).catch((error: Error) => { if (active) { setPlaying(false); setNotice(error.message) } }).finally(() => { simulating.current = false })
      }
      if (active) animation = requestAnimationFrame(tick)
    }
    animation = requestAnimationFrame(tick)
    return () => { active = false; cancelAnimationFrame(animation) }
  }, [autoSlowdown, baseRate, client, document, initial, map, playing, session.phase])

  if (!document || !map || !initial || snapshots.length === 0) return <main className="training-workspace"><div className="notice"><i />{notice}</div></main>
  const state = snapshots[frame] ?? snapshots.at(-1) ?? initial
  const fuzzInputIndex = verifiedInputs(document)[session.nextVerifiedInput]?.fuzzInputIndex
  const target = fuzzInputIndex === undefined ? undefined : nextTargetFrame(session.candidates, fuzzInputIndex)
  const globalTarget = fuzzStartFrame === null || target === undefined ? undefined : fuzzStartFrame + target
  const windows = fuzzInputIndex === undefined ? [] : candidateWindow(session.candidates, fuzzInputIndex).map((window) => ({ from: (fuzzStartFrame ?? 0) + window.from, to: (fuzzStartFrame ?? 0) + window.to }))
  const prompt = session.phase === 'pre_fuzz' ? document.entry.hint : document.teaching.steps[session.nextVerifiedInput]?.prompt ?? '继续保持。'
  const effective = autoSlowdown ? assistedRate(baseRate, fuzzStartFrame === null ? 0 : frame - fuzzStartFrame, target, document.assist.auto_slowdown.radius_frames, document.assist.auto_slowdown.minimum_multiplier) : baseRate
  const bestFinalSpeed = session.candidates[0]?.final_state?.speed?.x
  return <main className="training-workspace">
    <section className="training-stage panel-frame">
      <div className="stage-header"><div><small>TRAINING / {document.id}</small><h1>{document.title} <em>第 {Math.min(session.nextVerifiedInput + 1, document.teaching.steps.length)}/{document.teaching.steps.length} 步</em></h1></div><div className="cache-meter"><span>{bestFinalSpeed === undefined ? '有效倍率' : '当前最佳候选最终 X 速度'}</span><strong>{bestFinalSpeed === undefined ? `${effective.toFixed(2)}×` : bestFinalSpeed.toFixed(2)}</strong></div></div>
      <GameView map={map} state={state} states={snapshots} frame={frame} stale={false} />
      <div className="training-prompt">{prompt}</div>
      {session.phase === 'failed' && <div className="training-failure"><strong>{session.failure?.kind === 'entry_check_failed' ? document.entry.failure.title : session.failure?.kind === 'input_order_mismatch' ? document.teaching.steps[session.nextVerifiedInput]?.order_error.title : document.teaching.steps[session.nextVerifiedInput]?.window_error.title}</strong><span>{notice}</span><button onClick={() => resetTo()}>R 重试</button></div>}
      {session.phase === 'success' && <div className="training-success"><strong>成功</strong><span>实际输入已保留在训练时间线。</span><button onClick={() => resetTo(0)}>再试一次</button></div>}
      <div className="transport"><button aria-label="回到 R 点" onClick={() => resetTo()}>R</button><button aria-label="上一帧" onClick={() => seek(frame - 1)}>◀</button><button className="play-button" onClick={() => setPlaying((value) => !value)}>{playing ? 'Ⅱ' : '▶'}</button><button aria-label="下一帧" onClick={() => setPlaying(true)}>▶</button><select aria-label="训练基础速度" value={baseRate} onChange={(event) => setBaseRate(Number(event.target.value))}><option value={.25}>0.25×</option><option value={.5}>0.5×</option><option value={1}>1×</option><option value={2}>2×</option></select><label className="training-assist"><input type="checkbox" checked={autoSlowdown} onChange={(event) => setAutoSlowdown(event.target.checked)} />自动慢放</label></div>
    </section>
    <TrainingTimeline frame={frame} frameCount={Math.max(40, snapshots.length - 1)} fuzzStart={fuzzStartFrame} targetFrame={globalTarget} windows={windows} actualInputs={session.actualInputs.map((input) => ({ ...input, frame: (fuzzStartFrame ?? 0) + input.frame }))} failureFrame={session.failure ? (fuzzStartFrame ?? 0) + session.failure.frame : undefined} resetFrame={resetFrame} bestFinalSpeed={bestFinalSpeed} onSeek={seek} onSetReset={(value) => { setResetFrame(value); setNotice(`临时 R 点已设为 F${value}`) }} />
    <section className="training-log panel-frame" aria-label="关键输入日志"><div className="training-log-head"><small>VERIFY LOG</small><span>判定只发生在 verify:true 输入</span></div>{verificationLog.length === 0 ? <p>尚无关键输入。暂停时按任意游戏键会继续模拟；Pre-Fuzz 的非 Dash 输入不会判错。</p> : <ol>{verificationLog.map((entry, index) => <li key={`${entry.frame}-${index}`} className={entry.outcome === 'accepted' ? 'accepted' : 'rejected'}><b>F{entry.frame}</b><span>实际 {entry.actual.join('+') || '—'}</span><span>期望 {entry.expected.join('+') || '—'}</span><em>{entry.outcome === 'accepted' ? '通过' : entry.outcome}</em></li>)}</ol>}</section>
    <div className="notice" role="status"><i className={session.phase === 'failed' ? 'error' : 'ready'} />{notice} · 已记录 {attemptRevision ? session.actualInputs.length : 0} 次关键输入</div>
  </main>
}
