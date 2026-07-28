import { useEffect, useMemo, useRef, useState, type CSSProperties } from 'react'
import { DEFAULT_BINDINGS, buttonsToInput, makeEmptyButtons, type FrameButtons, type GymMap, type KeyBindings, type SimState } from '../model'
import { WasmClient } from '../simulator/wasmClient'
import { assistedRate, candidateWindow, createTrainingSession, keySemantics, nextTargetFrame, rebuildTrainingSession, verifiedInputs, verifyTrainingInput, type TrainingCandidate, type TrainingDefinition, type TrainingSession } from '../training/session'
import { trainingCatalog, type TrainingDocument, type TrainingVariant } from '../training/catalog'
import { GameView } from './GameView'
import { TrainingCatalogSidebar, TrainingVariantThumbnail } from './TrainingCatalogSidebar'
import { TrainingPrompt } from './TrainingPrompt'
import { TrainingResultTimeline, TrainingTimeline } from './TrainingTimeline'

interface Attempt { frame: number; keys: string[]; entryCheckPassed?: boolean }
interface PredictionPreview {
  targetFrame?: number
  windows: Array<{ from: number; to: number }>
  bestFinalSpeed?: number
}
export interface OutcomeAnimation {
  phase: 'failed' | 'success'
  startedAt: number
  durationMs: number
  resultSpeedX: number
  timelineFrame: number
}

const FAILURE_SLOWDOWN_MS = 1_000
const SUCCESS_SLOWDOWN_MS = 3_000
const MAX_AUTO_SLOWDOWN_REDUCTION = .7

function buttonsFromKeyboard(keys: ReadonlySet<string>, bindings: KeyBindings): FrameButtons {
  return {
    up: keys.has(bindings.up), down: keys.has(bindings.down), left: keys.has(bindings.left), right: keys.has(bindings.right),
    jump: keys.has(bindings.jump), dash: keys.has(bindings.dash), grab: keys.has(bindings.grab),
  }
}

function pressedVerification(current: FrameButtons, previous: FrameButtons): boolean {
  return (current.dash && !previous.dash) || (current.jump && !previous.jump) || (current.grab && !previous.grab)
}

export function trainingEntryDirectionPassed(buttons: FrameButtons): boolean {
  return buttons.right && buttons.down
}

export function timingAssessment(actualFrame: number | undefined, targetFrame: number | undefined): string {
  if (actualFrame === undefined || targetFrame === undefined) return '无可比较的最佳点'
  const difference = actualFrame - targetFrame
  if (difference === 0) return '正中最佳点'
  return difference < 0 ? `早了 ${Math.abs(difference)} 帧` : `晚了 ${difference} 帧`
}

export function outcomeSpeedX(after: SimState): number {
  return after.speed.x
}

export function trainingInputLocked(outcome: OutcomeAnimation | null): boolean {
  return outcome !== null
}

export function verificationKeys(buttons: FrameButtons, previous: FrameButtons, document: TrainingDefinition, step: number): string[] {
  const expected = verifiedInputs(document)[step]?.keys ?? []
  const actual = keySemantics(buttons)
  const newlyPressed = actual.filter((key) => !previous[key as keyof FrameButtons])
  // With verify:false direction holds, only the new action belongs to the
  // teaching input. A definition that includes a direction in verify:true
  // deliberately opts into strict directional matching.
  return expected.some((key) => ['up', 'down', 'left', 'right'].includes(key))
    ? actual.filter((key) => ['up', 'down', 'left', 'right'].includes(key) || newlyPressed.includes(key))
    : newlyPressed.filter((key) => !['up', 'down', 'left', 'right'].includes(key))
}

