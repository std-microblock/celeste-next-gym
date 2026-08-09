import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tsxApi = resolve(
  repoRoot,
  "services",
  "collector",
  "node_modules",
  "tsx",
  "dist",
  "esm",
  "api",
  "index.mjs",
);
const { register } = await import(pathToFileURL(tsxApi).href);
register();
const module = await import(
  pathToFileURL(resolve(repoRoot, "scripts", "gym-actors.ts")).href
);
await module.runActorLauncher();
