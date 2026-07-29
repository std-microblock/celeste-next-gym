import { runCommand } from "./commands.js";

export function compareRealTrace(options: {
  readonly repoRoot: string;
  readonly tracePath: string;
  readonly mapPath: string;
  readonly room?: string;
}): void {
  const args = [
    "run",
    "-q",
    "-p",
    "celeste-physics",
    "--example",
    "compare_real_trace",
    "--",
    options.tracePath,
    options.mapPath,
  ];
  if (options.room) args.push(options.room);
  runCommand("cargo", args, options.repoRoot);
}
