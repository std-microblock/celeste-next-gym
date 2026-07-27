import { useEffect, useMemo, useRef, useState } from 'react'
import type { GymMap, MapEntity, SimState, Vec2 } from '../model'

interface AtlasEntry {
  x: number
  y: number
  width: number
  height: number
  drawOffsetX: number
  drawOffsetY: number
  frameWidth: number
  frameHeight: number
}

interface GameAssets {
  image: HTMLImageElement
  entries: Record<string, AtlasEntry>
  keys: string[]
  frameLists: Map<string, string[]>
  tinted: Map<string, HTMLCanvasElement>
}

interface AnimationChoice {
  prefix: string
  delay: number
  indices?: number[]
}

const HAIR_OFFSETS: Record<string, string[]> = {
  idle: ['0,-2', '0,-2', '0,-2', '0,-2', '0,-1', '0,-1', '0,-1', '0,-1', '0,-1'],
  runSlow: ['1,-2', '1,-1', '1,-1', '1,-1', '1,-3', '1,-2', '1,-1', '1,-1', '1,-1', '1,-1', '1,-3', '1,-2'],
  runFast: ['1,-2', '1,-1', '1,-1', '1,-1', '1,-3', '1,-2', '1,-1', '1,-1', '1,-1', '1,-1', '1,-3', '1,-2'],
  dash: ['2,0', '2,0', '2,0', '2,1'],
  dreamDash: ['2,0', '1,0', '0,0', '0,0', '1,1', '2,1', '2,1', '2,0', '1,0', '1,0', '1,0', '1,0', '1,0', '1,0', '1,0', '1,0', '1,0', '0,0', '1,0', '1,0', '2,1'],
  jumpSlow: ['1,-3', '1,-3', '1,-2', '0,-2'],
  jumpFast: ['1,-3', '1,-3', '1,-2', '0,-2'],
  climb: ['0,-2', '0,-2', '0,-2', '0,-2', '-1,-2', '-1,-2', '0,-1', '1,-2:1', '2,-2:2'],
  duck: ['0,3'],
  swim: ['0,-2', '0,-2', '0,-2', '0,-2', '0,-2', '0,-2', '0,-3', '0,-3', '0,-2', '0,-2', '0,-2', '0,-2', '1,-1', '1,-1', '1,0', '1,0', '1,0', '1,0'],
  startStarFly: ['0,-2', '0,-2', '0,-2', '0,-2'],
  starFly: ['0,3'],
}

let assetsPromise: Promise<GameAssets> | null = null

function loadAssets(): Promise<GameAssets> {
  if (!assetsPromise) {
    assetsPromise = Promise.all([
      fetch('/assets/original/gameplay/gameplay-selected.json').then((response) => response.json()) as Promise<{ entries: Record<string, AtlasEntry> }>,
      new Promise<HTMLImageElement>((resolve, reject) => {
        const image = new Image()
        image.src = '/assets/original/gameplay/gameplay-selected.png'
        image.onload = () => resolve(image)
        image.onerror = () => reject(new Error('原版 Gameplay 图集加载失败'))
      }),
    ]).then(([manifest, image]) => ({
      image,
      entries: manifest.entries,
      keys: Object.keys(manifest.entries),
      frameLists: new Map(),
      tinted: new Map(),
    }))
  }
  return assetsPromise
}

function frames(assets: GameAssets, prefix: string): string[] {
  const cached = assets.frameLists.get(prefix)
  if (cached) return cached
  const escaped = prefix.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const matcher = new RegExp(`^${escaped}\\d+$`)
  const found = assets.keys.filter((key) => matcher.test(key)).sort((left, right) => left.localeCompare(right, undefined, { numeric: true }))
  assets.frameLists.set(prefix, found)
  return found
}

