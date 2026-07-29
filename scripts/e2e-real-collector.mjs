import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tsxCli = resolve(
  repoRoot,
  "services",
  "collector",
  "node_modules",
  "tsx",
  "dist",
  "cli.mjs",
);
const entry = resolve(repoRoot, "scripts", "e2e-real", "run.ts");
const result = spawnSync(
  process.execPath,
  [tsxCli, entry, ...process.argv.slice(2)],
  {
    cwd: repoRoot,
    stdio: "inherit",
    shell: false,
  },
);
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
