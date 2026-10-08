// Groups the gate's remaining mismatches into divergence classes and writes a work list.
// Streams the report: never whole-file JSON.parse.
import fs from 'node:fs';
import { forEachJsonArrayItem } from './guard.mjs';

const report = process.argv[2];
const out = process.argv[3];

const groups = new Map();
await forEachJsonArrayItem(report, 'segments', (s) => {
  if (s.status !== 'mismatch') return;
  const m = s.firstMismatch || {};
  const reasons = (m.reasons || []).slice().sort().join('+');
  const anchor = (m.rust && m.rust.state) || s.unsupportedState || '?';
  const key = `${reasons} | anchor=${anchor}`;
  if (!groups.has(key)) {
    groups.set(key, { n: 0, frames: 0, exact: 0, rooms: new Set(), deltas: new Map(), stalled: 0 });
  }
  const e = groups.get(key);
  e.n++;
  e.frames += s.frames;
  e.exact += s.exactPrefixFrames;
  e.stalled += s.stalledFrames || 0;
  e.rooms.add(`${s.sid}|${s.mode}|${s.room}`);
  if (m.rust && m.game && Array.isArray(m.rust.pos) && Array.isArray(m.game.pos)) {
    const dx = m.rust.pos[0] - m.game.pos[0];
    const dy = m.rust.pos[1] - m.game.pos[1];
    const k = `d=(${dx.toFixed(0)},${dy.toFixed(0)})`;
    e.deltas.set(k, (e.deltas.get(k) || 0) + 1);
  }
});

const list = [...groups.entries()]
  .map(([k, v]) => ({ k, ...v }))
  .sort((a, b) => b.frames - a.frames);

const lines = [];
lines.push('# Remaining divergence classes');
lines.push('');
lines.push(`Source report: \`${report}\``);
lines.push('');
lines.push('| class (reasons \\| anchor state) | segments | replayed frames | exact | stalled | rooms | top pos deltas (rust - game) |');
lines.push('| --- | ---: | ---: | ---: | ---: | ---: | --- |');
for (const e of list) {
  const d = [...e.deltas.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, 4)
    .map(([k, v]) => `${k} x${v}`)
    .join(' · ');
  lines.push(`| \`${e.k}\` | ${e.n} | ${e.frames} | ${e.exact} | ${e.stalled} | ${e.rooms.size} | ${d} |`);
}
lines.push('');
lines.push('Read this as a work queue, biggest first. Pick a class, pull its members with');
lines.push('`forEachJsonArrayItem(report, \'segments\', visit)` filtered on `status === \'mismatch\'`,');
lines.push('then group them by recovered per-frame move surplus');
lines.push('(`T_axis = dpos_axis + movementCounter_after - movementCounter_before`) rather than guessing');
lines.push('per segment. A surplus of `amount * dt` is a missing or extra fractional `Move`; an integer');
lines.push('surplus is an exact move or a rounding boundary. The gate does not compare `movementCounter`,');
lines.push('so sub-pixel remainder drift stays invisible until it flips a `Math.Round` step - that is the');
lines.push('shape of most entries above, and any fix must follow `Player.cs`, not just move the pixel.');

const text = lines.join('\n') + '\n';
fs.writeFileSync(out, text);
console.log(text.split('\n').slice(0, 22).join('\n'));
console.log(`\n(${list.length} classes written to ${out})`);
