import { render, screen, within } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { TrainingResultTimeline, type TrainingObjectiveSeries } from './TrainingTimeline'

const objectives: TrainingObjectiveSeries[] = [{
  expression: 'final.speed.x',
  points: [
    { frame: 0, value: 325, successful: true },
    { frame: 1, value: 325, successful: true },
    { frame: 2, value: 191.67, successful: false },
  ],
}]

describe('training result timeline', () => {
  it('layers the objective curve, feasible window, and timing markers without point glyphs', () => {
    const { container } = render(<TrainingResultTimeline
      targetFrame={0}
      windows={[{ from: 0, to: 1 }]}
      actualInputs={[{ frame: 0, keys: ['dash'] }, { frame: 1, keys: ['jump'] }]}
      objectives={objectives}
    />)

    expect(container.querySelector('.training-objective-curve polyline')).toBeInTheDocument()
    expect(container.querySelector('.training-window')).toBeInTheDocument()
    expect(container.querySelectorAll('circle')).toHaveLength(0)
    expect(container.querySelectorAll('.training-result-axis-end')).toHaveLength(0)
    expect(screen.getByText('最佳 F0')).toBeInTheDocument()
    expect(screen.getByText('F0 DASH')).toBeInTheDocument()
    expect(screen.getByText('F1 JUMP')).toBeInTheDocument()
  })

  it('exposes per-frame Fuzzer speed and success explanation through custom hover details', () => {
    const { container } = render(<TrainingResultTimeline
      windows={[{ from: 0, to: 1 }]}
      actualInputs={[]}
      objectives={objectives}
    />)

    const timeline = within(container)
    expect(timeline.getAllByText('F0')).toHaveLength(1)
    expect(timeline.getByLabelText('F2 未通过；final.speed.x 191.67')).toBeInTheDocument()
    expect(timeline.getByText('191.67')).toBeInTheDocument()
    expect(timeline.queryByText('终态未满足成功条件')).not.toBeInTheDocument()

    const hit = container.querySelector<HTMLElement>('.training-objective-hit')
    expect(Number.parseFloat(hit?.style.width ?? '')).toBeCloseTo(100 / 22)
  })
})
