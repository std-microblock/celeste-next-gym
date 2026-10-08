// Example `celeste-gym run` script: drive the TypeScript bindings directly.
//
//   celeste-gym run tools/gym-cli/examples/script-dash-sweep.ts -- out-dir
//
// The default export receives the bindings (same as src/index.ts) and the
// arguments after the script path. Whatever it returns is printed as JSON.
import type * as Gym from "../src/index.ts";

export default async function (gym: typeof Gym, args: string[]) {
  const out = args[0] ?? ".tmp/cli/sweep";
  const map = await gym.openMap("playground");
  const rows = [];
  // Compare the 8 dash directions after 10 frames on the ground.
  for (const dir of ["R", "L", "U", "D", "R,U", "R,D", "L,U", "L,D"]) {
    const trace = await map.simulate(`10;1,${dir},X;20,${dir}`);
    rows.push({ dir, final: gym.compactState(trace.final) });
    if (dir === "R,U") await gym.renderTrace(trace, `${out}/dash-${dir.replace(",", "")}.gif`, { hud: true });
  }
  return rows;
}
