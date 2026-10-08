# celeste-gym CLI

Agent-oriented command line and TypeScript bindings over the `celeste-physics`
WASM simulator: simulate inputs, replay CelesteTAS files, run the Fuzz searcher
(JSON or `.ts` specs), check whether `.bin` / Everest mod `.zip` maps load, and
render to PNG / contact sheets / GIF / MP4-WebM with the web app's own renderer
(`web/src/render/gameRenderer.ts`, running on `@napi-rs/canvas`).

```powershell
cd tools/gym-cli; npm install           # once
node scripts/build-wasm.mjs             # from repo root, when crates/ changed or `info` says so
node tools/gym-cli/bin/celeste-gym.mjs info
node tools/gym-cli/bin/celeste-gym.mjs help
```

```powershell
# simulate TAS-style inputs and look at them
node tools/gym-cli/bin/celeste-gym.mjs sim vanilla:1 --room 1 --input "20,R;1,R,J;11,R,J;20,R" --table --render .tmp/run.png --sheet 8 --hud
# replay a TAS room segment to video
node tools/gym-cli/bin/celeste-gym.mjs tas path/to/1A.tas --from-label lvl_2 --to-label lvl_3 --render .tmp/1a-2.mp4 --hud
# fuzz (TypeScript spec runs directly) and render the best candidate
node tools/gym-cli/bin/celeste-gym.mjs fuzz tools/gym-cli/examples/longest-jump.fuzz.ts --top 3 --render .tmp/best.gif --camera room --path
# is this mod map loadable?
node tools/gym-cli/bin/celeste-gym.mjs check vendor/celeste-game/Mods/SomeMod.zip --all-maps
# script the bindings
node tools/gym-cli/bin/celeste-gym.mjs run tools/gym-cli/examples/script-dash-sweep.ts .tmp/sweep
```

Full reference (commands, input syntax, fuzz spec, rendering options, caveats):
[`.agents/skills/celeste-gym-cli/SKILL.md`](../../.agents/skills/celeste-gym-cli/SKILL.md).

Layout: `src/wasm.ts` (WASM binding), `src/maps.ts` (map sources), `src/inputs.ts`
(TAS/JSON inputs, uses `tools/tas-replay`), `src/gym.ts` (MapSession / Trace /
renderTrace), `src/render.ts` (napi canvas backend + encoders), `src/check.ts`,
`src/cli.ts`. Tests: `npm test`; types: `npm run typecheck`.
