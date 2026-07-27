import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { mkdir, mkdtemp, readFile, rm, stat, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { afterEach, describe, it } from 'node:test'

import { parseConfig } from '../config.js'
import { buildRegistry } from '../registry.js'
import { scenarios } from '../scenarios/index.js'
import { collectorOwnershipEnvironment, runRecordingHarness } from '../runtime/runner.js'
import { validateRecordingStatus, type RecordingStatus } from '../runtime/collector-client.js'
import { captureScenario } from './runner.js'
import { createRecordingPlan } from './plan.js'
import { loadTechniqueCatalog } from './techniques.js'
import { RAW_FRAME_BYTES, type PresentationManifest } from './manifest.js'
import { encodeScenarioArtifacts } from './output.js'
import { createArtifactEntry, writeArtifactManifest, type RecordingArtifactEntry } from './artifacts.js'
import type { EncodingResult } from './encoder.js'

const roots: string[] = []
const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..')

afterEach(async () => await Promise.all(roots.splice(0).map(async (root) => await rm(root, { recursive: true, force: true }))))

describe('recording planning and lifecycle orchestration', () => {
  it('passes the authenticated game ownership into the Everest collector backend', () => {
    assert.deepEqual(collectorOwnershipEnvironment('nonce-1', 42), {
      EVEREST_RUN_NONCE: 'nonce-1',
      EVEREST_PROCESS_ID: '42',
    })
    assert.throws(() => collectorOwnershipEnvironment('', 42), /run nonce/)
    assert.throws(() => collectorOwnershipEnvironment('nonce-1', 0), /positive process id/)
  })

  it('validates all 69 handbook primaries before running one isolated lifecycle per target', async () => {
    const config = parseConfig(['--record-all'], {
      FFMPEG_PATH: path.resolve(repoRoot, 'fake-ffmpeg'),
      FFPROBE_PATH: path.resolve(repoRoot, 'fake-ffprobe'),
    }, repoRoot)
    const catalog = loadTechniqueCatalog(repoRoot)
    const registry = buildRegistry(scenarios, { implementedTechniqueIds: catalog.implementedIds })
    const plan = createRecordingPlan(config, registry, catalog)
    assert.equal(plan.techniqueCount, 69)
    assert.ok(plan.scenarioCount <= 69)
    assert.deepEqual(plan.groups.map((group) => group.target.id), ['playground', 'area-1', 'area-2'])

    const calls: string[] = []
    const summary = await runRecordingHarness(config, plan, {
      runTarget: async (_config, selected, group) => {
        calls.push(group!.target.id)
        assert.deepEqual(selected, group!.scenarios.map((item) => item.scenario))
        return { health: {}, scenarios: [] }
      },
    })
    assert.deepEqual(calls, ['playground', 'area-1', 'area-2'])
    assert.equal(summary.techniqueCount, 69)
  })

  it('rejects unknown, unimplemented, and candidate selections while deduplicating multi-primary scenarios', () => {
    const catalog = loadTechniqueCatalog(repoRoot)
    const registry = buildRegistry(scenarios, { implementedTechniqueIds: catalog.implementedIds })
    const unknown = parseConfig(['--record-tech', '999.999'], {}, repoRoot)
    assert.throws(() => createRecordingPlan(unknown, registry, catalog), /unknown technique/)
    const unimplementedId = catalog.techniques.find((technique) => technique.status === 'unimplemented')!.id
    const unimplemented = parseConfig(['--record-tech', unimplementedId], {}, repoRoot)
    assert.throws(() => createRecordingPlan(unimplemented, registry, catalog), /unimplemented/)
    const candidate = registry.scenarios.find((scenario) => scenario.status === 'candidate')!
    const candidateConfig = parseConfig(['--record', '--scenario', candidate.name], {}, repoRoot)
    assert.throws(() => createRecordingPlan(candidateConfig, registry, catalog), /candidate|excluded/)

    const base = scenarios.find((scenario) => scenario.status === 'active')!
    const shared = { ...base, name: 'shared-primary-test', techniqueIds: ['a', 'b'],
      recording: { primaryFor: ['a', 'b'], startFrame: 0, endFrame: base.inputs.length } } as typeof base
    const sharedCatalog = {
      techniques: [{ id: 'a', status: 'implemented' }, { id: 'b', status: 'implemented' }] as const,
      byId: new Map([['a', { id: 'a', status: 'implemented' }], ['b', { id: 'b', status: 'implemented' }]] as const),
      implementedIds: new Set(['a', 'b']),
    }
    const sharedPlan = createRecordingPlan(
      parseConfig(['--record-tech', 'a', '--record-tech', 'b'], {}, repoRoot),
      buildRegistry([shared], { implementedTechniqueIds: sharedCatalog.implementedIds }),
      sharedCatalog,
    )
    assert.equal(sharedPlan.scenarioCount, 1)
    assert.equal(sharedPlan.techniqueCount, 2)
    assert.deepEqual(sharedPlan.groups[0]!.scenarios[0]!.techniqueIds, ['a', 'b'])
  })
})

describe('recording service state machine', () => {
  it('uses one token for start and simulate, then finalizes and validates the owned raw manifest', async () => {
    const token = 'a'.repeat(32)
    const fixture = await createRawFixture(token)
    const events: string[] = []
    const service = fakeService(path.relative(fixture.root, fixture.manifestPath), events, 'ready')
    const result = await captureScenario({
      service, recordingRoot: fixture.root, scenarioId: 'scenario-1', endStateIndex: 1,
      runNonce: 'nonce-1', gameProcessId: 42, timeoutMs: 1000, pollTimeoutMs: 1000,
      createToken: () => token,
      execute: async (received) => { events.push(`simulate:${received}`); return 7 },
    })
    assert.equal(result.execution, 7)
    assert.deepEqual(events, [`start:${token}`, `simulate:${token}`, 'status', 'finalize'])
  })

  it('rejects a raw manifest without the one-second final-state tail', async () => {
    const token = 'd'.repeat(32)
    const fixture = await createRawFixture(token)
    const manifest = JSON.parse(await readFile(fixture.manifestPath, 'utf8')) as PresentationManifest
    manifest.frames.splice(1)
    manifest.repeated_presentation_count = 0
    await writeFile(fixture.manifestPath, JSON.stringify(manifest), 'utf8')

    await assert.rejects(() => captureScenario({
      service: fakeService(fixture.manifestPath, [], 'ready'),
      recordingRoot: fixture.root, scenarioId: 'scenario-1', endStateIndex: 1,
      runNonce: 'nonce-1', gameProcessId: 42, timeoutMs: 1000, pollTimeoutMs: 1000,
      createToken: () => token, execute: async () => undefined,
    }), /one-second final-state tail/)
  })

  it('stops and finalizes on poll timeout while preserving the finalized manifest', async () => {
    const token = 'b'.repeat(32)
    const fixture = await createRawFixture(token)
    const events: string[] = []
    const service = fakeService(fixture.manifestPath, events, 'active')
    const times = [0, 101]
    await captureScenario({
      service, recordingRoot: fixture.root, scenarioId: 'scenario-1', endStateIndex: 1,
      runNonce: 'nonce-1', gameProcessId: 42, timeoutMs: 1000, pollTimeoutMs: 100,
      createToken: () => token, now: () => times.shift() ?? 101, sleep: async () => undefined,
      execute: async () => undefined,
    })
    assert.deepEqual(events, [`start:${token}`, 'status', 'stop:runner poll timeout', 'finalize'])
  })

  it('attempts finalize after a failed stop and preserves the scenario error', async () => {
    const events: string[] = []
    const status: RecordingStatus = {
      state: 'active', scenario_id: 'scenario-1', start_state_index: 0, end_state_index: 1,
      latest_state_index: -1, render_frame_count: 0, final_state_presented: false,
      repeated_presentation_count: 0, unpresented_update_ranges: [],
    }
    await assert.rejects(() => captureScenario({
      service: {
        recordingStart: async () => status,
        recordingStatus: async () => status,
        recordingStop: async () => { events.push('stop'); throw new Error('stop failed') },
        recordingFinalize: async () => { events.push('finalize'); throw new Error('finalize failed') },
      },
      recordingRoot: repoRoot, scenarioId: 'scenario-1', endStateIndex: 1,
      runNonce: 'nonce-1', gameProcessId: 42, timeoutMs: 1000, pollTimeoutMs: 1000,
      createToken: () => 'c'.repeat(32), execute: async () => { throw new Error('scenario failed') },
    }), /scenario failed/)
    assert.deepEqual(events, ['stop', 'finalize'])
  })

  it('rejects negative counters and reversed ranges in collector responses', () => {
    const valid = {
      state: 'active', scenario_id: 'scenario-1', start_state_index: 0, end_state_index: 1,
      latest_state_index: -1, render_frame_count: 0, final_state_presented: false,
      repeated_presentation_count: 0, unpresented_update_ranges: [],
    }
    assert.throws(() => validateRecordingStatus({ ...valid, render_frame_count: -1 }), /negative/)
    assert.throws(() => validateRecordingStatus({ ...valid, unpresented_update_ranges: [{ start_state_index: 2, end_state_index: 1 }] }), /reversed/)
  })
})

describe('recording outputs', () => {
  it('captures once but emits one windowed clip and poster per primary technique', async () => {
    const root = await mkdtemp(path.join(tmpdir(), 'recording-output-'))
    roots.push(root)
    const base = scenarios.find((scenario) => scenario.inputs.length >= 4)!
    const scenario = { ...base, name: 'multi-output', techniqueIds: ['a', 'b'],
      recording: { primaryFor: ['a', 'b'], startFrame: 2, endFrame: 4 } } as typeof base
    const calls: Array<{ outputPath: string; posterPath?: string; stateWindow?: unknown }> = []
    const fakeResult: EncodingResult = {
      output_path: path.join(root, 'fake.mp4'), sha256: 'a'.repeat(64), bytes: 1,
      probe: { codec: 'h264', pixel_format: 'yuv420p', width: 320, height: 180, frame_rate: '60/1', frame_count: 3 },
      processes: [],
    }
    const artifacts = await encodeScenarioArtifacts({
      item: { scenario, techniqueIds: ['a', 'b'] }, manifestPath: path.join(root, 'raw', 'manifest.json'),
      tracePath: path.join(root, 'raw', 'trace.json'), recordingRoot: root,
      ffmpegPath: path.join(root, 'ffmpeg'), ffprobePath: path.join(root, 'ffprobe'),
    }, {
      encode: async (options) => { calls.push(options); return fakeResult },
      createEntry: async (entry) => ({ ...entry, raw_manifest: {} as never, trace: {} as never,
        video: {} as never, media_processes: [] }) as unknown as RecordingArtifactEntry,
    })
    assert.equal(artifacts.length, 3)
    assert.equal(calls.length, 3)
    assert.equal(calls[0]!.posterPath, undefined)
    for (const [index, technique] of ['a', 'b'].entries()) {
      const call = calls[index + 1]!
      assert.deepEqual(call.stateWindow, { startStateIndex: 2, endStateIndex: 4 })
      assert.ok(call.outputPath.endsWith(path.join('techniques', technique, 'multi-output.mp4')))
      assert.ok(call.posterPath!.endsWith(path.join('techniques', technique, 'multi-output.poster.png')))
    }
  })

  it('rejects artifact paths outside the recording root and atomically writes the manifest', async () => {
    const root = await mkdtemp(path.join(tmpdir(), 'recording-artifacts-'))
    const outside = await mkdtemp(path.join(tmpdir(), 'recording-outside-'))
    roots.push(root, outside)
    const outsideFile = path.join(outside, 'trace.json')
    await writeFile(outsideFile, '{}')
    await assert.rejects(() => createArtifactEntry({
      recordingRoot: root, kind: 'scenario-master', scenarioId: 'scenario-1',
      stateSpan: { start_state_index: 0, end_state_index: 1 }, rawManifestPath: outsideFile,
      tracePath: outsideFile, encoding: { output_path: outsideFile, sha256: 'a'.repeat(64), bytes: 2,
        probe: { codec: 'h264', pixel_format: 'yuv420p', width: 320, height: 180, frame_rate: '60/1' }, processes: [] },
    }), /escapes/)
    const manifestPath = await writeArtifactManifest({
      recordingRoot: root, runNonce: 'nonce',
      gameProcess: { processId: 42, executablePath: 'game.exe', creationTimeUtc: new Date(0).toISOString() }, artifacts: [],
    })
    assert.equal(JSON.parse(await readFile(manifestPath, 'utf8')).schema_version, 1)
    await assert.rejects(() => stat(`${manifestPath}.tmp`))
  })
})

function fakeService(manifestPath: string, events: string[], statusState: 'active' | 'ready') {
  const status = (state: RecordingStatus['state']): RecordingStatus => ({
    state, scenario_id: 'scenario-1', start_state_index: 0, end_state_index: 1,
    latest_state_index: 1, render_frame_count: 1, final_state_presented: true,
    repeated_presentation_count: 0, unpresented_update_ranges: [],
    ...(state === 'finalized' ? { manifest_path: manifestPath } : {}),
  })
  return {
    recordingStart: async (request: { capture_token: string }) => { events.push(`start:${request.capture_token}`); return status('active') },
    recordingStatus: async () => { events.push('status'); return status(statusState) },
    recordingStop: async (request: { reason?: string }) => { events.push(`stop:${request.reason}`); return status('stopped') },
    recordingFinalize: async () => { events.push('finalize'); return status('finalized') },
  }
}

async function createRawFixture(token: string): Promise<{ root: string; manifestPath: string }> {
  const root = await mkdtemp(path.join(tmpdir(), 'recording-wiring-'))
  roots.push(root)
  const session = path.join(root, 'scenarios', 'scenario-1', token)
  const frames = path.join(session, 'frames')
  await mkdir(frames, { recursive: true })
  const raw = Buffer.alloc(RAW_FRAME_BYTES)
  const presentationFrames = Array.from({ length: 61 }, (_, renderIndex) => ({
    render_index: renderIndex,
    state_index: 1,
    timestamp_ns: renderIndex * 16_666_667,
    path: `frames/${renderIndex.toString().padStart(6, '0')}.bgra`,
    sha256: createHash('sha256').update(raw).digest('hex'),
    bytes: raw.byteLength,
    repeated_state_presentation: renderIndex > 0,
  }))
  await Promise.all(presentationFrames.map(async (frame) => {
    await writeFile(path.join(session, frame.path), raw)
  }))
  const manifest: PresentationManifest = {
    schema_version: 1, capture_semantics: 'presentation_frames', scenario_id: 'scenario-1',
    run_nonce: 'nonce-1', process_id: 42,
    capture_token_sha256: createHash('sha256').update(token).digest('hex'),
    width: 320, height: 180, pixel_format: 'bgra', encoding_frame_rate: 60,
    started_at: new Date(0).toISOString(), finalized_at: new Date(1).toISOString(), outcome: 'ready',
    start_state_index: 0, end_state_index: 1, latest_state_index: 1, final_state_presented: true,
    repeated_presentation_count: 60, unpresented_update_ranges: [],
    frames: presentationFrames,
  }
  const manifestPath = path.join(session, 'manifest.json')
  await writeFile(manifestPath, JSON.stringify(manifest), 'utf8')
  return { root, manifestPath }
}
