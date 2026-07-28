# Timeline regression fixtures

This directory stores Celeste Next Gym v2 timeline exports that reproduce
player-visible physics bugs. Keep each recording unchanged so it can still be
loaded in the web UI for manual inspection.

- Use a short kebab-case mechanic name, for example `delayed-wallboost.json`.
- Add a regression test that replays every checked-in timeline.
- Assert source-backed behavior at the critical frames instead of snapshotting
  simulator output wholesale.

`delayed-wallboost.json` was recorded in the Mechanics Playground and supplied
on 2026-07-28. It reproduces an away input delayed until the last wallboost
frame.
