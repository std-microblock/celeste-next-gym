# Bundled playground map

This directory keeps the test map next to the recorder so neither the game nor
the browser needs a hand-copied map. `playground.map.fixture.json` is a checked,
byte-identical mirror of the editable fixture source. `CelesteGymPlayground/Playground.bin` is the generated
Celeste BinaryPacker file loaded by Everest and decoded directly by the WASM
browser worker.

Run `npm run maps:check` from `interactive-recorder` to verify this copy, the
Everest Mod copy, and the Web copy are byte-identical outputs of the fixture.
