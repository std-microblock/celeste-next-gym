import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const mod = resolve(root, "mods/MicroblocksQolUtils");
const output = resolve(mod, "Build");
const dll = resolve(mod, "Source/bin/Release/net8.0/MicroblocksQolUtils.dll");
const nativeName = process.platform === "win32"
  ? "microblocks_qol_native.dll"
  : process.platform === "darwin"
    ? "libmicroblocks_qol_native.dylib"
    : "libmicroblocks_qol_native.so";

const run = (command, args) => {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit", shell: process.platform === "win32" });
  if (result.status !== 0) process.exit(result.status ?? 1);
};

run("cargo", ["build", "-q", "-p", "microblocks-qol-native", "--release"]);
run("dotnet", ["build", resolve(mod, "Source/MicroblocksQolUtils.csproj"), "-c", "Release"]);
rmSync(output, { recursive: true, force: true });
mkdirSync(resolve(output, "Code"), { recursive: true });
cpSync(dll, resolve(output, "Code/MicroblocksQolUtils.dll"));
cpSync(resolve(root, "target", "release", nativeName), resolve(output, "Code", nativeName));
for (const path of ["everest.yaml", "Dialog", "Graphics", "Native"]) {
  const source = resolve(mod, path);
  if (!existsSync(source)) continue;
  const target = resolve(output, path);
  mkdirSync(dirname(target), { recursive: true });
  cpSync(source, target, { recursive: true });
}
console.log(`Built ${output}`);
