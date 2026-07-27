import { useEffect, useState } from 'react'
import { ACTIONS, ACTION_GLYPHS, ACTION_LABELS, bindingLabel, type Action, type KeyBindings as Bindings } from '../model'

export function KeyBindings({ bindings, onChange, onClose }: { bindings: Bindings; onChange(action: Action, code: string): void; onClose(): void }) {
  const [listening, setListening] = useState<Action | null>(null)

  useEffect(() => {
    if (!listening) return
    const capture = (event: KeyboardEvent) => {
      event.preventDefault()
      event.stopPropagation()
      if (event.code === 'Escape') setListening(null)
      else {
        onChange(listening, event.code)
        setListening(null)
      }
    }
    window.addEventListener('keydown', capture, true)
    return () => window.removeEventListener('keydown', capture, true)
  }, [listening, onChange])

  return <div className="modal-backdrop" onPointerDown={(event) => event.target === event.currentTarget && onClose()}>
    <section className="bindings-modal panel-frame" role="dialog" aria-modal="true" aria-labelledby="bindings-title">
      <div className="panel-heading">
        <div><small>CONTROLS</small><h2 id="bindings-title">按键绑定</h2></div>
        <button className="icon-button" onClick={onClose} aria-label="关闭">×</button>
      </div>
      <p className="modal-copy">点击一个键位，再按下新按键。重复键位会自动交换。</p>
      <div className="binding-list">
        {ACTIONS.map((action) => <button key={action} className={listening === action ? 'binding listening' : 'binding'} onClick={() => setListening(action)}>
          <span className={`action-glyph ${action}`}>{ACTION_GLYPHS[action]}</span>
          <strong>{ACTION_LABELS[action]}</strong>
          <kbd>{listening === action ? '按下按键…' : bindingLabel(bindings[action])}</kbd>
        </button>)}
      </div>
    </section>
  </div>
}
