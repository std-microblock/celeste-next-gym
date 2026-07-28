# Timeline regression fixtures

This directory stores Celeste Next Gym v2 timeline exports that reproduce
player-visible physics bugs. Keep the exported map, initial state, inputs, and
bindings unchanged so each fixture still loads in the web UI for inspection;
repository-only regression metadata may be added at the document root.

- Use a short kebab-case mechanic name, for example `delayed-wallbounce.json`.
- Add source-backed assertions to the timeline's `regression.assertions` array.
- Set `regression.e2e_scenario` to the real Everest scenario that covers the
  same bug. `npm run e2e:timeline-regressions` discovers all JSON files and
  runs those scenarios through the owned-process E2E harness.
- The TypeScript E2E harness discovers and replays every `*.json` file in this
  directory; adding a fixture does not require editing Rust test code.
- Assert only behavior at critical frames instead of snapshotting simulator
  output wholesale. Supported operators are `eq`, `near`, `gt`, `gte`, `lt`,
  and `lte`; `near` also requires a positive `tolerance`.

`delayed-wallbounce.json` was recorded in the Mechanics Playground and supplied
on 2026-07-28. Its final attempt presses jump after the up-dash state has ended
but while the original game's dash-attack window is still active.