function choosePlayerAnimation(state: SimState): AnimationChoice {
  if (state.dead) return { prefix: 'characters/player/death_h', delay: 1 }
  if (state.state === 'StarFly') {
    return state.star_fly_transforming || (state.star_fly_transform_frames ?? 0) > 0
      ? { prefix: 'characters/player/startStarFly', delay: 5 }
      : { prefix: 'characters/player/starFly', delay: 5 }
  }
  if (state.state === 'DreamDash') return { prefix: 'characters/player/dreamDash', delay: 2, indices: Array.from({ length: 13 }, (_, index) => index + 4) }
  if (state.state === 'Dash' || state.state === 'RedDash') return { prefix: 'characters/player/dash', delay: 5 }
  if (state.state === 'Swim') return { prefix: 'characters/player/swim', delay: 5, indices: [0, 1, 2, 3, 4, 5] }
  if (state.state === 'SummitLaunch' || state.state === 'Launch') return { prefix: 'characters/player/launch', delay: 4 }
  if (state.state === 'Climb') return { prefix: 'characters/player/climb', delay: Math.abs(state.speed.y) > 5 ? 3 : 99, indices: Math.abs(state.speed.y) > 5 ? [0, 1, 2, 3, 4, 5] : [0] }
  if (state.ducking) return { prefix: 'characters/player/duck', delay: 99 }
  if (!state.on_ground) {
    const prefix = Math.abs(state.speed.x) > 90 ? 'characters/player/jumpFast' : 'characters/player/jumpSlow'
    return { prefix, delay: 6, indices: state.speed.y < 0 ? [0, 1] : [2, 3] }
  }
  if (Math.abs(state.speed.x) > 20) return { prefix: Math.abs(state.speed.x) > 70 ? 'characters/player/runFast' : 'characters/player/runSlow', delay: Math.abs(state.speed.x) > 70 ? 3 : 4 }
  return { prefix: 'characters/player/idle', delay: 6 }
}

function playerFrameKey(assets: GameAssets, state: SimState, frame: number): string | undefined {
  const animation = choosePlayerAnimation(state)
  const available = frames(assets, animation.prefix)
  const indices = animation.indices ?? available.map((_, index) => index)
  const animationIndex = indices[Math.floor(frame / animation.delay) % Math.max(1, indices.length)] ?? 0
  return available[animationIndex] ?? available[0]
}

function tintedEntry(assets: GameAssets, key: string, color: string): HTMLCanvasElement | undefined {
  const entry = assets.entries[key]
  if (!entry) return undefined
  const cacheKey = `${key}:${color}`
  const cached = assets.tinted.get(cacheKey)
  if (cached) return cached
  const canvas = document.createElement('canvas')
  canvas.width = entry.width
  canvas.height = entry.height
  const context = canvas.getContext('2d')
  if (!context) return undefined
  context.drawImage(assets.image, entry.x, entry.y, entry.width, entry.height, 0, 0, entry.width, entry.height)
  context.globalCompositeOperation = 'source-in'
  context.fillStyle = color
  context.fillRect(0, 0, entry.width, entry.height)
  assets.tinted.set(cacheKey, canvas)
  return canvas
}

function dreamParticleTexture(assets: GameAssets, sourceX: number, color: string): HTMLCanvasElement | undefined {
  const entry = assets.entries['objects/dreamblock/particles']
  if (!entry) return undefined
  const cacheKey = `dream-particle:${sourceX}:${color}`
  const cached = assets.tinted.get(cacheKey)
  if (cached) return cached
  const canvas = document.createElement('canvas')
  canvas.width = 7
  canvas.height = 7
  const context = canvas.getContext('2d')
  if (!context) return undefined
  context.drawImage(
    assets.image,
    entry.x, entry.y, entry.width, entry.height,
    entry.drawOffsetX - sourceX, entry.drawOffsetY, entry.width, entry.height,
  )
  context.globalCompositeOperation = 'source-in'
  context.fillStyle = color
  context.fillRect(0, 0, 7, 7)
  assets.tinted.set(cacheKey, canvas)
  return canvas
}

function drawEntry(
  context: CanvasRenderingContext2D,
  assets: GameAssets,
  key: string | undefined,
  x: number,
  y: number,
  originX = 0,
  originY = 0,
  scaleX = 1,
  scaleY = 1,
  tint?: string,
): void {
  if (!key) return
  const entry = assets.entries[key]
  if (!entry) return
  context.save()
  context.translate(Math.round(x), Math.round(y))
  context.scale(scaleX, scaleY)
  const source = tint ? tintedEntry(assets, key, tint) : undefined
  if (source) {
    context.drawImage(source, -originX + entry.drawOffsetX, -originY + entry.drawOffsetY)
  } else {
    context.drawImage(
      assets.image,
      entry.x, entry.y, entry.width, entry.height,
      -originX + entry.drawOffsetX, -originY + entry.drawOffsetY, entry.width, entry.height,
    )
  }
  context.restore()
}

