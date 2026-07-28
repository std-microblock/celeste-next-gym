import type { TrainingTechnique } from '../catalog.ts'
import { hyperVariant, room, snapshot } from '../helpers.ts'

const openGapMap = room('Hyper · 跨越断层', 'hyper-open-gap', {
  width: 640,
  floorSegments: [
    { x: 0, y: 240, width: 176, height: 30 },
    { x: 208, y: 240, width: 432, height: 30 },
    { x: 176, y: 262, width: 32, height: 8 },
  ],
})
const spikeMap = room('Hyper · 越过尖刺', 'hyper-spikes', {
  entities: [{ kind: 'spikes', bounds: { x: 174, y: 237, width: 80, height: 3 }, direction: { x: 0, y: -1 }, name: 'spikesUp' }],
})
const bubbleMap = room('Hyper · 泡泡起手', 'hyper-bubble', {
  spawn: { x: 112, y: 226 },
  entities: [{ kind: 'booster', bounds: { x: 104, y: 216, width: 16, height: 16 }, direction: { x: 0, y: 0 }, name: 'booster' }],
})

export const hyper: TrainingTechnique = {
  id: 'hyper',
  title: 'Hyper',
  summary: '右下冲后快速起跳，把冲刺转成高速地面移动。',
  related: ['wavedash', 'super'],
  variants: [
    hyperVariant({
      id: 'open-gap', title: '跨越断层', summary: '地面中间缺了一大片，用 Hyper 跨到另一侧。', map: openGapMap, initial: snapshot(openGapMap.spawn),
      routeValidation: { observeUntil: 'jump_frame+25', success: ['!final.dead', 'final.pos.x >= 208'] },
    }),
    hyperVariant({
      id: 'spike-gap', title: '越过尖刺', summary: '控制起跳窗口，越过地面的尖刺带。', map: spikeMap, initial: snapshot(spikeMap.spawn),
      routeValidation: { observeUntil: 'jump_frame+20', success: ['!final.dead', 'final.pos.x > 174'] },
    }),
    hyperVariant({
      id: 'bubble-exit', title: '泡泡起手', summary: '从泡泡状态离开后建立 Hyper 节奏。', map: bubbleMap,
      initial: snapshot(bubbleMap.spawn, {
        state: 'Boost', on_ground: false, boost_target: { x: 112, y: 226 }, last_booster_target: { x: 112, y: 226 }, booster_reuse_timer: .45, state_timer: .28,
      }),
      routeValidation: { observeUntil: 'jump_frame+30', success: ['!final.dead', 'final.pos.x > 180'] },
    }),
  ],
}
