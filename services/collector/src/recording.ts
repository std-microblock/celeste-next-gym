import { decode } from "@msgpack/msgpack";
import { ProtocolValidationError } from "./protocol.js";

export const CAPTURE_TOKEN_PATTERN = /^[A-Za-z0-9_-]{32,128}$/;
export const SCENARIO_ID_PATTERN = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;

export interface StateIndexRange {
  start_state_index: number;
  end_state_index: number;
}

export interface RecordingStatus {
  state: "active" | "ready" | "stopped" | "timed_out" | "faulted" | "finalized";
  scenario_id: string;
  start_state_index: number;
  end_state_index: number;
  latest_state_index: number;
  render_frame_count: number;
  final_state_presented: boolean;
  repeated_presentation_count: number;
  unpresented_update_ranges: StateIndexRange[];
  manifest_path?: string;
  reason?: string;
}

export interface RecordingStartRequest {
  capture_token: string;
  scenario_id: string;
  start_state_index: number;
  end_state_index: number;
  timeout_ms: number;
}

export interface RecordingControlRequest {
  capture_token: string;
  reason?: string;
}

export interface RecordingSuccess {
  success: true;
  recording: RecordingStatus;
}

export interface RecordingFailure {
  success: false;
  error: string;
  code: string;
}

export type RecordingResponse = RecordingSuccess | RecordingFailure;
export type RecordingAction = "start" | "status" | "stop" | "finalize";

export function decodeRecordingRequest(
  body: Uint8Array,
  action: RecordingAction,
): RecordingStartRequest | RecordingControlRequest {
  let decoded: unknown;
  try {
    decoded = decode(body);
  } catch {
    throw new ProtocolValidationError("Request body is not valid MessagePack");
  }
  const root = requireRecord(decoded, "request");
  const captureToken = requireCaptureToken(root.capture_token);
  if (action !== "start") {
    const reason =
      root.reason === undefined
        ? undefined
        : requireBoundedString(root.reason, "reason", 256);
    return {
      capture_token: captureToken,
      ...(reason === undefined ? {} : { reason }),
    };
  }

  const scenarioId = requireBoundedString(root.scenario_id, "scenario_id", 128);
  if (!SCENARIO_ID_PATTERN.test(scenarioId)) {
    throw new ProtocolValidationError(
      "scenario_id contains characters that are unsafe for artifact paths",
    );
  }
  const start = requireUnsignedInteger(
    root.start_state_index,
    "start_state_index",
  );
  const end = requireUnsignedInteger(root.end_state_index, "end_state_index");
  if (end < start) {
    throw new ProtocolValidationError(
      "end_state_index must not precede start_state_index",
    );
  }
  const timeout = requireUnsignedInteger(root.timeout_ms, "timeout_ms");
  if (timeout < 1_000 || timeout > 600_000) {
    throw new ProtocolValidationError(
      "timeout_ms must be between 1000 and 600000",
    );
  }
  return {
    capture_token: captureToken,
    scenario_id: scenarioId,
    start_state_index: start,
    end_state_index: end,
    timeout_ms: timeout,
  };
}

export function requireCaptureToken(value: unknown): string {
  if (typeof value !== "string" || !CAPTURE_TOKEN_PATTERN.test(value)) {
    throw new ProtocolValidationError(
      "capture_token must be 32-128 URL-safe characters",
    );
  }
  return value;
}

function requireRecord(value: unknown, path: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new ProtocolValidationError(`${path} must be a map`);
  }
  return value as Record<string, unknown>;
}

function requireUnsignedInteger(value: unknown, path: string): number {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw new ProtocolValidationError(`${path} must be a non-negative integer`);
  }
  return value as number;
}

function requireBoundedString(
  value: unknown,
  path: string,
  maximumLength: number,
): string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > maximumLength
  ) {
    throw new ProtocolValidationError(
      `${path} must be a non-empty string no longer than ${maximumLength} characters`,
    );
  }
  return value;
}
