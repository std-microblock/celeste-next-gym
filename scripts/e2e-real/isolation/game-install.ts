import { lstatSync, realpathSync } from 'node:fs'
import { dirname, relative, resolve, sep } from 'node:path'

import type { GameInstall } from '../types.js'

export function comparablePath(path: string): string {
  const normalized = resolve(path).replaceAll('\\', '/').replace(/\/$/, '')
  return process.platform === 'win32' ? normalized.toLowerCase() : normalized
}

function assertNoLinkedSegments(base: string, target: string): void {
  const rel = relative(base, target)
  if (rel.startsWith('..') || resolve(base, rel) !== resolve(target)) {
    throw new Error(`path escapes repository root: ${target}`)
  }
  let current = resolve(base)
  for (const segment of rel.split(sep).filter(Boolean)) {
    current = resolve(current, segment)
    if (lstatSync(current).isSymbolicLink()) {
      throw new Error(`reparse/symlink path is not allowed for the E2E game install: ${current}`)
    }
  }
}

export function validateGameInstall(options: {
  readonly repoRoot: string
  readonly gameRoot: string
  readonly steamRoots?: readonly string[]
}): GameInstall {
  const expectedRoot = resolve(options.repoRoot, 'vendor', 'celeste-game')
  if (comparablePath(options.gameRoot) !== comparablePath(expectedRoot)) {
    throw new Error(`E2E game root must be ${expectedRoot}, got ${resolve(options.gameRoot)}`)
  }
  assertNoLinkedSegments(options.repoRoot, options.gameRoot)

  const realRepoRoot = realpathSync(options.repoRoot)
  const realGameRoot = realpathSync(options.gameRoot)
  const expectedRealRoot = resolve(realRepoRoot, 'vendor', 'celeste-game')
  if (comparablePath(realGameRoot) !== comparablePath(expectedRealRoot)) {
    throw new Error(`E2E game root resolves outside the repository vendor install: ${realGameRoot}`)
  }

  const executable = resolve(options.gameRoot, 'Celeste.exe')
  assertNoLinkedSegments(options.repoRoot, executable)
  const realExecutable = realpathSync(executable)
  if (comparablePath(dirname(realExecutable)) !== comparablePath(realGameRoot)) {
    throw new Error(`Celeste executable resolves outside the isolated game root: ${realExecutable}`)
  }

  const normalizedGameRoot = comparablePath(realGameRoot)
  if (/\/steamapps\/common\/celeste(?:\/|$)/i.test(normalizedGameRoot)) {
    throw new Error(`refusing to use a Steam Celeste install for E2E: ${realGameRoot}`)
  }
  for (const steamRoot of options.steamRoots ?? []) {
    let resolvedSteamRoot: string
    try {
      resolvedSteamRoot = realpathSync(steamRoot)
    } catch {
      resolvedSteamRoot = resolve(steamRoot)
    }
    if (normalizedGameRoot === comparablePath(resolvedSteamRoot)) {
      throw new Error(`refusing to use configured Steam Celeste install for E2E: ${realGameRoot}`)
    }
  }
  return { gameRoot: realGameRoot, executable: realExecutable }
}
