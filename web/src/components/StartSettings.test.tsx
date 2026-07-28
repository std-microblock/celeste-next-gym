import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { GymMap } from '../model'
import { StartSettings } from './StartSettings'

const rooms: GymMap[] = [
  {
    name: 'Playground',
    room: 'playground',
    bounds: { x: 0, y: 0, width: 320, height: 184 },
    spawn: { x: 64, y: 160 },
    solids: [{ x: 0, y: 160, width: 320, height: 24 }],
    entities: [],
    source_package: null,
  },
  {
    name: 'Transition',
    room: 'transition_0',
    bounds: { x: 0, y: -544, width: 960, height: 544 },
    spawn: { x: 24, y: -16 },
    solids: [],
    entities: [],
    source_package: null,
  },
]

afterEach(cleanup)

describe('StartSettings', () => {
  it('selects a room and its default spawn without typed coordinates', () => {
    const onApply = vi.fn()
    render(<StartSettings
      rooms={rooms}
      room="playground"
      position={{ x: 64, y: 160 }}
      busy={false}
      onApply={onApply}
      onClose={() => {}}
    />)

    fireEvent.click(screen.getByRole('button', { name: /过渡训练场/ }))
    expect(screen.getByLabelText('已选坐标')).toHaveTextContent('X 24 · Y -16')
    fireEvent.click(screen.getByRole('button', { name: '从这里开始' }))

    expect(onApply).toHaveBeenCalledWith({
      room: 'transition_0',
      position: { x: 24, y: -16 },
    })
  })

  it('places the start point by clicking the room preview', () => {
    const onApply = vi.fn()
    render(<StartSettings
      rooms={rooms}
      room="playground"
      position={{ x: 64, y: 160 }}
      busy={false}
      onApply={onApply}
      onClose={() => {}}
    />)

    const preview = screen.getByRole('img', { name: /主训练场 起点地图/ })
    vi.spyOn(preview, 'getBoundingClientRect').mockReturnValue({
      x: 0, y: 0, left: 0, top: 0, right: 320, bottom: 184, width: 320, height: 184, toJSON: () => ({}),
    })
    fireEvent.pointerDown(preview, { clientX: 160, clientY: 92 })
    fireEvent.click(screen.getByRole('button', { name: '从这里开始' }))

    expect(onApply).toHaveBeenCalledWith({
      room: 'playground',
      position: { x: 160, y: 92 },
    })
  })
})