function hairMetadata(key: string | undefined): { offset: Vec2; frame: number } {
  const match = key?.match(/\/([^/]+?)(\d+)$/)
  if (!match) return { offset: { x: 0, y: -2 }, frame: 0 }
  const values = HAIR_OFFSETS[match[1]]
  const raw = values?.[Number(match[2])] ?? '0,-2'
  const [position, frame = '0'] = raw.split(':')
  const [x, y] = position.split(',').map(Number)
  return { offset: { x, y }, frame: Number(frame) }
}

function approach(from: Vec2, to: Vec2, amount: number): Vec2 {
  const dx = to.x - from.x
  const dy = to.y - from.y
  const distance = Math.hypot(dx, dy)
  if (distance <= amount || distance === 0) return { ...to }
  return { x: from.x + dx / distance * amount, y: from.y + dy / distance * amount }
}

function hairRoot(state: SimState, key: string | undefined): Vec2 {
  const metadata = hairMetadata(key)
  const facing = state.facing ? 1 : -1
  return { x: state.pos.x + metadata.offset.x * facing, y: state.pos.y - 9 + metadata.offset.y }
}

function computeHairNodes(assets: GameAssets, states: readonly (SimState | undefined)[], state: SimState, frame: number): Vec2[] {
  const start = Math.max(0, frame - 180)
  let nodes: Vec2[] | undefined
  for (let value = start; value <= frame; value += 1) {
    const sample = states[value] ?? (value === frame ? state : undefined)
    if (!sample) continue
    const key = playerFrameKey(assets, sample, value)
    const root = hairRoot(sample, key)
    const facing = sample.facing ? 1 : -1
    if (!nodes) nodes = Array.from({ length: 4 }, (_, index) => ({ x: root.x - facing * index * .5, y: root.y + index * 2 }))
    nodes[0] = root
    let target = { x: root.x - facing, y: root.y + 2 }
    let previous = root
    for (let index = 1; index < 4; index += 1) {
      if (sample.state !== 'StarFly') {
        const amount = (1 - index / 4 * .5) * 64 / 60
        nodes[index] = approach(nodes[index], target, amount)
      }
      const distance = Math.hypot(nodes[index].x - previous.x, nodes[index].y - previous.y)
      if (distance > 3) nodes[index] = approach(previous, nodes[index], 3)
      target = { x: nodes[index].x - facing * .5, y: nodes[index].y + 2 }
      previous = nodes[index]
    }
  }
  return nodes ?? Array.from({ length: 4 }, (_, index) => ({ x: state.pos.x, y: state.pos.y - 11 + index * 2 }))
}

function hairColor(state: SimState): string {
  if (state.state === 'StarFly') return '#ffd65c'
  if (state.dashes === 0) return '#44b7ff'
  if (state.dashes >= 2) return '#ff6def'
  return '#ac3232'
}

function drawPlayer(context: CanvasRenderingContext2D, assets: GameAssets, states: readonly (SimState | undefined)[], state: SimState, frame: number): void {
  const key = playerFrameKey(assets, state, frame)
  const facing = state.facing ? 1 : -1
  context.save()
  context.globalAlpha = state.dead ? .75 : 1
  if (!state.dead) {
    const nodes = computeHairNodes(assets, states, state, frame)
    const metadata = hairMetadata(key)
    for (let index = 3; index >= 0; index -= 1) {
      const texture = index === 0 ? `characters/player/bangs0${metadata.frame}` : 'characters/player/hair00'
      const scale = .25 + (1 - index / 4) * .75
      const scaleX = index === 0 ? facing : scale
      const scaleY = index === 0 ? 1 : scale
      for (const [dx, dy] of [[-1, 0], [1, 0], [0, -1], [0, 1]] as const) {
        drawEntry(context, assets, texture, nodes[index].x + dx, nodes[index].y + dy, 5, 5, scaleX, scaleY, '#000000')
      }
      drawEntry(context, assets, texture, nodes[index].x, nodes[index].y, 5, 5, scaleX, scaleY, hairColor(state))
    }
  }
  drawEntry(context, assets, key, state.pos.x, state.pos.y, 16, 32, facing, 1)
  context.restore()
}

