import net from "node:net";
import { execFileSync } from "node:child_process";
import { randomInt } from "node:crypto";

import type { PortReservation } from "../types.js";

export const LOOPBACK = "127.0.0.1";
export const LONG_LIVED_PORT_MIN = 20_000;
export const LONG_LIVED_PORT_MAX = 40_000;

export interface TcpPortRange {
  readonly start: number;
  readonly end: number;
}

export async function reserveLoopbackPort(): Promise<PortReservation> {
  const server = net.createServer();
  await new Promise<void>((resolveListen, reject) => {
    server.once("error", reject);
    server.listen({ host: LOOPBACK, port: 0, exclusive: true }, resolveListen);
  });
  const address = server.address();
  if (address === null || typeof address === "string") {
    server.close();
    throw new Error("failed to reserve a loopback TCP port");
  }
  let released = false;
  return {
    port: address.port,
    async release(): Promise<void> {
      if (released) return;
      released = true;
      await new Promise<void>((resolveClose, reject) => {
        server.close((error) => (error ? reject(error) : resolveClose()));
      });
    },
  };
}

export async function reserveLongLivedLoopbackPort(
  dynamicRange = readSystemDynamicTcpPortRange(),
): Promise<PortReservation> {
  const candidateCount = LONG_LIVED_PORT_MAX - LONG_LIVED_PORT_MIN + 1;
  const first = randomInt(candidateCount);
  for (let offset = 0; offset < candidateCount; offset++) {
    const port = LONG_LIVED_PORT_MIN + ((first + offset) % candidateCount);
    if (port >= dynamicRange.start && port <= dynamicRange.end) continue;
    const reservation = await tryReservePort(port);
    if (reservation) return reservation;
  }
  throw new Error(
    `no long-lived loopback TCP port is available outside dynamic range ` +
    `${dynamicRange.start}-${dynamicRange.end}`,
  );
}

export function readSystemDynamicTcpPortRange(): TcpPortRange {
  if (process.platform !== "win32") return { start: 49_152, end: 65_535 };
  try {
    const output = execFileSync(
      "netsh",
      ["int", "ipv4", "show", "dynamicport", "tcp"],
      { encoding: "utf8", windowsHide: true },
    );
    const values = [...output.matchAll(/:\s*(\d+)\s*$/gm)].map((match) =>
      Number.parseInt(match[1]!, 10)
    );
    const [start, count] = values;
    if (Number.isSafeInteger(start) && Number.isSafeInteger(count)
        && start! > 0 && count! > 0 && start! + count! - 1 <= 65_535) {
      return { start: start!, end: start! + count! - 1 };
    }
  } catch {
  }
  return { start: 49_152, end: 65_535 };
}

export function isOutsidePortRange(port: number, range: TcpPortRange): boolean {
  return port < range.start || port > range.end;
}

async function tryReservePort(port: number): Promise<PortReservation | undefined> {
  const server = net.createServer();
  try {
    await new Promise<void>((resolveListen, reject) => {
      server.once("error", reject);
      server.listen({ host: LOOPBACK, port, exclusive: true }, resolveListen);
    });
  } catch (error) {
    server.removeAllListeners();
    if ((error as NodeJS.ErrnoException).code === "EADDRINUSE"
        || (error as NodeJS.ErrnoException).code === "EACCES") {
      return undefined;
    }
    throw error;
  }
  let released = false;
  return {
    port,
    async release(): Promise<void> {
      if (released) return;
      released = true;
      await new Promise<void>((resolveClose, reject) => {
        server.close((error) => (error ? reject(error) : resolveClose()));
      });
    },
  };
}
