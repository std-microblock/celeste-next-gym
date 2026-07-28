export const ACTIONS = ['up', 'down', 'left', 'right', 'jump', 'dash', 'grab'] as const

export type Action = (typeof ACTIONS)[number]

export type FrameButtons = Record<Action, boolean>

export interface SimInput {
  move_x: -1 | 0 | 1
  move_y: -1 | 0 | 1
  jump_pressed: boolean
  jump_held: boolean
  dash_pressed: boolean
  crouch_dash_pressed: boolean
  grab_held: boolean
}

export interface Vec2 {
  x: number
  y: number
}

export interface ZipMoverState {
  phase: number
  wait_timer: number
  at: number
  position: Vec2
  remainder: Vec2
  lift_speed: Vec2
  start: Vec2
}

export interface BounceBlockState {
  phase: number
  move_speed: number
  bounce_dir: Vec2
  bounce_lift: Vec2
  bounce_end_timer: number
  respawn_timer: number
  position: Vec2
  remainder: Vec2
  lift_speed: Vec2
  start: Vec2
}

export interface TheoCrystalState {
  position: Vec2
  speed: Vec2
  remainder: Vec2
  held: boolean
  cannot_hold_timer: number
  gravity_timer: number
}

export interface GliderState {
  position: Vec2
  speed: Vec2
  remainder: Vec2
  held: boolean
  cannot_hold_timer: number
  gravity_timer: number
  no_gravity_timer: number
  high_friction_timer: number
}

export interface SimState {
  pos: Vec2
  speed: Vec2
  state: string
  facing: boolean
  dashes: number
  stamina: number
  on_ground: boolean
  player_on_ground?: boolean
  player_on_ground_initialized?: boolean
  ducking: boolean
  can_dream_dash: boolean
  dead: boolean
  death_freeze_pending: boolean
  respawn_frames: number
  dash_dir: Vec2
  state_timer?: number
  boost_target?: Vec2
  boost_red?: boolean
  last_booster_target?: Vec2
  booster_reuse_timer?: number
  wind?: Vec2
  star_fly_transforming?: boolean
  star_fly_transform_frames?: number
  last_feather_target?: Vec2
  feather_reuse_timer?: number
  last_bumper_target?: Vec2
  bumper_reuse_timer?: number
  star_fly_hitbox_preserved?: boolean
  last_bounce_target?: Vec2
  bounce_reuse_timer?: number
  moving_solid_time?: number
  zip_movers?: ZipMoverState[]
  bounce_blocks?: BounceBlockState[]
  theo_crystals?: TheoCrystalState[]
  gliders?: GliderState[]
  holding_theo?: number | null
  holding_glider?: number | null
  badeline_boost_active?: boolean
  badeline_boost_entity_origin?: Vec2
  badeline_boost_current_position?: Vec2
  badeline_boost_relocating?: boolean
  [key: string]: unknown
}

export type EntityKind = 'jump_thru' | 'spikes' | 'water' | 'dream_block' | 'booster' | 'red_booster' | 'fly_feather' | 'bumper' | 'ice_ball' | 'badeline_boost' | 'spring' | 'strawberry' | 'wind' | 'bounce_block' | 'theo_crystal' | 'glider' | 'zip_mover' | 'moving_solid' | 'unknown'

export interface MapEntity {
  kind: EntityKind
  bounds: { x: number; y: number; width: number; height: number }
  direction: Vec2
  shielded?: boolean
  single_use?: boolean
  nodes?: Vec2[]
  name: string
}

export interface GymMap {
  name: string
  room?: string
  bounds: { x: number; y: number; width: number; height: number }
  spawn: Vec2
  solids: { x: number; y: number; width: number; height: number }[]
  entities: MapEntity[]
  source_package: string | null
}

export type KeyBindings = Record<Action, string>

export const ACTION_LABELS: Record<Action, string> = {
  up: '上',
  down: '下',
  left: '左',
  right: '右',
  jump: '跳跃',
  dash: '冲刺',
  grab: '抓取',
}

export const ACTION_GLYPHS: Record<Action, string> = {
  up: '▲',
  down: '▼',
  left: '◀',
  right: '▶',
  jump: 'J',
  dash: 'D',
  grab: 'G',
}

export const DEFAULT_BINDINGS: KeyBindings = {
  up: 'KeyW',
  down: 'KeyS',
  left: 'KeyA',
  right: 'KeyD',
  jump: 'KeyL',
  dash: 'Semicolon',
  grab: 'Quote',
}

export const EMPTY_BUTTONS: FrameButtons = {
  up: false,
  down: false,
  left: false,
  right: false,
  jump: false,
  dash: false,
  grab: false,
}

export function makeEmptyButtons(): FrameButtons {
  return { ...EMPTY_BUTTONS }
}

