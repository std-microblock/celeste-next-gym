/**
 * Public TypeScript bindings of the Celeste Next Gym CLI.
 *
 * Import from scripts run with `celeste-gym run script.ts` (the API object is
 * also passed to the script's default export), or directly by path:
 *   import { openMap, renderTrace } from "<repo>/tools/gym-cli/src/index.ts";
 */
export { openMap, MapSession, Trace, renderTrace, renderState, toInputs, compactState } from "./gym.ts";
export type { InputsLike, StartOptions, TraceRenderOptions, TraceSummary } from "./gym.ts";
export { openMapSource, listZipMaps, findVanillaMap, loadRoom, sourceRooms } from "./maps.ts";
export { checkMap } from "./check.ts";
export {
  parseTasLines,
  resolveTasFile,
  tasFramesToInputs,
  buttonsToInputs,
  inputsFromJson,
  inputsFromFile,
  inputsToTas,
  describeInput,
} from "./inputs.ts";
export { SceneRenderer, chooseTheme, listThemes, contactSheet, writePng } from "./render.ts";
export { defineFuzz } from "./fuzzTypes.ts";
export type * from "./fuzzTypes.ts";
export type * from "./types.ts";
export {
  loadWasm,
  listRooms,
  auditMap,
  decodeRoom,
  simulateRaw,
  fuzzRaw,
  fuzzResolveInputs,
  checkState,
} from "./wasm.ts";
