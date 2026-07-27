import { copyFileSync, mkdirSync } from 'node:fs'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { dirname, resolve } from 'node:path'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const output = resolve(root, 'web', 'src', 'wasm')
mkdirSync(output, { recursive: true })
const playgroundSource = resolve(root, 'mods', 'CelesteGymPlayground', 'Maps', 'CelesteGymPlayground', 'Playground.bin')
const playgroundOutput = resolve(root, 'web', 'public', 'assets', 'original', 'maps', 'CelesteGymPlayground-Playground.bin')
mkdirSync(dirname(playgroundOutput), { recursive: true })
copyFileSync(playgroundSource, playgroundOutput)

function run(command, args) {
  const result = spawnSync(command, args, { cwd: root, stdio: 'inherit', shell: false })
  if (result.error) throw result.error
  if (result.status !== 0) process.exit(result.status ?? 1)
}

run('cargo', ['build', '-p', 'celeste-wasm', '--release', '--target', 'wasm32-unknown-unknown'])
run('wasm-bindgen', [
  '--target', 'web',
  '--out-dir', output,
  '--out-name', 'celeste_wasm',
  resolve(root, 'target', 'wasm32-unknown-unknown', 'release', 'celeste_wasm.wasm'),
])

console.log(`WASM browser bundle written to ${output}; synced ${playgroundOutput}`)
