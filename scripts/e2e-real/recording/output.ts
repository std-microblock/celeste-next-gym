import path from 'node:path'

import type { ScenarioDefinition } from '../types.js'
import { createArtifactEntry, type RecordingArtifactEntry } from './artifacts.js'
import { encodePresentationRecording } from './encoder.js'
import type { RecordingScenarioPlan } from './plan.js'

export async function encodeScenarioArtifacts(options: {
  readonly item: RecordingScenarioPlan
  readonly manifestPath: string
  readonly tracePath: string
  readonly recordingRoot: string
  readonly ffmpegPath: string
  readonly ffprobePath: string
}, dependencies: {
  readonly encode?: typeof encodePresentationRecording
  readonly createEntry?: typeof createArtifactEntry
} = {}): Promise<RecordingArtifactEntry[]> {
  const encode = dependencies.encode ?? encodePresentationRecording
  const makeEntry = dependencies.createEntry ?? createArtifactEntry
  const stateWindow = resolveRecordingWindow(options.item.scenario)
  const media = { ffmpegPath: options.ffmpegPath, ffprobePath: options.ffprobePath }
  const master = await encode({
    recordingRoot: options.recordingRoot, manifestPath: options.manifestPath,
    outputPath: path.resolve(options.manifestPath, '..', 'master.mp4'), ...media,
  })
  const result: RecordingArtifactEntry[] = [await makeEntry({
    recordingRoot: options.recordingRoot, kind: 'scenario-master', scenarioId: options.item.scenario.name,
    stateSpan: { start_state_index: 0, end_state_index: options.item.scenario.inputs.length },
    rawManifestPath: options.manifestPath, tracePath: options.tracePath, encoding: master,
  })]
  for (const techniqueId of options.item.techniqueIds) {
    const techniqueDirectory = path.resolve(options.recordingRoot, 'techniques', techniqueId)
    const clip = await encode({
      recordingRoot: options.recordingRoot, manifestPath: options.manifestPath,
      outputPath: path.resolve(techniqueDirectory, `${options.item.scenario.name}.mp4`),
      posterPath: path.resolve(techniqueDirectory, `${options.item.scenario.name}.poster.png`),
      stateWindow: { startStateIndex: stateWindow.start_state_index, endStateIndex: stateWindow.end_state_index },
      ...media,
    })
    result.push(await makeEntry({
      recordingRoot: options.recordingRoot, kind: 'technique-clip', scenarioId: options.item.scenario.name, techniqueId,
      stateSpan: stateWindow, rawManifestPath: options.manifestPath, tracePath: options.tracePath, encoding: clip,
    }))
  }
  return result
}

export function resolveRecordingWindow(scenario: ScenarioDefinition): { start_state_index: number; end_state_index: number } {
  const recording = scenario.recording
  if (recording?.startFrame !== undefined) {
    return { start_state_index: recording.startFrame, end_state_index: Math.min(recording.endFrame, scenario.inputs.length) }
  }
  return { start_state_index: 0, end_state_index: scenario.inputs.length }
}
