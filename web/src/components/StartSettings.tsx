import { useEffect, useMemo, useState, type FormEvent, type KeyboardEvent, type PointerEvent } from 'react'
import type { GymMap, Vec2 } from '../model'

export interface StartConfiguration {
  room: string
  position: Vec2
}

interface StartSettingsProps {
  rooms: readonly GymMap[]
  room: string
  position: Vec2
  busy: boolean
  onApply(configuration: StartConfiguration): void
  onClose(): void
}

function roomName(map: GymMap): string {
  return map.room ?? map.name
}

function roomLabel(room: string): string {
  if (room === 'playground') return '主训练场'
  if (room.startsWith('transition')) return '过渡训练场'
  return room
}

function clampPosition(map: GymMap, position: Vec2): Vec2 {
  return {
    x: Math.round(Math.min(map.bounds.x + map.bounds.width, Math.max(map.bounds.x, position.x))),
    y: Math.round(Math.min(map.bounds.y + map.bounds.height, Math.max(map.bounds.y, position.y))),
  }
}

export function StartSettings({ rooms, room, position, busy, onApply, onClose }: StartSettingsProps) {
  const initialRoom = rooms.find((candidate) => roomName(candidate) === room) ?? rooms[0]
  const [draftRoom, setDraftRoom] = useState(initialRoom ? roomName(initialRoom) : room)
  const [draftPosition, setDraftPosition] = useState<Vec2>({ ...position })
  const selectedMap = useMemo(
    () => rooms.find((candidate) => roomName(candidate) === draftRoom) ?? initialRoom,
    [draftRoom, initialRoom, rooms],
  )

  useEffect(() => {
    const closeOnEscape = (event: globalThis.KeyboardEvent) => {
      if (event.key === 'Escape' && !busy) onClose()
    }
    window.addEventListener('keydown', closeOnEscape)
    return () => window.removeEventListener('keydown', closeOnEscape)
  }, [busy, onClose])

  const selectRoom = (map: GymMap) => {
    setDraftRoom(roomName(map))
    setDraftPosition({ ...map.spawn })
  }

  const selectPoint = (event: PointerEvent<SVGSVGElement>) => {
    if (!selectedMap || busy) return
    const bounds = event.currentTarget.getBoundingClientRect()
    if (bounds.width <= 0 || bounds.height <= 0) return
    setDraftPosition(clampPosition(selectedMap, {
      x: selectedMap.bounds.x + (event.clientX - bounds.left) / bounds.width * selectedMap.bounds.width,
      y: selectedMap.bounds.y + (event.clientY - bounds.top) / bounds.height * selectedMap.bounds.height,
    }))
  }

  const movePoint = (event: KeyboardEvent<SVGSVGElement>) => {
    if (!selectedMap || busy) return
    const offsets: Record<string, Vec2> = {
      ArrowUp: { x: 0, y: -8 },
      ArrowDown: { x: 0, y: 8 },
      ArrowLeft: { x: -8, y: 0 },
      ArrowRight: { x: 8, y: 0 },
    }
    const offset = offsets[event.key]
    if (!offset) return
    event.preventDefault()
    setDraftPosition((current) => clampPosition(selectedMap, {
      x: current.x + offset.x,
      y: current.y + offset.y,
    }))
  }

  const submit = (event: FormEvent) => {
    event.preventDefault()
    if (!selectedMap) return
    onApply({ room: roomName(selectedMap), position: { ...draftPosition } })
  }

  return <div className="modal-backdrop" onPointerDown={(event) => event.target === event.currentTarget && !busy && onClose()}>
    <section className="start-modal panel-frame" role="dialog" aria-modal="true" aria-labelledby="start-settings-title">
      <div className="panel-heading">
        <div><small>SIMULATION ORIGIN</small><h2 id="start-settings-title">选择开始位置</h2></div>
        <button className="icon-button" disabled={busy} onClick={onClose} aria-label="关闭">×</button>
      </div>
      <form onSubmit={submit}>
        <p className="modal-copy">先选房间，再直接点击地图放置玩家；方向键可按 8 像素微调。</p>
        <div className="start-map-grid">
          <div className="room-picker" role="group" aria-label="可用房间">
            {rooms.map((candidate) => {
              const candidateRoom = roomName(candidate)
              const selected = candidateRoom === draftRoom
              return <button
                type="button"
                key={candidateRoom}
                className={selected ? 'selected' : ''}
                aria-pressed={selected}
                disabled={busy}
                onClick={() => selectRoom(candidate)}
              >
                <small>ROOM</small>
                <strong>{roomLabel(candidateRoom)}</strong>
                <span>{candidateRoom}</span>
              </button>
            })}
          </div>
          {selectedMap && <div className="start-preview">
            <div className="start-preview-heading">
              <div><small>点击地图选择</small><strong>{roomLabel(roomName(selectedMap))}</strong></div>
              <output aria-label="已选坐标">X {draftPosition.x} · Y {draftPosition.y}</output>
            </div>
            <svg
              className="start-map"
              style={{ aspectRatio: `${selectedMap.bounds.width} / ${selectedMap.bounds.height}` }}
              viewBox={`${selectedMap.bounds.x} ${selectedMap.bounds.y} ${selectedMap.bounds.width} ${selectedMap.bounds.height}`}
              role="img"
              aria-label={`${roomLabel(roomName(selectedMap))} 起点地图，点击选择位置`}
              tabIndex={0}
              onPointerDown={selectPoint}
              onKeyDown={movePoint}
            >
              <rect className="map-background" x={selectedMap.bounds.x} y={selectedMap.bounds.y} width={selectedMap.bounds.width} height={selectedMap.bounds.height} />
              {selectedMap.solids.map((solid, index) => <rect className="map-solid" key={`solid-${index}`} {...solid} />)}
              {selectedMap.entities.map((entity, index) => <rect className="map-entity" data-kind={entity.kind} key={`entity-${index}`} {...entity.bounds} />)}
              <g className="default-spawn" transform={`translate(${selectedMap.spawn.x} ${selectedMap.spawn.y})`}>
                <circle r="7" /><path d="M -4 0 H 4 M 0 -4 V 4" />
              </g>
              <g className="selected-spawn" transform={`translate(${draftPosition.x} ${draftPosition.y})`}>
                <circle r="10" /><path d="M -6 0 H 6 M 0 -6 V 6" />
              </g>
            </svg>
            <button className="default-spawn-button" type="button" disabled={busy} onClick={() => setDraftPosition({ ...selectedMap.spawn })}>使用房间默认出生点</button>
          </div>}
        </div>
        <div className="modal-actions">
          <button type="button" disabled={busy} onClick={onClose}>取消</button>
          <button className="primary" type="submit" disabled={busy || !selectedMap}>{busy ? '正在加载房间…' : '从这里开始'}</button>
        </div>
      </form>
    </section>
  </div>
}
