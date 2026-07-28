import { useEffect, useState, type FormEvent } from 'react'
import type { Vec2 } from '../model'

export interface StartConfiguration {
  room: string
  position: Vec2
}

interface StartSettingsProps {
  room: string
  position: Vec2
  busy: boolean
  onApply(configuration: StartConfiguration): void
  onClose(): void
}

export function StartSettings({ room, position, busy, onApply, onClose }: StartSettingsProps) {
  const [draftRoom, setDraftRoom] = useState(room)
  const [x, setX] = useState(String(position.x))
  const [y, setY] = useState(String(position.y))
  const [error, setError] = useState('')

  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !busy) onClose()
    }
    window.addEventListener('keydown', closeOnEscape)
    return () => window.removeEventListener('keydown', closeOnEscape)
  }, [busy, onClose])

  const submit = (event: FormEvent) => {
    event.preventDefault()
    const selectedRoom = draftRoom.trim().replace(/^lvl_/, '')
    const positionX = Number(x)
    const positionY = Number(y)
    if (!selectedRoom) {
      setError('请输入开始房间')
      return
    }
    if (x.trim() === '' || y.trim() === '' || !Number.isFinite(positionX) || !Number.isFinite(positionY)) {
      setError('开始位置必须是有效的 X / Y 数值')
      return
    }
    setError('')
    onApply({ room: selectedRoom, position: { x: positionX, y: positionY } })
  }

  return <div className="modal-backdrop" onPointerDown={(event) => event.target === event.currentTarget && !busy && onClose()}>
    <section className="start-modal panel-frame" role="dialog" aria-modal="true" aria-labelledby="start-settings-title">
      <div className="panel-heading">
        <div><small>SIMULATION ORIGIN</small><h2 id="start-settings-title">开始房间与位置</h2></div>
        <button className="icon-button" disabled={busy} onClick={onClose} aria-label="关闭">×</button>
      </div>
      <form onSubmit={submit}>
        <p className="modal-copy">房间名可写 <code>playground</code> 或 <code>lvl_playground</code>；位置使用 Celeste 地图中的绝对像素坐标。</p>
        <label className="start-field">
          <span>开始房间</span>
          <input autoFocus name="room" value={draftRoom} onChange={(event) => setDraftRoom(event.target.value)} placeholder="playground" disabled={busy} />
        </label>
        <div className="start-position">
          <label className="start-field">
            <span>X</span>
            <input name="start-x" type="number" step="any" value={x} onChange={(event) => setX(event.target.value)} disabled={busy} />
          </label>
          <label className="start-field">
            <span>Y</span>
            <input name="start-y" type="number" step="any" value={y} onChange={(event) => setY(event.target.value)} disabled={busy} />
          </label>
        </div>
        {error && <p className="start-error" role="alert">{error}</p>}
        <div className="modal-actions">
          <button type="button" disabled={busy} onClick={onClose}>取消</button>
          <button className="primary" type="submit" disabled={busy}>{busy ? '正在加载房间…' : '应用并回到 F0'}</button>
        </div>
      </form>
    </section>
  </div>
}
