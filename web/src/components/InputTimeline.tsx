import { useCallback, useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from 'react'
import { ACTIONS, ACTION_GLYPHS, ACTION_LABELS, type Action, type FrameButtons, type SimState } from '../model'

interface TimelineProps {
  frame: number
  inputs: readonly FrameButtons[]
  states: readonly (SimState | undefined)[]
  onSeek(frame: number): void
  onPaint(action: Action, from: number, to: number, value: boolean): void
  onMove(action: Action, targetAction: Action, start: number, end: number, delta: number): { start: number; end: number }
  onEditComplete(): void
  onResize(frames: number): void
}

interface TimelineSelection {
  action: Action
  start: number
  end: number
}

type DragState =
  | { mode: 'paint'; action: Action; anchor: number; value: boolean }
  | { mode: 'move'; action: Action; anchor: number; start: number; end: number }
  | { mode: 'playhead'; source: 'overview' | 'ruler' }
  | { mode: 'pan'; startX: number; startScroll: number }
  | { mode: 'overview'; startX: number; startScroll: number; contentWidth: number; trackWidth: number }
  | { mode: 'scrollbar'; startX: number; startScroll: number; maxScroll: number; thumbTravel: number }

const LABEL_WIDTH = 108
const OVERVIEW_HEIGHT = 34
const RULER_HEIGHT = 32
const HEADER_HEIGHT = OVERVIEW_HEIGHT + RULER_HEIGHT
const ROW_HEIGHT = 34
const LANE_COUNT = 8
const SCROLLBAR_HEIGHT = 16
const ROWS_HEIGHT = ROW_HEIGHT * LANE_COUNT
const CANVAS_HEIGHT = HEADER_HEIGHT + ROWS_HEIGHT + SCROLLBAR_HEIGHT
const PLAYHEAD_HIT_RADIUS = 14
const LANE_COLORS: Record<Action, string> = {
  up: '#65d9ff', down: '#65d9ff', left: '#65d9ff', right: '#65d9ff',
  jump: '#72f0b0', dash: '#ff4f81', crouch_dash: '#c96cff', grab: '#ffd37a',
}

export function heldRuns(inputs: readonly FrameButtons[], action: Action, from: number, to: number): Array<[number, number]> {
  const runs: Array<[number, number]> = []
  let cursor = Math.max(0, from)
  const limit = Math.min(inputs.length, to)
  while (cursor < limit) {
    if (!inputs[cursor][action]) {
      cursor += 1
      continue
    }
    const start = cursor
    while (cursor < limit && inputs[cursor][action]) cursor += 1
    runs.push([start, cursor])
  }
  return runs
}

export function cachedRuns(states: readonly (SimState | undefined)[], from: number, to: number): Array<[number, number]> {
  const runs: Array<[number, number]> = []
  let cursor = Math.max(0, from)
  const limit = Math.min(states.length, to)
  while (cursor < limit) {
    if (!states[cursor]) {
      cursor += 1
      continue
    }
    const start = cursor
    while (cursor < limit && states[cursor]) cursor += 1
    runs.push([start, cursor])
  }
  return runs
}

function heldRunAt(inputs: readonly FrameButtons[], action: Action, frame: number): [number, number] | undefined {
  if (!inputs[frame]?.[action]) return undefined
  let start = frame
  let end = frame + 1
  while (start > 0 && inputs[start - 1][action]) start -= 1
  while (end < inputs.length && inputs[end][action]) end += 1
  return [start, end]
}

function stateColor(state: SimState): string {
  if (state.state === 'Dash' || state.state === 'RedDash' || state.state === 'DreamDash') return '#ff4f81'
  if (state.state === 'Climb') return '#ffd37a'
  if (state.state === 'Swim') return '#65d9ff'
  return '#72f0b0'
}

export function InputTimeline({ frame, inputs, states, onSeek, onPaint, onMove, onEditComplete, onResize }: TimelineProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const scrollRef = useRef(0)
  const dragRef = useRef<DragState | null>(null)
  const clipboardRef = useRef<TimelineSelection | null>(null)
  const [selection, setSelection] = useState<TimelineSelection | null>(null)
  const [cellSize, setCellSize] = useState(26)
  const [viewportRevision, setViewportRevision] = useState(0)
  const frameCount = inputs.length

  const maxScrollFor = useCallback((width: number, size = cellSize) => Math.max(0, (frameCount + 1) * size - Math.max(1, width - LABEL_WIDTH)), [cellSize, frameCount])

  const clampScroll = useCallback((value: number, width: number, size = cellSize) => {
    scrollRef.current = Math.max(0, Math.min(maxScrollFor(width, size), value))
    setViewportRevision((revision) => revision + 1)
  }, [cellSize, maxScrollFor])

  const zoomTo = useCallback((requested: number, anchorCanvasX?: number) => {
    const canvas = canvasRef.current
    if (!canvas) return
    const next = Math.max(5, Math.min(72, Math.round(requested)))
    if (next === cellSize) return
    const visibleWidth = Math.max(1, canvas.clientWidth - LABEL_WIDTH)
    const anchor = Math.max(0, Math.min(visibleWidth, (anchorCanvasX ?? LABEL_WIDTH + visibleWidth / 2) - LABEL_WIDTH))
    const anchorFrame = (scrollRef.current + anchor) / cellSize
    scrollRef.current = Math.max(0, Math.min(maxScrollFor(canvas.clientWidth, next), anchorFrame * next - anchor))
    setCellSize(next)
    setViewportRevision((revision) => revision + 1)
  }, [cellSize, maxScrollFor])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const wheel = (event: WheelEvent) => {
      event.preventDefault()
      const rect = canvas.getBoundingClientRect()
      if (event.ctrlKey) zoomTo(cellSize - Math.sign(event.deltaY) * 3, event.clientX - rect.left)
      else clampScroll(scrollRef.current + (Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY), canvas.clientWidth)
    }
    canvas.addEventListener('wheel', wheel, { passive: false })
    return () => canvas.removeEventListener('wheel', wheel)
  }, [cellSize, clampScroll, zoomTo])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const observer = new ResizeObserver(() => setViewportRevision((revision) => revision + 1))
    observer.observe(canvas)
    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const visibleWidth = Math.max(1, canvas.clientWidth - LABEL_WIDTH)
    const playhead = frame * cellSize
    if (playhead < scrollRef.current || playhead + cellSize > scrollRef.current + visibleWidth) {
      clampScroll(playhead - visibleWidth * .35, canvas.clientWidth)
    }
  }, [cellSize, clampScroll, frame])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const context = canvas.getContext('2d')
    if (!context) return
    const width = canvas.clientWidth
    const dpr = window.devicePixelRatio || 1
    canvas.width = Math.max(1, Math.round(width * dpr))
    canvas.height = Math.round(CANVAS_HEIGHT * dpr)
    context.setTransform(dpr, 0, 0, dpr, 0, 0)
    context.imageSmoothingEnabled = false
    context.clearRect(0, 0, width, CANVAS_HEIGHT)
    context.fillStyle = '#08050f'
    context.fillRect(0, 0, width, CANVAS_HEIGHT)

    const visibleWidth = Math.max(1, width - LABEL_WIDTH)
    const contentWidth = Math.max(1, (frameCount + 1) * cellSize)
    const maxScroll = maxScrollFor(width)
    const scroll = Math.min(scrollRef.current, maxScroll)
    scrollRef.current = scroll
    const firstFrame = Math.max(0, Math.floor(scroll / cellSize))
    const lastFrame = Math.min(frameCount, Math.ceil((scroll + visibleWidth) / cellSize) + 1)
    const overviewX = LABEL_WIDTH + 6
    const overviewWidth = Math.max(1, width - overviewX - 6)
    const overviewY = 8
    const overviewTrackHeight = 17
    const viewportX = overviewX + scroll / contentWidth * overviewWidth
    const viewportWidth = Math.max(18, Math.min(overviewWidth, visibleWidth / contentWidth * overviewWidth))
    const overviewPlayheadX = overviewX + frame / Math.max(1, frameCount) * overviewWidth

    context.save()
    context.beginPath()
    context.rect(LABEL_WIDTH, 0, visibleWidth, CANVAS_HEIGHT - SCROLLBAR_HEIGHT)
    context.clip()

    context.fillStyle = '#171020'
    context.fillRect(overviewX, overviewY, overviewWidth, overviewTrackHeight)
    for (const [start, end] of cachedRuns(states, 0, states.length)) {
      const x = overviewX + start / Math.max(1, frameCount) * overviewWidth
      const runWidth = Math.max(1, (end - start) / Math.max(1, frameCount) * overviewWidth)
      context.fillStyle = '#72f0b055'
      context.fillRect(x, overviewY + 3, runWidth, overviewTrackHeight - 6)
    }
    context.fillStyle = '#79598955'
    context.fillRect(viewportX, overviewY, viewportWidth, overviewTrackHeight)
    context.strokeStyle = '#b996c9'
    context.strokeRect(Math.round(viewportX) + .5, overviewY + .5, Math.max(1, Math.round(viewportWidth) - 1), overviewTrackHeight - 1)
    context.fillStyle = '#ffffff'
    context.fillRect(Math.round(overviewPlayheadX), overviewY - 4, 3, overviewTrackHeight + 8)
    context.beginPath()
    context.moveTo(overviewPlayheadX, OVERVIEW_HEIGHT - 1)
    context.lineTo(overviewPlayheadX - 7, OVERVIEW_HEIGHT - 10)
    context.lineTo(overviewPlayheadX + 7, OVERVIEW_HEIGHT - 10)
    context.closePath()
    context.fill()

    context.fillStyle = '#10091b'
    context.fillRect(LABEL_WIDTH, OVERVIEW_HEIGHT, visibleWidth, RULER_HEIGHT)
    for (let value = firstFrame; value <= lastFrame; value += 1) {
      const x = LABEL_WIDTH + value * cellSize - scroll
      if (value % 10 === 0) {
        context.fillStyle = '#ffffff08'
        context.fillRect(x, OVERVIEW_HEIGHT, 1, RULER_HEIGHT + ROWS_HEIGHT)
      }
      if (cellSize >= 10) {
        context.strokeStyle = '#302140'
        context.lineWidth = 1
        context.beginPath()
        context.moveTo(Math.round(x) + .5, OVERVIEW_HEIGHT)
        context.lineTo(Math.round(x) + .5, HEADER_HEIGHT + ROWS_HEIGHT)
        context.stroke()
      }
      if (value % (cellSize < 8 ? 50 : cellSize < 16 ? 20 : 10) === 0) {
        context.fillStyle = '#83708f'
        context.font = '10px Consolas, monospace'
        context.textBaseline = 'middle'
        context.fillText(String(value), x + 3, OVERVIEW_HEIGHT + RULER_HEIGHT / 2)
      }
    }

    for (let lane = 0; lane < LANE_COUNT; lane += 1) {
      const y = HEADER_HEIGHT + lane * ROW_HEIGHT
      context.strokeStyle = '#38264a'
      context.beginPath()
      context.moveTo(LABEL_WIDTH, y + ROW_HEIGHT - .5)
      context.lineTo(width, y + ROW_HEIGHT - .5)
      context.stroke()
    }

    for (let lane = 0; lane < ACTIONS.length; lane += 1) {
      const action = ACTIONS[lane]
      const color = LANE_COLORS[action]
      const y = HEADER_HEIGHT + lane * ROW_HEIGHT
      const scanFrom = firstFrame > 0 && inputs[firstFrame - 1]?.[action] ? firstFrame - 1 : firstFrame
      for (const [start, end] of heldRuns(inputs, action, scanFrom, lastFrame + 1)) {
        const x = LABEL_WIDTH + start * cellSize - scroll
        const runWidth = (end - start) * cellSize
        context.globalAlpha = .2
        context.fillStyle = color
        context.fillRect(x, y + 2, runWidth, ROW_HEIGHT - 4)
        context.globalAlpha = 1
        context.fillStyle = color
        context.fillRect(x + 3, y + 7, Math.max(3, runWidth - 6), ROW_HEIGHT - 14)
        context.fillStyle = '#ffffffcc'
        context.fillRect(x, y + 5, 2, ROW_HEIGHT - 10)
        context.fillStyle = color
        context.fillRect(x + runWidth - 2, y + 5, 2, ROW_HEIGHT - 10)
      }
    }

    if (selection) {
      const lane = ACTIONS.indexOf(selection.action)
      const x = LABEL_WIDTH + selection.start * cellSize - scroll
      const y = HEADER_HEIGHT + lane * ROW_HEIGHT
      const runWidth = (selection.end - selection.start) * cellSize
      context.save()
      context.strokeStyle = '#ffffff'
      context.lineWidth = 2
      context.setLineDash([6, 3])
      context.strokeRect(Math.round(x) + 2, y + 3, Math.max(2, Math.round(runWidth) - 4), ROW_HEIGHT - 6)
      context.setLineDash([])
      context.fillStyle = '#ffffff'
      context.fillRect(Math.round(x) - 1, y + 2, 4, 4)
      context.fillRect(Math.round(x + runWidth) - 3, y + ROW_HEIGHT - 6, 4, 4)
      context.restore()
    }

    const stateY = HEADER_HEIGHT + ACTIONS.length * ROW_HEIGHT
    context.strokeStyle = '#5c4868'
    context.setLineDash([7, 5])
    context.beginPath()
    context.moveTo(LABEL_WIDTH, stateY + ROW_HEIGHT / 2)
    context.lineTo(width, stateY + ROW_HEIGHT / 2)
    context.stroke()
    context.setLineDash([])
    for (const [start, end] of cachedRuns(states, Math.max(0, firstFrame - 1), lastFrame + 1)) {
      const x = LABEL_WIDTH + start * cellSize - scroll
      const runWidth = (end - start) * cellSize
      context.fillStyle = '#72f0b033'
      context.fillRect(x, stateY + 6, runWidth, ROW_HEIGHT - 12)
    }
    let stateCursor = Math.max(0, firstFrame)
    while (stateCursor <= lastFrame && stateCursor < states.length) {
      const state = states[stateCursor]
      if (!state) {
        stateCursor += 1
        continue
      }
      const color = stateColor(state)
      const start = stateCursor
      while (stateCursor <= lastFrame && states[stateCursor] && stateColor(states[stateCursor]!) === color) stateCursor += 1
      const x = LABEL_WIDTH + start * cellSize - scroll
      context.fillStyle = color
      context.fillRect(x, stateY + 10, (stateCursor - start) * cellSize, 5)
    }

    const playheadX = LABEL_WIDTH + frame * cellSize - scroll
    context.fillStyle = '#ffffff10'
    context.fillRect(playheadX, OVERVIEW_HEIGHT, cellSize, RULER_HEIGHT + ROWS_HEIGHT)
    context.fillStyle = '#fff'
    context.fillRect(Math.round(playheadX), OVERVIEW_HEIGHT, 2, RULER_HEIGHT + ROWS_HEIGHT)
    context.shadowColor = '#fff'
    context.shadowBlur = 7
    context.fillRect(Math.round(playheadX), OVERVIEW_HEIGHT, 1, RULER_HEIGHT + ROWS_HEIGHT)
    context.shadowBlur = 0
    context.restore()

    context.fillStyle = '#130b27'
    context.fillRect(0, 0, LABEL_WIDTH, CANVAS_HEIGHT - SCROLLBAR_HEIGHT)
    context.strokeStyle = '#604676'
    context.beginPath()
    context.moveTo(LABEL_WIDTH - .5, 0)
    context.lineTo(LABEL_WIDTH - .5, CANVAS_HEIGHT - SCROLLBAR_HEIGHT)
    context.stroke()
    context.font = '9px Consolas, monospace'
    context.textBaseline = 'middle'
    context.fillStyle = '#a88bb5'
    context.fillText('VIEW', 9, OVERVIEW_HEIGHT / 2)
    context.fillText('FRAME', 9, OVERVIEW_HEIGHT + RULER_HEIGHT / 2)
    for (let lane = 0; lane < ACTIONS.length; lane += 1) {
      const action = ACTIONS[lane]
      const y = HEADER_HEIGHT + lane * ROW_HEIGHT
      context.fillStyle = LANE_COLORS[action]
      context.strokeStyle = LANE_COLORS[action]
      context.strokeRect(10.5, y + 6.5, 20, 20)
      context.font = '9px Consolas, monospace'
      context.fillText(ACTION_GLYPHS[action], 16, y + ROW_HEIGHT / 2)
      context.fillStyle = '#b09bb9'
      context.font = '11px Consolas, monospace'
      context.fillText(ACTION_LABELS[action], 40, y + ROW_HEIGHT / 2)
    }
    const labelStateY = HEADER_HEIGHT + ACTIONS.length * ROW_HEIGHT
    context.fillStyle = '#cdb8d2'
    context.fillText('◆  STATE', 9, labelStateY + ROW_HEIGHT / 2)

    const trackY = CANVAS_HEIGHT - SCROLLBAR_HEIGHT + 4
    const trackWidth = Math.max(1, width - LABEL_WIDTH - 8)
    context.fillStyle = '#1c112d'
    context.fillRect(LABEL_WIDTH + 4, trackY, trackWidth, 8)
    const visibleRatio = Math.min(1, visibleWidth / contentWidth)
    const thumbWidth = Math.max(32, trackWidth * visibleRatio)
    const thumbTravel = trackWidth - thumbWidth
    const thumbX = LABEL_WIDTH + 4 + (maxScroll ? scroll / maxScroll * thumbTravel : 0)
    context.fillStyle = '#795989'
    context.fillRect(thumbX, trackY, thumbWidth, 8)
  }, [cellSize, frame, frameCount, inputs, maxScrollFor, selection, states, viewportRevision])

  const pointerFrame = (event: ReactPointerEvent<HTMLCanvasElement>): number => {
    const rect = event.currentTarget.getBoundingClientRect()
    return Math.max(0, Math.min(frameCount, Math.floor((event.clientX - rect.left - LABEL_WIDTH + scrollRef.current) / cellSize)))
  }

  const overviewFrame = (event: ReactPointerEvent<HTMLCanvasElement>): number => {
    const rect = event.currentTarget.getBoundingClientRect()
    const overviewX = LABEL_WIDTH + 6
    const overviewWidth = Math.max(1, rect.width - overviewX - 6)
    const ratio = Math.max(0, Math.min(1, (event.clientX - rect.left - overviewX) / overviewWidth))
    return Math.round(ratio * frameCount)
  }

  const copySelection = (): void => {
    if (!selection) return
    clipboardRef.current = { ...selection }
  }

  const pasteSelection = (): void => {
    const copied = clipboardRef.current
    if (!copied || frameCount === 0) return
    const start = Math.min(frameCount - 1, frame)
    const end = Math.min(frameCount, start + copied.end - copied.start)
    if (end <= start) return
    onPaint(copied.action, start, end - 1, true)
    setSelection({ action: copied.action, start, end })
    onSeek(start)
    onEditComplete()
  }

  const deleteSelection = (): void => {
    if (!selection) return
    onPaint(selection.action, selection.start, selection.end - 1, false)
    setSelection(null)
    onEditComplete()
  }

  return <section className="timeline-panel panel-frame">
    <div className="timeline-toolbar">
      <div><small>INPUT EDITOR / CANVAS</small><h2>逐帧时间线</h2></div>
      <div className="timeline-tools">
        <span>{states.filter(Boolean).length}/{states.length} STATES</span>
        <label>容量<input aria-label="总帧数" type="number" min="1" max="1048576" value={frameCount} onChange={(event) => onResize(Number(event.target.value))} /></label>
        <span className="timeline-zoom">Ctrl + 滚轮缩放 <output>{cellSize}px</output></span>
      </div>
    </div>
    <canvas
      ref={canvasRef}
      className="timeline-canvas"
      style={{ height: CANVAS_HEIGHT }}
      tabIndex={0}
      aria-label="Canvas 时间线。点击按键块选中操作，可复制、粘贴、删除并拖到其他轨道；方向键左右移动播放头或选中操作。"
      onContextMenu={(event) => event.preventDefault()}
      onKeyDown={(event) => {
        const modifier = event.ctrlKey || event.metaKey
        if (modifier && event.code === 'KeyC') {
          if (!selection) return
          event.preventDefault()
          event.stopPropagation()
          copySelection()
          return
        }
        if (modifier && event.code === 'KeyV') {
          if (!clipboardRef.current) return
          event.preventDefault()
          event.stopPropagation()
          pasteSelection()
          return
        }
        if ((event.code === 'Delete' || event.code === 'Backspace') && selection) {
          event.preventDefault()
          event.stopPropagation()
          deleteSelection()
          return
        }
        if (event.code === 'Escape' && selection) {
          event.preventDefault()
          event.stopPropagation()
          setSelection(null)
          return
        }
        if (event.code !== 'ArrowLeft' && event.code !== 'ArrowRight') return
        event.preventDefault()
        event.stopPropagation()
        const delta = event.code === 'ArrowLeft' ? -1 : 1
        if (selection) {
          const moved = onMove(selection.action, selection.action, selection.start, selection.end, delta)
          setSelection({ ...selection, ...moved })
          onEditComplete()
        } else {
          onSeek(frame + delta)
        }
      }}
      onPointerDown={(event) => {
        event.currentTarget.focus({ preventScroll: true })
        const rect = event.currentTarget.getBoundingClientRect()
        const x = event.clientX - rect.left
        const y = event.clientY - rect.top
        if (x < LABEL_WIDTH) return
        const contentWidth = Math.max(1, (frameCount + 1) * cellSize)
        const visibleWidth = Math.max(1, rect.width - LABEL_WIDTH)
        const maxScroll = maxScrollFor(rect.width)
        if (y < OVERVIEW_HEIGHT) {
          setSelection(null)
          const overviewX = LABEL_WIDTH + 6
          const trackWidth = Math.max(1, rect.width - overviewX - 6)
          const pointerX = overviewX + frame / Math.max(1, frameCount) * trackWidth
          event.currentTarget.setPointerCapture(event.pointerId)
          if (Math.abs(x - pointerX) <= PLAYHEAD_HIT_RADIUS) {
            dragRef.current = { mode: 'playhead', source: 'overview' }
            onSeek(overviewFrame(event))
          } else {
            const clickedFrame = overviewFrame(event)
            clampScroll(clickedFrame * cellSize - visibleWidth / 2, rect.width)
            dragRef.current = { mode: 'overview', startX: x, startScroll: scrollRef.current, contentWidth, trackWidth }
          }
          return
        }
        if (y < HEADER_HEIGHT) {
          setSelection(null)
          const pointerX = LABEL_WIDTH + frame * cellSize - scrollRef.current
          if (Math.abs(x - pointerX) <= PLAYHEAD_HIT_RADIUS) {
            dragRef.current = { mode: 'playhead', source: 'ruler' }
            event.currentTarget.setPointerCapture(event.pointerId)
            onSeek(pointerFrame(event))
            return
          }
          dragRef.current = { mode: 'pan', startX: x, startScroll: scrollRef.current }
          event.currentTarget.setPointerCapture(event.pointerId)
          return
        }
        if (y >= HEADER_HEIGHT + ROWS_HEIGHT) {
          setSelection(null)
          const trackWidth = Math.max(1, rect.width - LABEL_WIDTH - 8)
          const thumbWidth = Math.max(32, trackWidth * Math.min(1, visibleWidth / contentWidth))
          dragRef.current = { mode: 'scrollbar', startX: x, startScroll: scrollRef.current, maxScroll, thumbTravel: Math.max(1, trackWidth - thumbWidth) }
          event.currentTarget.setPointerCapture(event.pointerId)
          return
        }
        const value = Math.min(frameCount - 1, pointerFrame(event))
        const lane = Math.floor((y - HEADER_HEIGHT) / ROW_HEIGHT)
        if (lane >= ACTIONS.length) {
          setSelection(null)
          onSeek(value)
          return
        }
        const action = ACTIONS[lane]
        const erase = event.button === 2 || event.altKey
        const run = !erase ? heldRunAt(inputs, action, value) : undefined
        event.currentTarget.setPointerCapture(event.pointerId)
        if (run) {
          setSelection({ action, start: run[0], end: run[1] })
          dragRef.current = { mode: 'move', action, anchor: value, start: run[0], end: run[1] }
          onSeek(value)
        } else {
          const paintValue = erase ? false : !inputs[value]?.[action]
          dragRef.current = { mode: 'paint', action, anchor: value, value: paintValue }
          onPaint(action, value, value, paintValue)
          setSelection(paintValue ? { action, start: value, end: value + 1 } : null)
          onSeek(value)
        }
      }}
      onPointerMove={(event) => {
        const rect = event.currentTarget.getBoundingClientRect()
        const x = event.clientX - rect.left
        const y = event.clientY - rect.top
        const drag = dragRef.current
        if (!drag) {
          if (x < LABEL_WIDTH) event.currentTarget.style.cursor = 'default'
          else if (y < OVERVIEW_HEIGHT) {
            const overviewX = LABEL_WIDTH + 6
            const trackWidth = Math.max(1, rect.width - overviewX - 6)
            const pointerX = overviewX + frame / Math.max(1, frameCount) * trackWidth
            event.currentTarget.style.cursor = Math.abs(x - pointerX) <= PLAYHEAD_HIT_RADIUS ? 'ew-resize' : 'grab'
          } else if (y < HEADER_HEIGHT) {
            const pointerX = LABEL_WIDTH + frame * cellSize - scrollRef.current
            event.currentTarget.style.cursor = Math.abs(x - pointerX) <= PLAYHEAD_HIT_RADIUS ? 'ew-resize' : 'grab'
          } else if (y >= HEADER_HEIGHT + ROWS_HEIGHT) event.currentTarget.style.cursor = 'ew-resize'
          else {
            const lane = Math.floor((y - HEADER_HEIGHT) / ROW_HEIGHT)
            const action = ACTIONS[lane]
            event.currentTarget.style.cursor = action && heldRunAt(inputs, action, Math.min(frameCount - 1, pointerFrame(event))) ? 'grab' : action ? 'crosshair' : 'pointer'
          }
          return
        }
        if (drag.mode === 'playhead') onSeek(drag.source === 'overview' ? overviewFrame(event) : pointerFrame(event))
        else if (drag.mode === 'pan') clampScroll(drag.startScroll - (x - drag.startX), rect.width)
        else if (drag.mode === 'overview') clampScroll(drag.startScroll + (x - drag.startX) / drag.trackWidth * drag.contentWidth, rect.width)
        else if (drag.mode === 'scrollbar') clampScroll(drag.startScroll + (x - drag.startX) / drag.thumbTravel * drag.maxScroll, rect.width)
        else if (drag.mode === 'paint') {
          const current = Math.min(frameCount - 1, pointerFrame(event))
          onPaint(drag.action, drag.anchor, current, drag.value)
          if (drag.value) setSelection({ action: drag.action, start: Math.min(drag.anchor, current), end: Math.max(drag.anchor, current) + 1 })
        } else {
          const lane = Math.max(0, Math.min(ACTIONS.length - 1, Math.floor((y - HEADER_HEIGHT) / ROW_HEIGHT)))
          const targetAction = ACTIONS[lane]
          const desired = Math.max(-drag.start, Math.min(frameCount - drag.end, pointerFrame(event) - drag.anchor))
          if (desired !== 0 || targetAction !== drag.action) {
            const previousStart = drag.start
            const moved = onMove(drag.action, targetAction, drag.start, drag.end, desired)
            drag.action = targetAction
            drag.start = moved.start
            drag.end = moved.end
            drag.anchor += moved.start - previousStart
            setSelection({ action: targetAction, start: moved.start, end: moved.end })
          }
        }
      }}
      onPointerUp={(event) => {
        const drag = dragRef.current
        dragRef.current = null
        if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId)
        if (drag?.mode === 'paint' || drag?.mode === 'move') onEditComplete()
      }}
      onPointerCancel={() => { dragRef.current = null }}
    />
    <div className="timeline-legend"><span><i className="held" />点击选中 · 拖动/跨轨移动</span><span><i className="cached" />Ctrl+C / Ctrl+V · Delete</span><span><i className="invalid" />Alt/右键拖动擦除</span><em>Ctrl+滚轮缩放 · 滚轮横移 · ←/→ 调整帧或操作</em></div>
  </section>
}
