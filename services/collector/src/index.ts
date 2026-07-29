import {
  MockCollectorBackend,
  NotConfiguredBackend,
  type CollectorBackend,
} from "./backend.js";
import { EverestTcpBackend } from "./everestBackend.js";
import { createCollectorServer, listen } from "./server.js";

function backendFromEnvironment(): CollectorBackend {
  const backend = process.env.COLLECTOR_BACKEND?.trim().toLowerCase();
  if (backend === "mock") {
    return new MockCollectorBackend();
  }
  if (backend === "everest") {
    const areaSid = process.env.EVEREST_AREA_SID?.trim() || undefined;
    const runNonce = process.env.EVEREST_RUN_NONCE?.trim() || undefined;
    const processId = readOptionalPositiveInteger("EVEREST_PROCESS_ID");
    return new EverestTcpBackend({
      host: process.env.EVEREST_COLLECTOR_HOST ?? "127.0.0.1",
      port: readPositiveInteger("EVEREST_COLLECTOR_PORT", 32270),
      areaId: readUnsignedInteger("EVEREST_AREA_ID", 1),
      ...(runNonce === undefined ? {} : { runNonce }),
      ...(processId === undefined ? {} : { processId }),
      ...(areaSid === undefined ? {} : { areaSid }),
    });
  }
  if (backend && backend !== "none") {
    console.warn(
      `Unknown COLLECTOR_BACKEND=${JSON.stringify(backend)}; collector is not configured`,
    );
  }
  return new NotConfiguredBackend();
}

function readOptionalPositiveInteger(name: string): number | undefined {
  const raw = process.env[name];
  if (raw === undefined || raw.trim() === "") return undefined;
  const value = Number(raw);
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return value;
}

function readUnsignedInteger(name: string, fallback: number): number {
  const raw = process.env[name];
  if (raw === undefined) return fallback;
  const value = Number(raw);
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${name} must be a non-negative integer`);
  }
  return value;
}

function readPositiveInteger(name: string, fallback: number): number {
  const raw = process.env[name];
  if (raw === undefined) return fallback;
  const value = Number(raw);
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new Error(`${name} must be a positive integer`);
  }
  return value;
}

const backend = backendFromEnvironment();
const host = process.env.COLLECTOR_HOST ?? "127.0.0.1";
const port = readPositiveInteger("COLLECTOR_PORT", 4318);
const timeoutMs = readPositiveInteger("COLLECTOR_TIMEOUT_MS", 30_000);

const server = createCollectorServer({ backend, timeoutMs, logger: console });
const running = await listen(server, port, host, backend);

console.info(
  `Celeste collector listening at ${running.url} (backend: ${backend.name})`,
);

async function shutdown(signal: string): Promise<void> {
  console.info(`Received ${signal}; shutting down collector`);
  await running.close();
}

process.once("SIGINT", () => void shutdown("SIGINT"));
process.once("SIGTERM", () => void shutdown("SIGTERM"));
