import { useMemo, useRef } from 'react'
import type { FrameWindow } from '../training/session'

export interface TrainingTimelineProps {
  frame: number
  frameCount: number
  fuzzStart: number | null
  targetFrame?: number
  windows: FrameWindow[]
  actualInputs: readonly { frame: number; keys: readonly string[] }[]
  failureFrame?: number
  resetFrame: number
  bestFinalSpeed?: number
  onSeek(frame: number): void
  onSetReset(frame: number): void
}

/** A review-only timeline for lessons.  It intentionally has no TAS editing affordances. */
export function TrainingTimeline({ frame, frameCount, fuzzStart, targetFrame, windows, actualInputs, failureFrame, resetFrame, bestFinalSpeed, onSeek, onSetReset }: TrainingTimelineProps) {
  const track = useRef<HTMLDivElement>(null)
  const pointerStart = useRef<number | null>(null)
  const maximum = Math.max(1, frameCount)
  const viewportFrames = Math.min(maximum, 48)
  const viewportStart = Math.max(0, Math.min(maximum - viewportFrames, frame - Math.floor(viewportFrames / 2)))
  const viewportEnd = viewportStart + viewportFrames
  const inViewport = (value: number) => value >= viewportStart && value <= viewportEnd
  const percent = (value: number) => `${(value - viewportStart) / Math.max(1, viewportFrames) * 100}%`
  const clampedPercent = (value: number) => `${Math.max(0, Math.min(100, (value - viewportStart) / Math.max(1, viewportFrames) * 100))}%`
  const visibleWindows = windows.flatMap((window) => {
    const from = Math.max(window.from, viewportStart)
    const to = Math.min(window.to, viewportEnd)
    return from <= to ? [{ ...window, from, to }] : []
  })
  const inputLabels = useMemo(() => actualInputs.map((input) => ({ ...input, label: input.keys.join('+').toUpperCase() })), [actualInputs])
  const selectFrame = (clientX: number): number | undefined => {
    const rect = track.current?.getBoundingClientRect()
    if (!rect) return undefined
    const next = Math.round(viewportStart + Math.max(0, Math.min(1, (clientX - rect.left) / rect.width)) * viewportFrames)
    onSeek(next)
    return next
  }
  return <section className="training-timeline panel-frame" aria-label="训练时间线">
    <div className="training-timeline-head"><div><small>TRAINING REVIEW · F{viewportStart}–F{viewportStart + viewportFrames}</small><h2>参考输入 / 成功窗口 / 玩家输入</h2></div><output>F{frame}</output></div>
    <div
      ref={track}
      className="training-track"
      role="slider"
      aria-label="训练回看时间线"
      aria-valuemin={0}
      aria-valuemax={maximum}
      aria-valuenow={frame}
      tabIndex={0}
      onPointerDown={(event) => { pointerStart.current = event.clientX; event.currentTarget.setPointerCapture?.(event.pointerId); selectFrame(event.clientX) }}
      onPointerMove={(event) => { if (!event.currentTarget.hasPointerCapture || event.currentTarget.hasPointerCapture(event.pointerId)) selectFrame(event.clientX) }}
      onPointerUp={(event) => {
        const selected = selectFrame(event.clientX)
        if (pointerStart.current !== null && Math.abs(pointerStart.current - event.clientX) < 4 && selected !== undefined) onSetReset(selected)
        pointerStart.current = null
        if (event.currentTarget.hasPointerCapture?.(event.pointerId)) event.currentTarget.releasePointerCapture?.(event.pointerId)
      }}
      onKeyDown={(event) => {
        if (event.code === 'ArrowLeft' || event.code === 'ArrowRight') { event.preventDefault(); onSeek(frame + (event.code === 'ArrowLeft' ? -1 : 1)) }
      }}
    >
      {visibleWindows.map((window, index) => <i key={`${window.from}-${window.to}-${index}`} className="training-window" style={{ left: percent(window.from), width: `${Math.max(.8, (window.to - window.from + 1) / viewportFrames * 100)}%` }} title={`成功窗口 F${window.from}–F${window.to}`} />)}
      {fuzzStart !== null && inViewport(fuzzStart) && <b className="training-marker fuzz" style={{ left: percent(fuzzStart) }}>F0<span className="training-tooltip">Fuzz 起点：入口 Dash 是本地 F0</span></b>}
      {targetFrame !== undefined && <b className={`training-marker target ${inViewport(targetFrame) ? '' : targetFrame < viewportStart ? 'offscreen before' : 'offscreen after'}`} style={{ left: clampedPercent(targetFrame) }}>{inViewport(targetFrame) ? '◆' : targetFrame < viewportStart ? '‹' : '›'}<span className="training-tooltip">下一最佳关键点：F{targetFrame}{bestFinalSpeed === undefined ? '' : `；该候选最终 X 速度 ${bestFinalSpeed.toFixed(2)}`}</span></b>}
      {inputLabels.filter((input) => inViewport(input.frame)).map((input) => <b key={`${input.frame}-${input.label}`} className="training-marker input" style={{ left: percent(input.frame) }}>●<span className="training-tooltip">你的输入：F{input.frame} {input.label}</span></b>)}
      {failureFrame !== undefined && inViewport(failureFrame) && <b className="training-marker failure" style={{ left: percent(failureFrame) }}>×<span className="training-tooltip">失败发生在 F{failureFrame}</span></b>}
      {inViewport(resetFrame) && <button className="training-marker reset" style={{ left: percent(resetFrame) }} title={`R 点 F${resetFrame}`} onClick={(event) => { event.stopPropagation(); onSetReset(resetFrame) }}>R</button>}
      <b className="training-playhead" style={{ left: percent(frame) }} />
    </div>
    <div className="training-timeline-actions"><span className="training-legend"><i className="fuzz" />Fuzz 起点<i className="target" />下一关键点<i className="window" />成功窗口<i className="input" />你的输入<i className="failure" />失败<i className="reset" />R 点</span><button onClick={() => onSetReset(frame)}>设为 R 点 F{frame}</button></div>
  </section>
}
