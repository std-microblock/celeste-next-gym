import { createHash } from "node:crypto";
import { copyFileSync, readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const fixture = resolve(root, "fixtures", "e2e", "playground.map.fixture.json");
const bundledFixture = resolve(
  root,
  "interactive-recorder",
  "maps",
  "playground.map.fixture.json",
);
const outputs = [
  resolve(
    root,
    "mods",
    "CelesteGymPlayground",
    "Maps",
    "CelesteGymPlayground",
    "Playground.bin",
  ),
  resolve(
    root,
    "web",
    "public",
    "assets",
    "original",
    "maps",
    "CelesteGymPlayground-Playground.bin",
  ),
  resolve(
    root,
    "interactive-recorder",
    "maps",
    "CelesteGymPlayground",
    "Playground.bin",
  ),
];
const check = process.argv.slice(2).includes("--check");
const unknown = process.argv
  .slice(2)
  .filter((argument) => argument !== "--check");
if (unknown.length > 0)
  throw new Error(`unknown arguments: ${unknown.join(", ")}`);

const args = [
  "run",
  "-q",
  "-p",
  "celeste-physics",
  "--example",
  "compile_map_fixture",
  "--",
  ...(check ? ["--check"] : []),
  ...(check ? [] : ["--canonicalize"]),
  "--legacy-playground",
  fixture,
  ...outputs,
];
const result = spawnSync("cargo", args, {
  cwd: root,
  encoding: "utf8",
  shell: false,
  stdio: ["ignore", "pipe", "inherit"],
});
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
copyFileSync(fixture, bundledFixture);

const hashes = outputs.map((path) =>
  createHash("sha256").update(readFileSync(path)).digest("hex"),
);
if (new Set(hashes).size !== 1)
  throw new Error(`generated map mirrors differ: ${hashes.join(", ")}`);
const fixtureHashes = [fixture, bundledFixture].map((path) =>
  createHash("sha256").update(readFileSync(path)).digest("hex"),
);
if (new Set(fixtureHashes).size !== 1)
  throw new Error(
    `playground fixture mirrors differ: ${fixtureHashes.join(", ")}`,
  );
process.stdout.write(result.stdout);
console.log(`sha256=${hashes[0]}`);
