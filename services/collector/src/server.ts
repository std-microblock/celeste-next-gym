import { createServer, type IncomingMessage, type Server, type ServerResponse } from "node:http";
import { once } from "node:events";
import {
  BackendNotConfiguredError,
  BackendTimeoutError,
  collectWithTimeout,
  type CollectorBackend,
} from "./backend.js";
import {
  CONTENT_TYPE,
  ProtocolValidationError,
  decodeSimulateRequest,
  encodeResponse,
  validateBackendStates,
} from "./protocol.js";
import {
  decodeRecordingRequest,
  type RecordingAction,
  type RecordingControlRequest,
  type RecordingStartRequest,
  type RecordingStatus,
} from "./recording.js";

export interface CollectorServerOptions {
  backend: CollectorBackend;
  timeoutMs?: number;
  maxBodyBytes?: number;
  maxMapBytes?: number;
  maxFrames?: number;
  logger?: Pick<Console, "error" | "info">;
}

export interface RunningCollectorServer {
  server: Server;
  url: string;
  close(): Promise<void>;
}

const DEFAULTS = {
  timeoutMs: 30_000,
  maxBodyBytes: 32 * 1024 * 1024,
  maxMapBytes: 24 * 1024 * 1024,
  maxFrames: 36_000,
};

export function createCollectorServer(options: CollectorServerOptions): Server {
  const config = { ...DEFAULTS, ...options };

  return createServer(async (request, response) => {
    try {
      await route(request, response, config);
    } catch (error) {
      config.logger?.error(error);
      if (!response.headersSent) {
        sendMessagePackError(response, 500, "INTERNAL_ERROR", "Internal server error");
      } else {
        response.destroy();
      }
    }
  });
}

export async function listen(
  server: Server,
  port = 0,
  host = "127.0.0.1",
  backend?: CollectorBackend,
): Promise<RunningCollectorServer> {
  server.listen(port, host);
  await once(server, "listening");
  const address = server.address();
  if (address === null || typeof address === "string") {
    throw new Error("Collector server did not bind to a TCP address");
  }

  return {
    server,
    url: `http://${host}:${address.port}`,
    async close() {
      server.close();
      await once(server, "close");
      await backend?.close?.();
    },
  };
}

async function route(
  request: IncomingMessage,
  response: ServerResponse,
  config: Required<
    Pick<
      CollectorServerOptions,
      "backend" | "timeoutMs" | "maxBodyBytes" | "maxMapBytes" | "maxFrames"
    >
  > &
    Pick<CollectorServerOptions, "logger">,
): Promise<void> {
  const url = new URL(request.url ?? "/", "http://collector.local");

  if (request.method === "GET" && url.pathname === "/health") {
    const health = (await config.backend.health?.()) ?? { ready: true };
    sendJson(response, 200, {
      status: "ok",
      ready: health.ready,
      backend: config.backend.name,
      detail: health.detail,
    });
    return;
  }

  const recordingMatch = /^\/api\/recording\/(start|status|stop|finalize)$/.exec(
    url.pathname,
  );
  if (request.method === "POST" && recordingMatch) {
    await routeRecording(
      recordingMatch[1] as RecordingAction,
      request,
      response,
      config,
    );
    return;
  }

  if (request.method !== "POST" || url.pathname !== "/api/simulate") {
    sendMessagePackError(response, 404, "NOT_FOUND", "Route not found");
    return;
  }

  const mediaType = request.headers["content-type"]?.split(";", 1)[0]?.trim().toLowerCase();
  if (mediaType !== CONTENT_TYPE) {
    sendMessagePackError(
      response,
      415,
      "UNSUPPORTED_MEDIA_TYPE",
      `Content-Type must be ${CONTENT_TYPE}`,
    );
    return;
  }

  let body: Uint8Array;
  try {
    body = await readBody(request, config.maxBodyBytes);
  } catch (error) {
    if (error instanceof BodyTooLargeError) {
      sendMessagePackError(response, 413, error.code, error.message);
      return;
    }
    throw error;
  }

  try {
    const simulation = decodeSimulateRequest(body, {
      maxMapBytes: config.maxMapBytes,
      maxFrames: config.maxFrames,
    });
    const states = await collectWithTimeout(
      config.backend,
      simulation,
      config.timeoutMs,
    );
    validateBackendStates(states, simulation.frames);
    sendMessagePack(response, 200, { success: true, states });
  } catch (error) {
    if (error instanceof ProtocolValidationError) {
      sendMessagePackError(response, 400, error.code, error.message);
      return;
    }
    if (error instanceof BackendNotConfiguredError) {
      sendMessagePackError(response, 503, error.code, error.message);
      return;
    }
    if (error instanceof BackendTimeoutError) {
      sendMessagePackError(response, 504, error.code, error.message);
      return;
    }

    config.logger?.error(error);
    sendMessagePackError(
      response,
      502,
      "BACKEND_ERROR",
      error instanceof Error ? error.message : "Collector backend failed",
    );
  }
}