/** The lesson runner owns simulation and review state, while its timeline is read-only. */
export function TrainingGround({ techniqueId, variantId, onSelectTraining }: { techniqueId: string; variantId: string; onSelectTraining(techniqueId: string, variantId: string): void }) {
  const client = useMemo(() => new WasmClient(), [])
  const technique = trainingCatalog.find((item) => item.id === techniqueId) ?? trainingCatalog[0]
  const variantIndex = Math.max(0, technique.variants.findIndex((variant) => variant.id === variantId))
  const selectedVariant = technique.variants[variantIndex] ?? technique.variants[0]
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
  const [prediction, setPrediction] = useState<PredictionPreview>({ windows: [] })
  const predictionDirty = useRef(false)
  const [followReference, setFollowReference] = useState(true)
  const [outcome, setOutcome] = useState<OutcomeAnimation | null>(null)
  const outcomeRef = useRef<OutcomeAnimation | null>(null)
  const [outcomeProgress, setOutcomeProgress] = useState(0)
  const [notice, setNotice] = useState('正在加载训练定义…')
  const keys = useRef(new Set<string>())
  const previousButtons = useRef<FrameButtons>(makeEmptyButtons())
  const outcomeButtons = useRef<FrameButtons>(makeEmptyButtons())
  const attempts = useRef<Attempt[]>([])
  const simulating = useRef(false)

  const installVariant = async (variant: TrainingVariant) => {
    setPlaying(false)
    clearOutcome()
    setNotice(`正在加载 ${technique.title} · ${variant.title}…`)
    const result = await client.fuzzSearch(variant.initial, JSON.stringify(variant.document.fuzz), variant.map)
    if (result.candidates.length === 0) throw new Error('该训练 Variant 没有成功候选，无法开始')
    setDocument(variant.document)
    setMap(variant.map)
    setInitial(variant.initial)
    setCandidates(result.candidates)
    applySession(createTrainingSession(result.candidates))
    snapshotsRef.current = [variant.initial]
    setSnapshots([variant.initial])
    frameRef.current = 0
    setFrame(0)
    setResetFrame(0)
    fuzzStartRef.current = null
    setFuzzStartFrame(null)
    attempts.current = []
    previousButtons.current = makeEmptyButtons()
    outcomeButtons.current = makeEmptyButtons()
    keys.current.clear()
    applyPrediction({ windows: [] })
    setFollowReference(true)
    setAutoSlowdown(variant.document.assist.auto_slowdown.enabled_by_default)
    console.info('[training] loaded', { technique: technique.id, variant: variant.id, candidates: result.candidates.length })
    setNotice(`已加载 ${technique.title} · ${variant.title} · ${result.candidates.length} 个可行候选`)
  }

  const applySession = (next: TrainingSession) => { sessionRef.current = next; setSession(next) }
  const applyPrediction = (next: PredictionPreview) => setPrediction(next)
  const beginOutcome = (phase: OutcomeAnimation['phase'], after: SimState, timelineFrame: number) => {
    const next = {
      phase,
      startedAt: performance.now(),
      durationMs: phase === 'success' ? SUCCESS_SLOWDOWN_MS : FAILURE_SLOWDOWN_MS,
      resultSpeedX: outcomeSpeedX(after),
      timelineFrame,
    }
    outcomeRef.current = next
    outcomeButtons.current = { ...previousButtons.current }
    setOutcome(next)
    setOutcomeProgress(0)
  }
  const clearOutcome = () => {
    outcomeRef.current = null
    outcomeButtons.current = makeEmptyButtons()
    setOutcome(null)
    setOutcomeProgress(0)
  }
  const capturePrediction = (source: TrainingSession, start: number | null) => {
    const inputIndex = verifiedInputs(document!)[source.nextVerifiedInput]?.fuzzInputIndex
    if (inputIndex === undefined || start === null) return
    const next: PredictionPreview = {
      targetFrame: (() => {
        const target = nextTargetFrame(source.candidates, inputIndex)
        return target === undefined ? undefined : start + target
      })(),
      windows: candidateWindow(source.candidates, inputIndex).map((window) => ({
        from: start + window.from,
        to: start + window.to,
      })),
      bestFinalSpeed: source.candidates[0]?.final_state?.speed?.x,
    }
    applyPrediction(next)
  }

  useEffect(() => () => client.dispose(), [client])
  useEffect(() => {
    let active = true
    void (async () => {
      try {
        await client.ready()
        if (!active) return
        await installVariant(selectedVariant)
      } catch (error) {
        if (active) setNotice(error instanceof Error ? error.message : '训练加载失败')
      }
    })()
    return () => { active = false }
  // installVariant reads the selected catalog entry for this render.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [client, selectedVariant])

  const seek = (requested: number) => {
    const next = Math.max(0, Math.min(snapshotsRef.current.length - 1, Math.round(requested)))
    setPlaying(false)
    clearOutcome()
    predictionDirty.current = true
    frameRef.current = next
    setFrame(next)
    if (!document) return
    const start = attempts.current.find((attempt) => attempt.entryCheckPassed === true && attempt.keys.includes('dash'))?.frame ?? null
    if (start === null || next < start) {
      fuzzStartRef.current = null
      setFuzzStartFrame(null)
      applySession(createTrainingSession(candidates))
      return
    }
    fuzzStartRef.current = start
    const replay = start === null ? [] : attempts.current
      .filter((attempt) => attempt.frame >= start && attempt.frame <= next)
      .map((attempt) => ({ ...attempt, frame: attempt.frame - start }))
    applySession(rebuildTrainingSession(document, candidates, replay))
    setFuzzStartFrame(start)
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
    outcomeButtons.current = makeEmptyButtons()
    keys.current.clear()
    seek(target)
    setNotice(`已回到 R 点 F${target}`)
  }

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLElement && event.target.matches('input, select, button')) return
      const gameInput = Object.values(DEFAULT_BINDINGS).includes(event.code)
      if (gameInput || event.code === 'KeyR') event.preventDefault()
      if (event.code === 'KeyR' && !event.repeat) {
        resetTo()
        return
      }
      if (trainingInputLocked(outcomeRef.current)) return
      keys.current.add(event.code)
      if (gameInput) {
        setFollowReference(true)
        if (predictionDirty.current) {
          predictionDirty.current = false
          applyPrediction({ windows: [] })
        }
        setPlaying(true)
      }
    }
    const up = (event: KeyboardEvent) => {
      if (trainingInputLocked(outcomeRef.current)) return
      keys.current.delete(event.code)
    }
    window.addEventListener('keydown', down)
    window.addEventListener('keyup', up)
    return () => { window.removeEventListener('keydown', down); window.removeEventListener('keyup', up) }
    // resetTo intentionally reads current state through React's render closure.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [resetFrame, initial, document, candidates])

  useEffect(() => {
    if (!playing || !document || !map || !initial) return
    let active = true
    let animation = 0
    let last = performance.now()
    let carry = 0
    const tick = (now: number) => {
      const fuzzInputIndex = verifiedInputs(document)[sessionRef.current.nextVerifiedInput]?.fuzzInputIndex
      const target = fuzzInputIndex === undefined ? undefined : nextTargetFrame(sessionRef.current.candidates, fuzzInputIndex)
      const fuzzFrame = fuzzStartRef.current === null ? 0 : Math.max(0, frameRef.current - fuzzStartRef.current)
      const assisted = autoSlowdown
        ? assistedRate(baseRate, fuzzFrame, target, document.assist.auto_slowdown.radius_frames, Math.max(MAX_AUTO_SLOWDOWN_REDUCTION, document.assist.auto_slowdown.minimum_multiplier))
        : baseRate
      const activeOutcome = outcomeRef.current
      const progress = activeOutcome === null ? 0 : Math.max(0, Math.min(1, (now - activeOutcome.startedAt) / activeOutcome.durationMs))
      const rate = assisted * (1 - progress)
      if (activeOutcome !== null) {
        setOutcomeProgress(progress)
        if (progress >= 1) {
          setPlaying(false)
          return
        }
      } else {
        capturePrediction(sessionRef.current, fuzzStartRef.current)
      }
      carry += Math.min(250, now - last) * 60 / 1000 * rate
      last = now
      if (carry >= 1 && !simulating.current) {
        carry -= 1
        const currentFrame = frameRef.current
        const current = activeOutcome === null
          ? buttonsFromKeyboard(keys.current, DEFAULT_BINDINGS)
          : outcomeButtons.current
        const input = buttonsToInput(current, previousButtons.current)
        const beforeSession = sessionRef.current
        const triggers = pressedVerification(current, previousButtons.current)
        const shouldVerify = beforeSession.phase === 'pre_fuzz' ? current.dash && !previousButtons.current.dash : triggers
        const previous = previousButtons.current
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
            const semanticKeys = verificationKeys(current, previous, document, beforeSession.nextVerifiedInput)
            const entryPassed = beforeSession.phase === 'pre_fuzz'
              ? trainingEntryDirectionPassed(current) && await client.entryCheck(after, document.entry.check)
              : true
            const nextSession = verifyTrainingInput(beforeSession, document, localFrame, semanticKeys, entryPassed)
            attempts.current = [...attempts.current, { frame: currentFrame, keys: semanticKeys, entryCheckPassed: entryPassed }]
            const expected = verifiedInputs(document)[beforeSession.nextVerifiedInput]?.keys ?? []
            const verificationOutcome = nextSession.phase === 'failed'
              ? nextSession.failure?.kind ?? 'input_order_mismatch'
              : 'accepted'
            console.info('[training] verify', {
              frame: currentFrame,
              actual: semanticKeys,
              expected,
              outcome: verificationOutcome,
              after
            })
            if (beforeSession.phase === 'pre_fuzz' && nextSession.phase === 'fuzz') {
              fuzzStartRef.current = currentFrame
              setFuzzStartFrame(currentFrame)
            }
            capturePrediction(nextSession.phase === 'failed' ? beforeSession : nextSession, fuzzStartRef.current ?? currentFrame)
            applySession(nextSession)
            if (nextSession.phase === 'failed') {
              setNotice(nextSession.failure?.kind === 'input_order_mismatch' ? document.teaching.steps[nextSession.nextVerifiedInput]?.order_error.body ?? '输入顺序不正确。' : nextSession.failure?.kind === 'timing_window_miss' ? document.teaching.steps[nextSession.nextVerifiedInput]?.window_error.body ?? '错过输入窗口。' : document.entry.failure.body)
              beginOutcome('failed', after, nextFrame)
            } else if (nextSession.phase === 'success') {
              setNotice('成功：已命中可行候选。')
              beginOutcome('success', after, nextFrame)
            }
          }
        }).catch((error: Error) => { if (active) { setPlaying(false); setNotice(error.message) } }).finally(() => { simulating.current = false })
      }
      if (active) animation = requestAnimationFrame(tick)
    }
    animation = requestAnimationFrame(tick)
    return () => { active = false; cancelAnimationFrame(animation) }
  }, [autoSlowdown, baseRate, client, document, initial, map, playing, session.phase])

  if (!document || !map || !initial || snapshots.length === 0) return <main className="training-workspace">
    <TrainingCatalogSidebar techniqueId={technique.id} variantId={selectedVariant.id} onSelectTraining={onSelectTraining} />
    <div className="training-loading notice"><i />{notice}</div>
  </main>
  const state = snapshots[frame] ?? snapshots.at(-1) ?? initial
  const fuzzInputIndex = verifiedInputs(document)[session.nextVerifiedInput]?.fuzzInputIndex
  const target = fuzzInputIndex === undefined ? undefined : nextTargetFrame(session.candidates, fuzzInputIndex)
  const prompt = session.phase === 'pre_fuzz' ? document.entry.hint : document.teaching.steps[session.nextVerifiedInput]?.prompt ?? '继续保持。'
  const effective = (autoSlowdown ? assistedRate(baseRate, fuzzStartFrame === null ? 0 : frame - fuzzStartFrame, target, document.assist.auto_slowdown.radius_frames, Math.max(MAX_AUTO_SLOWDOWN_REDUCTION, document.assist.auto_slowdown.minimum_multiplier)) : baseRate) * (1 - outcomeProgress)
  const bestFinalSpeed = prediction.bestFinalSpeed
  const actualInputs = session.actualInputs.map((input) => ({ ...input, frame: (fuzzStartFrame ?? 0) + input.frame }))
  const actualActionFrame = actualInputs.at(-1)?.frame
  const timing = timingAssessment(actualActionFrame, prediction.targetFrame)
  const finalSpeed = outcome?.resultSpeedX
  const recommendations = technique.variants.filter((variant) => variant.id !== selectedVariant.id).slice(0, 2)
  const failureFrame = session.failure ? (fuzzStartFrame ?? 0) + session.failure.frame : undefined
  const timelineFrame = outcome?.timelineFrame ?? frame
  const timelineFrameCount = outcome?.timelineFrame ?? Math.max(40, snapshots.length - 1, (prediction.targetFrame ?? 0) + 24)
  const resultPanel = outcome && <div className={`training-result-card ${outcome.phase}`}>
    <div className="training-result-heading">
      <div><small>ATTEMPT RESULT</small><strong>{outcome.phase === 'success' ? '成功' : session.failure?.kind === 'entry_check_failed' ? document.entry.failure.title : session.failure?.kind === 'input_order_mismatch' ? document.teaching.steps[session.nextVerifiedInput]?.order_error.title : document.teaching.steps[session.nextVerifiedInput]?.window_error.title}</strong></div>
      <em>{timing}</em>
    </div>
    <TrainingResultTimeline targetFrame={prediction.targetFrame} windows={prediction.windows} actualInputs={actualInputs} failureFrame={failureFrame} />
    <div className="training-result-stats">
      <div><span>本次最终 X 速度</span><b>{finalSpeed === undefined ? '—' : finalSpeed.toFixed(2)}</b></div>
      <div><span>最佳点结算速度</span><b>{bestFinalSpeed === undefined ? '—' : bestFinalSpeed.toFixed(2)}</b></div>
      <div><span>输入时机</span><b>{timing}</b></div>
    </div>
    <p>{outcome.phase === 'success' ? '' : notice}</p>
    <div className="training-result-actions">
      <button onClick={() => outcome.phase === 'success' ? resetTo(0) : resetTo()}>{outcome.phase === 'success' ? '再试一次' : 'R 重试'}</button>
      {outcome.phase === 'success' && variantIndex + 1 < technique.variants.length && <button className="primary" onClick={() => onSelectTraining(technique.id, technique.variants[variantIndex + 1].id)}>下一个 {variantIndex + 2}</button>}
    </div>
  </div>
  return <main className="training-workspace">
    <TrainingCatalogSidebar techniqueId={technique.id} variantId={selectedVariant.id} onSelectTraining={onSelectTraining} />
    <section className="training-stage panel-frame">
      <div className="stage-header"><div><small>TRAINING / {document.technique_id} / {document.variant_id}</small><h1>{document.title} · {document.variant_title} <em>第 {Math.min(session.nextVerifiedInput + 1, document.teaching.steps.length)}/{document.teaching.steps.length} 步</em></h1></div><div className="cache-meter"><span>{bestFinalSpeed === undefined ? '有效倍率' : '当前最佳候选最终 X 速度'}</span><strong>{bestFinalSpeed === undefined ? `${effective.toFixed(2)}×` : bestFinalSpeed.toFixed(2)}</strong></div></div>
      <GameView map={map} state={state} states={snapshots} frame={frame} stale={false}>
        {(viewport) => <TrainingPrompt map={map} state={state} viewport={viewport} text={prompt} hidden={outcome !== null} />}
      </GameView>
      {outcome && <div className={outcome.phase === 'success' ? 'training-success' : 'training-failure'} style={{ '--outcome-progress': outcomeProgress } as CSSProperties}>
        <div className={`training-outcome-layout ${outcome.phase}`}>
          {outcome.phase === 'success' && recommendations.length > 0 && <aside className="training-recommendations" aria-label="你还可以看看">
            <div className="training-recommendations-heading"><small>KEEP CLIMBING</small><h2>你还可以看看</h2><p>继续挑战其他训练场景。</p></div>
            <div className="training-recommendation-list">
              {recommendations.map((variant) => <button type="button" key={variant.id} onClick={() => onSelectTraining(technique.id, variant.id)}>
                <TrainingVariantThumbnail variant={variant} />
                <span><small>{technique.title} · VARIANT</small><strong>{variant.title}</strong><em>{variant.summary}</em><b>开始训练 <i aria-hidden="true">→</i></b></span>
              </button>)}
            </div>
          </aside>}
          {resultPanel}
        </div>
      </div>}
      <div className="transport"><button aria-label="回到 R 点" onClick={() => resetTo()}>R</button><button aria-label="上一帧" onClick={() => seek(frame - 1)}>◀</button><button className="play-button" onClick={() => { if (!playing && predictionDirty.current) { predictionDirty.current = false; applyPrediction({ windows: [] }) } setPlaying((value) => !value) }}>{playing ? 'Ⅱ' : '▶'}</button><button aria-label="下一帧" onClick={() => { if (predictionDirty.current) { predictionDirty.current = false; applyPrediction({ windows: [] }) } setPlaying(true) }}>▶</button><select aria-label="训练基础速度" value={baseRate} onChange={(event) => setBaseRate(Number(event.target.value))}><option value={.25}>0.25×</option><option value={.5}>0.5×</option><option value={1}>1×</option><option value={2}>2×</option></select><label className="training-assist"><input type="checkbox" checked={autoSlowdown} onChange={(event) => setAutoSlowdown(event.target.checked)} />自动慢放</label></div>
    </section>
    <TrainingTimeline frame={timelineFrame} frameCount={timelineFrameCount} fuzzStart={fuzzStartFrame} targetFrame={prediction.targetFrame} windows={prediction.windows} actualInputs={actualInputs} failureFrame={failureFrame} resetFrame={resetFrame} bestFinalSpeed={bestFinalSpeed} followTarget={followReference && !outcome} onSeek={(value, manual) => { if (manual && value < timelineFrame) setFollowReference(false); seek(value) }} onSetReset={(value) => { setResetFrame(value); setNotice(`临时 R 点已设为 F${value}`) }} />
  </main>
}
