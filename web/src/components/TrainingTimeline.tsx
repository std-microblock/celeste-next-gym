import { useMemo, useRef, useState } from 'react'
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
  followTarget?: boolean
  onSeek(frame: number, manual?: boolean): void
  onSetReset(frame: number): void
}

/** A review-only timeline for lessons.  It intentionally has no TAS editing affordances. */
export function TrainingTimeline({ frame, frameCount, fuzzStart, targetFrame, windows, actualInputs, failureFrame, resetFrame, bestFinalSpeed, followTarget = false, onSeek, onSetReset }: TrainingTimelineProps) {
  const track = useRef<HTMLDivElement>(null)
  const pointerStart = useRef<number | null>(null)
  const dragViewportStartRef = useRef<number | null>(null)
  const [dragViewportStart, setDragViewportStart] = useState<number | null>(null)
  const maximum = Math.max(1, frameCount)
  const viewportFrames = Math.min(maximum, 48)
  const viewportFocus = followTarget && targetFrame !== undefined ? targetFrame : frame
  const automaticViewportStart = Math.max(0, Math.min(maximum - viewportFrames, viewportFocus - Math.floor(viewportFrames / 2)))
  const viewportStart = dragViewportStart ?? automaticViewportStart
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
  const selectFrame = (clientX: number, allowEdgeScroll = false): number | undefined => {
    const rect = track.current?.getBoundingClientRect()
    if (!rect) return undefined
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
    let start = dragViewportStartRef.current ?? viewportStart
    if (allowEdgeScroll && ratio <= .04 && start > 0) start -= 1
    else if (allowEdgeScroll && ratio >= .96 && start < maximum - viewportFrames) start += 1
    if (start !== dragViewportStartRef.current) {
      dragViewportStartRef.current = start
      setDragViewportStart(start)
    }
    const next = Math.round(start + ratio * viewportFrames)
    onSeek(next, true)
    return next
  }
  return <section className="training-timeline panel-frame" aria-label="训练时间线">
    <div className="training-timeline-head"><div><small>TIMELINE · F{viewportStart}–F{viewportStart + viewportFrames}</small><h2>时间线</h2></div><output>F{frame}</output></div>
    <div
      ref={track}
      className="training-track"
      role="slider"
      aria-label="训练回看时间线"
      aria-valuemin={0}
      aria-valuemax={maximum}
      aria-valuenow={frame}
      tabIndex={0}
      onPointerDown={(event) => {
        pointerStart.current = event.clientX
        dragViewportStartRef.current = viewportStart
        setDragViewportStart(viewportStart)
        event.currentTarget.setPointerCapture?.(event.pointerId)
        selectFrame(event.clientX)
      }}
      onPointerMove={(event) => { if (!event.currentTarget.hasPointerCapture || event.currentTarget.hasPointerCapture(event.pointerId)) selectFrame(event.clientX, true) }}
      onPointerUp={(event) => {
        const selected = selectFrame(event.clientX)
        if (pointerStart.current !== null && Math.abs(pointerStart.current - event.clientX) < 4 && selected !== undefined) onSetReset(selected)
        pointerStart.current = null
        dragViewportStartRef.current = null
        setDragViewportStart(null)
        if (event.currentTarget.hasPointerCapture?.(event.pointerId)) event.currentTarget.releasePointerCapture?.(event.pointerId)
      }}
      onKeyDown={(event) => {
        if (event.code === 'ArrowLeft' || event.code === 'ArrowRight') { event.preventDefault(); onSeek(frame + (event.code === 'ArrowLeft' ? -1 : 1), true) }
      }}
    >
      {visibleWindows.map((window, index) => <i key={`${window.from}-${window.to}-${index}`} className="training-window" style={{ left: percent(window.from), width: `${Math.max(.8, (window.to - window.from + 1) / viewportFrames * 100)}%` }} title={`成功窗口 F${window.from}–F${window.to}`} />)}
      {fuzzStart !== null && inViewport(fuzzStart) && <b className="training-marker fuzz" style={{ left: percent(fuzzStart) }}>F0<span className="training-tooltip">操作起点：训练定义的入口输入是本地 F0</span></b>}
      {targetFrame !== undefined && <b className={`training-marker target ${inViewport(targetFrame) ? '' : targetFrame < viewportStart ? 'offscreen before' : 'offscreen after'}`} style={{ left: clampedPercent(targetFrame) }}>{inViewport(targetFrame) ? '◆' : targetFrame < viewportStart ? '‹' : '›'}<span className="training-tooltip">下一最佳关键点：F{targetFrame}{bestFinalSpeed === undefined ? '' : `；该候选最终 X 速度 ${bestFinalSpeed.toFixed(2)}`}</span></b>}
      {inputLabels.filter((input) => inViewport(input.frame)).map((input) => <b key={`${input.frame}-${input.label}`} className="training-marker input" style={{ left: percent(input.frame) }}>●<span className="training-tooltip">你的输入：F{input.frame} {input.label}</span></b>)}
      {failureFrame !== undefined && inViewport(failureFrame) && <b className="training-marker failure" style={{ left: percent(failureFrame) }}>×<span className="training-tooltip">失败发生在 F{failureFrame}</span></b>}
      {inViewport(resetFrame) && <button className="training-marker reset" style={{ left: percent(resetFrame) }} title={`R 点 F${resetFrame}`} onClick={(event) => { event.stopPropagation(); onSetReset(resetFrame) }}>R</button>}
      <b className="training-playhead" style={{ left: percent(frame) }} />
    </div>
    <div className="training-timeline-actions"><span className="training-legend"><i className="fuzz" />操作起点<i className="target" />下一关键点<i className="window" />成功窗口<i className="input" />你的输入<i className="failure" />失败<i className="reset" />R 点</span><button onClick={() => onSetReset(frame)}>设为 R 点 F{frame}</button></div>
  </section>
}

export interface TrainingResultTimelineProps {
  targetFrame?: number
  windows: FrameWindow[]
  actualInputs: readonly { frame: number; keys: readonly string[] }[]
  failureFrame?: number
}

export function TrainingResultTimeline({ targetFrame, windows, actualInputs, failureFrame }: TrainingResultTimelineProps) {
  const inputFrames = actualInputs.map((input) => input.frame)
  const points = [...inputFrames, ...windows.flatMap((window) => [window.from, window.to]), ...(targetFrame === undefined ? [] : [targetFrame]), ...(failureFrame === undefined ? [] : [failureFrame])]
  const minimum = Math.min(...points, 0)
  const maximum = Math.max(...points, minimum + 16)
  const padding = 3
  const from = minimum - padding
  const span = Math.max(16, maximum - minimum + padding * 2)
  const percent = (value: number) => `${(value - from) / span * 100}%`
  return <div className="training-result-timeline" aria-label="本次操作时间线">
    {windows.map((window, index) => <i key={`${window.from}-${window.to}-${index}`} className="training-window" style={{ left: percent(window.from), width: `${Math.max(1.5, (window.to - window.from + 1) / span * 100)}%` }} />)}
    {targetFrame !== undefined && <b className="training-result-target" style={{ left: percent(targetFrame) }}>◆<span>最佳 F{targetFrame}</span></b>}
    {actualInputs.map((input, index) => <b key={`${input.frame}-${index}`} className="training-result-input" style={{ left: percent(input.frame) }}>●<span>F{input.frame} {input.keys.join('+').toUpperCase()}</span></b>)}
    {failureFrame !== undefined && <b className="training-result-failure" style={{ left: percent(failureFrame) }}>×</b>}
  </div>
}
