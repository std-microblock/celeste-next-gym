import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { compareTraces, validateTrace } from "./trace.js";
import { recordGame } from "./game.js";

const [command, ...args] = process.argv.slice(2);

if (command === "compare") {
  const [actualPath, expectedPath, toleranceRaw] = args;
  if (!actualPath || !expectedPath) {
    throw new Error(
      "usage: npm run compare -- <actual.trace.json> <expected.trace.json> [tolerance]",
    );
  }
  const actual = loadTrace(actualPath);
  const expected = loadTrace(expectedPath);
  const result = compareTraces(
    actual,
    expected,
    toleranceRaw === undefined ? undefined : Number(toleranceRaw),
  );
  console.log(JSON.stringify(result, null, 2));
  if (!result.matched) process.exitCode = 1;
} else if (command === "game") {
  const maxFramesArgument = args.find((argument) =>
    argument.startsWith("--max-frames="),
  );
  const maxFrames = maxFramesArgument
    ? Number(maxFramesArgument.slice("--max-frames=".length))
    : undefined;
  await recordGame(maxFrames === undefined ? {} : { maxFrames });
} else {
  throw new Error("usage: cli.ts <compare|game>");
}

function loadTrace(path: string) {
  return validateTrace(JSON.parse(readFileSync(resolve(path), "utf8")));
}