export function buttonsToInput(current: FrameButtons, previous: FrameButtons = EMPTY_BUTTONS): SimInput {
  return {
    move_x: current.left === current.right ? 0 : current.left ? -1 : 1,
    move_y: current.up === current.down ? 0 : current.up ? -1 : 1,
    jump_pressed: current.jump && !previous.jump,
    jump_held: current.jump,
    dash_pressed: current.dash && !previous.dash,
    crouch_dash_pressed: false,
    grab_held: current.grab,
  }
}

export function bindingLabel(code: string): string {
  if (code.startsWith('Key')) return code.slice(3)
  if (code.startsWith('Digit')) return code.slice(5)
  return ({ Semicolon: ';', Quote: "'", Space: 'SPACE', ArrowUp: '↑', ArrowDown: '↓', ArrowLeft: '←', ArrowRight: '→' } as Record<string, string>)[code] ?? code
}

export const PLAYGROUND: GymMap = {
  name: 'Mechanics Playground',
  room: 'playground',
  bounds: { x: 0, y: 0, width: 960, height: 544 },
  spawn: { x: 64, y: 496 },
  solids: [
    { x: 0, y: 496, width: 960, height: 48 },
    { x: 0, y: 0, width: 24, height: 496 },
    { x: 936, y: 0, width: 24, height: 496 },
    { x: 272, y: 304, width: 32, height: 192 },
    { x: 688, y: 360, width: 24, height: 136 },
    { x: 864, y: 240, width: 24, height: 256 },
    { x: 480, y: 240, width: 96, height: 24 },
    { x: 800, y: 248, width: 120, height: 16 },
    { x: 168, y: 120, width: 72, height: 64 },
    { x: 240, y: 112, width: 8, height: 8 },
    { x: 296, y: 120, width: 16, height: 8 },
    { x: 328, y: 120, width: 48, height: 8 },
    { x: 400, y: 120, width: 80, height: 64 },
    { x: 480, y: 104, width: 8, height: 16 },
    { x: 544, y: 112, width: 80, height: 8 },
    { x: 688, y: 120, width: 80, height: 64 },
    { x: 768, y: 112, width: 8, height: 8 },
    { x: 840, y: 120, width: 80, height: 8 },
  ],
  entities: [
    { kind: 'jump_thru', bounds: { x: 112, y: 400, width: 112, height: 8 }, direction: { x: 0, y: 0 }, name: 'jumpThru' },
    { kind: 'spikes', bounds: { x: 328, y: 493, width: 96, height: 3 }, direction: { x: 0, y: -1 }, name: 'spikesUp' },
    { kind: 'water', bounds: { x: 448, y: 416, width: 112, height: 80 }, direction: { x: 0, y: 0 }, name: 'water' },
    { kind: 'dream_block', bounds: { x: 600, y: 352, width: 64, height: 144 }, direction: { x: 0, y: 0 }, name: 'dreamBlock' },
    { kind: 'booster', bounds: { x: 752, y: 432, width: 16, height: 16 }, direction: { x: 0, y: 0 }, name: 'booster' },
    { kind: 'red_booster', bounds: { x: 816, y: 432, width: 16, height: 16 }, direction: { x: 0, y: 0 }, name: 'redBooster' },
    { kind: 'fly_feather', bounds: { x: 100, y: 180, width: 20, height: 20 }, direction: { x: 0, y: 0 }, name: 'infiniteStar' },
    { kind: 'bumper', bounds: { x: 588, y: 188, width: 24, height: 24 }, direction: { x: 0, y: 0 }, name: 'bigSpinner' },
    { kind: 'badeline_boost', bounds: { x: 304, y: 384, width: 32, height: 32 }, direction: { x: 0, y: 0 }, nodes: [{ x: 320, y: 288 }], name: 'badelineBoost' },
    { kind: 'theo_crystal', bounds: { x: 846, y: 486, width: 8, height: 10 }, direction: { x: 0, y: 0 }, name: 'theoCrystal' },
    { kind: 'bounce_block', bounds: { x: 352, y: 360, width: 64, height: 16 }, direction: { x: 0, y: 0 }, name: 'bounceBlock' },
    { kind: 'wind', bounds: { x: 640, y: 128, width: 280, height: 120 }, direction: { x: 400, y: 0 }, name: 'windTrigger' },
  ],
  source_package: 'CelesteGymPlayground',
}

export function createInitialState(map: GymMap): SimState {
  return {
    pos: { ...map.spawn },
    speed: { x: 0, y: 0 },
    state: 'Normal',
    facing: true,
    dashes: 1,
    stamina: 110,
    on_ground: false,
    ducking: false,
    can_dream_dash: true,
    dead: false,
    death_freeze_pending: false,
    respawn_frames: 0,
    dash_dir: { x: 0, y: 0 },
  }
}
