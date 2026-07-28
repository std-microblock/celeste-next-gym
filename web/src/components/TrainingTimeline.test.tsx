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
    expect(onSeek).toHaveBeenCalledWith(10, true)
    fireEvent.click(view.getByRole('button', { name: /设为 R 点 F4/ }))
    expect(onSetReset).toHaveBeenCalledWith(4)
    expect(view.queryByText(/复制|粘贴|删除|编辑/)).not.toBeInTheDocument()
    expect(view.getByText((_, element) => element?.classList.contains('training-legend') === true && element.textContent?.startsWith('操作起点') === true)).toBeInTheDocument()
    expect(view.getByText(/操作起点：入口 Dash/)).toBeInTheDocument()
  })

  it('centers the review window and retains an offscreen next-key hint', () => {
    const view = render(<TrainingTimeline frame={50} frameCount={100} fuzzStart={0} targetFrame={90} windows={[{ from: 88, to: 92 }]} actualInputs={[]} resetFrame={0} onSeek={vi.fn()} onSetReset={vi.fn()} />)

    expect(view.getByText('TRAINING REVIEW · F26–F74')).toBeInTheDocument()
    expect(view.getByText('›')).toHaveClass('offscreen', 'after')
    expect(view.getByText(/下一最佳关键点：F90/)).toBeInTheDocument()
    expect(view.queryByTitle('成功窗口 F88–F92')).not.toBeInTheDocument()
  })

  it('keeps the reference input centered while follow mode is active', () => {
    const view = render(<TrainingTimeline frame={8} frameCount={120} fuzzStart={0} targetFrame={70} windows={[{ from: 68, to: 72 }]} actualInputs={[]} resetFrame={0} followTarget onSeek={vi.fn()} onSetReset={vi.fn()} />)

    expect(view.getByText('TRAINING REVIEW · F46–F94')).toBeInTheDocument()
    expect(view.getByTitle('成功窗口 F68–F72')).toBeInTheDocument()
  })
})
