import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { copyFileSync, existsSync, lstatSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(import.meta.url);

export function compileWorkspace(workspaceRoot) {
  const root = resolve(workspaceRoot);
  const manifestPath = resolve(root, "celeste-gym.workspace.json");
  assertRegularFileWithin(root, manifestPath);
  const manifest = readJson(manifestPath);
  if (manifest.version !== 1 || !Array.isArray(manifest.projects) || manifest.projects.length === 0) {
    throw new Error("workspace must be version 1 with at least one project");
  }
  const rooms = [];
  const projects = [];
  const usedIds = new Set();
  const usedRooms = new Set();
  let nextRoomY = 0;
  for (const entry of manifest.projects) {
    if (!entry || typeof entry.id !== "string" || !entry.id.trim()) throw new Error("workspace project id is required");
    if (usedIds.has(entry.id)) throw new Error(`duplicate workspace project id: ${entry.id}`);
    usedIds.add(entry.id);
    const mapPath = resolveProjectFile(root, entry.map, `${entry.id}.map`);
    const trainingPath = resolveProjectFile(root, entry.training, `${entry.id}.training`);
    const map = readJson(mapPath);
    const training = readJson(trainingPath);
    validateMap(map, entry.id);
    validateTraining(training, entry.id);
    const room = map.room || entry.id;
    if (!/^[A-Za-z0-9_-]+$/.test(room)) throw new Error(`${entry.id}: room must use letters, digits, underscore, or dash`);
    if (usedRooms.has(room)) throw new Error(`duplicate workspace room: ${room}`);
    usedRooms.add(room);
    const normalized = normalizeRoom(map, room, nextRoomY);
    nextRoomY = normalized.bounds[1] + normalized.bounds[3] + 64;
    rooms.push(normalized.fixture);
    projects.push({
      id: entry.id,
      title: training.title,
      summary: training.summary,
      room,
      difficulty: typeof entry.difficulty === "string" && entry.difficulty.trim() ? entry.difficulty : "入门",
      thumbnail: typeof entry.thumbnail === "string" && entry.thumbnail.trim() ? entry.thumbnail : null,
      coordinateOffset: normalized.offset,
      map: transformMap(map, normalized.offset, normalized.bounds),
      training: transformTraining(training, normalized.offset),
    });
  }
  return {
    fixture: { formatVersion: 1, package: "CelesteGymTraining", sid: "CelesteGymTraining/Training", rooms },
    catalog: {
      version: 1,
      areaSid: "CelesteGymTraining/Training",
      skin: "strawberry-jam-2021-beginner-gym",
      sourceWorkspace: relative(repoRoot, root).replaceAll("\\", "/"),
      projects,
    },
  };
}

function normalizeRoom(map, room, targetY) {
  const left = floor8(map.bounds.x);
  const top = floor8(map.bounds.y);
  const right = Math.max(left + 320, ceil8(map.bounds.x + map.bounds.width));
  const bottom = Math.max(top + 184, ceil8(map.bounds.y + map.bounds.height));
  const offset = { x: -left, y: targetY - top };
  const bounds = [0, targetY, right - left, bottom - top];
  const fixture = {
    name: room,
    bounds,
    spawn: point(map.spawn, offset),
    solids: map.solids.map((solid) => gridRect(solid, offset)),
    entities: map.entities.map((entity, index) => fixtureEntity(entity, index, offset)),
  };
  return { fixture, bounds, offset };
}

function fixtureEntity(entity, index, offset) {
  if (entity.kind === "unknown") throw new Error(`entities[${index}]: unknown entities cannot be built into a Celeste map`);
  return {
    id: `entity-${String(index).padStart(4, "0")}`,
    kind: entity.kind,
    bounds: rect(entity.bounds, offset),
    direction: [entity.direction.x, entity.direction.y],
    shielded: entity.shielded === true,
    singleUse: entity.single_use === true,
    nodes: (entity.nodes ?? []).map((node) => point(node, offset)),
    name: entity.name,
  };
}

function transformMap(map, offset, bounds) {
  return {
    ...map,
    bounds: { x: bounds[0], y: bounds[1], width: bounds[2], height: bounds[3] },
    spawn: objectPoint(map.spawn, offset),
    solids: map.solids.map((solid) => objectRect(solid, offset)),
    entities: map.entities.map((entity) => ({
      ...entity,
      bounds: objectRect(entity.bounds, offset),
      nodes: entity.nodes?.map((node) => objectPoint(node, offset)),
    })),
  };
}

function transformTraining(training, offset) {
  const copy = structuredClone(training);
  for (const module of copy.modules) {
    module.trigger.bounds = objectRect(module.trigger.bounds, offset);
    module.end_trigger.bounds = objectRect(module.end_trigger.bounds, offset);
    module.validation.initial_state.pos = objectPoint(module.validation.initial_state.pos, offset);
  }
  copy.finish.trigger.bounds = objectRect(copy.finish.trigger.bounds, offset);
  return copy;
}

function validateMap(map, id) {
  if (!map || typeof map.name !== "string" || !map.bounds || !map.spawn || !Array.isArray(map.solids) || !Array.isArray(map.entities)) {
    throw new Error(`${id}: invalid map document`);
  }
  validateRect(map.bounds, `${id}.bounds`);
  validatePoint(map.spawn, `${id}.spawn`);
  map.solids.forEach((value, index) => validateRect(value, `${id}.solids[${index}]`));
  map.entities.forEach((entity, index) => {
    if (!entity || typeof entity.kind !== "string" || typeof entity.name !== "string" || !entity.direction) {
      throw new Error(`${id}.entities[${index}]: invalid entity`);
    }
    validateRect(entity.bounds, `${id}.entities[${index}].bounds`);
    validatePoint(entity.direction, `${id}.entities[${index}].direction`);
    entity.nodes?.forEach((node, nodeIndex) => validatePoint(node, `${id}.entities[${index}].nodes[${nodeIndex}]`));
  });
}

function validateTraining(training, id) {
  if (!training || training.version !== 2 || training.id !== id || !Array.isArray(training.modules) || !training.finish?.trigger) {
    throw new Error(`${id}: invalid version 2 training document or id mismatch`);
  }
  if (typeof training.title !== "string" || typeof training.summary !== "string") throw new Error(`${id}: training title and summary are required`);
  training.modules.forEach((module, index) => {
    validateRect(module?.trigger?.bounds, `${id}.modules[${index}].trigger`);
    validateRect(module?.end_trigger?.bounds, `${id}.modules[${index}].end_trigger`);
    validatePoint(module?.validation?.initial_state?.pos, `${id}.modules[${index}].initial_state.pos`);
  });
  validateRect(training.finish.trigger.bounds, `${id}.finish.trigger`);
}

function validateRect(value, label) {
  if (!value || ![value.x, value.y, value.width, value.height].every(Number.isFinite) || value.width <= 0 || value.height <= 0) {
    throw new Error(`${label}: expected a finite positive rectangle`);
  }
}

function validatePoint(value, label) {
  if (!value || !Number.isFinite(value.x) || !Number.isFinite(value.y)) throw new Error(`${label}: expected a finite point`);
}

function resolveProjectFile(root, path, label) {
  if (typeof path !== "string" || !path || isAbsolute(path)) throw new Error(`${label}: path must be relative`);
  const target = resolve(root, path);
  assertRegularFileWithin(root, target);
  return target;
}

function assertRegularFileWithin(root, target) {
  const relativePath = relative(root, target);
  if (relativePath.startsWith("..") || isAbsolute(relativePath)) throw new Error(`workspace path escapes root: ${target}`);
  if (!existsSync(target) || !lstatSync(target).isFile() || lstatSync(target).isSymbolicLink()) throw new Error(`workspace file must be a regular non-link file: ${target}`);
  if (relative(root, realpathSync(target)).startsWith("..")) throw new Error(`workspace file resolves outside root: ${target}`);
}

const readJson = (path) => JSON.parse(readFileSync(path, "utf8"));
const floor8 = (value) => Math.floor(value / 8) * 8;
const ceil8 = (value) => Math.ceil(value / 8) * 8;
const point = (value, offset) => [value.x + offset.x, value.y + offset.y];
const rect = (value, offset) => [value.x + offset.x, value.y + offset.y, value.width, value.height];
const objectPoint = (value, offset) => ({ x: value.x + offset.x, y: value.y + offset.y });
const objectRect = (value, offset) => ({ x: value.x + offset.x, y: value.y + offset.y, width: value.width, height: value.height });
const gridRect = (value, offset) => {
  const left = floor8(value.x + offset.x);
  const top = floor8(value.y + offset.y);
  const right = ceil8(value.x + offset.x + value.width);
  const bottom = ceil8(value.y + offset.y + value.height);
  return [left, top, right - left, bottom - top];
};

async function main() {
  const workspaceRoot = resolve(repoRoot, process.argv[2] ?? ".tmp/example-workfolder-tutorial");
  const modRoot = resolve(repoRoot, "mods/CelesteGymTraining");
  const generated = compileWorkspace(workspaceRoot);
  const fixturePath = resolve(modRoot, "Build/generated.fixture.json");
  const catalogPath = resolve(modRoot, "Content/CelesteGymTraining/training-catalog.json");
  const rawMapPath = resolve(repoRoot, ".tmp/training-build/Training.raw.bin");
  const skinnedMapPath = resolve(repoRoot, ".tmp/training-build/Training.skinned.bin");
  const mapPath = resolve(modRoot, "Maps/CelesteGymTraining/Training.bin");
  const nativeName = process.platform === "win32"
    ? "celeste_gym_native.dll"
    : process.platform === "darwin"
      ? "libceleste_gym_native.dylib"
      : "libceleste_gym_native.so";
  const nativePath = resolve(modRoot, "Build", nativeName);
  mkdirSync(dirname(fixturePath), { recursive: true });
  mkdirSync(dirname(catalogPath), { recursive: true });
  mkdirSync(dirname(mapPath), { recursive: true });
  mkdirSync(dirname(rawMapPath), { recursive: true });
  writeFileSync(fixturePath, `${JSON.stringify(generated.fixture, null, 2)}\n`, "utf8");
  writeFileSync(catalogPath, `${JSON.stringify(generated.catalog, null, 2)}\n`, "utf8");
  run("cargo", ["run", "-q", "-p", "celeste-physics", "--example", "compile_map_fixture", "--", fixturePath, rawMapPath]);
  run("cargo", ["run", "-q", "-p", "celeste-physics", "--example", "skin_training_map", "--", rawMapPath, skinnedMapPath]);
  run("cargo", ["run", "-q", "-p", "celeste-physics", "--example", "inject_training_triggers", "--", skinnedMapPath, catalogPath, mapPath]);
  run("cargo", ["build", "-q", "-p", "celeste-gym-native", "--release"]);
  copyFileSync(resolve(repoRoot, "target", "release", nativeName), nativePath);
  await buildSkinTexture(modRoot);
  run("dotnet", ["build", resolve(modRoot, "Source/CelesteGymTraining.csproj"), "-c", "Release"]);
  console.log(JSON.stringify({ workspace: workspaceRoot, projects: generated.catalog.projects.length, map: mapPath, catalog: catalogPath, native: nativePath }, null, 2));
}

async function buildSkinTexture(modRoot) {
  const sharp = require(resolve(repoRoot, "web/node_modules/sharp"));
  const source = resolve(repoRoot, "web/public/assets/strawberry-jam/gameplay/theme-selected.png");
  const output = resolve(modRoot, "Graphics/Atlases/Gameplay/tilesets/CelesteGymTraining/BeginnerGym.png");
  mkdirSync(dirname(output), { recursive: true });
  await sharp(source).extract({ left: 2, top: 2, width: 24, height: 136 }).png().toFile(output);
}

function run(command, args) {
  const result = spawnSync(command, args, { cwd: repoRoot, stdio: "inherit", shell: false });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} exited with ${result.status}`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
