import { useMemo, useRef, useState, type PointerEvent as ReactPointerEvent } from 'react'
import type { EntityKind, GymMap, MapEntity, SimState } from '../model'
import type { VisualTheme } from '../visualThemes'
import { GameView } from './GameView'

const GRID_SIZE = 8

type EditorTool = 'select' | 'solid' | 'spawn' | 'erase' | `entity:${string}`
type EditorSelection = { type: 'solid' | 'entity'; index: number }
type EditableBounds = { x: number; y: number; width: number; height: number }

interface EntityTemplate {
  id: string
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
  { id: 'jump-thru', kind: 'jump_thru', label: '木板', name: 'jumpThru', width: 32, height: 8 },
  { id: 'spikes-up', kind: 'spikes', label: '上刺', name: 'spikesUp', width: 32, height: 3, direction: { x: 0, y: -1 } },
  { id: 'spikes-down', kind: 'spikes', label: '下刺', name: 'spikesDown', width: 32, height: 3, direction: { x: 0, y: 1 } },
  { id: 'spikes-left', kind: 'spikes', label: '左刺', name: 'spikesLeft', width: 3, height: 32, direction: { x: -1, y: 0 } },
  { id: 'spikes-right', kind: 'spikes', label: '右刺', name: 'spikesRight', width: 3, height: 32, direction: { x: 1, y: 0 } },
  { id: 'crystal-spinner', kind: 'crystal_static_spinner', label: '圆刺', name: 'spinner', width: 16, height: 12 },
  { id: 'water', kind: 'water', label: '水', name: 'water', width: 32, height: 32 },
  { id: 'dream-block', kind: 'dream_block', label: '梦块', name: 'dreamBlock', width: 32, height: 32 },
  { id: 'booster', kind: 'booster', label: '绿泡', name: 'booster', width: 16, height: 16 },
  { id: 'red-booster', kind: 'red_booster', label: '红泡', name: 'redBooster', width: 16, height: 16 },
  { id: 'spring', kind: 'spring', label: '弹簧', name: 'spring', width: 16, height: 8, direction: { x: 0, y: -1 } },
  { id: 'strawberry', kind: 'strawberry', label: '草莓', name: 'strawberry', width: 16, height: 16 },
  { id: 'fly-feather', kind: 'fly_feather', label: '羽毛', name: 'infiniteStar', width: 20, height: 20 },
  { id: 'bumper', kind: 'bumper', label: '碰碰球', name: 'bigSpinner', width: 24, height: 24 },
  { id: 'theo-crystal', kind: 'theo_crystal', label: 'Theo 水晶', name: 'theoCrystal', width: 8, height: 10 },
  { id: 'glider', kind: 'glider', label: '水母', name: 'glider', width: 8, height: 10 },
] as const

const SPIKE_DIRECTIONS = [
  { label: '上', name: 'spikesUp', direction: { x: 0, y: -1 } },
  { label: '下', name: 'spikesDown', direction: { x: 0, y: 1 } },
  { label: '左', name: 'spikesLeft', direction: { x: -1, y: 0 } },
  { label: '右', name: 'spikesRight', direction: { x: 1, y: 0 } },
] as const

const SPINNER_VARIANTS = [
  { id: 'theme', label: '主题' },
  { id: 'blue', label: '蓝' },
  { id: 'red', label: '红' },
  { id: 'purple', label: '紫' },
  { id: 'rainbow', label: '彩虹' },
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

export function createEditorEntity(templateIdOrKind: string, x: number, y: number): MapEntity | null {
  const template = ENTITY_TEMPLATES.find((candidate) => candidate.id === templateIdOrKind)
    ?? ENTITY_TEMPLATES.find((candidate) => candidate.kind === templateIdOrKind)
  if (!template) return null
  return {
    kind: template.kind,
    bounds: { x, y, width: template.width, height: template.height },
    direction: template.direction ? { ...template.direction } : { x: 0, y: 0 },
    name: template.name,
  }
}

export function setEditorSpikeDirection(entity: MapEntity, direction: { x: number; y: number }): MapEntity {
  const config = SPIKE_DIRECTIONS.find((candidate) => candidate.direction.x === direction.x && candidate.direction.y === direction.y)
  if (entity.kind !== 'spikes' || !config) return entity
  const wasHorizontal = Math.abs(entity.direction.y) > 0
  const willBeHorizontal = Math.abs(direction.y) > 0
  const length = wasHorizontal ? entity.bounds.width : entity.bounds.height
  return {
    ...entity,
    name: config.name,
    direction: { ...direction },
    bounds: {
      ...entity.bounds,
      width: willBeHorizontal ? length : 3,
      height: willBeHorizontal ? 3 : length,
    },
  }
}

export function editorEntityHitBounds(entity: MapEntity): MapEntity['bounds'] {
  const box = entity.bounds
  if (entity.kind !== 'spikes') return box
  if (Math.abs(entity.direction.y) > 0) return { x: box.x, y: box.y - 3, width: box.width, height: 9 }
  return { x: box.x - 3, y: box.y, width: 9, height: box.height }
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
  if (tool === 'select') return '选择'
  if (tool === 'solid') return '实心块'
  if (tool === 'spawn') return '出生点'
  if (tool === 'erase') return '删除'
  const templateId = tool.slice('entity:'.length)
  return ENTITY_TEMPLATES.find((template) => template.id === templateId)?.label ?? templateId
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
      const entity = createEditorEntity(tool.slice('entity:'.length), snapToGrid(point.x, map.bounds.x), snapToGrid(point.y, map.bounds.y))
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

  const updateSelectedEntity = (update: (entity: MapEntity) => MapEntity) => {
    if (selection?.type !== 'entity') return
    rememberAndChange({
      ...map,
      entities: map.entities.map((entity, index) => index === selection.index ? update(entity) : entity),
    })
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
          const candidate = `entity:${template.id}` as EditorTool
          return <button key={template.id} className={tool === candidate ? 'active' : ''} onClick={() => chooseTool(candidate)} aria-pressed={tool === candidate}>{template.label}</button>
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
          {map.entities.map((entity, index) => <rect key={`entity-${index}`} data-kind={entity.kind} className={`editor-object entity ${selection?.type === 'entity' && selection.index === index ? 'selected' : ''}`} {...editorEntityHitBounds(entity)} onPointerDown={(event) => beginSelectionDrag(event, { type: 'entity', index })} />)}
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
        {selectedEntity?.kind === 'spikes' && <div className="editor-option-group"><small>尖刺方向</small><div className="editor-option-buttons">
          {SPIKE_DIRECTIONS.map((option) => <button key={option.name} className={selectedEntity.name === option.name ? 'active' : ''} onClick={() => updateSelectedEntity((entity) => setEditorSpikeDirection(entity, option.direction))}>{option.label}</button>)}
        </div></div>}
        {selectedEntity?.kind === 'crystal_static_spinner' && <div className="editor-option-group"><small>圆刺外观</small><div className="editor-option-buttons spinner-variants">
          {SPINNER_VARIANTS.map((option) => <button key={option.id} className={(selectedEntity.variant ?? 'theme') === option.id ? 'active' : ''} onClick={() => updateSelectedEntity((entity) => ({ ...entity, variant: option.id === 'theme' ? undefined : option.id }))}>{option.label}</button>)}
        </div></div>}
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
