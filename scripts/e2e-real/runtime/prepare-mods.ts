import {
  copyFileSync,
  existsSync,
  lstatSync,
  mkdirSync,
  realpathSync,
  rmSync,
} from "node:fs";
import { dirname, resolve } from "node:path";

import type { GameInstall } from "../types.js";
import { comparablePath } from "../isolation/game-install.js";
import { runCommand } from "./commands.js";

export function playgroundAssemblyPath(playgroundModRoot: string): string {
  return resolve(
    playgroundModRoot,
    "Source",
    "bin",
    "Release",
    "net8.0",
    "CelesteGymPlayground.dll",
  );
}

export function stagePlaygroundAssembly(
  sourceModRoot: string,
  stagedModRoot: string,
): void {
  const codeRoot = resolve(stagedModRoot, "Code");
  mkdirSync(codeRoot, { recursive: true });
  copyFileSync(
    playgroundAssemblyPath(sourceModRoot),
    resolve(codeRoot, "CelesteGymPlayground.dll"),
  );
}

export interface HarnessPaths {
  readonly repoRoot: string;
  readonly gameRoot: string;
  readonly modRoot: string;
  readonly playgroundModRoot: string;
  readonly serviceRoot: string;
}

export function createHarnessPaths(repoRoot: string): HarnessPaths {
  return Object.freeze({
    repoRoot,
    gameRoot: resolve(repoRoot, "vendor", "celeste-game"),
    modRoot: resolve(repoRoot, "mods", "CelesteGymCollector"),
    playgroundModRoot: resolve(repoRoot, "mods", "CelesteGymPlayground"),
    serviceRoot: resolve(repoRoot, "services", "collector"),
  });
}

export function prepareMods(
  paths: HarnessPaths,
  gameInstall: GameInstall,
  playgroundModRoot: string = paths.playgroundModRoot,
): void {
  runCommand(
    "dotnet",
    [
      "build",
      resolve(
        paths.playgroundModRoot,
        "Source",
        "CelesteGymPlayground.csproj",
      ),
      "-c",
      "Release",
      `-p:CelesteRoot=${paths.gameRoot}`,
    ],
    paths.repoRoot,
  );
  stagePlaygroundAssembly(paths.playgroundModRoot, playgroundModRoot);
  runCommand(
    "dotnet",
    [
      "build",
      resolve(paths.modRoot, "Source", "CelesteGymCollector.csproj"),
      "-c",
      "Release",
    ],
    paths.repoRoot,
  );
  const gameModsRoot = resolve(gameInstall.gameRoot, "Mods");
  mkdirSync(gameModsRoot, { recursive: true });
  if (
    lstatSync(gameModsRoot).isSymbolicLink() ||
    comparablePath(realpathSync(gameModsRoot)) !== comparablePath(gameModsRoot)
  ) {
    throw new Error(
      `refusing to use a linked game Mods directory: ${gameModsRoot}`,
    );
  }
  const installedMod = resolve(gameModsRoot, "CelesteGymCollector");
  assertUnlinkedTarget(installedMod, gameModsRoot);
  const installedCode = resolve(installedMod, "Code");
  const installedManifest = resolve(installedMod, "everest.yaml");
  const installedAssembly = resolve(installedCode, "CelesteGymCollector.dll");
  assertUnlinkedTarget(installedCode, installedMod);
  assertUnlinkedTarget(installedManifest, installedMod);
  assertUnlinkedTarget(installedAssembly, installedCode);
  mkdirSync(installedCode, { recursive: true });
  copyFileSync(resolve(paths.modRoot, "everest.yaml"), installedManifest);
  copyFileSync(
    resolve(
      paths.modRoot,
      "Source",
      "bin",
      "Release",
      "net8.0",
      "CelesteGymCollector.dll",
    ),
    installedAssembly,
  );

  const installedPlaygroundMod = resolve(gameModsRoot, "CelesteGymPlayground");
  const installedPlaygroundZip = resolve(
    gameModsRoot,
    "CelesteGymPlayground.zip",
  );
  if (
    dirname(installedPlaygroundMod) !== gameModsRoot ||
    dirname(installedPlaygroundZip) !== gameModsRoot
  ) {
    throw new Error(
      "refusing to replace a playground mod outside the game Mods directory",
    );
  }
  assertUnlinkedTarget(installedPlaygroundMod, gameModsRoot);
  assertUnlinkedTarget(installedPlaygroundZip, gameModsRoot);
  removeValidatedTarget(installedPlaygroundMod, gameModsRoot);
  removeValidatedTarget(installedPlaygroundZip, gameModsRoot);
  runCommand(
    "7z",
    [
      "a",
      "-tzip",
      "-mx=0",
      installedPlaygroundZip,
      "everest.yaml",
      "Maps",
      "Code",
    ],
    playgroundModRoot,
  );
  runCommand(
    process.execPath,
    [
      resolve(paths.serviceRoot, "node_modules", "typescript", "bin", "tsc"),
      "-p",
      "tsconfig.json",
    ],
    paths.serviceRoot,
  );
}

export function removeValidatedTarget(
  target: string,
  expectedParent: string,
  dependencies: {
    readonly remove?: typeof rmSync;
    readonly wait?: (milliseconds: number) => void;
  } = {},
): void {
  assertUnlinkedTarget(target, expectedParent);
  const remove = dependencies.remove ?? rmSync;
  const wait =
    dependencies.wait ??
    ((milliseconds: number) => {
      Atomics.wait(
        new Int32Array(new SharedArrayBuffer(4)),
        0,
        0,
        milliseconds,
      );
    });
  for (let attempt = 0; ; attempt++) {
    try {
      remove(target, { recursive: true, force: true });
      return;
    } catch (error) {
      const code =
        error instanceof Error && "code" in error ? String(error.code) : "";
      if ((code !== "EPERM" && code !== "EBUSY") || attempt >= 49) throw error;
      wait(100);
    }
  }
}

function assertUnlinkedTarget(target: string, expectedParent: string): void {
  if (dirname(target) !== expectedParent)
    throw new Error(`refusing to modify a path outside game Mods: ${target}`);
  if (existsSync(target) && lstatSync(target).isSymbolicLink()) {
    throw new Error(`refusing to modify a linked path in game Mods: ${target}`);
  }
}