function buildSolidGrid(map: GymMap): string[][] {
  const columns = Math.ceil(map.bounds.width / 8)
  const rows = Math.ceil(map.bounds.height / 8)
  const grid = Array.from({ length: rows }, () => Array.from({ length: columns }, () => '0'))
  for (const solid of map.solids) {
    const left = Math.max(0, Math.floor((solid.x - map.bounds.x) / 8))
    const top = Math.max(0, Math.floor((solid.y - map.bounds.y) / 8))
    const right = Math.min(columns, Math.ceil((solid.x + solid.width - map.bounds.x) / 8))
    const bottom = Math.min(rows, Math.ceil((solid.y + solid.height - map.bounds.y) / 8))
    for (let y = top; y < bottom; y += 1) for (let x = left; x < right; x += 1) grid[y][x] = '1'
  }
  return grid
}

function tileCoordinate(grid: string[][], x: number, y: number): [number, number] {
  const filled = (dx: number, dy: number) => grid[y + dy]?.[x + dx] !== undefined && grid[y + dy][x + dx] !== '0'
  const top = filled(0, -1); const bottom = filled(0, 1); const left = filled(-1, 0); const right = filled(1, 0)
  const variant = Math.abs(x * 17 + y * 31) % 4
  if (!top && bottom && left && right) return [variant, 0]
  if (top && !bottom && left && right) return [variant, 1]
  if (top && bottom && !left && right) return [variant, 2]
  if (top && bottom && left && !right) return [variant, 3]
  if (!top && !bottom && left && right) return [variant, 4]
  if (top && bottom && !left && !right) return [variant, 5]
  if (!top && bottom && !left && right) return [variant, 6]
  if (!top && bottom && left && !right) return [variant, 7]
  if (!top && !bottom && !left && right) return [variant, 8]
  if (!top && !bottom && left && !right) return [variant, 9]
  if (!top && !bottom && !left && !right) return [variant, 10]
  return [5, 12]
}

function drawTiles(context: CanvasRenderingContext2D, assets: GameAssets, map: GymMap, grid: string[][]): void {
  const tileset = assets.entries['tilesets/dirt']
  if (!tileset) return
  for (let y = 0; y < grid.length; y += 1) {
    for (let x = 0; x < grid[y].length; x += 1) {
      if (grid[y][x] === '0') continue
      const [tileX, tileY] = tileCoordinate(grid, x, y)
      context.drawImage(assets.image, tileset.x + tileX * 8, tileset.y + tileY * 8, 8, 8, map.bounds.x + x * 8, map.bounds.y + y * 8, 8, 8)
    }
  }
}

function drawRepeated(context: CanvasRenderingContext2D, assets: GameAssets, key: string, entity: MapEntity, step: number): void {
  const entry = assets.entries[key]
  if (!entry) return
  const box = entity.bounds
  const horizontal = box.width >= box.height
  const length = horizontal ? box.width : box.height
  for (let offset = 0; offset < length; offset += step) {
    const x = horizontal ? box.x + offset : box.x
    const y = horizontal ? box.y : box.y + offset
    context.save()
    context.translate(x, y)
    if (entity.direction.x < 0) context.rotate(-Math.PI / 2)
    else if (entity.direction.x > 0) context.rotate(Math.PI / 2)
    else if (entity.direction.y > 0) context.rotate(Math.PI)
    context.drawImage(assets.image, entry.x, entry.y, entry.width, entry.height, 0, -entry.height, entry.width, entry.height)
    context.restore()
  }
}

function pseudo(seed: number): number {
  const value = Math.sin(seed * 12.9898 + 78.233) * 43758.5453
  return value - Math.floor(value)
}

function waterSurfaceY(box: MapEntity['bounds'], x: number, frame: number): number {
  return box.y + 8 - (6 + Math.sin(frame / 60 + (x - box.x) * .1))
}

