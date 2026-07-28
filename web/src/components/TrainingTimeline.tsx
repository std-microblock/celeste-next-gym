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
  onSeek(frame: number): void
  onSetReset(frame: number): void
}

/** A review-only timeline for lessons.  It intentionally has no TAS editing affordances. */
export function TrainingTimeline({ frame, frameCount, fuzzStart, targetFrame, windows, actualInputs, failureFrame, resetFrame, onSeek, onSetReset }: TrainingTimelineProps) {
  const track = useRef<HTMLDivElement>(null)
  const maximum = Math.max(1, frameCount)
  const percent = (value: number) => `${Math.max(0, Math.min(100, value / maximum * 100))}%`
  const inputLabels = useMemo(() => actualInputs.map((input) => ({ ...input, label: input.keys.join('+').toUpperCase() })), [actualInputs])
  const selectFrame = (clientX: number) => {
    const rect = track.current?.getBoundingClientRect()
    if (!rect) return
    onSeek(Math.round(Math.max(0, Math.min(1, (clientX - rect.left) / rect.width)) * maximum))
  }
  return <section className="training-timeline panel-frame" aria-label="训练时间线">
    <div className="training-timeline-head"><div><small>TRAINING REVIEW</small><h2>参考输入 / 成功窗口 / 玩家输入</h2></div><output>F{frame}</output></div>
    <div
      ref={track}
      className="training-track"
      role="slider"
      aria-label="训练回看时间线"
      aria-valuemin={0}
      aria-valuemax={maximum}
      aria-valuenow={frame}
      tabIndex={0}
      onPointerDown={(event) => { event.currentTarget.setPointerCapture?.(event.pointerId); selectFrame(event.clientX) }}
      onPointerMove={(event) => { if (!event.currentTarget.hasPointerCapture || event.currentTarget.hasPointerCapture(event.pointerId)) selectFrame(event.clientX) }}
      onPointerUp={(event) => { if (event.currentTarget.hasPointerCapture?.(event.pointerId)) event.currentTarget.releasePointerCapture?.(event.pointerId) }}
      onKeyDown={(event) => {
        if (event.code === 'ArrowLeft' || event.code === 'ArrowRight') { event.preventDefault(); onSeek(frame + (event.code === 'ArrowLeft' ? -1 : 1)) }
      }}
    >
      {windows.map((window, index) => <i key={`${window.from}-${window.to}`} className="training-window" style={{ left: percent(window.from), width: `${Math.max(.8, (window.to - window.from + 1) / maximum * 100)}%` }} title={`成功窗口 F${window.from}–F${window.to}`} />)}
      {fuzzStart !== null && <b className="training-marker fuzz" style={{ left: percent(fuzzStart) }} title="Fuzz 起点">F0</b>}
      {targetFrame !== undefined && <b className="training-marker target" style={{ left: percent(targetFrame) }} title={`下一关键点 F${targetFrame}`}>◆</b>}
      {inputLabels.map((input) => <b key={`${input.frame}-${input.label}`} className="training-marker input" style={{ left: percent(input.frame) }} title={input.label}>●</b>)}
      {failureFrame !== undefined && <b className="training-marker failure" style={{ left: percent(failureFrame) }} title="失败">×</b>}
      <button className="training-marker reset" style={{ left: percent(resetFrame) }} title={`R 点 F${resetFrame}`} onClick={(event) => { event.stopPropagation(); onSetReset(resetFrame) }}>R</button>
      <b className="training-playhead" style={{ left: percent(frame) }} />
    </div>
    <div className="training-timeline-actions"><span>拖动回看会暂停；点击任意帧设为临时 R 点</span><button onClick={() => onSetReset(frame)}>设为 R 点 F{frame}</button></div>
  </section>
}
