import { describe, expect, it } from "vitest";
import { createInitialState, PLAYGROUND, type SimState } from "./model";
import {
  playerParticleSamples,
  playerTrailColor,
  playerTrailOpacity,
  playerTrailSamples,
} from "./playerEffects";

function state(state: string, dashes = 1): SimState {
  return { ...createInitialState(PLAYGROUND), state, dashes };
}

describe("player effects", () => {
  it("rebuilds the three vanilla dash trail emissions", () => {
    const states = [
      state("Normal"),
      ...Array.from({ length: 9 }, () => state("Dash", 0)),
      state("Normal", 0),
    ];
    expect(playerTrailSamples(states, 10).map((sample) => sample.frame)).toEqual([
      1, 6, 9,
    ]);
  });

  it("uses vanilla DreamDash and StarFly trail intervals", () => {
    const dream = Array.from({ length: 14 }, () => state("DreamDash"));
    expect(playerTrailSamples(dream, 13).map((sample) => sample.frame)).toEqual([
      0, 6, 12,
    ]);
    const feather = Array.from({ length: 8 }, () => state("StarFly"));
    expect(playerTrailSamples(feather, 7).map((sample) => sample.frame)).toEqual([
      0, 3, 6,
    ]);
  });

  it("supports bounded live-state windows with an absolute frame offset", () => {
    const window = [
      state("Normal"),
      ...Array.from({ length: 9 }, () => state("Dash", 0)),
      state("Normal", 0),
    ];
    expect(
      playerTrailSamples(window, 250, 240).map((sample) => sample.frame),
    ).toEqual([241, 246, 249]);
  });

  it("fades old trails and follows the hair dash palette", () => {
    expect(playerTrailOpacity(0)).toBeCloseTo(0.55);
    expect(playerTrailOpacity(24)).toBe(0);
    expect(playerTrailColor(state("Dash", 0))).toBe("#44b7ff");
    expect(playerTrailColor(state("Dash", 2))).toBe("#ff6def");
  });

  it("emits slash and streaming particles throughout a dash", () => {
    const normal = state("Normal");
    const dash = {
      ...state("Dash", 0),
      speed: { x: 240, y: 0 },
      dash_dir: { x: 1, y: 0 },
    };
    const particles = playerParticleSamples([normal, dash, dash], 2);
    expect(particles.filter((particle) => particle.kind === "slash")).toHaveLength(1);
    expect(
      particles.filter((particle) => particle.kind === "dash-streak"),
    ).toHaveLength(2);
  });

  it("matches Player.cs jump and hard-landing dust counts", () => {
    const grounded = { ...state("Normal"), on_ground: true };
    const jumping = {
      ...state("Normal"),
      on_ground: false,
      speed: { x: 0, y: -105 },
    };
    expect(playerParticleSamples([grounded, jumping], 1)[0]?.count).toBe(4);

    const falling = {
      ...state("Normal"),
      on_ground: false,
      speed: { x: 0, y: 80 },
    };
    const landed = { ...state("Normal"), on_ground: true };
    expect(playerParticleSamples([falling, landed], 1)[0]?.count).toBe(8);
  });

  it("replays sustained special-state particles and water splashes", () => {
    const normal = state("Normal");
    const dream = state("DreamDash");
    const starFly = state("StarFly");
    const swim = state("Swim");
    expect(
      playerParticleSamples([normal, dream, dream], 2).filter(
        (particle) => particle.kind === "dream-spark",
      ),
    ).toHaveLength(1);
    expect(
      playerParticleSamples([normal, starFly], 1).some(
        (particle) => particle.kind === "feather",
      ),
    ).toBe(true);
    expect(
      playerParticleSamples([normal, swim], 1).find(
        (particle) => particle.kind === "water-splash",
      )?.count,
    ).toBe(8);
  });

  it("emits one long-lived shard burst when the player dies", () => {
    const alive = state("Normal", 0);
    const dead = { ...state("Normal", 0), dead: true };
    const burst = playerParticleSamples([alive, dead, dead], 2).find(
      (particle) => particle.kind === "death-shard",
    );
    expect(burst?.count).toBe(12);
    expect(burst?.age).toBe(1);
  });
});
