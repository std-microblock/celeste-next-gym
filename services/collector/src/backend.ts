import {
  createDefaultSnapshot,
  type PlayerSnapshot,
  type SimulateRequest,
} from "./protocol.js";
import type {
  RecordingControlRequest,
  RecordingStartRequest,
  RecordingStatus,
} from "./recording.js";

export interface BackendHealth {
  ready: boolean;
  detail?: string;
}

export interface CollectorBackend {
  readonly name: string;
  collect(
    request: SimulateRequest,
    signal: AbortSignal,
  ): Promise<PlayerSnapshot[]>;
  health?(): Promise<BackendHealth>;
  close?(): Promise<void>;
  recordingStart?(
    request: RecordingStartRequest,
    signal: AbortSignal,
  ): Promise<RecordingStatus>;
  recordingStatus?(
    request: RecordingControlRequest,
    signal: AbortSignal,
  ): Promise<RecordingStatus>;
  recordingStop?(
    request: RecordingControlRequest,
    signal: AbortSignal,
  ): Promise<RecordingStatus>;
  recordingFinalize?(
    request: RecordingControlRequest,
    signal: AbortSignal,
  ): Promise<RecordingStatus>;
}

export class BackendNotConfiguredError extends Error {
  readonly code = "NOT_CONFIGURED";

  constructor(
    message = "A real Celeste/Everest collector backend is not configured",
  ) {
    super(message);
    this.name = "BackendNotConfiguredError";
  }
}

export class BackendTimeoutError extends Error {
  readonly code = "BACKEND_TIMEOUT";

  constructor(timeoutMs: number) {
    super(`Collector backend did not finish within ${timeoutMs} ms`);
    this.name = "BackendTimeoutError";
  }
}

export class NotConfiguredBackend implements CollectorBackend {
  readonly name = "not-configured";

  async collect(): Promise<PlayerSnapshot[]> {
    throw new BackendNotConfiguredError();
  }

  async health(): Promise<BackendHealth> {
    return {
      ready: false,
      detail:
        "Set COLLECTOR_BACKEND=mock for protocol testing, or supply a real backend implementation",
    };
  }
}

/**
 * Protocol-only backend for local development. It intentionally performs no
 * Celeste physics and must never be used as reference data.
 */
export class MockCollectorBackend implements CollectorBackend {
  readonly name = "mock-no-physics";

  async collect(
    request: SimulateRequest,
    signal: AbortSignal,
  ): Promise<PlayerSnapshot[]> {
    if (signal.aborted) {
      throw signal.reason;
    }

    const initial = request.initial_snapshot ?? createDefaultSnapshot();
    return Array.from({ length: request.frames + 1 }, (_, frame) => ({
      ...structuredClone(initial),
      _collector_mock: true,
      _frame: frame,
    }));
  }

  async health(): Promise<BackendHealth> {
    return {
      ready: true,
      detail:
        "Mock backend is active; no game process and no physics simulation",
    };
  }
}

export async function collectWithTimeout(
  backend: CollectorBackend,
  request: SimulateRequest,
  timeoutMs: number,
  requestSignal?: AbortSignal,
): Promise<PlayerSnapshot[]> {
  const controller = new AbortController();
  const timeoutError = new BackendTimeoutError(timeoutMs);
  let rejectTimeout: ((reason: unknown) => void) | undefined;

  const timeoutPromise = new Promise<never>((_, reject) => {
    rejectTimeout = reject;
  });
  const timer = setTimeout(() => {
    controller.abort(timeoutError);
    rejectTimeout?.(timeoutError);
  }, timeoutMs);
  timer.unref();

  const abortFromRequest = () => controller.abort(requestSignal?.reason);
  requestSignal?.addEventListener("abort", abortFromRequest, { once: true });

  try {
    return await Promise.race([
      backend.collect(request, controller.signal),
      timeoutPromise,
    ]);
  } finally {
    clearTimeout(timer);
    requestSignal?.removeEventListener("abort", abortFromRequest);
  }
}
