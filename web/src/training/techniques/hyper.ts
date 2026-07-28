import type { TrainingMapDocument, TrainingTechnique, TrainingVariant } from '../catalog.ts'
import { room, snapshot } from '../helpers.ts'
import routeTrainingJson from '../maps/hyper-route.training.json' with { type: 'json' }

const routeMap = room('Hyper · 山路训练', 'hyper-route', {
  width: 960,
  spawn: { x: 32, y: 240 },
  floorSegments: [
    { x: 0, y: 240, width: 176, height: 30 },
    { x: 208, y: 240, width: 752, height: 30 },
  ],
  entities: [{ kind: 'spikes', bounds: { x: 520, y: 237, width: 64, height: 3 }, direction: { x: 0, y: -1 }, name: 'spikesUp' }],
})

const spikeMap = room('Hyper · 越过尖刺', 'hyper-spikes', {
  width: 480,
  spawn: { x: 32, y: 240 },
  entities: [{ kind: 'spikes', bounds: { x: 174, y: 237, width: 80, height: 3 }, direction: { x: 0, y: -1 }, name: 'spikesUp' }],
})

const bubbleMap = room('Hyper · 泡泡起手', 'hyper-bubble', {
  width: 480,
  spawn: { x: 112, y: 226 },
  entities: [{ kind: 'booster', bounds: { x: 104, y: 216, width: 16, height: 16 }, direction: { x: 0, y: 0 }, name: 'booster' }],
})

function trainingDocument(value: unknown): TrainingMapDocument {
  return value as TrainingMapDocument
}

const routeTraining = trainingDocument(routeTrainingJson)
const bubbleInitial = snapshot(bubbleMap.spawn, {
  state: 'Boost', on_ground: false, boost_target: { x: 112, y: 226 }, last_booster_target: { x: 112, y: 226 }, booster_reuse_timer: .45, state_timer: .28,
})

function singleModuleTraining(options: {
  id: string
  title: string
  summary: string
  trigger: { x: number; y: number; width: number; height: number }
  finish: { x: number; y: number; width: number; height: number }
  initial: ReturnType<typeof snapshot>
}): TrainingMapDocument {
  const module = structuredClone(routeTraining.modules[0])
  module.id = options.id
  module.trigger = { id: `${options.id}-start`, bounds: options.trigger }
  module.tutorial.id = `${options.id}-tutorial`
  module.tutorial.title = options.title
  module.tutorial.summary = options.summary
  module.validation.initial_state = options.initial
  return {
    version: 2,
    id: options.id,
    title: options.title,
    summary: options.summary,
    modules: [module],
    finish: { trigger: { id: `${options.id}-finish`, bounds: options.finish }, require_all_modules: true },
  }
}

const spikeTraining = singleModuleTraining({
  id: 'spike-gap', title: '越过尖刺', summary: '控制起跳窗口，越过地面的尖刺带。',
  trigger: { x: 56, y: 190, width: 104, height: 50 }, finish: { x: 320, y: 180, width: 96, height: 60 }, initial: snapshot({ x: 100, y: 240 }),
})
const bubbleTraining = singleModuleTraining({
  id: 'bubble-exit', title: '泡泡起手', summary: '从泡泡状态离开后建立 Hyper 节奏。',
  trigger: { x: 88, y: 180, width: 72, height: 60 }, finish: { x: 352, y: 180, width: 80, height: 60 }, initial: bubbleInitial,
})

function variant(id: string, map: ReturnType<typeof room>, training: TrainingMapDocument, initial = snapshot(map.spawn)): TrainingVariant {
  return { id, title: training.title, summary: training.summary, map, initial, training }
}

export const hyper: TrainingTechnique = {
  id: 'hyper',
  title: 'Hyper',
  summary: '右下冲后快速起跳，把冲刺转成高速地面移动。',
  related: ['wavedash', 'super'],
  variants: [
    variant('route', routeMap, routeTraining),
    variant('spike-gap', spikeMap, spikeTraining),
    variant('bubble-exit', bubbleMap, bubbleTraining, bubbleInitial),
  ],
}
