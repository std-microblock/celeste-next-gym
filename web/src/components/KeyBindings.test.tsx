import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { KeyBindings } from './KeyBindings'
import { DEFAULT_BINDINGS } from '../model'

afterEach(cleanup)

describe('KeyBindings gamepad settings', () => {
  it('shows the connected controller and changes the direction source', () => {
    const onGamepadDirectionSourceChange = vi.fn()
    render(<KeyBindings
      bindings={DEFAULT_BINDINGS}
      gamepadDirectionSource="stick"
      gamepadName="Xbox Wireless Controller"
      gamepadSupported
      onChange={vi.fn()}
      onGamepadDirectionSourceChange={onGamepadDirectionSourceChange}
      onClose={vi.fn()}
    />)

    expect(screen.getByText('已连接')).toBeInTheDocument()
    expect(screen.getByTitle('Xbox Wireless Controller')).toBeInTheDocument()
    fireEvent.change(screen.getByLabelText('手柄方向输入'), { target: { value: 'dpad' } })
    expect(onGamepadDirectionSourceChange).toHaveBeenCalledWith('dpad')
  })

  it('disables direction selection when the browser has no Gamepad API', () => {
    render(<KeyBindings
      bindings={DEFAULT_BINDINGS}
      gamepadDirectionSource="stick"
      gamepadName={null}
      gamepadSupported={false}
      onChange={vi.fn()}
      onGamepadDirectionSourceChange={vi.fn()}
      onClose={vi.fn()}
    />)

    expect(screen.getByText('浏览器不支持')).toBeInTheDocument()
    expect(screen.getByLabelText('手柄方向输入')).toBeDisabled()
  })
})
