import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'
import { describe, it } from 'node:test'

import { discoverTimelineFixtures, runTimelineRegression } from '../timelines.js'

const repoRoot = fileURLToPath(new URL('../../..', import.meta.url))
const fixtures = discoverTimelineFixtures(resolve(repoRoot, 'tests', 'timelines'))

describe('timeline regression fixtures', () => {
  for (const fixture of fixtures) {
    it(fixture.name, () => runTimelineRegression(fixture, repoRoot))
  }
})
