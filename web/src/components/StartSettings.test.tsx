import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { StartSettings } from './StartSettings'

afterEach(cleanup)

describe('StartSettings', () => {
  it('submits a normalized room and numeric start position', () => {
    const onApply = vi.fn()
    render(<StartSettings
      room="playground"
      position={{ x: 64, y: 496 }}
      busy={false}
      onApply={onApply}
      onClose={() => {}}
    />)

    fireEvent.change(screen.getByRole('textbox', { name: '开始房间' }), { target: { value: ' lvl_transition_0 ' } })
    fireEvent.change(screen.getByRole('spinbutton', { name: 'X' }), { target: { value: '24.5' } })
    fireEvent.change(screen.getByRole('spinbutton', { name: 'Y' }), { target: { value: '-16' } })
    fireEvent.click(screen.getByRole('button', { name: '应用并回到 F0' }))

    expect(onApply).toHaveBeenCalledWith({
      room: 'transition_0',
      position: { x: 24.5, y: -16 },
    })
  })

  it('requires a room and both coordinates', () => {
    const onApply = vi.fn()
    render(<StartSettings
      room="playground"
      position={{ x: 64, y: 496 }}
      busy={false}
      onApply={onApply}
      onClose={() => {}}
    />)

    fireEvent.change(screen.getByRole('textbox', { name: '开始房间' }), { target: { value: ' ' } })
    fireEvent.change(screen.getByRole('spinbutton', { name: 'X' }), { target: { value: '' } })
    fireEvent.click(screen.getByRole('button', { name: '应用并回到 F0' }))

    expect(screen.getByRole('alert')).toHaveTextContent('请输入开始房间')
    expect(onApply).not.toHaveBeenCalled()
  })
})
