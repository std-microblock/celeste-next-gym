import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

import { parseConfig } from "./config.js";
import { assembleFixturePackage } from "./map-parts.js";
import { buildRegistry, selectScenarios } from "./registry.js";
import { scenarios } from "./scenarios/index.js";
import { runHarness } from "./runtime/runner.js";
import {
  runRecordingHarness,
  type HarnessSummary,
  type RecordingHarnessSummary,
} from "./runtime/runner.js";
import {
  createRecordingPlan,
  loadTechniqueCatalog,
  type RecordingPlan,
} from "./recording/index.js";
import { discoverTimelineFixtures } from "./timelines.js";
import type { HarnessConfig } from "./config.js";
import type { ScenarioDefinition } from "./types.js";

export interface MainDependencies {
  run(
    config: HarnessConfig,
    selected: readonly ScenarioDefinition[],
  ): Promise<HarnessSummary>;
  runRecording?(
    config: HarnessConfig,
    plan: RecordingPlan,
  ): Promise<RecordingHarnessSummary>;
  planRecording?(
    config: HarnessConfig,
    registry: ReturnType<typeof buildRegistry>,
    catalog: ReturnType<typeof loadTechniqueCatalog>,
  ): RecordingPlan;
}

export async function main(
  argv: readonly string[],
  env: NodeJS.ProcessEnv,
  repoRoot: string,
  dependencies: MainDependencies = { run: runHarness },
): Promise<void> {
  const config = parseConfig(argv, env, repoRoot);
  const catalog = config.recording ? loadTechniqueCatalog(repoRoot) : undefined;
  const registry = buildRegistry(
    scenarios,
    catalog ? { implementedTechniqueIds: catalog.implementedIds } : {},
  );
  if (config.recording) {
    const plan = (dependencies.planRecording ?? createRecordingPlan)(
      config,
      registry,
      catalog!,
    );
    const summary = await (dependencies.runRecording ?? runRecordingHarness)(
      config,
      plan,
    );
    console.log(JSON.stringify(summary, null, 2));
    return;
  }
  const requestedScenarios = config.timelineRegressions
    ? new Set(
        discoverTimelineFixtures(resolve(repoRoot, "tests", "timelines")).map(
          (fixture) => fixture.e2eScenario,
        ),
      )
    : config.requestedScenarios;
  const selected = selectScenarios(registry, {
    ...(requestedScenarios.size > 0 ? { names: requestedScenarios } : {}),
    ...(config.target && (!config.timelineRegressions || config.targetExplicit)
      ? { target: config.target }
      : {}),
    includeCandidates: config.includeCandidates,
    disabledTags: config.disabledTags,
  });

  if (config.listOnly) {
    console.log(
      JSON.stringify(
        { counts: registry.counts, scenarios: selected.map(describeScenario) },
        null,
        2,
      ),
    );
    return;
  }
  if (config.fixtureOutput) {
    const requestedParts = [
      ...new Map(
        selected
          .flatMap((scenario) => scenario.mapParts)
          .map((part) => [part.id, part]),
      ).values(),
    ];
    const fixture = assembleFixturePackage(requestedParts, registry.mapParts);
    mkdirSync(dirname(config.fixtureOutput), { recursive: true });
    writeFileSync(
      config.fixtureOutput,
      `${JSON.stringify(fixture, null, 2)}\n`,
      "utf8",
    );
    console.log(
      JSON.stringify(
        {
          fixture: config.fixtureOutput,
          scenarios: selected.length,
          parts: requestedParts.length,
        },
        null,
        2,
      ),
    );
    return;
  }

  const summaries: HarnessSummary[] = [];
  const groups = config.timelineRegressions
    ? groupByTarget(selected)
    : [selected];
  for (const group of groups)
    summaries.push(await dependencies.run(config, group));
  const summary =
    summaries.length === 1
      ? summaries[0]
      : {
          health: { targets: summaries.map((item) => item.health) },
          scenarios: summaries.flatMap((item) => item.scenarios),
        };
  console.log(JSON.stringify(summary, null, 2));
}

function groupByTarget(
  selected: readonly ScenarioDefinition[],
): readonly (readonly ScenarioDefinition[])[] {
  const byTarget = new Map<string, ScenarioDefinition[]>();
  for (const scenario of selected) {
    const group = byTarget.get(scenario.target.id) ?? [];
    if (group.length === 0) byTarget.set(scenario.target.id, group);
    group.push(scenario);
  }
  return [...byTarget.values()];
}

function describeScenario(
  scenario: (typeof scenarios)[number],
): Record<string, unknown> {
  return {
    name: scenario.name,
    target: scenario.target.id,
    status: scenario.status,
    tags: scenario.tags,
    techniqueIds: scenario.techniqueIds,
    mapParts: scenario.mapParts.map((part: { readonly id: string }) => part.id),
  };
}
