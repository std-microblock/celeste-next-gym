import { createRequire } from "node:module";
import { resolve } from "node:path";

import { SERVICE_READY_TIMEOUT_MS } from "../constants.js";
import type { E2EState, SimulateRequest } from "../types.js";

interface Codec {
  encode(value: unknown): Uint8Array;
  decode(value: Uint8Array): unknown;
}

interface SimulationBody {
  readonly success?: boolean;
  readonly error?: unknown;
  readonly states?: unknown;
}

export type RecordingState =
  | "active"
  | "ready"
  | "stopped"
  | "timed_out"
  | "faulted"
  | "finalized";

export interface RecordingStatus {
  readonly state: RecordingState;
  readonly scenario_id: string;
  readonly start_state_index: number;
  readonly end_state_index: number;
  readonly latest_state_index: number;
  readonly render_frame_count: number;
  readonly final_state_presented: boolean;
  readonly repeated_presentation_count: number;
  readonly unpresented_update_ranges: readonly {
    readonly start_state_index: number;
    readonly end_state_index: number;
  }[];
  readonly manifest_path?: string;
  readonly reason?: string;
}

export interface RecordingStartRequest {
  readonly capture_token: string;
  readonly scenario_id: string;
  readonly start_state_index: number;
  readonly end_state_index: number;
  readonly timeout_ms: number;
}

export interface RecordingControlRequest {
  readonly capture_token: string;
  readonly reason?: string;
}

export interface CollectorClient {
  health(): Promise<Record<string, unknown>>;
  waitUntilReady(timeoutMs?: number): Promise<Record<string, unknown>>;
  simulate(request: SimulateRequest): Promise<unknown>;
  recordingStart(request: RecordingStartRequest): Promise<RecordingStatus>;
  recordingStatus(request: RecordingControlRequest): Promise<RecordingStatus>;
  recordingStop(request: RecordingControlRequest): Promise<RecordingStatus>;
  recordingFinalize(request: RecordingControlRequest): Promise<RecordingStatus>;
}

export function createCollectorClient(
  serviceRoot: string,
  port: number,
): CollectorClient {
  const requireFromService = createRequire(
    resolve(serviceRoot, "package.json"),
  );
  const codec = requireFromService("@msgpack/msgpack") as Codec;
  return {
    async health(): Promise<Record<string, unknown>> {
      const value = await fetch(`http://127.0.0.1:${port}/health`).then(
        async (response) => await response.json(),
      );
      if (!value || typeof value !== "object")
        throw new Error("collector health response is not an object");
      return value as Record<string, unknown>;
    },
    async waitUntilReady(
      timeoutMs = SERVICE_READY_TIMEOUT_MS,
    ): Promise<Record<string, unknown>> {
      const deadline = Date.now() + timeoutMs;
      let health: Record<string, unknown> = {};
      do {
        health = await this.health();
        if (health.ready === true) return health;
        await new Promise<void>((resolveWait) => setTimeout(resolveWait, 250));
      } while (Date.now() < deadline);
      throw new Error(`HTTP collector is not ready: ${JSON.stringify(health)}`);
    },
    async simulate(request: SimulateRequest): Promise<unknown> {
      const response = await fetch(`http://127.0.0.1:${port}/api/simulate`, {
        method: "POST",
        headers: { "content-type": "application/octet-stream" },
        body: Buffer.from(codec.encode(request)),
      });
      const body = codec.decode(
        new Uint8Array(await response.arrayBuffer()),
      ) as SimulationBody;
      if (!response.ok || body.success !== true)
        throw new Error(`simulation failed: ${JSON.stringify(body)}`);
      return body.states;
    },
    recordingStart: async (request) => await postRecording("start", request),
    recordingStatus: async (request) => await postRecording("status", request),
    recordingStop: async (request) => await postRecording("stop", request),
    recordingFinalize: async (request) =>
      await postRecording("finalize", request),
  };

  async function postRecording(
    action: string,
    request: RecordingStartRequest | RecordingControlRequest,
  ): Promise<RecordingStatus> {
    const response = await fetch(
      `http://127.0.0.1:${port}/api/recording/${action}`,
      {
        method: "POST",
        headers: { "content-type": "application/octet-stream" },
        body: Buffer.from(codec.encode(request)),
      },
    );
    const decoded = codec.decode(new Uint8Array(await response.arrayBuffer()));
    const body = requireRecord(decoded, "recording response");
    if (!response.ok || body.success !== true)
      throw new Error(`recording ${action} failed: ${JSON.stringify(decoded)}`);
    return validateRecordingStatus(body.recording);
  }
}

export function asStates(value: unknown): readonly E2EState[] {
  if (!Array.isArray(value))
    throw new Error("collector states are not an array");
  return value as readonly E2EState[];
}

export function validateRecordingStatus(value: unknown): RecordingStatus {
  const status = requireRecord(value, "recording status");
  const states: readonly RecordingState[] = [
    "active",
    "ready",
    "stopped",
    "timed_out",
    "faulted",
    "finalized",
  ];
  if (!states.includes(status.state as RecordingState))
    throw new Error("recording status.state is invalid");
  const ranges = status.unpresented_update_ranges;
  if (!Array.isArray(ranges))
    throw new Error(
      "recording status.unpresented_update_ranges is not an array",
    );
  for (const name of ["scenario_id"] as const)
    requireString(status[name], `recording status.${name}`);
  for (const name of [
    "start_state_index",
    "end_state_index",
    "render_frame_count",
    "repeated_presentation_count",
  ] as const) {
    requireUnsignedInteger(status[name], `recording status.${name}`);
  }
  requireInteger(
    status.latest_state_index,
    "recording status.latest_state_index",
  );
  if ((status.latest_state_index as number) < -1)
    throw new Error("recording status.latest_state_index is less than -1");
  if ((status.end_state_index as number) < (status.start_state_index as number))
    throw new Error("recording status state range is reversed");
  if (typeof status.final_state_presented !== "boolean")
    throw new Error("recording status.final_state_presented is not boolean");
  for (const [index, rangeValue] of ranges.entries()) {
    const range = requireRecord(rangeValue, `recording range ${index}`);
    const start = requireUnsignedInteger(
      range.start_state_index,
      `recording range ${index}.start`,
    );
    const end = requireUnsignedInteger(
      range.end_state_index,
      `recording range ${index}.end`,
    );
    if (end < start) throw new Error(`recording range ${index} is reversed`);
  }
  if (status.manifest_path !== undefined)
    requireString(status.manifest_path, "recording status.manifest_path");
  if (status.reason !== undefined)
    requireString(status.reason, "recording status.reason");
  return status as unknown as RecordingStatus;
}

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error(`${label} is not an object`);
  return value as Record<string, unknown>;
}

function requireInteger(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value))
    throw new Error(`${label} is not an integer`);
  return value as number;
}

function requireUnsignedInteger(value: unknown, label: string): number {
  const integer = requireInteger(value, label);
  if (integer < 0) throw new Error(`${label} is negative`);
  return integer;
}

function requireString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0)
    throw new Error(`${label} is not a string`);
  return value;
}
