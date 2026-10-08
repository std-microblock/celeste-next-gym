#!/usr/bin/env node
// Entry point: register tsx so the CLI, the shared web renderer sources and
// user `.ts` fuzz specs / scripts all run as TypeScript directly.
import { register } from "tsx/esm/api";

register();
const { main } = await import("../src/cli.ts");
try {
  await main(process.argv.slice(2));
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  process.stdout.write(JSON.stringify({ ok: false, error: message }, null, 2) + "\n");
  if (process.env.CELESTE_GYM_DEBUG && error instanceof Error) process.stderr.write(`${error.stack}\n`);
  process.exitCode = 1;
}
