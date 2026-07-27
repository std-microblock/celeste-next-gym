import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

export type TechniqueStatus = 'implemented' | 'unimplemented'

export interface TechniqueRecord {
  readonly id: string
  readonly status: TechniqueStatus
}

export interface TechniqueCatalog {
  readonly techniques: readonly TechniqueRecord[]
  readonly byId: ReadonlyMap<string, TechniqueRecord>
  readonly implementedIds: ReadonlySet<string>
}

export function loadTechniqueCatalog(repoRoot: string): TechniqueCatalog {
  const root = join(repoRoot, 'docs', 'tech-handbook', 'techs')
  const techniques = typstFiles(root).map((file) => parseTechnique(file))
    .sort((left, right) => compareTechniqueIds(left.id, right.id))
  const byId = new Map<string, TechniqueRecord>()
  for (const technique of techniques) {
    if (byId.has(technique.id)) throw new Error(`duplicate technique id in handbook: ${technique.id}`)
    byId.set(technique.id, technique)
  }
  const implementedIds = new Set(techniques.filter((technique) => technique.status === 'implemented').map((technique) => technique.id))
  if (techniques.length !== 120) throw new Error(`unexpected handbook coverage count: ${techniques.length} total`)
  return Object.freeze({ techniques: Object.freeze(techniques), byId, implementedIds })
}

function typstFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name)
    return entry.isDirectory() ? typstFiles(path) : entry.name.endsWith('.typ') ? [path] : []
  })
}

function parseTechnique(file: string): TechniqueRecord {
  const text = readFileSync(file, 'utf8')
  const id = text.match(/\bid:\s*"([^"]+)"/)?.[1]
  const status = text.match(/\bstatus:\s*"([^"]+)"/)?.[1]
  if (!id) throw new Error(`${file}: missing technique id`)
  if (status !== 'implemented' && status !== 'unimplemented') throw new Error(`${file}: invalid technique status ${status ?? '(missing)'}`)
  return { id, status }
}

function compareTechniqueIds(left: string, right: string): number {
  const leftParts = left.split('.').map(Number)
  const rightParts = right.split('.').map(Number)
  for (let index = 0; index < Math.max(leftParts.length, rightParts.length); index++) {
    const difference = (leftParts[index] ?? -1) - (rightParts[index] ?? -1)
    if (difference !== 0) return difference
  }
  return left.localeCompare(right)
}