function drawWater(context: CanvasRenderingContext2D, entity: MapEntity, frame: number): void {
  const box = entity.bounds
  context.beginPath()
  context.moveTo(box.x, waterSurfaceY(box, box.x, frame))
  for (let x = box.x + 4; x <= box.x + box.width; x += 4) context.lineTo(x, waterSurfaceY(box, x, frame))
  context.lineTo(box.x + box.width, box.y + box.height)
  context.lineTo(box.x, box.y + box.height)
  context.closePath()
  context.fillStyle = 'rgba(135, 206, 250, .30)'
  context.fill()

  const rayCount = Math.floor(box.width * .2)
  for (let index = 0; index < rayCount; index += 1) {
    const duration = 2 + pseudo(index + box.x * .01) * 6
    const percent = (frame / 60 / duration + pseudo(index * 3.7 + box.y)) % 1
    const alpha = percent < .1 ? percent / .1 : percent > .9 ? (1 - percent) / .1 : 1
    const center = pseudo(index * 7.1 + box.x) * box.width
    const width = 2 + Math.floor(pseudo(index * 11.3 + box.y) * 14)
    const length = 8 + pseudo(index * 13.7 + box.x + box.y) * 120
    const left = Math.max(0, center - width / 2)
    const right = Math.min(box.width, center + width / 2)
    const depth = Math.min(box.height, length * .7)
    const slant = length * .3
    context.beginPath()
    context.moveTo(box.x + left, waterSurfaceY(box, box.x + left, frame))
    context.lineTo(box.x + right, waterSurfaceY(box, box.x + right, frame))
    context.lineTo(box.x + right - slant, box.y + 8 + depth)
    context.lineTo(box.x + left - slant, box.y + 8 + depth)
    context.closePath()
    const topY = (waterSurfaceY(box, box.x + left, frame) + waterSurfaceY(box, box.x + right, frame)) * .5
    const bottomY = box.y + 8 + depth
    const ray = context.createLinearGradient(0, topY, 0, bottomY)
    ray.addColorStop(0, `rgba(135, 206, 250, ${Math.max(0, alpha) * .6})`)
    ray.addColorStop(1, 'rgba(135, 206, 250, 0)')
    context.fillStyle = ray
    context.fill()
  }

  context.beginPath()
  context.moveTo(box.x, waterSurfaceY(box, box.x, frame) - 1)
  for (let x = box.x + 4; x <= box.x + box.width; x += 4) context.lineTo(x, waterSurfaceY(box, x, frame) - 1)
  context.lineTo(box.x + box.width, waterSurfaceY(box, box.x + box.width, frame))
  for (let x = box.x + box.width; x >= box.x; x -= 4) context.lineTo(x, waterSurfaceY(box, x, frame))
  context.closePath()
  context.fillStyle = 'rgba(135, 206, 250, .80)'
  context.fill()
}

function drawWind(context: CanvasRenderingContext2D, assets: GameAssets, map: GymMap, state: SimState, frame: number): void {
  const wind = state.wind
  const texture = assets.entries['particles/snow']
  if (!wind || !texture || Math.hypot(wind.x, wind.y) < 1) return
  const horizontal = wind.y === 0
  const scaleX = Math.max(1, Math.abs(horizontal ? wind.x : wind.y) / (horizontal ? 100 : 40))
  const scaleY = 1 / Math.max(1, scaleX * .25)
  const rotation = horizontal ? 0 : -Math.PI / 2
  const areaScale = map.bounds.width * map.bounds.height / (640 * 360)
  const count = Math.min(600, Math.ceil(240 * areaScale * (horizontal ? 1 : .6)))
  const elapsed = frame / 60
  for (let index = 0; index < count; index += 1) {
    const frequency = .8 + pseudo(index * 3.17) * .4
    const wave = Math.sin(elapsed * frequency * Math.PI * 2 + pseudo(index * 7.91) * Math.PI * 2)
    const velocity = horizontal ? { x: wind.x + wave * 10, y: 20 } : { x: 0, y: wind.y * 3 + wave * 10 }
    const baseX = pseudo(index * 11.73 + 2) * map.bounds.width
    const baseY = pseudo(index * 17.37 + 5) * map.bounds.height
    const x = map.bounds.x + ((baseX + velocity.x * elapsed) % map.bounds.width + map.bounds.width) % map.bounds.width
    const y = map.bounds.y + ((baseY + velocity.y * elapsed) % map.bounds.height + map.bounds.height) % map.bounds.height
    context.save()
    context.globalAlpha = .85
    context.translate(Math.round(x), Math.round(y))
    context.rotate(rotation)
    context.scale(scaleX, scaleY)
    context.drawImage(
      assets.image,
      texture.x, texture.y, texture.width, texture.height,
      -texture.frameWidth / 2 + texture.drawOffsetX,
      -texture.frameHeight / 2 + texture.drawOffsetY,
      texture.width, texture.height,
    )
    context.restore()
  }
}

function lineAmplitude(seed: number, index: number): number {
  return (Math.sin(seed + index / 16 + Math.sin(seed * 2 + index / 32) * Math.PI * 2) + 1) * 1.5
}