async function routeRecording(
  action: RecordingAction,
  request: IncomingMessage,
  response: ServerResponse,
  config: Required<
    Pick<
      CollectorServerOptions,
      "backend" | "timeoutMs" | "maxBodyBytes" | "maxMapBytes" | "maxFrames"
    >
  > & Pick<CollectorServerOptions, "logger">,
): Promise<void> {
  const mediaType = request.headers["content-type"]?.split(";", 1)[0]?.trim().toLowerCase();
  if (mediaType !== CONTENT_TYPE) {
    sendMessagePackError(
      response,
      415,
      "UNSUPPORTED_MEDIA_TYPE",
      `Content-Type must be ${CONTENT_TYPE}`,
    );
    return;
  }

  try {
    const body = await readBody(request, Math.min(config.maxBodyBytes, 64 * 1024));
    const decoded = decodeRecordingRequest(body, action);
    const signal = AbortSignal.timeout(config.timeoutMs);
    let recording: RecordingStatus;
    switch (action) {
      case "start":
        if (!config.backend.recordingStart) throw new RecordingNotConfiguredError();
        recording = await config.backend.recordingStart(
          decoded as RecordingStartRequest,
          signal,
        );
        break;
      case "status":
        if (!config.backend.recordingStatus) throw new RecordingNotConfiguredError();
        recording = await config.backend.recordingStatus(
          decoded as RecordingControlRequest,
          signal,
        );
        break;
      case "stop":
        if (!config.backend.recordingStop) throw new RecordingNotConfiguredError();
        recording = await config.backend.recordingStop(
          decoded as RecordingControlRequest,
          signal,
        );
        break;
      case "finalize":
        if (!config.backend.recordingFinalize) throw new RecordingNotConfiguredError();
        recording = await config.backend.recordingFinalize(
          decoded as RecordingControlRequest,
          signal,
        );
        break;
    }
    sendMessagePack(response, 200, { success: true, recording });
  } catch (error) {
    if (error instanceof BodyTooLargeError) {
      sendMessagePackError(response, 413, error.code, error.message);
      return;
    }
    if (error instanceof ProtocolValidationError) {
      sendMessagePackError(response, 400, error.code, error.message);
      return;
    }
    if (error instanceof RecordingNotConfiguredError) {
      sendMessagePackError(response, 503, error.code, error.message);
      return;
    }
    config.logger?.error(error);
    sendMessagePackError(
      response,
      502,
      "RECORDING_BACKEND_ERROR",
      error instanceof Error ? error.message : "Recording backend failed",
    );
  }
}

async function readBody(
  request: IncomingMessage,
  maxBodyBytes: number,
): Promise<Uint8Array> {
  const declaredLength = Number(request.headers["content-length"]);
  if (Number.isFinite(declaredLength) && declaredLength > maxBodyBytes) {
    request.resume();
    throw new BodyTooLargeError(maxBodyBytes);
  }

  const chunks: Buffer[] = [];
  let size = 0;
  for await (const chunk of request) {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    size += buffer.byteLength;
    if (size > maxBodyBytes) {
      throw new BodyTooLargeError(maxBodyBytes);
    }
    chunks.push(buffer);
  }
  return Buffer.concat(chunks, size);
}

class BodyTooLargeError extends Error {
  readonly code = "BODY_TOO_LARGE";

  constructor(limit: number) {
    super(`Request body exceeds the ${limit} byte limit`);
    this.name = "BodyTooLargeError";
  }
}

class RecordingNotConfiguredError extends Error {
  readonly code = "RECORDING_NOT_CONFIGURED";

  constructor() {
    super("Collector backend does not support authenticated presentation recording");
    this.name = "RecordingNotConfiguredError";
  }
}

function sendMessagePack(
  response: ServerResponse,
  status: number,
  payload: Parameters<typeof encodeResponse>[0],
): void {
  const bytes = encodeResponse(payload);
  response.writeHead(status, {
    "content-type": CONTENT_TYPE,
    "content-length": bytes.byteLength,
    "cache-control": "no-store",
  });
  response.end(bytes);
}

function sendMessagePackError(
  response: ServerResponse,
  status: number,
  code: string,
  error: string,
): void {
  sendMessagePack(response, status, { success: false, code, error });
}

function sendJson(response: ServerResponse, status: number, payload: unknown): void {
  const body = Buffer.from(JSON.stringify(payload));
  response.writeHead(status, {
    "content-type": "application/json; charset=utf-8",
    "content-length": body.byteLength,
    "cache-control": "no-store",
  });
  response.end(body);
}
