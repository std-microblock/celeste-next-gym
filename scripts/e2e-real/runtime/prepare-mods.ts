import {
  cpSync,
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

export interface HarnessPaths {
  readonly repoRoot: string;
  readonly gameRoot: string;
  readonly modRoot: string;
  readonly playgroundModRoot: string;
  readonly trainingModRoot?: string;
  readonly serviceRoot: string;
}

export function createHarnessPaths(repoRoot: string): HarnessPaths {
  return Object.freeze({
    repoRoot,
    gameRoot: resolve(repoRoot, "vendor", "celeste-game"),
    modRoot: resolve(repoRoot, "mods", "CelesteGymCollector"),
    playgroundModRoot: resolve(repoRoot, "mods", "CelesteGymPlayground"),
    trainingModRoot: resolve(repoRoot, "mods", "CelesteGymTraining"),
    serviceRoot: resolve(repoRoot, "services", "collector"),
  });
}

export function prepareTrainingMod(
  paths: HarnessPaths,
  gameInstall: GameInstall,
): void {
  const trainingModRoot = paths.trainingModRoot ?? resolve(paths.repoRoot, "mods", "CelesteGymTraining");
  runCommand(
    "dotnet",
    [
      "build",
      resolve(trainingModRoot, "Source", "CelesteGymTraining.csproj"),
      "-c",
      "Release",
    ],
    paths.repoRoot,
  );
  runCommand(
    "cargo",
    ["build", "-q", "-p", "celeste-gym-native", "--release"],
    paths.repoRoot,
  );
  const gameModsRoot = resolve(gameInstall.gameRoot, "Mods");
  mkdirSync(gameModsRoot, { recursive: true });
  if (
    lstatSync(gameModsRoot).isSymbolicLink() ||
    comparablePath(realpathSync(gameModsRoot)) !== comparablePath(gameModsRoot)
  ) {
    throw new Error(`refusing to use a linked game Mods directory: ${gameModsRoot}`);
  }
  const installed = resolve(gameModsRoot, "CelesteGymTraining");
  const installedZip = resolve(gameModsRoot, "CelesteGymTraining.zip");
  assertUnlinkedTarget(installed, gameModsRoot);
  assertUnlinkedTarget(installedZip, gameModsRoot);
  removeValidatedTarget(installed, gameModsRoot);
  removeValidatedTarget(installedZip, gameModsRoot);
  mkdirSync(resolve(installed, "Code"), { recursive: true });
  copyFileSync(
    resolve(trainingModRoot, "everest.yaml"),
    resolve(installed, "everest.yaml"),
  );
  copyFileSync(
    resolve(
      trainingModRoot,
      "Source",
      "bin",
      "Release",
      "net8.0",
      "CelesteGymTraining.dll",
    ),
    resolve(installed, "Code", "CelesteGymTraining.dll"),
  );
  copyFileSync(
    resolve(
      paths.repoRoot,
      "target",
      "release",
      process.platform === "win32" ? "celeste_gym_native.dll" : "libceleste_gym_native.so",
    ),
    resolve(
      installed,
      "Code",
      process.platform === "win32" ? "celeste_gym_native.dll" : "libceleste_gym_native.so",
    ),
  );
  for (const directory of ["Dialog", "Maps", "Graphics", "Content"]) {
    const source = resolve(trainingModRoot, directory);
    if (existsSync(source)) cpSync(source, resolve(installed, directory), { recursive: true });
  }
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
  installRandomizerArchive(gameModsRoot);

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
    ["a", "-tzip", "-mx=0", installedPlaygroundZip, "everest.yaml", "Maps"],
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

export function resolveRandomizerArchive(
  environment: NodeJS.ProcessEnv = process.env,
  fileExists: (path: string) => boolean = existsSync,
): string | null {
  const candidates = [
    environment.CELESTE_GYM_RANDOMIZER_ZIP,
    process.platform === "win32"
      ? "C:\\SteamLibrary\\steamapps\\common\\Celeste\\Mods\\Randomizer.zip"
      : undefined,
  ];
  for (const candidate of candidates) {
    if (!candidate) continue;
    const archive = resolve(candidate);
    if (fileExists(archive)) return archive;
  }
  return null;
}

function installRandomizerArchive(gameModsRoot: string): void {
  const source = resolveRandomizerArchive();
  if (source === null) return;
  const destination = resolve(gameModsRoot, "Randomizer.zip");
  assertUnlinkedTarget(destination, gameModsRoot);
  if (comparablePath(source) === comparablePath(destination)) return;
  if (!lstatSync(source).isFile()) {
    throw new Error(`Randomizer archive is not a file: ${source}`);
  }
  copyFileSync(source, destination);
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
