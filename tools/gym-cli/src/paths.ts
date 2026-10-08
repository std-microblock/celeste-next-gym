import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** Repository root (`tools/gym-cli/src` → `../../..`). */
export const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
/** Static web assets (atlases, themes) served at `/` by the web app. */
export const WEB_PUBLIC = resolve(REPO_ROOT, "web", "public");
/** Vanilla Celeste maps of the repository-owned game install. */
export const VANILLA_MAPS = resolve(REPO_ROOT, "vendor", "celeste-game", "Content", "Maps");
/** Bundled Mechanics Playground map. */
export const PLAYGROUND_BIN = resolve(
  WEB_PUBLIC,
  "assets",
  "original",
  "maps",
  "CelesteGymPlayground-Playground.bin",
);
/** CelesteTAS `.tas` flattener shared with the fidelity tooling. */
export const TAS_RESOLVER = resolve(REPO_ROOT, "tools", "tas-replay", "src", "resolve.mjs");
