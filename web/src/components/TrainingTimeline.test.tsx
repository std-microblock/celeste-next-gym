import { fireEvent, render } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { TrainingTimeline } from './TrainingTimeline'

describe('TrainingTimeline', () => {
  it('only seeks and sets R points; it never exposes input editing', () => {
    const onSeek = vi.fn()
    const onSetReset = vi.fn()
    const view = render(<TrainingTimeline frame={4} frameCount={20} fuzzStart={0} targetFrame={6} windows={[{ from: 5, to: 6 }]} actualInputs={[]} resetFrame={0} onSeek={onSeek} onSetReset={onSetReset} />)
    const timeline = view.getByRole('slider', { name: '训练回看时间线' })
    vi.spyOn(timeline, 'getBoundingClientRect').mockReturnValue({ x: 0, y: 0, left: 0, top: 0, right: 200, bottom: 50, width: 200, height: 50, toJSON: () => ({}) })
    fireEvent.pointerDown(timeline, { pointerId: 1, clientX: 100 })
    expect(onSeek).toHaveBeenCalledWith(10)
    fireEvent.click(view.getByRole('button', { name: /设为 R 点 F4/ }))
    expect(onSetReset).toHaveBeenCalledWith(4)
    expect(view.queryByText(/复制|粘贴|删除|编辑/)).not.toBeInTheDocument()
  })
})
