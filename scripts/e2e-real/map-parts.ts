import type { CanonicalFixtureEntity, FixtureEntity, FixturePackage, MapPart, Rect, RoomContribution, Vector2 } from './types.js'

function stableJson(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(',')}]`
  if (value && typeof value === 'object') {
    const entries = Object.entries(value as Record<string, unknown>).sort(([left], [right]) => left.localeCompare(right))
    return `{${entries.map(([key, item]) => `${JSON.stringify(key)}:${stableJson(item)}`).join(',')}}`
  }
  return JSON.stringify(value)
}

export function defineMapPart<const T extends MapPart>(part: T): T {
  if (!/^[a-z0-9][a-z0-9.-]*$/.test(part.id)) throw new Error(`invalid map part id: ${part.id}`)
  return Object.freeze(part)
}

export function dependencyClosure(requested: readonly MapPart[], catalog: ReadonlyMap<string, MapPart>): readonly MapPart[] {
  const result = new Map<string, MapPart>()
  const visiting = new Set<string>()
  const visit = (part: MapPart): void => {
    if (result.has(part.id)) return
    if (visiting.has(part.id)) throw new Error(`map part dependency cycle at ${part.id}`)
    visiting.add(part.id)
    for (const dependency of [...part.dependencies].sort()) {
      const resolved = catalog.get(dependency)
      if (!resolved) throw new Error(`${part.id}: unknown map part dependency ${dependency}`)
      visit(resolved)
    }
    visiting.delete(part.id)
    result.set(part.id, part)
  }
  for (const part of [...requested].sort((left, right) => left.id.localeCompare(right.id))) visit(part)
  return Object.freeze([...result.values()])
}

export function assembleFixturePackage(
  requested: readonly MapPart[],
  catalog: ReadonlyMap<string, MapPart>,
): FixturePackage {
  const parts = dependencyClosure(requested, catalog)
  if (parts.length === 0) throw new Error('cannot assemble an empty map-part set')
  const packageName = parts[0]?.package
  const sid = parts[0]?.sid
  if (!packageName || !sid) throw new Error('map part package and sid must not be empty')
  const rooms = new Map<string, {
    bounds?: Rect
    spawn?: Vector2
    solids: Map<string, Rect>
    entities: Map<string, FixtureEntity>
  }>()
  const entityRooms = new Map<string, string>()
  for (const part of parts) {
    if (part.package !== packageName || part.sid !== sid) throw new Error(`${part.id}: package or sid conflicts with dependency closure`)
    for (const contribution of part.rooms) {
      for (const entity of contribution.entities ?? []) {
        const previousRoom = entityRooms.get(entity.id)
        if (previousRoom) throw new Error(`${part.id}: duplicate map-global fixture entity id ${entity.id} in ${previousRoom} and ${contribution.name}`)
        entityRooms.set(entity.id, contribution.name)
      }
      mergeRoom(part.id, contribution, rooms)
    }
  }
  const fixture: FixturePackage = {
    formatVersion: 1,
    package: packageName,
    sid,
    rooms: [...rooms.entries()].sort(([left], [right]) => left.localeCompare(right)).map(([name, room]) => {
      if (!room.bounds || !room.spawn) throw new Error(`room ${name} is missing bounds or spawn`)
      return {
        name,
        bounds: room.bounds,
        spawn: room.spawn,
        solids: [...room.solids.values()].sort((left, right) => stableJson(left).localeCompare(stableJson(right))),
        entities: [...room.entities.values()]
          .sort((left, right) => left.id.localeCompare(right.id))
          .map(canonicalizeEntity),
      }
    }),
  }
  validateFixturePackage(fixture)
  return fixture
}

function mergeRoom(
  partId: string,
  contribution: RoomContribution,
  rooms: Map<string, { bounds?: Rect; spawn?: Vector2; solids: Map<string, Rect>; entities: Map<string, FixtureEntity> }>,
): void {
  const room = rooms.get(contribution.name) ?? { solids: new Map<string, Rect>(), entities: new Map<string, FixtureEntity>() }
  if (contribution.bounds) room.bounds = mergeScalar(`${partId}:${contribution.name}:bounds`, room.bounds, contribution.bounds)
  if (contribution.spawn) room.spawn = mergeScalar(`${partId}:${contribution.name}:spawn`, room.spawn, contribution.spawn)
  for (const solid of contribution.solids ?? []) room.solids.set(stableJson(solid), solid)
  for (const entity of contribution.entities ?? []) {
    validateAuthoringEntity(entity, contribution.name)
    const previous = room.entities.get(entity.id)
    if (previous && stableJson(previous) !== stableJson(entity)) throw new Error(`${partId}: conflicting fixture entity ${entity.id}`)
    room.entities.set(entity.id, entity)
  }
  rooms.set(contribution.name, room)
}

function mergeScalar<T>(label: string, previous: T | undefined, next: T): T {
  if (previous !== undefined && stableJson(previous) !== stableJson(next)) throw new Error(`conflicting ${label}`)
  return next
}

const ENTITY_KINDS = new Set<FixtureEntity['kind']>([
  'jump_thru', 'dream_block', 'spikes', 'water', 'booster', 'red_booster',
  'fly_feather', 'bumper', 'ice_ball', 'badeline_boost', 'spring', 'strawberry',
  'puffer', 'angry_oshiro', 'seeker', 'snowball', 'cloud',
  'wind', 'bounce_block', 'theo_crystal', 'zip_mover', 'move_block', 'moving_solid',
])

export function validateFixturePackage(fixture: FixturePackage): void {
  if (fixture.formatVersion !== 1) throw new Error('fixture formatVersion must be 1')
  if (!fixture.package || !fixture.sid) throw new Error('fixture package and sid must not be empty')
  const roomNames = new Set<string>()
  const entityIds = new Set<string>()
  for (const room of fixture.rooms) {
    if (!room.name || roomNames.has(room.name)) throw new Error(`duplicate or empty fixture room name: ${room.name}`)
    roomNames.add(room.name)
    assertRect(room.bounds, `room ${room.name} bounds`, true)
    assertVector(room.spawn, `room ${room.name} spawn`)
    assertPointInRoom(room.spawn, room.bounds, `room ${room.name} spawn`)
    for (const solid of room.solids) {
      assertRect(solid, `room ${room.name} solid`, true)
      assertRectInRoom(solid, room.bounds, `room ${room.name} solid`)
    }
    for (const entity of room.entities) {
      validateCanonicalEntity(entity, room.name)
      if (entityIds.has(entity.id)) throw new Error(`duplicate map-global fixture entity id: ${entity.id}`)
      entityIds.add(entity.id)
      assertRectInRoom(entity.bounds, room.bounds, `entity ${entity.id}`)
      for (const node of 'nodes' in entity ? entity.nodes ?? [] : []) {
        assertPointInRoom(node, room.bounds, `entity ${entity.id} node`)
      }
    }
  }
}

function validateAuthoringEntity(entity: FixtureEntity, roomName: string): void {
  if (!entity.id) throw new Error(`room ${roomName} has an entity with an empty id`)
  if (!ENTITY_KINDS.has(entity.kind)) throw new Error(`entity ${entity.id} has unknown kind ${String(entity.kind)}`)
  assertRect(entity.bounds, `entity ${entity.id} bounds`, false)
  const allowed = new Set(['id', 'kind', 'bounds', 'name'])
  if (entity.kind === 'fly_feather') { allowed.add('shielded'); allowed.add('singleUse') }
  if (entity.kind === 'ice_ball') { allowed.add('nodes'); allowed.add('singleUse') }
  if (entity.kind === 'badeline_boost' || entity.kind === 'zip_mover') allowed.add('nodes')
  if (entity.kind === 'spikes' || entity.kind === 'spring' || entity.kind === 'wind' || entity.kind === 'move_block' || entity.kind === 'moving_solid') allowed.add('direction')
  for (const key of Object.keys(entity)) if (!allowed.has(key)) throw new Error(`entity ${entity.id} kind ${entity.kind} forbids field ${key}`)

  if ('direction' in entity) {
    assertVector(entity.direction, `entity ${entity.id} direction`)
    const [x, y] = entity.direction
    if (entity.kind === 'spikes' || entity.kind === 'spring') {
      if (Math.abs(x) + Math.abs(y) !== 1) throw new Error(`entity ${entity.id} requires a cardinal unit direction`)
    } else if (entity.kind === 'wind' && (x === 0) === (y === 0)) {
      throw new Error(`entity ${entity.id} wind direction must have exactly one nonzero axis`)
    } else if ((entity.kind === 'move_block' || entity.kind === 'moving_solid') && x === 0 && y === 0) {
      throw new Error(`entity ${entity.id} ${entity.kind} direction must be nonzero`)
    }
  }
  if ('nodes' in entity && entity.nodes) {
    for (const [index, node] of entity.nodes.entries()) assertVector(node, `entity ${entity.id} node ${index}`)
    if (entity.kind === 'zip_mover' && entity.nodes.length !== 1) throw new Error(`entity ${entity.id} zip_mover requires exactly one node`)
    if (entity.kind === 'ice_ball' && entity.nodes.length > 1) throw new Error(`entity ${entity.id} ice_ball accepts at most one node`)
  }
  if ('shielded' in entity && typeof entity.shielded !== 'boolean') throw new Error(`entity ${entity.id} shielded must be boolean`)
  if ('singleUse' in entity && typeof entity.singleUse !== 'boolean') throw new Error(`entity ${entity.id} singleUse must be boolean`)
}

function canonicalizeEntity(entity: FixtureEntity): CanonicalFixtureEntity {
  return {
    id: entity.id,
    kind: entity.kind,
    bounds: entity.bounds,
    direction: 'direction' in entity ? entity.direction : [0, 0],
    shielded: 'shielded' in entity ? entity.shielded ?? false : false,
    singleUse: 'singleUse' in entity ? entity.singleUse ?? false : false,
    nodes: 'nodes' in entity ? entity.nodes ?? [] : [],
    name: entity.name ?? null,
  }
}

function validateCanonicalEntity(entity: CanonicalFixtureEntity, roomName: string): void {
  const keys = Object.keys(entity).sort()
  const expectedKeys = ['bounds', 'direction', 'id', 'kind', 'name', 'nodes', 'shielded', 'singleUse']
  if (stableJson(keys) !== stableJson(expectedKeys)) throw new Error(`entity ${entity.id} does not use the canonical entity schema`)
  if (!entity.id) throw new Error(`room ${roomName} has an entity with an empty id`)
  if (!ENTITY_KINDS.has(entity.kind)) throw new Error(`entity ${entity.id} has unknown kind ${String(entity.kind)}`)
  assertRect(entity.bounds, `entity ${entity.id} bounds`, false)
  assertVector(entity.direction, `entity ${entity.id} direction`)
  for (const [index, node] of entity.nodes.entries()) assertVector(node, `entity ${entity.id} node ${index}`)
  if (typeof entity.shielded !== 'boolean' || typeof entity.singleUse !== 'boolean') {
    throw new Error(`entity ${entity.id} canonical booleans are invalid`)
  }
  if (entity.name !== null && !entity.name) throw new Error(`entity ${entity.id} name must be null or non-empty`)

  const [x, y] = entity.direction
  if (entity.kind === 'spikes' || entity.kind === 'spring') {
    if (Math.abs(x) + Math.abs(y) !== 1) throw new Error(`entity ${entity.id} requires a cardinal unit direction`)
  } else if (entity.kind === 'wind') {
    if ((x === 0) === (y === 0)) throw new Error(`entity ${entity.id} wind direction must have exactly one nonzero axis`)
  } else if (entity.kind !== 'move_block' && entity.kind !== 'moving_solid' && (x !== 0 || y !== 0)) {
    throw new Error(`entity ${entity.id} kind ${entity.kind} forbids nonzero direction`)
  }
  if (entity.kind === 'zip_mover' && entity.nodes.length !== 1) throw new Error(`entity ${entity.id} zip_mover requires exactly one node`)
  if (entity.kind === 'ice_ball' && entity.nodes.length > 1) throw new Error(`entity ${entity.id} ice_ball accepts at most one node`)
  if (entity.nodes.length > 0 && entity.kind !== 'zip_mover' && entity.kind !== 'ice_ball' && entity.kind !== 'badeline_boost') {
    throw new Error(`entity ${entity.id} kind ${entity.kind} forbids nodes`)
  }
  if (entity.shielded && entity.kind !== 'fly_feather') throw new Error(`entity ${entity.id} kind ${entity.kind} forbids shielded`)
  if (entity.singleUse && entity.kind !== 'fly_feather' && entity.kind !== 'ice_ball') {
    throw new Error(`entity ${entity.id} kind ${entity.kind} forbids singleUse`)
  }
}

function assertRect(rect: Rect, label: string, aligned: boolean): void {
  if (rect.length !== 4 || !rect.every(Number.isSafeInteger)) throw new Error(`${label} must contain four integers`)
  if (rect[2] <= 0 || rect[3] <= 0) throw new Error(`${label} width and height must be positive`)
  if (aligned && !rect.every((value) => value % 8 === 0)) throw new Error(`${label} must be 8px aligned`)
}

function assertVector(vector: Vector2, label: string): void {
  if (vector.length !== 2 || !vector.every(Number.isSafeInteger)) throw new Error(`${label} must contain two integers`)
}

function assertRectInRoom(rect: Rect, room: Rect, label: string): void {
  const [x, y, width, height] = rect
  const [roomX, roomY, roomWidth, roomHeight] = room
  if (x < roomX || y < roomY || x + width > roomX + roomWidth || y + height > roomY + roomHeight) {
    throw new Error(`${label} is outside room bounds`)
  }
}

function assertPointInRoom(point: Vector2, room: Rect, label: string): void {
  const [x, y] = point
  const [roomX, roomY, roomWidth, roomHeight] = room
  if (x < roomX || y < roomY || x > roomX + roomWidth || y > roomY + roomHeight) {
    throw new Error(`${label} is outside room bounds`)
  }
}