function drawWobbleLine(context: CanvasRenderingContext2D, from: Vec2, to: Vec2, offset: number, frame: number, color: string, background: string): void {
  const length = Math.hypot(to.x - from.x, to.y - from.y)
  const along = { x: (to.x - from.x) / length, y: (to.y - from.y) / length }
  const normal = { x: along.y, y: -along.x }
  const cycle = frame / 30
  const ease = cycle - Math.floor(cycle)
  const seedA = Math.floor(cycle) * 1.873 + offset
  const seedB = seedA + 1.873
  let previousAmplitude = 0
  for (let distance = 2; distance < length - 2; distance += 16) {
    const nextDistance = Math.min(16, length - 2 - distance)
    const nextAmplitude = distance + 16 >= length ? 0 : lineAmplitude(seedA, distance) * (1 - ease) + lineAmplitude(seedB, distance) * ease
    const start = { x: from.x + along.x * distance + normal.x * previousAmplitude, y: from.y + along.y * distance + normal.y * previousAmplitude }
    const end = { x: from.x + along.x * (distance + nextDistance) + normal.x * nextAmplitude, y: from.y + along.y * (distance + nextDistance) + normal.y * nextAmplitude }
    context.strokeStyle = background
    context.lineWidth = 3
    context.beginPath(); context.moveTo(start.x - normal.x * 1.5, start.y - normal.y * 1.5); context.lineTo(end.x - normal.x * 1.5, end.y - normal.y * 1.5); context.stroke()
    context.strokeStyle = color
    context.lineWidth = 1
    context.beginPath(); context.moveTo(start.x, start.y); context.lineTo(end.x, end.y); context.stroke()
    previousAmplitude = nextAmplitude
  }
}

function drawDreamBlock(context: CanvasRenderingContext2D, assets: GameAssets, entity: MapEntity, frame: number, active: boolean): void {
  const box = entity.bounds
  const background = active ? '#000000' : '#1f2e2d'
  const line = active ? '#ffffff' : '#6a8480'
  context.fillStyle = background
  context.fillRect(box.x, box.y, box.width, box.height)
  const particle = assets.entries['objects/dreamblock/particles']
  const colors = active
    ? [['#ffef11', '#ff00d0', '#08a310'], ['#5fcde4', '#7fb25e', '#e0564c'], ['#5b6ee1', '#cc3b3b', '#7daa64']]
    : [['#8a8a8a'], ['#aaaaaa'], ['#d0d0d0']]
  const count = Math.floor(box.width / 8 * (box.height / 8) * .7)
  if (particle) {
    for (let index = 0; index < count; index += 1) {
      const layerRoll = pseudo(index * 5.31 + box.x)
      const layer = layerRoll < 1 / 6 ? 0 : layerRoll < .5 ? 1 : 2
      const px = box.x + 2 + pseudo(index * 7.17 + box.y) * Math.max(1, box.width - 4)
      const py = box.y + 2 + pseudo(index * 9.73 + box.x + box.y) * Math.max(1, box.height - 4)
      const timeOffset = pseudo(index * 13.11)
      const animation = layer === 0 ? 3 - Math.floor((timeOffset * 4 + frame / 10) % 4) : layer === 1 ? 1 + Math.floor((timeOffset * 2 + frame / 10) % 2) : 2
      const sourceX = [14, 7, 0, 7][animation]
      const palette = colors[layer]
      const color = palette[Math.floor(pseudo(index * 17.41) * palette.length)]
      const tile = dreamParticleTexture(assets, sourceX, color)
      if (tile) context.drawImage(tile, Math.round(px - 3.5), Math.round(py - 3.5))
    }
  }
  drawWobbleLine(context, { x: box.x, y: box.y }, { x: box.x + box.width, y: box.y }, 0, frame, line, background)
  drawWobbleLine(context, { x: box.x + box.width, y: box.y }, { x: box.x + box.width, y: box.y + box.height }, .7, frame, line, background)
  drawWobbleLine(context, { x: box.x + box.width, y: box.y + box.height }, { x: box.x, y: box.y + box.height }, 1.5, frame, line, background)
  drawWobbleLine(context, { x: box.x, y: box.y + box.height }, { x: box.x, y: box.y }, 2.5, frame, line, background)
  context.fillStyle = line
  for (const [x, y] of [[box.x, box.y], [box.x + box.width - 2, box.y], [box.x, box.y + box.height - 2], [box.x + box.width - 2, box.y + box.height - 2]]) context.fillRect(x, y, 2, 2)
}

