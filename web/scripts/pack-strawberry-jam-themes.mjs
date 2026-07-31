import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import sharp from "sharp";

const source = process.argv[2];
const destination = process.argv[3];

if (!source || !destination) {
  console.error(
    "usage: node scripts/pack-strawberry-jam-themes.mjs <extracted StrawberryJam2021 root> <output>",
  );
  process.exit(2);
}

/**
 * Original Gameplay atlas paths selected from Strawberry Jam 2021. Every key in
 * the packed atlas is the exact path under Graphics/Atlases/Gameplay minus the
 * .png extension, i.e. the same key Everest produces when the mod's loose PNGs
 * are merged into the Gameplay atlas. No renaming happens here, so the themes
 * can reference the same values a Celeste map would use:
 *   - tilesets via ForegroundTiles XML path="tilesets/..."
 *   - spikes via the Spike meta / entity type "danger/spikes/<type>_<dir>"
 *   - spinners via their atlas prefix
 *   - backdrops via bgs/<...>
 */
const themeAssets = [
  // Gym tilesets (the five official SJ2021 palettes).
  "tilesets/SJ2021/Gym/BeginnerGym.png",
  "tilesets/SJ2021/Gym/IntermediateGym.png",
  "tilesets/SJ2021/Gym/AdvancedGym.png",
  "tilesets/SJ2021/Gym/ExpertGym.png",
  "tilesets/SJ2021/Gym/GrandmasterGym.png",
  // Lobby tilesets.
  "tilesets/SJ2021/BeginnerLobby/lobbyCliff.png",
  "tilesets/SJ2021/Int_Lobby/IntGirderFg.png",
  "tilesets/SJ2021/Advanced_Lobby/advCloudSunset.png",
  "tilesets/SJ2021/ExpertLobby/spaceVegetation.png",
  "tilesets/SJ2021/Grandmaster/elysianGrass.png",
  // Gym dark-mode backgrounds.
  "bgs/SJ2021/Gym/begGymDarkBG.png",
  "bgs/SJ2021/Gym/intGymDarkBG.png",
  "bgs/SJ2021/Gym/advGymDarkBG.png",
  "bgs/SJ2021/Gym/expGymDarkBG.png",
  "bgs/SJ2021/Gym/gmGymDarkBG.png",
  // Lobby backgrounds.
  "bgs/SJ2021/BeginnerLobby/main/sky.png",
  "bgs/SJ2021/BeginnerLobby/main/sky_top_gradient.png",
  "bgs/SJ2021/BeginnerLobby/main/sun.png",
  "bgs/SJ2021/BeginnerLobby/main/sun_reflection.png",
  "bgs/SJ2021/BeginnerLobby/main/clouds.png",
  "bgs/SJ2021/BeginnerLobby/main/clouds_reflection.png",
  "bgs/SJ2021/BeginnerLobby/main/islands.png",
  "bgs/SJ2021/BeginnerLobby/main/islands_reflection.png",
  "bgs/SJ2021/BeginnerLobby/main/water_gradient.png",
  "bgs/SJ2021/BeginnerLobby/main/water_bg.png",
  "bgs/SJ2021/Int_Lobby/skybox.png",
  "bgs/SJ2021/Int_Lobby/bghills.png",
  "bgs/SJ2021/Int_Lobby/fghills.png",
  "bgs/SJ2021/Advanced Lobby/sunset/sunset.png",
  "bgs/SJ2021/Advanced Lobby/sunset/sunsetmountains.png",
  "bgs/SJ2021/Advanced Lobby/sunset/sunsetdunes.png",
  "bgs/SJ2021/ExpertLobby/space.png",
  "bgs/SJ2021/ExpertLobby/nebulae.png",
  "bgs/SJ2021/ExpertLobby/planets.png",
  "bgs/SJ2021/GMLobby/sky.png",
  "bgs/SJ2021/GMLobby/mountains.png",
  "bgs/SJ2021/GMLobby/cloud-group-1.png",
];

function directionalSpikes(prefix, directions) {
  const paths = [];
  for (const names of Object.values(directions)) {
    for (const name of names) paths.push(`${prefix}/${name}.png`);
  }
  return paths;
}

// Gym spikes. Note intermediate has no numeric suffix on its left texture
// (int_left.png), exactly like the mod ships it.
for (const tier of ["beg", "int", "adv", "exp", "gm"]) {
  themeAssets.push(
    ...directionalSpikes("danger/spikes/SJ2021/Gym", {
      up: [`${tier}_up00`],
      down: [`${tier}_down00`],
      left: [tier === "int" ? "int_left" : `${tier}_left00`],
      right: [`${tier}_right00`],
    }),
  );
}

