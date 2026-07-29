import type {
  MapPart,
  ScenarioDefinition,
  ScenarioStatus,
  TargetId,
} from "./types.js";

export interface ScenarioRegistry {
  readonly scenarios: readonly ScenarioDefinition[];
  readonly byName: ReadonlyMap<string, ScenarioDefinition>;
  readonly byTarget: ReadonlyMap<TargetId, readonly ScenarioDefinition[]>;
  readonly counts: Readonly<Record<ScenarioStatus, number>>;
  readonly mapParts: ReadonlyMap<string, MapPart>;
}

export function buildRegistry(
  definitions: readonly ScenarioDefinition[],
  options: { readonly implementedTechniqueIds?: ReadonlySet<string> } = {},
): ScenarioRegistry {
  const byName = new Map<string, ScenarioDefinition>();
  const byTargetMutable = new Map<TargetId, ScenarioDefinition[]>();
  const mapParts = new Map<string, MapPart>();
  const counts: Record<ScenarioStatus, number> = { active: 0, candidate: 0 };
  const implementedTechniqueIds =
    options.implementedTechniqueIds ??
    new Set(
      definitions
        .filter((scenario) => scenario.status === "active")
        .flatMap((scenario) => scenario.techniqueIds),
    );
  const primaryByTechnique = new Map<string, string>();

  for (const scenario of definitions) {
    if (byName.has(scenario.name))
      throw new Error(`duplicate scenario name: ${scenario.name}`);
    byName.set(scenario.name, scenario);
    validateScenarioMetadata(
      scenario,
      implementedTechniqueIds,
      primaryByTechnique,
    );
    counts[scenario.status]++;
    const targetScenarios = byTargetMutable.get(scenario.target.id) ?? [];
    targetScenarios.push(scenario);
    byTargetMutable.set(scenario.target.id, targetScenarios);
    for (const part of scenario.mapParts) {
      const previous = mapParts.get(part.id);
      if (
        previous &&
        previous !== part &&
        JSON.stringify(previous) !== JSON.stringify(part)
      ) {
        throw new Error(`conflicting map part stable id: ${part.id}`);
      }
      mapParts.set(part.id, part);
    }
  }

  const scenarios = Object.freeze(
    [...definitions].sort((a, b) => a.name.localeCompare(b.name)),
  );
  const byTarget = new Map<TargetId, readonly ScenarioDefinition[]>(
    [...byTargetMutable].map(([target, values]) => [
      target,
      Object.freeze([...values].sort((a, b) => a.name.localeCompare(b.name))),
    ]),
  );
  return Object.freeze({
    scenarios,
    byName,
    byTarget,
    counts: Object.freeze(counts),
    mapParts,
  });
}

function validateScenarioMetadata(
  scenario: ScenarioDefinition,
  implementedTechniqueIds: ReadonlySet<string>,
  primaryByTechnique: Map<string, string>,
): void {
  if (new Set(scenario.techniqueIds).size !== scenario.techniqueIds.length) {
    throw new Error(`${scenario.name}: duplicate techniqueIds`);
  }
  const recording = scenario.recording;
  if (!recording) return;
  if (recording.primaryFor.length === 0)
    throw new Error(`${scenario.name}: recording.primaryFor must not be empty`);
  if (
    recording.posterFrame !== undefined &&
    (!Number.isSafeInteger(recording.posterFrame) ||
      recording.posterFrame < 0 ||
      recording.posterFrame > scenario.inputs.length)
  ) {
    throw new Error(`${scenario.name}: invalid recording poster frame`);
  }
  for (const techniqueId of recording.primaryFor) {
    if (!scenario.techniqueIds.includes(techniqueId)) {
      throw new Error(
        `${scenario.name}: recording primary ${techniqueId} is not listed in techniqueIds`,
      );
    }
    if (!implementedTechniqueIds.has(techniqueId)) continue;
    if (scenario.status === "candidate") {
      throw new Error(
        `${scenario.name}: candidate cannot be primary recording for implemented technique ${techniqueId}`,
      );
    }
    const previous = primaryByTechnique.get(techniqueId);
    if (previous)
      throw new Error(
        `implemented technique ${techniqueId} has multiple primary recordings: ${previous}, ${scenario.name}`,
      );
    primaryByTechnique.set(techniqueId, scenario.name);
  }
  if (recording.startFrame !== undefined) {
    const { startFrame, endFrame } = recording;
    if (
      endFrame === undefined ||
      !Number.isSafeInteger(startFrame) ||
      !Number.isSafeInteger(endFrame) ||
      startFrame < 0 ||
      endFrame < startFrame
    ) {
      throw new Error(
        `${scenario.name}: invalid absolute recording frame window`,
      );
    }
    if (
      recording.posterFrame !== undefined &&
      (recording.posterFrame < startFrame || recording.posterFrame > endFrame)
    ) {
      throw new Error(
        `${scenario.name}: recording poster frame is outside the recording window`,
      );
    }
  } else if (
    !Number.isSafeInteger(recording.preRollFrames) ||
    !Number.isSafeInteger(recording.postRollFrames) ||
    recording.preRollFrames < 0 ||
    recording.postRollFrames < 0
  ) {
    throw new Error(`${scenario.name}: invalid recording pre/post-roll window`);
  }
}

export function selectScenarios(
  registry: ScenarioRegistry,
  options: {
    readonly names?: ReadonlySet<string>;
    readonly target?: TargetId;
    readonly includeCandidates?: boolean;
    readonly disabledTags?: ReadonlySet<string>;
  },
): readonly ScenarioDefinition[] {
  if (options.names) {
    const missing = [...options.names].filter(
      (name) => !registry.byName.has(name),
    );
    if (missing.length > 0)
      throw new Error(`unknown E2E scenarios: ${missing.sort().join(", ")}`);
  }
  const selected = registry.scenarios.filter((scenario) => {
    if (options.names && !options.names.has(scenario.name)) return false;
    if (options.target && scenario.target.id !== options.target) return false;
    if (!options.includeCandidates && scenario.status === "candidate")
      return false;
    if (
      options.disabledTags &&
      [...options.disabledTags].some((tag) => scenario.tags.includes(tag))
    )
      return false;
    return true;
  });
  if (options.names) {
    const rejected = [...options.names].filter(
      (name) => !selected.some((scenario) => scenario.name === name),
    );
    if (rejected.length > 0) {
      throw new Error(
        `requested scenarios are excluded by target, candidate, or feature filters: ${rejected.sort().join(", ")}`,
      );
    }
  }
  return Object.freeze(selected);
}