function targetMatches(target: Vec2 | undefined, x: number, y: number, tolerance = 1): boolean {
  return Boolean(target && Math.abs(target.x - x) <= tolerance && Math.abs(target.y - y) <= tolerance)
}

export function activeBoosterCenter(entity: MapEntity, state: SimState): Vec2 | undefined {
  const center = { x: entity.bounds.x + entity.bounds.width / 2, y: entity.bounds.y + entity.bounds.height / 2 }
  const active = (state.booster_reuse_timer ?? 0) > 0 && targetMatches(state.last_booster_target, center.x, center.y + 2)
  return active ? { x: state.pos.x, y: state.pos.y - 7.5 } : undefined
}

function drawBooster(context: CanvasRenderingContext2D, assets: GameAssets, entity: MapEntity, frame: number, state: SimState): void {
  const box = entity.bounds
  const center = { x: box.x + box.width / 2, y: box.y + box.height / 2 }
  const red = entity.kind === 'red_booster'
  const prefix = red ? 'objects/booster/boosterRed' : 'objects/booster/booster'
  const activeCenter = activeBoosterCenter(entity, state)
  if (activeCenter) {
    if (state.state === 'Dash' || state.state === 'RedDash') drawEntry(context, assets, 'objects/booster/outline', center.x, center.y, 16, 16)
    const indices = state.state === 'Boost' ? [5, 6, 7, 8] : [18, 19, 20, 21, 22, 23, 24, 25]
    const index = indices[Math.floor(frame / (state.state === 'Boost' ? 6 : 4)) % indices.length]
    const key = `${prefix}${String(index).padStart(2, '0')}`
    drawEntry(context, assets, key, activeCenter.x, activeCenter.y, 16, 16)
    return
  }
  const index = Math.floor(frame / 6) % 5
  drawEntry(context, assets, `${prefix}${String(index).padStart(2, '0')}`, center.x, center.y, 16, 16)
}

function drawFlyFeather(context: CanvasRenderingContext2D, assets: GameAssets, entity: MapEntity, frame: number, state: SimState): void {
  const box = entity.bounds
  const center = { x: box.x + box.width / 2, y: box.y + box.height / 2 }
  const consumed = (state.feather_reuse_timer ?? 0) > 0 && targetMatches(state.last_feather_target, center.x, center.y)
  if (consumed) {
    if (!entity.single_use) drawEntry(context, assets, 'objects/flyFeather/outline', center.x, center.y, 16, 16)
    return
  }
  const phase = Math.floor(frame / 4) % 42
  const prefix = phase < 21 ? 'objects/flyFeather/idle' : 'objects/flyFeather/flash'
  const index = phase % 21
  const y = center.y + Math.sin(frame / 12 + center.x) * 2
  drawEntry(context, assets, `${prefix}${String(index).padStart(2, '0')}`, center.x, y, 16, 16)
  if (entity.shielded) {
    context.strokeStyle = '#ffffff'
    context.lineWidth = 3
    context.beginPath(); context.arc(center.x, y, 10, 0, Math.PI * 2); context.stroke()
  }
}

function drawBumper(context: CanvasRenderingContext2D, assets: GameAssets, entity: MapEntity, frame: number, state: SimState): void {
  const box = entity.bounds
  const center = { x: box.x + box.width / 2 + Math.sin(frame / 20) * 3, y: box.y + box.height / 2 + Math.sin(frame / 40) * 2 }
  const cooling = (state.bumper_reuse_timer ?? 0) > 0 && targetMatches(state.last_bumper_target, box.x + box.width / 2, box.y + box.height / 2)
  const index = cooling ? 42 : Math.floor(frame / 4) % 34
  drawEntry(context, assets, `objects/Bumper/Idle${String(index).padStart(2, '0')}`, center.x, center.y, 32, 32)
}

function drawBadelineBoost(context: CanvasRenderingContext2D, assets: GameAssets, entity: MapEntity, frame: number, state: SimState): void {
  const box = entity.bounds
  const origin = { x: box.x + box.width / 2, y: box.y + box.height / 2 }
  const selected = targetMatches(state.badeline_boost_entity_origin, origin.x, origin.y)
  const position = selected && state.badeline_boost_current_position ? state.badeline_boost_current_position : origin
  if (selected && state.badeline_boost_active) return
  const index = Math.floor(frame / 5) % 6
  drawEntry(context, assets, `objects/badelineboost/idle${String(index).padStart(2, '0')}`, position.x, position.y, 12, 12)
}

