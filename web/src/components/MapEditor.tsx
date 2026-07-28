import { useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from 'react'
import type { EntityKind, GymMap, MapEntity, SimState } from '../model'
import type { VisualTheme } from '../visualThemes'
import { GameView } from './GameView'

const GRID_SIZE = 8

type EditorTool = 'select' | 'solid' | 'spawn' | 'erase' | `entity:${EntityKind}`
type EditorSelection = { type: 'solid' | 'entity'; index: number }
type EditableBounds = { x: number; y: number; width: number; height: number }

interface EntityTemplate {
  kind: EntityKind
  label: string
  name: string
  width: number
  height: number
  direction?: { x: number; y: number }
}

interface DragState {
  kind: 'create-solid' | 'move-selection'
  start: { x: number; y: number }
  originalMap: GymMap
  selection?: EditorSelection
}

const ENTITY_TEMPLATES: readonly EntityTemplate[] = [
  { kind: 'jump_thru', label: '木板', name: 'jumpThru', width: 32, height: 8 },
  { kind: 'spikes', label: '尖刺', name: 'spikesUp', width: 32, height: 3, direction: { x: 0, y: -1 } },
  { kind: 'water', label: '水', name: 'water', width: 32, height: 32 },
  { kind: 'dream_block', label: '梦块', name: 'dreamBlock', width: 32, height: 32 },
  { kind: 'booster', label: '绿泡', name: 'booster', width: 16, height: 16 },
  { kind: 'red_booster', label: '红泡', name: 'redBooster', width: 16, height: 16 },
  { kind: 'spring', label: '弹簧', name: 'spring', width: 16, height: 8, direction: { x: 0, y: -1 } },
  { kind: 'strawberry', label: '草莓', name: 'strawberry', width: 16, height: 16 },
  { kind: 'fly_feather', label: '羽毛', name: 'infiniteStar', width: 20, height: 20 },
  { kind: 'bumper', label: '碰碰球', name: 'bigSpinner', width: 24, height: 24 },
  { kind: 'theo_crystal', label: 'Theo 水晶', name: 'theoCrystal', width: 8, height: 10 },
  { kind: 'glider', label: '水母', name: 'glider', width: 8, height: 10 },
] as const

export interface MapEditorProps {
  map: GymMap
  state: SimState
  frame: number
  theme: VisualTheme
  experiencing: boolean
  ready: boolean
  onChange: (map: GymMap) => void
  onExperienceChange: (experiencing: boolean) => void
  onResetExperience: () => void
}

export function snapToGrid(value: number, origin = 0, grid = GRID_SIZE): number {
  return Math.round((value - origin) / grid) * grid + origin
}

export function createEditorEntity(kind: EntityKind, x: number, y: number): MapEntity | null {
  const template = ENTITY_TEMPLATES.find((candidate) => candidate.kind === kind)
  if (!template) return null
  return {
    kind,
    bounds: { x, y, width: template.width, height: template.height },
    direction: template.direction ? { ...template.direction } : { x: 0, y: 0 },
    name: template.name,
  }
}

function selectionBounds(map: GymMap, selection: EditorSelection | null): EditableBounds | null {
  if (!selection) return null
  return selection.type === 'solid'
    ? map.solids[selection.index] ?? null
    : map.entities[selection.index]?.bounds ?? null
}

function replaceSelectionBounds(map: GymMap, selection: EditorSelection, bounds: EditableBounds): GymMap {
  if (selection.type === 'solid') {
    const solids = map.solids.map((solid, index) => index === selection.index ? bounds : solid)
    return { ...map, solids }
  }
  const entities = map.entities.map((entity, index) => index === selection.index
    ? { ...entity, bounds }
    : entity)
  return { ...map, entities }
}

function deleteSelection(map: GymMap, selection: EditorSelection): GymMap {
  return selection.type === 'solid'
    ? { ...map, solids: map.solids.filter((_, index) => index !== selection.index) }
    : { ...map, entities: map.entities.filter((_, index) => index !== selection.index) }
}

function pointInMap(clientX: number, clientY: number, svg: SVGSVGElement, map: GymMap): { x: number; y: number } {
  const rect = svg.getBoundingClientRect()
  const scale = Math.min(rect.width / map.bounds.width, rect.height / map.bounds.height)
  const offsetX = (rect.width - map.bounds.width * scale) / 2
  const offsetY = (rect.height - map.bounds.height * scale) / 2
  return {
    x: Math.max(map.bounds.x, Math.min(map.bounds.x + map.bounds.width, map.bounds.x + (clientX - rect.left - offsetX) / scale)),
    y: Math.max(map.bounds.y, Math.min(map.bounds.y + map.bounds.height, map.bounds.y + (clientY - rect.top - offsetY) / scale)),
  }
}

function normalizedRect(from: { x: number; y: number }, to: { x: number; y: number }, map: GymMap): EditableBounds {
  const x1 = snapToGrid(from.x, map.bounds.x)
  const y1 = snapToGrid(from.y, map.bounds.y)
  const x2 = snapToGrid(to.x, map.bounds.x)
  const y2 = snapToGrid(to.y, map.bounds.y)
  return {
    x: Math.min(x1, x2),
    y: Math.min(y1, y2),
    width: Math.max(GRID_SIZE, Math.abs(x2 - x1)),
    height: Math.max(GRID_SIZE, Math.abs(y2 - y1)),
  }
}

function toolLabel(tool: EditorTool): string {
  if (!tool.startsWith('entity:')) return ({ select: '选择', solid: '实心块', spawn: '出生点', erase: '删除' } as const)[tool]
  const kind = tool.slice('entity:'.length) as EntityKind
  return ENTITY_TEMPLATES.find((template) => template.kind === kind)?.label ?? kind
}

export function MapEditor({ map, state, frame, theme, experiencing, ready, onChange, onExperienceChange, onResetExperience }: MapEditorProps) {
  const [tool, setTool] = useState<EditorTool>('select')
  const [selection, setSelection] = useState<EditorSelection | null>(null)
  const [draft, setDraft] = useState<EditableBounds | null>(null)
  const [historyRevision, setHistoryRevision] = useState(0)
  const drag = useRef<DragState | null>(null)
  const undoStack = useRef<GymMap[]>([])
  const redoStack = useRef<GymMap[]>([])
  const bounds = selectionBounds(map, selection)
  const selectedEntity = selection?.type === 'entity' ? map.entities[selection.index] : null
  const stats = useMemo(() => ({ solids: map.solids.length, entities: map.entities.length }), [map.entities.length, map.solids.length])

  const rememberAndChange = (next: GymMap) => {
    undoStack.current.push(structuredClone(map))
    redoStack.current = []
    setHistoryRevision((value) => value + 1)
    onChange(next)
  }

  const finishContinuousChange = (originalMap: GymMap) => {
    undoStack.current.push(structuredClone(originalMap))
    redoStack.current = []
    setHistoryRevision((value) => value + 1)
  }

  const undo = () => {
    const previous = undoStack.current.pop()
    if (!previous) return
    redoStack.current.push(structuredClone(map))
    setSelection(null)
    setHistoryRevision((value) => value + 1)
    onChange(previous)
  }

  const redo = () => {
    const next = redoStack.current.pop()
    if (!next) return
    undoStack.current.push(structuredClone(map))
    setSelection(null)
    setHistoryRevision((value) => value + 1)
    onChange(next)
  }

  const chooseTool = (next: EditorTool) => {
    setTool(next)
    if (next !== 'select') setSelection(null)
  }

  const beginSelectionDrag = (event: ReactPointerEvent<SVGRectElement>, nextSelection: EditorSelection) => {
    event.stopPropagation()
    if (tool === 'erase') {
      rememberAndChange(deleteSelection(map, nextSelection))
      setSelection(null)
      return
    }
    if (tool !== 'select') return
    const svg = event.currentTarget.ownerSVGElement
    if (!svg) return
    drag.current = { kind: 'move-selection', start: pointInMap(event.clientX, event.clientY, svg, map), originalMap: structuredClone(map), selection: nextSelection }
    setSelection(nextSelection)
    event.currentTarget.setPointerCapture(event.pointerId)
  }

  const pointerDown = (event: ReactPointerEvent<SVGSVGElement>) => {
    if (event.target !== event.currentTarget && !(event.target instanceof SVGRectElement && event.target.dataset.editorBackground === 'true')) return
    const point = pointInMap(event.clientX, event.clientY, event.currentTarget, map)
    if (tool === 'select') {
      setSelection(null)
    } else if (tool === 'solid') {
      drag.current = { kind: 'create-solid', start: point, originalMap: structuredClone(map) }
      setDraft(normalizedRect(point, point, map))
      event.currentTarget.setPointerCapture(event.pointerId)
    } else if (tool === 'spawn') {
      rememberAndChange({ ...map, spawn: { x: snapToGrid(point.x, map.bounds.x), y: snapToGrid(point.y, map.bounds.y) } })
    } else if (tool.startsWith('entity:')) {
      const entity = createEditorEntity(tool.slice('entity:'.length) as EntityKind, snapToGrid(point.x, map.bounds.x), snapToGrid(point.y, map.bounds.y))
      if (!entity) return
      rememberAndChange({ ...map, entities: [...map.entities, entity] })
      setSelection({ type: 'entity', index: map.entities.length })
      setTool('select')
    }
  }

  const pointerMove = (event: ReactPointerEvent<SVGSVGElement>) => {
    const currentDrag = drag.current
    if (!currentDrag) return
    const point = pointInMap(event.clientX, event.clientY, event.currentTarget, map)
    if (currentDrag.kind === 'create-solid') {
      setDraft(normalizedRect(currentDrag.start, point, map))
      return
    }
    const originalBounds = selectionBounds(currentDrag.originalMap, currentDrag.selection ?? null)
    if (!originalBounds || !currentDrag.selection) return
    const dx = snapToGrid(point.x - currentDrag.start.x)
    const dy = snapToGrid(point.y - currentDrag.start.y)
    const moved = {
      ...originalBounds,
      x: Math.max(map.bounds.x, Math.min(map.bounds.x + map.bounds.width - originalBounds.width, originalBounds.x + dx)),
      y: Math.max(map.bounds.y, Math.min(map.bounds.y + map.bounds.height - originalBounds.height, originalBounds.y + dy)),
    }
    onChange(replaceSelectionBounds(currentDrag.originalMap, currentDrag.selection, moved))
  }

  const pointerUp = () => {
    const currentDrag = drag.current
    if (!currentDrag) return
    if (currentDrag.kind === 'create-solid' && draft) {
      rememberAndChange({ ...map, solids: [...map.solids, draft] })
      setSelection({ type: 'solid', index: map.solids.length })
      setTool('select')
    } else if (currentDrag.kind === 'move-selection') {
      finishContinuousChange(currentDrag.originalMap)
    }
    drag.current = null
    setDraft(null)
  }

  const updateBounds = (field: keyof EditableBounds, value: number) => {
    if (!selection || !bounds || !Number.isFinite(value)) return
    const next = {
      ...bounds,
      [field]: field === 'width' || field === 'height' ? Math.max(1, value) : value,
    }
    rememberAndChange(replaceSelectionBounds(map, selection, next))
  }

  const removeSelected = () => {
    if (!selection) return
    rememberAndChange(deleteSelection(map, selection))
    setSelection(null)
  }

  return <main className={`map-editor ${experiencing ? 'experiencing' : ''}`}>
    <aside className="editor-palette">
      <div className="editor-panel-heading"><small>MAP TOOLS</small><h1>地图编辑器</h1></div>
      <div className="editor-tool-group editor-primary-tools">
        {(['select', 'solid', 'spawn', 'erase'] as const).map((candidate) => <button key={candidate} className={tool === candidate ? 'active' : ''} onClick={() => chooseTool(candidate)} aria-pressed={tool === candidate}>
          <span>{candidate === 'select' ? '↖' : candidate === 'solid' ? '▦' : candidate === 'spawn' ? '◆' : '⌫'}</span>{toolLabel(candidate)}
        </button>)}
      </div>
      <div className="editor-tool-section"><small>实体</small><div className="editor-entity-tools">
        {ENTITY_TEMPLATES.map((template) => {
          const candidate = `entity:${template.kind}` as EditorTool
          return <button key={template.kind} className={tool === candidate ? 'active' : ''} onClick={() => chooseTool(candidate)} aria-pressed={tool === candidate}>{template.label}</button>
        })}
      </div></div>
      <div className="editor-history" data-revision={historyRevision}>
        <button onClick={undo} disabled={undoStack.current.length === 0}>↶ 撤销</button>
        <button onClick={redo} disabled={redoStack.current.length === 0}>↷ 重做</button>
      </div>
      <p className="editor-hint">8 px 网格 · 拖动画实心块<br />选择对象后可拖动或精确修改</p>
    </aside>

    <section className="editor-stage">
      <div className="editor-stage-bar">
        <div><small>{experiencing ? 'LIVE EXPERIENCE' : 'EDITING'}</small><strong>{map.name}</strong><span>{stats.solids} solids · {stats.entities} entities</span></div>
        <div className="editor-stage-actions">
          {experiencing && <button onClick={() => onResetExperience()}>重生</button>}
          <button className={experiencing ? 'stop' : 'experience'} disabled={!ready} onClick={() => onExperienceChange(!experiencing)}>{experiencing ? '■ 返回编辑' : '▶ 实时体验'}</button>
        </div>
      </div>
      <GameView map={map} state={state} states={[]} frame={frame} stale={false} theme={theme}>
        {!experiencing && <svg
          className={`map-editor-overlay tool-${tool.replace(':', '-')}`}
          viewBox={`${map.bounds.x} ${map.bounds.y} ${map.bounds.width} ${map.bounds.height}`}
          preserveAspectRatio="xMidYMid meet"
          aria-label="地图编辑画布"
          onPointerDown={pointerDown}
          onPointerMove={pointerMove}
          onPointerUp={pointerUp}
          onPointerCancel={pointerUp}
        >
          <defs><pattern id="editor-grid" width={GRID_SIZE} height={GRID_SIZE} patternUnits="userSpaceOnUse"><path d={`M ${GRID_SIZE} 0 L 0 0 0 ${GRID_SIZE}`} /></pattern></defs>
          <rect data-editor-background="true" className="editor-map-hitarea" x={map.bounds.x} y={map.bounds.y} width={map.bounds.width} height={map.bounds.height} />
          <rect className="editor-grid" x={map.bounds.x} y={map.bounds.y} width={map.bounds.width} height={map.bounds.height} />
          {map.solids.map((solid, index) => <rect key={`solid-${index}`} className={`editor-object solid ${selection?.type === 'solid' && selection.index === index ? 'selected' : ''}`} {...solid} onPointerDown={(event) => beginSelectionDrag(event, { type: 'solid', index })} />)}
          {map.entities.map((entity, index) => <rect key={`entity-${index}`} data-kind={entity.kind} className={`editor-object entity ${selection?.type === 'entity' && selection.index === index ? 'selected' : ''}`} {...entity.bounds} onPointerDown={(event) => beginSelectionDrag(event, { type: 'entity', index })} />)}
          {draft && <rect className="editor-draft" {...draft} />}
          <g className="editor-spawn" transform={`translate(${map.spawn.x} ${map.spawn.y})`}><circle r="7" /><path d="M -4 0 H 4 M 0 -4 V 4" /></g>
        </svg>}
      </GameView>
      {experiencing && <div className="editor-live-note"><i />WASM 60 FPS · 与游玩模式相同 · 不记录 state</div>}
    </section>

    <aside className="editor-inspector">
      <div className="editor-panel-heading"><small>INSPECTOR</small><h2>{selection ? selectedEntity?.name ?? '实心块' : '房间'}</h2></div>
      {selection && bounds ? <>
        <div className="editor-kind"><span>{selection.type === 'solid' ? 'SOLID' : selectedEntity?.kind}</span><button onClick={removeSelected}>删除对象</button></div>
        <div className="editor-field-grid">
          {(['x', 'y', 'width', 'height'] as const).map((field) => <label key={field}><small>{field.toUpperCase()}</small><input type="number" value={bounds[field]} onChange={(event) => updateBounds(field, Number(event.target.value))} /></label>)}
        </div>
        {selectedEntity && <div className="editor-readout"><small>方向</small><code>{selectedEntity.direction.x}, {selectedEntity.direction.y}</code></div>}
      </> : <>
        <label className="editor-room-name"><small>ROOM NAME</small><input value={map.name} onChange={(event) => onChange({ ...map, name: event.target.value })} /></label>
        <div className="editor-readout"><small>边界</small><code>{map.bounds.width} × {map.bounds.height}</code></div>
        <div className="editor-readout"><small>出生点</small><code>{map.spawn.x}, {map.spawn.y}</code></div>
        <div className="editor-readout"><small>来源</small><code>{map.source_package ?? 'custom'}</code></div>
      </>}
      <div className="editor-inspector-tip"><strong>{experiencing ? '正在实时体验' : toolLabel(tool)}</strong><span>{experiencing ? '键盘和手柄输入直接送入 WASM；返回编辑时地图保持不变。' : tool === 'select' ? '点击对象以选择，拖动对象会按网格吸附。' : tool === 'solid' ? '在画布空白处拖动，创建新的碰撞块。' : '在画布中点击即可应用当前工具。'}</span></div>
    </aside>
  </main>
}
