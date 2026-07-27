# Celeste Next Gym agent instructions

- Treat the annotated upstream Celeste player implementation as the primary source of truth for player states, constants, coroutine timing, collision order, and comments:
  <https://raw.githubusercontent.com/NoelFB/Celeste/refs/heads/master/Source/Player/Player.cs>
- Before implementing or changing a mechanic, locate and follow its corresponding `Player.cs` state callbacks and related entity source. Use recorded Everest traces to verify frame ordering and floating-point behavior, not to invent the core logic.
- Real Celeste/Everest E2E comparisons must cover position, speed, state, facing, dashes, stamina, grounded, ducking, and death with tolerance at most `0.01`.
- Treat `docs/tech-coverage.md` as the pinned authoritative product coverage list for player techniques. It is a local snapshot of <https://celeste.ink/wiki/Tech> last revised on 2026-05-28; do not revisit or silently refresh the online Wiki unless the user explicitly requests it. Every listed technique must have an explicit implementation verdict plus a real Celeste/Everest E2E scenario before claiming 100% coverage.
- `FinalBoss` and the player `Attract` state are intentionally unsupported product exclusions. Do not count them in the Tech coverage denominator and do not implement them unless the user explicitly reverses this decision.
- When using PowerShell to read or write files, account for encoding differences. Most repository text is UTF-8 while older PowerShell defaults can be UTF-16. Prefer `apply_patch` for text edits and avoid PowerShell text-writing commands.
