import type { SimState } from '../model'

function number(value: number): string {
  return Number(value).toFixed(2)
}

function BoolLight({ value }: { value: boolean }) {
  return <span className={value ? 'bool on' : 'bool'}>{value ? 'YES' : 'NO'}</span>
}

export function StateInspector({ frame, state, exact }: { frame: number; state: SimState; exact: boolean }) {
  return <section className="inspector panel-frame">
    <div className="panel-heading">
      <div><small>FRAME SNAPSHOT</small><h2>逐帧状态</h2></div>
      <span className={exact ? 'cache-pill valid' : 'cache-pill'}>{exact ? 'CACHED' : 'STALE'}</span>
    </div>
    <div className="frame-number">{String(frame).padStart(4, '0')}</div>
    <dl className="state-list">
      <div><dt>Position</dt><dd>{number(state.pos.x)}<i>/</i>{number(state.pos.y)}</dd></div>
      <div><dt>Speed</dt><dd>{number(state.speed.x)}<i>/</i>{number(state.speed.y)}</dd></div>
      <div><dt>State</dt><dd className="state-name">{state.state}</dd></div>
      <div><dt>Facing</dt><dd>{state.facing ? 'RIGHT' : 'LEFT'}</dd></div>
      <div><dt>Dashes</dt><dd>{state.dashes}</dd></div>
      <div><dt>Stamina</dt><dd>{number(state.stamina)}</dd></div>
      <div><dt>Grounded</dt><dd><BoolLight value={state.on_ground} /></dd></div>
      <div><dt>Ducking</dt><dd><BoolLight value={state.ducking} /></dd></div>
      <div><dt>Death</dt><dd><BoolLight value={state.dead} /></dd></div>
    </dl>
  </section>
}
