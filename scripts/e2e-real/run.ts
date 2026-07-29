import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { main } from "./cli.js";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
await main(process.argv.slice(2), process.env, repoRoot);
