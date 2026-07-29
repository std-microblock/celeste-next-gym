import { randomUUID } from "node:crypto";
import { mkdirSync, renameSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

import type { GameInstall, GitIdentity, RunContext } from "../types.js";

export function createRunContext(options: {
  readonly repoRoot: string;
  readonly gameInstall: GameInstall;
  readonly modPort: number;
  readonly httpPort: number;
  readonly git: GitIdentity;
}): RunContext {
  const runId = `${new Date().toISOString().replaceAll(":", "-")}-${process.pid}-${randomUUID()}`;
  const runNonce = randomUUID();
  const runRoot = resolve(options.repoRoot, ".tmp", "e2e-runs", runId);
  const saveRoot = resolve(runRoot, "userdata");
  const tempRoot = resolve(runRoot, "tmp");
  mkdirSync(saveRoot, { recursive: true });
  mkdirSync(tempRoot, { recursive: true });
  const context: RunContext = {
    runId,
    runNonce,
    runRoot,
    saveRoot,
    tempRoot,
    manifestPath: resolve(runRoot, "manifest.json"),
    manifest: {
      version: 1,
      run_id: runId,
      run_nonce: runNonce,
      status: "preparing",
      created_at: new Date().toISOString(),
      launcher_pid: process.pid,
      git: options.git,
      game: {
        root: options.gameInstall.gameRoot,
        executable: options.gameInstall.executable,
      },
      ports: { mod: options.modPort, http: options.httpPort },
      isolation: { save_root: saveRoot, temp_root: tempRoot },
    },
  };
  updateRunManifest(context);
  return context;
}

export function updateRunManifest(
  context: RunContext,
  patch: Readonly<Record<string, unknown>> = {},
): void {
  context.manifest = {
    ...context.manifest,
    ...patch,
    updated_at: new Date().toISOString(),
  };
  const temporaryPath = `${context.manifestPath}.tmp`;
  writeFileSync(
    temporaryPath,
    `${JSON.stringify(context.manifest, null, 2)}\n`,
    "utf8",
  );
  renameSync(temporaryPath, context.manifestPath);
}