function drawEntity(context: CanvasRenderingContext2D, assets: GameAssets, entity: MapEntity, frame: number, state: SimState): void {
  const box = entity.bounds
  if (entity.kind === 'spikes') {
    const direction = entity.direction.x < 0 ? 'left' : entity.direction.x > 0 ? 'right' : entity.direction.y > 0 ? 'down' : 'up'
    drawRepeated(context, assets, `danger/spikes/default_${direction}00`, entity, 8)
  } else if (entity.kind === 'jump_thru') {
    const entry = assets.entries['objects/jumpthru/wood']
    if (!entry) return
    for (let x = box.x; x < box.x + box.width; x += 8) context.drawImage(assets.image, entry.x + 8, entry.y, 8, Math.min(8, entry.height), x, box.y, 8, Math.min(8, entry.height))
  } else if (entity.kind === 'water') {
    drawWater(context, entity, frame)
  } else if (entity.kind === 'dream_block') {
    drawDreamBlock(context, assets, entity, frame, state.can_dream_dash)
  } else if (entity.kind === 'booster' || entity.kind === 'red_booster') {
    drawBooster(context, assets, entity, frame, state)
  } else if (entity.kind === 'fly_feather') {
    drawFlyFeather(context, assets, entity, frame, state)
  } else if (entity.kind === 'bumper') {
    drawBumper(context, assets, entity, frame, state)
  } else if (entity.kind === 'ice_ball') {
    const centerX = box.x + box.width / 2
    const centerY = box.y + box.height / 2
    context.fillStyle = '#9de7ff'
    context.strokeStyle = '#ffffff'
    context.lineWidth = 2
    context.beginPath()
    context.arc(centerX, centerY, box.width / 2, 0, Math.PI * 2)
    context.fill()
    context.stroke()
  } else if (entity.kind === 'badeline_boost') {
    drawBadelineBoost(context, assets, entity, frame, state)
  }
}

export function GameView({ map, state, states, frame, stale }: { map: GymMap; state: SimState; states: readonly (SimState | undefined)[]; frame: number; stale: boolean }) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [assets, setAssets] = useState<GameAssets | null>(null)
  const [viewportRevision, setViewportRevision] = useState(0)
  const solidGrid = useMemo(() => buildSolidGrid(map), [map])

  useEffect(() => { void loadAssets().then(setAssets) }, [])
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const observer = new ResizeObserver(() => setViewportRevision((value) => value + 1))
    observer.observe(canvas)
    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || !assets) return
    const context = canvas.getContext('2d')
    if (!context) return
    const dpr = window.devicePixelRatio || 1
    const rect = canvas.getBoundingClientRect()
    canvas.width = Math.round(rect.width * dpr)
    canvas.height = Math.round(rect.height * dpr)
    context.setTransform(dpr, 0, 0, dpr, 0, 0)
    context.imageSmoothingEnabled = false

    const scale = Math.min(rect.width / map.bounds.width, rect.height / map.bounds.height)
    const offsetX = (rect.width - map.bounds.width * scale) / 2
    const offsetY = (rect.height - map.bounds.height * scale) / 2
    context.fillStyle = '#000000'
    context.fillRect(0, 0, rect.width, rect.height)

    context.save()
    context.translate(offsetX - map.bounds.x * scale, offsetY - map.bounds.y * scale)
    context.scale(scale, scale)
    drawTiles(context, assets, map, solidGrid)
    for (const entity of map.entities) drawEntity(context, assets, entity, frame, state)
    context.globalAlpha = stale ? .45 : 1
    drawPlayer(context, assets, states, state, frame)
    drawWind(context, assets, map, state, frame)
    context.restore()
  }, [assets, frame, map, solidGrid, stale, state, states, viewportRevision])

  return <div className="game-screen">
    <canvas ref={canvasRef} aria-label="CelesteGymPlayground 原版资源渲染画面" />
    <div className="screen-vignette" />
    <div className="screen-noise" />
    {!assets && <div className="recompute-flag"><span />加载 Gameplay atlas</div>}
    {assets && stale && <div className="recompute-flag"><span />等待重算</div>}
  </div>
}
