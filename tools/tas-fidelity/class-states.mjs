// Decompose one reason-set class of a fidelity report by the *state pair* it diverges in.
//
// A class like `speed+state` loses its shape when it is read as a list: on the 202 trace its 41
// segments are 28 Badeline-boss `StAttract` frames (an intentional product exclusion, see
// AGENTS.md), 8 Core C-side `StLaunch` frames, and three small tails. Grouping by the diverging
// state pair is what separates "a mechanic the simulator does not model" from "a mechanic it is
// not supposed to model".
//
// Usage:
//   node tools/tas-fidelity/class-states.mjs <report.json> <reasons> [anchorState]
//
// `reasons` is the class's `+`-joined reason set as the work list prints it, e.g. `speed+state`;
// `anchorState` defaults to `StNormal` and matches the simulator's state at the divergence.
import { readJsonSmall } from './lib/guard.mjs';

const [reportPath, wantReasons, wantAnchor = 'StNormal'] = process.argv.slice(2);
if (!reportPath || !wantReasons) {
  console.error('usage: node tools/tas-fidelity/class-states.mjs <report.json> <reasons> [anchor]');
  process.exit(2);
}

const report = readJsonSmall(reportPath);
const groups = new Map();
let total = 0;
let segments = 0;
for (const segment of report.segments) {
  if (segment.status !== 'mismatch') continue;
  const mismatch = segment.firstMismatch ?? {};
  const reasons = (mismatch.reasons ?? []).slice().sort().join('+');
  if (reasons !== wantReasons) continue;
  if (mismatch.rust?.state !== wantAnchor) continue;
  segments += 1;
  total += segment.frames;
  const key = `game=${mismatch.game?.state} rust=${mismatch.rust?.state}`;
  const group = groups.get(key) ?? { segments: 0, frames: 0, areas: new Map() };
  group.segments += 1;
  group.frames += segment.frames;
  const area = segment.sid.replace('Celeste/', '').split('|')[0];
  group.areas.set(area, (group.areas.get(area) ?? 0) + 1);
  groups.set(key, group);
}

console.log(`${wantReasons} | anchor=${wantAnchor}: ${segments} segments / ${total} frames`);
for (const [key, group] of [...groups.entries()].sort((a, b) => b[1].frames - a[1].frames)) {
  const areas = [...group.areas.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, 4)
    .map(([name, count]) => `${name} x${count}`)
    .join(', ');
  console.log(`${key}\tsegments=${group.segments}\tframes=${group.frames}\tareas=${areas}`);
}
