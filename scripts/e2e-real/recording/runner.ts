import { createHash, randomBytes } from "node:crypto";

import type {
  RecordingControlRequest,
  RecordingStartRequest,
  RecordingStatus,
} from "../runtime/collector-client.js";
import {
  loadPresentationManifest,
  type LoadedPresentationManifest,
} from "./manifest.js";

const FINAL_STATE_TAIL_PRESENTATION_FRAMES = 60;

export interface RecordingService {
  recordingStart(request: RecordingStartRequest): Promise<RecordingStatus>;
  recordingStatus(request: RecordingControlRequest): Promise<RecordingStatus>;
  recordingStop(request: RecordingControlRequest): Promise<RecordingStatus>;
  recordingFinalize(request: RecordingControlRequest): Promise<RecordingStatus>;
}

export interface CaptureResult<T> {
  readonly captureToken: string;
  readonly execution: T;
  readonly status: RecordingStatus;
  readonly presentation: LoadedPresentationManifest;
}

export async function captureScenario<T>(options: {
  readonly service: RecordingService;
  readonly recordingRoot: string;
  readonly scenarioId: string;
  readonly endStateIndex: number;
  readonly runNonce: string;
  readonly gameProcessId: number;
  readonly timeoutMs: number;
  readonly pollTimeoutMs: number;
  readonly execute: (captureToken: string) => Promise<T>;
  readonly createToken?: () => string;
  readonly sleep?: (milliseconds: number) => Promise<void>;
  readonly now?: () => number;
}): Promise<CaptureResult<T>> {
  const captureToken = (
    options.createToken ?? (() => randomBytes(32).toString("base64url"))
  )();
  const control = { capture_token: captureToken };
  const started = await options.service.recordingStart({
    capture_token: captureToken,
    scenario_id: options.scenarioId,
    start_state_index: 0,
    end_state_index: options.endStateIndex,
    timeout_ms: options.timeoutMs,
  });
  assertStatusIdentity(started, options.scenarioId, 0, options.endStateIndex);
  if (started.state !== "active")
    throw new Error(`recording start returned ${started.state}`);

  let execution: T;
  try {
    execution = await options.execute(captureToken);
  } catch (error) {
    await stopAndFinalize(
      options.service,
      control,
      "scenario execution failed",
    ).catch(() => undefined);
    throw error;
  }

  const sleep =
    options.sleep ??
    (async (milliseconds) =>
      await new Promise<void>((resolve) => setTimeout(resolve, milliseconds)));
  const now = options.now ?? Date.now;
  const deadline = now() + options.pollTimeoutMs;
  let status = await options.service.recordingStatus(control);
  assertStatusIdentity(status, options.scenarioId, 0, options.endStateIndex);
  while (status.state === "active" && now() < deadline) {
    await sleep(100);
    status = await options.service.recordingStatus(control);
    assertStatusIdentity(status, options.scenarioId, 0, options.endStateIndex);
  }
  if (status.state === "active") {
    status = await options.service.recordingStop({
      ...control,
      reason: "runner poll timeout",
    });
    assertStatusIdentity(status, options.scenarioId, 0, options.endStateIndex);
  }
  if (status.state === "faulted") {
    await options.service.recordingFinalize(control).catch(() => undefined);
    throw new Error(`recording faulted: ${status.reason ?? "unknown reason"}`);
  }
  const finalized = await options.service.recordingFinalize(control);
  assertStatusIdentity(finalized, options.scenarioId, 0, options.endStateIndex);
  if (finalized.state !== "finalized" || !finalized.manifest_path) {
    throw new Error(
      `recording finalize returned ${finalized.state} without manifest_path`,
    );
  }
  const presentation = await loadPresentationManifest(
    options.recordingRoot,
    finalized.manifest_path,
  );
  const manifest = presentation.manifest;
  if (manifest.scenario_id !== options.scenarioId)
    throw new Error("recording manifest scenario mismatch");
  if (manifest.run_nonce !== options.runNonce)
    throw new Error("recording manifest run nonce mismatch");
  if (manifest.process_id !== options.gameProcessId)
    throw new Error("recording manifest game process mismatch");
  const tokenHash = createHash("sha256").update(captureToken).digest("hex");
  if (manifest.capture_token_sha256 !== tokenHash)
    throw new Error("recording manifest capture token mismatch");
  if (
    manifest.start_state_index !== 0 ||
    manifest.end_state_index !== options.endStateIndex
  ) {
    throw new Error("recording manifest state span mismatch");
  }
  if (
    !manifest.final_state_presented ||
    manifest.latest_state_index < options.endStateIndex
  ) {
    throw new Error("recording manifest does not present the final state");
  }
  const finalStatePresentations = manifest.frames.filter(
    (frame) => frame.state_index === options.endStateIndex,
  );
  if (finalStatePresentations.length <= FINAL_STATE_TAIL_PRESENTATION_FRAMES) {
    throw new Error(
      "recording manifest does not include the one-second final-state tail",
    );
  }
  return { captureToken, execution, status: finalized, presentation };
}

async function stopAndFinalize(
  service: RecordingService,
  request: RecordingControlRequest,
  reason: string,
): Promise<void> {
  try {
    await service.recordingStop({ ...request, reason });
  } finally {
    await service.recordingFinalize(request);
  }
}

function assertStatusIdentity(
  status: RecordingStatus,
  scenarioId: string,
  start: number,
  end: number,
): void {
  if (
    status.scenario_id !== scenarioId ||
    status.start_state_index !== start ||
    status.end_state_index !== end
  ) {
    throw new Error(
      "recording service returned mismatched scenario or state span",
    );
  }
}
