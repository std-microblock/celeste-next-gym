import assert from "node:assert/strict";
import net from "node:net";
import { afterEach, describe, it } from "node:test";
import { EverestTcpBackend } from "../src/everestBackend.js";
import { createDefaultSnapshot, type SimulateRequest } from "../src/protocol.js";

const servers: net.Server[] = [];

afterEach(async () => {
  await Promise.all(servers.splice(0).map((server) => new Promise<void>((resolve) => server.close(() => resolve()))));
});

describe("Everest TCP backend", () => {
  it("translates requests and preserves reflected frame fields", async () => {
    const { port } = await startFakeEverest((request) => {
      assert.equal(request.command, "simulate_area");
      assert.equal(request.area_id, 2);
      assert.equal(request.area_sid, "CelesteGymPlayground/Playground");
      assert.equal(request.room, "1");
      assert.equal(request.dream_dash, true);
      assert.equal(request.initial_snapshot.state, 0);
      assert.equal(request.inputs[0].crouch_dash_pressed, false);
      return {
        success: true,
        states: [0, 1].map((frame) => ({
          frame,
          pos: [10 + frame, 20],
          speed: [60, 0],
          state: 0,
          facing: 1,
          dashes: 1,
          stamina: 110,
          on_ground: true,
          ducking: false,
          dead: false,
          fields: { dashAttackTimer: 0.25 },
        })),
      };
    });
    const backend = new EverestTcpBackend({
      port,
      areaId: 2,
      areaSid: "CelesteGymPlayground/Playground",
    });
    const request = validRequest();
    const states = await backend.collect(request, new AbortController().signal);
    assert.equal(states.length, 2);
    assert.equal(states[1]?.facing, true);
    assert.deepEqual(states[1]?._everest_fields, { dashAttackTimer: 0.25 });
  });

  it("reports a live mod through ping", async () => {
    const { port } = await startFakeEverest(() => ({ success: true, version: "test" }));
    const health = await new EverestTcpBackend({ port }).health();
    assert.equal(health.ready, true);
    assert.match(health.detail ?? "", /test/);
  });
});

function validRequest(): SimulateRequest {
  return {
    map: Uint8Array.of(1),
    room: "1",
    dream_dash: true,
    inputs: [{ move_x: 1, move_y: 0, jump_pressed: false, jump_held: false, dash_pressed: false, crouch_dash_pressed: false, grab_held: false }],
    initial_snapshot: createDefaultSnapshot(),
    frames: 1,
  };
}

async function startFakeEverest(respond: (request: any) => unknown): Promise<{ port: number }> {
  const server = net.createServer((socket) => {
    socket.setEncoding("utf8");
    let data = "";
    socket.on("data", (chunk: string) => {
      data += chunk;
      const newline = data.indexOf("\n");
      if (newline < 0) return;
      socket.end(`${JSON.stringify(respond(JSON.parse(data.slice(0, newline))))}\n`);
    });
  });
  servers.push(server);
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (address === null || typeof address === "string") throw new Error("fake server did not bind TCP");
  return { port: address.port };
}
