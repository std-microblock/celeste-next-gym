import assert from 'node:assert/strict'
import { readdirSync, readFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { describe, it } from 'node:test'
import { fileURLToPath } from 'node:url'

import { scenarios } from './index.js'

type TechniqueStatus = 'implemented' | 'unimplemented'

interface TechniqueEvidence {
  readonly id: string
  readonly status: TechniqueStatus
  readonly e2eSymbols: readonly string[]
  readonly candidateSymbols: readonly string[]
}

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../../..')
const TECHNIQUE_ROOT = join(REPO_ROOT, 'docs', 'tech-handbook', 'techs')

function typstFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name)
    return entry.isDirectory() ? typstFiles(path) : entry.name.endsWith('.typ') ? [path] : []
  })
}

function evidenceSymbols(block: string | undefined): string[] {
  const symbol = block?.match(/symbol:\s*\[([^\]]*)\]/)?.[1] ?? ''
  return symbol.split(/\s*(?:;|\/)\s*/).map((value) => value.trim()).filter(Boolean)
}

function techniqueCatalog(): TechniqueEvidence[] {
  return typstFiles(TECHNIQUE_ROOT).map((path) => {
    const text = readFileSync(path, 'utf8')
    const id = text.match(/\bid:\s*"([^"]+)"/)?.[1]
    const status = text.match(/\bstatus:\s*"([^"]+)"/)?.[1]
    assert.ok(id, `${path}: missing technique id`)
    assert.ok(status === 'implemented' || status === 'unimplemented', `${path}: invalid status ${status}`)
    return {
      id,
      status,
      e2eSymbols: evidenceSymbols(text.match(/e2e-evidence:\s*evidence\(([\s\S]*?)\),\s*candidate-e2e:/)?.[1]),
      candidateSymbols: evidenceSymbols(text.match(/candidate-e2e:\s*evidence\(([\s\S]*?)\),?\s*\n\)/)?.[1]),
    }
  })
}

describe('authoritative technique recording metadata', () => {
  const techniques = techniqueCatalog()
  const byId = new Map(techniques.map((technique) => [technique.id, technique]))
  const byName = new Map(scenarios.map((scenario) => [scenario.name, scenario]))

  it('assigns exactly one primary to every implemented technique and none to unimplemented techniques', () => {
    assert.equal(techniques.length, 120)
    assert.equal(techniques.filter((technique) => technique.status === 'implemented').length, 52)
    assert.equal(techniques.filter((technique) => technique.status === 'unimplemented').length, 68)

    const primaryCounts = new Map<string, number>()
    for (const scenario of scenarios) {
      for (const techniqueId of scenario.techniqueIds) {
        assert.ok(byId.has(techniqueId), `${scenario.name}: unknown technique id ${techniqueId}`)
      }
      if (!scenario.recording) continue
      assert.equal(scenario.recording.startFrame, 0, `${scenario.name}: recording must preserve the full verified trace`)
      assert.equal(scenario.recording.endFrame, scenario.inputs.length, `${scenario.name}: recording must preserve the full verified trace`)
      for (const techniqueId of scenario.recording.primaryFor) {
        assert.ok(scenario.techniqueIds.includes(techniqueId), `${scenario.name}: primary is not a technique link`)
        primaryCounts.set(techniqueId, (primaryCounts.get(techniqueId) ?? 0) + 1)
      }
    }

    for (const technique of techniques) {
      assert.equal(primaryCounts.get(technique.id) ?? 0, technique.status === 'implemented' ? 1 : 0, technique.id)
    }
  })

  it('derives links and primary ownership from the ordered Typst evidence symbols', () => {
    for (const technique of techniques) {
      const symbols = technique.status === 'implemented' ? technique.e2eSymbols : technique.candidateSymbols
      const evidenceScenarios = symbols.map((symbol) => byName.get(symbol)).filter((scenario) => scenario !== undefined)
      if (technique.status === 'implemented') assert.ok(evidenceScenarios.length > 0, `${technique.id}: no E2E scenario symbol`)
      for (const scenario of evidenceScenarios) {
        assert.ok(scenario.techniqueIds.includes(technique.id), `${technique.id}: missing link on ${scenario.name}`)
      }
      const primaries = scenarios.filter((scenario) => scenario.recording?.primaryFor.includes(technique.id))
      if (technique.status === 'implemented') assert.deepEqual(primaries.map((scenario) => scenario.name), [evidenceScenarios[0]?.name])
      else assert.deepEqual(primaries, [], `${technique.id}: unimplemented technique has a primary`)
    }
  })
})