// Lobby spikes.
themeAssets.push(
  ...directionalSpikes("danger/spikes/SJ2021/1-Beginner", {
    up: ["bramble_up00", "bramble_up01", "bramble_up02", "bramble_up03"],
    down: ["bramble_down00", "bramble_down01", "bramble_down02"],
    left: ["bramble_left00", "bramble_left01", "bramble_left02"],
    right: ["bramble_right00", "bramble_right01", "bramble_right02"],
  }),
);
themeAssets.push(
  ...directionalSpikes("danger/spikes/SJ2021/pixelator", {
    up: ["v_up00"],
    down: ["v_down00"],
    left: ["v_left00"],
    right: ["v_right00"],
  }),
);
themeAssets.push(
  ...directionalSpikes("danger/spikes/SJ2021/Archire", {
    up: ["orange_up00"],
    down: ["orange_down00"],
    left: ["orange_left00"],
    right: ["orange_right00"],
  }),
);
// powerav ships its downward frame under the misspelled spacet_down00 name;
// the key stays untouched so the atlas matches the mod verbatim.
themeAssets.push(
  ...directionalSpikes("danger/spikes/SJ2021/powerav", {
    up: ["space_up00"],
    down: ["spacet_down00"],
    left: ["space_left00"],
    right: ["space_right00"],
  }),
);
themeAssets.push(
  ...directionalSpikes("danger/spikes/SJ2021/Grandmaster", {
    up: ["marble_up00"],
    down: ["marble_down00"],
    left: ["marble_left00"],
    right: ["marble_right00"],
  }),
);

// Custom lobby spinner sheets. Backgrounds may be a single bg (Ceph/Julia) or
// bg00 (brambles), exactly as the SJ mod's texture caches reference them.
themeAssets.push(
  ...directionalSpikes("danger/spikes/SJ2021/1-Beginner/brambles", {
    up: ["fg00", "fg01", "fg02", "fg03"],
    down: ["bg00"],
  }),
);
themeAssets.push(
  ...directionalSpikes("danger/SJ2021/Ceph/Spinner", {
    up: ["fg00", "fg01", "fg02", "fg03"],
    down: ["bg"],
  }),
);
themeAssets.push(
  ...directionalSpikes("danger/SJ2021/Julia/Spinner", {
    up: ["fg00", "fg01", "fg02", "fg03"],
    down: ["bg"],
  }),
);

const gameplayRoot = path.join(source, "Graphics", "Atlases", "Gameplay");
const sheetWidth = 1024;
const padding = 2;
let cursorX = padding;
let cursorY = padding;
let rowHeight = 0;
const placements = [];

for (const relativePath of themeAssets) {
  const key = relativePath.slice(0, -".png".length);
  const input = path.join(gameplayRoot, ...relativePath.split("/"));
  const metadata = await sharp(input).metadata();
  if (!metadata.width || !metadata.height)
    throw new Error(`Could not read dimensions for ${relativePath}`);
  if (cursorX + metadata.width + padding > sheetWidth) {
    cursorX = padding;
    cursorY += rowHeight + padding;
    rowHeight = 0;
  }
  placements.push({
    key,
    relativePath,
    input,
    x: cursorX,
    y: cursorY,
    width: metadata.width,
    height: metadata.height,
  });
  cursorX += metadata.width + padding;
  rowHeight = Math.max(rowHeight, metadata.height);
}

const sheetHeight = cursorY + rowHeight + padding;
await mkdir(destination, { recursive: true });
await sharp({
  create: {
    width: sheetWidth,
    height: sheetHeight,
    channels: 4,
    background: { r: 0, g: 0, b: 0, alpha: 0 },
  },
})
  .composite(
    await Promise.all(
      placements.map(async (entry) => ({
        input: await readFile(entry.input),
        left: entry.x,
        top: entry.y,
      })),
    ),
  )
  .png({ compressionLevel: 9 })
  .toFile(path.join(destination, "theme-selected.png"));

const entries = Object.fromEntries(
  placements.map((entry) => [
    entry.key,
    {
      x: entry.x,
      y: entry.y,
      width: entry.width,
      height: entry.height,
      drawOffsetX: 0,
      drawOffsetY: 0,
      frameWidth: entry.width,
      frameHeight: entry.height,
    },
  ]),
);
const sources = Object.fromEntries(
  placements.map((entry) => [
    entry.key,
    `Graphics/Atlases/Gameplay/${entry.relativePath}`,
  ]),
);
await writeFile(
  path.join(destination, "theme-selected.json"),
  `${JSON.stringify({ entries, sources }, null, 2)}\n`,
  "utf8",
);
console.log(
  `Packed ${placements.length} Strawberry Jam theme textures into ${destination}`,
);
