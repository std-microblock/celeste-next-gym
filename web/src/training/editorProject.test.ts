import { describe, expect, it } from 'vitest'
import { PLAYGROUND } from '../model'
import { createTrainingProject, validateTrainingProject } from './editorProject'

describe('training editor projects', () => {
  it('creates a valid map-owned training project', () => {
    const project = createTrainingProject(PLAYGROUND)
    expect(project.training.version).toBe(2)
    expect(project.training.modules).toHaveLength(1)
    expect(validateTrainingProject(project)).toEqual([])
  })

  it('reports trigger and entry mistakes before invoking WASM', () => {
    const project = createTrainingProject(PLAYGROUND)
    project.training.finish.trigger.id = project.training.modules[0].trigger.id
    project.training.modules[0].tutorial.fuzz.inputs[0].at = 2
    const issues = validateTrainingProject(project)
    expect(issues.some((item) => item.path === 'finish.trigger.id')).toBe(true)
    expect(issues.some((item) => item.path.endsWith('entry.input_id'))).toBe(true)
  })
})
