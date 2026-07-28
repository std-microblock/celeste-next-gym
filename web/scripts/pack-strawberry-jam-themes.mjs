import { mkdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import sharp from 'sharp'

const source = process.argv[2]
const destination = process.argv[3]

if (!source || !destination) {
  console.error('usage: node scripts/pack-strawberry-jam-themes.mjs <extracted StrawberryJam2021 root> <output>')
  process.exit(2)
}

const themeAssets = [
  ['sj/tilesets/gym/beginner', 'tilesets/SJ2021/Gym/BeginnerGym.png'],
  ['sj/tilesets/gym/intermediate', 'tilesets/SJ2021/Gym/IntermediateGym.png'],
  ['sj/tilesets/gym/advanced', 'tilesets/SJ2021/Gym/AdvancedGym.png'],
  ['sj/tilesets/gym/expert', 'tilesets/SJ2021/Gym/ExpertGym.png'],
  ['sj/tilesets/gym/grandmaster', 'tilesets/SJ2021/Gym/GrandmasterGym.png'],
  ['sj/bgs/gym/beginner-dark', 'bgs/SJ2021/Gym/begGymDarkBG.png'],
  ['sj/bgs/gym/intermediate-dark', 'bgs/SJ2021/Gym/intGymDarkBG.png'],
  ['sj/bgs/gym/advanced-dark', 'bgs/SJ2021/Gym/advGymDarkBG.png'],
  ['sj/bgs/gym/expert-dark', 'bgs/SJ2021/Gym/expGymDarkBG.png'],
  ['sj/bgs/gym/grandmaster-dark', 'bgs/SJ2021/Gym/gmGymDarkBG.png'],
  ['sj/tilesets/lobby/beginner-cliff', 'tilesets/SJ2021/BeginnerLobby/lobbyCliff.png'],
  ['sj/bgs/lobby/beginner/sky', 'bgs/SJ2021/BeginnerLobby/main/sky.png'],
  ['sj/bgs/lobby/beginner/clouds', 'bgs/SJ2021/BeginnerLobby/main/clouds.png'],
  ['sj/bgs/lobby/beginner/islands', 'bgs/SJ2021/BeginnerLobby/main/islands.png'],
  ['sj/tilesets/lobby/intermediate-girder', 'tilesets/SJ2021/Int_Lobby/IntGirderFg.png'],
  ['sj/bgs/lobby/intermediate/skybox', 'bgs/SJ2021/Int_Lobby/skybox.png'],
  ['sj/bgs/lobby/intermediate/hills', 'bgs/SJ2021/Int_Lobby/bghills.png'],
  ['sj/bgs/lobby/intermediate/foreground-hills', 'bgs/SJ2021/Int_Lobby/fghills.png'],
  ['sj/tilesets/lobby/advanced-cloud', 'tilesets/SJ2021/Advanced_Lobby/advCloudSunset.png'],
  ['sj/bgs/lobby/advanced/sunset', 'bgs/SJ2021/Advanced Lobby/sunset/sunset.png'],
  ['sj/bgs/lobby/advanced/mountains', 'bgs/SJ2021/Advanced Lobby/sunset/sunsetmountains.png'],
  ['sj/bgs/lobby/advanced/dunes', 'bgs/SJ2021/Advanced Lobby/sunset/sunsetdunes.png'],
  ['sj/tilesets/lobby/expert-vegetation', 'tilesets/SJ2021/ExpertLobby/spaceVegetation.png'],
  ['sj/bgs/lobby/expert/space', 'bgs/SJ2021/ExpertLobby/space.png'],
  ['sj/bgs/lobby/expert/nebulae', 'bgs/SJ2021/ExpertLobby/nebulae.png'],
  ['sj/bgs/lobby/expert/planets', 'bgs/SJ2021/ExpertLobby/planets.png'],
  ['sj/tilesets/lobby/grandmaster-grass', 'tilesets/SJ2021/Grandmaster/elysianGrass.png'],
  ['sj/bgs/lobby/grandmaster/sky', 'bgs/SJ2021/GMLobby/sky.png'],
  ['sj/bgs/lobby/grandmaster/mountains', 'bgs/SJ2021/GMLobby/mountains.png'],
  ['sj/bgs/lobby/grandmaster/clouds', 'bgs/SJ2021/GMLobby/cloud-group-1.png'],
]

const gameplayRoot = path.join(source, 'Graphics', 'Atlases', 'Gameplay')
const sheetWidth = 1024
const padding = 2
let cursorX = padding
let cursorY = padding
let rowHeight = 0
const placements = []

for (const [key, relativePath] of themeAssets) {
  const input = path.join(gameplayRoot, ...relativePath.split('/'))
  const metadata = await sharp(input).metadata()
  if (!metadata.width || !metadata.height) throw new Error(`Could not read dimensions for ${relativePath}`)
  if (cursorX + metadata.width + padding > sheetWidth) {
    cursorX = padding
    cursorY += rowHeight + padding
    rowHeight = 0
  }
  placements.push({ key, relativePath, input, x: cursorX, y: cursorY, width: metadata.width, height: metadata.height })
  cursorX += metadata.width + padding
  rowHeight = Math.max(rowHeight, metadata.height)
}

const sheetHeight = cursorY + rowHeight + padding
await mkdir(destination, { recursive: true })
await sharp({ create: { width: sheetWidth, height: sheetHeight, channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0 } } })
  .composite(await Promise.all(placements.map(async (entry) => ({
    input: await readFile(entry.input),
    left: entry.x,
    top: entry.y,
  }))))
  .png({ compressionLevel: 9 })
  .toFile(path.join(destination, 'theme-selected.png'))

const entries = Object.fromEntries(placements.map((entry) => [entry.key, {
  x: entry.x,
  y: entry.y,
  width: entry.width,
  height: entry.height,
  drawOffsetX: 0,
  drawOffsetY: 0,
  frameWidth: entry.width,
  frameHeight: entry.height,
}]))
const sources = Object.fromEntries(placements.map((entry) => [entry.key, `Graphics/Atlases/Gameplay/${entry.relativePath}`]))
await writeFile(path.join(destination, 'theme-selected.json'), `${JSON.stringify({ entries, sources }, null, 2)}\n`, 'utf8')
console.log(`Packed ${placements.length} Strawberry Jam theme textures into ${destination}`)
