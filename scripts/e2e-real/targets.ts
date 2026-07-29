import { PLAYGROUND_SID } from "./constants.js";
import type {
  ExternalMapTarget,
  PlaygroundTarget,
  ScenarioTarget,
  TargetId,
} from "./types.js";

export const PLAYGROUND_TARGET: PlaygroundTarget = Object.freeze({
  id: "playground",
  kind: "playground",
  areaId: 1,
  areaSid: PLAYGROUND_SID,
});
export const AREA_1_TARGET: ExternalMapTarget = Object.freeze({
  id: "area-1",
  kind: "external",
  areaId: 1,
  defaultMapFile: "1-ForsakenCity.bin",
});
export const AREA_2_TARGET: ExternalMapTarget = Object.freeze({
  id: "area-2",
  kind: "external",
  areaId: 2,
  defaultMapFile: "2-OldSite.bin",
});
export const AREA_4_TARGET: ExternalMapTarget = Object.freeze({
  id: "area-4",
  kind: "external",
  areaId: 4,
  defaultMapFile: "4-GoldenRidge.bin",
});

export const TARGETS: Readonly<Record<TargetId, ScenarioTarget>> =
  Object.freeze({
    playground: PLAYGROUND_TARGET,
    "area-1": AREA_1_TARGET,
    "area-2": AREA_2_TARGET,
    "area-4": AREA_4_TARGET,
  });

export function targetForArea(
  areaId: number,
  areaSid?: string,
): ScenarioTarget | undefined {
  if (areaSid === PLAYGROUND_SID) return PLAYGROUND_TARGET;
  return Object.values(TARGETS).find(
    (target) => target.kind === "external" && target.areaId === areaId,
  );
}
