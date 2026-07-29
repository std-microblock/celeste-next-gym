import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import sharp from "sharp";

const source = process.argv[2];
const destination = process.argv[3];
const listPattern = process.argv
  .find((argument) => argument.startsWith("--list="))
  ?.slice(7);

if (!source) {
  console.error(
    "usage: node scripts/extract-gameplay-atlas.mjs <Celeste Content/Graphics/Atlases> [output] [--list=regex]",
  );
  process.exit(2);
}

class Reader {
  offset = 0;
  constructor(buffer) {
    this.buffer = buffer;
  }
  i16() {
    const value = this.buffer.readInt16LE(this.offset);
    this.offset += 2;
    return value;
  }
  i32() {
    const value = this.buffer.readInt32LE(this.offset);
    this.offset += 4;
    return value;
  }
  string() {
    let length = 0;
    let shift = 0;
    let byte;
    do {
      byte = this.buffer[this.offset++];
      length |= (byte & 0x7f) << shift;
      shift += 7;
    } while (byte & 0x80);
    const value = this.buffer.toString(
      "utf8",
      this.offset,
      this.offset + length,
    );
    this.offset += length;
    return value;
  }
}

function parseMeta(buffer) {
  const reader = new Reader(buffer);
  reader.i32();
  const sourceHash = reader.string();
  reader.i32();
  const pages = [];
  const pageCount = reader.i16();
  for (let pageIndex = 0; pageIndex < pageCount; pageIndex += 1) {
    const page = { name: reader.string(), entries: [] };
    const entryCount = reader.i16();
    for (let entryIndex = 0; entryIndex < entryCount; entryIndex += 1) {
      page.entries.push({
        name: reader.string().replaceAll("\\", "/"),
        x: reader.i16(),
        y: reader.i16(),
        width: reader.i16(),
        height: reader.i16(),
        trimX: reader.i16(),
        trimY: reader.i16(),
        frameWidth: reader.i16(),
        frameHeight: reader.i16(),
      });
    }
    pages.push(page);
  }
  return { sourceHash, pages };
}

function decodeTexture(buffer) {
  const width = buffer.readInt32LE(0);
  const height = buffer.readInt32LE(4);
  const alpha = buffer[8] === 1;
  const pixels = Buffer.alloc(width * height * 4);
  let sourceOffset = 9;
  let pixel = 0;
  while (pixel < width * height) {
    const run = buffer[sourceOffset];
    if (run === 0)
      throw new Error(`invalid zero-length run at ${sourceOffset}`);
    let red = 0;
    let green = 0;
    let blue = 0;
    let opacity = 255;
    if (alpha) {
      opacity = buffer[sourceOffset + 1];
      if (opacity > 0) {
        blue = buffer[sourceOffset + 2];
        green = buffer[sourceOffset + 3];
        red = buffer[sourceOffset + 4];
        sourceOffset += 5;
      } else {
        sourceOffset += 2;
      }
    } else {
      blue = buffer[sourceOffset + 1];
      green = buffer[sourceOffset + 2];
      red = buffer[sourceOffset + 3];
      sourceOffset += 4;
    }
    for (let repeat = 0; repeat < run; repeat += 1) {
      const target = (pixel + repeat) * 4;
      pixels[target] = red;
      pixels[target + 1] = green;
      pixels[target + 2] = blue;
      pixels[target + 3] = opacity;
    }
    pixel += run;
  }
  return { width, height, pixels };
}

const PLAYER_PATHS = [
  "characters/player/idle",
  "characters/player/runSlow",
  "characters/player/runFast",
  "characters/player/jumpSlow",
  "characters/player/jumpFast",
  "characters/player/dash",
  "characters/player/climb",
  "characters/player/duck",
  "characters/player/death_h",
  "characters/player/dreamDash",
  "characters/player/swim",
  "characters/player/startStarFly",
  "characters/player/startStarFlyWhite",
  "characters/player/starFly",
  "characters/player/launch",
  "characters/player/hair",
  "characters/player/bangs",
];
const GAMEPLAY_PATHS = [
  ...PLAYER_PATHS,
  "tilesets/dirt",
  "tilesets/snow",
  "tilesets/stone",
  "tilesets/wood",
  "tilesets/cliffside",
  "tilesets/summit",
  "tilesets/bgDirt",
  "tilesets/bgSnow",
  "bgs/01/bg0",
  "bgs/01/bg1",
  "bgs/01/bg2",
  "bgs/02/stars/",
  "bgs/03/bg0",
  "bgs/03/bg1",
  "bgs/03/bg2",
  "bgs/03/bg3",
  "bgs/03/fg0",
  "bgs/04/bg0",
  "bgs/04/bg1",
  "bgs/04/bgCloud",
  "bgs/07/bg0",
  "bgs/07/00/bg1",
  "bgs/07/00/bg2",
  "danger/spikes/",
  "objects/woodPlatform/default",
  "objects/jumpthru/wood",
  "objects/booster/booster",
  "objects/booster/boosterRed",
  "objects/booster/outline",
  "objects/dreamblock/active",
  "objects/dreamblock/particles",
  "objects/flyFeather/",
  "objects/Bumper/",
  "objects/badelineboost/",
  "objects/spring/",
  "objects/fireball/",
  "characters/theoCrystal/idle",
  "objects/glider/",
  "objects/puffer/",
  "objects/clouds/cloud",
  "objects/moveBlock/",
  "characters/oshiro/boss",
  "characters/monsters/predator",
  "danger/snowball",
  "collectables/heartGem/",
  "objects/cassetteblock/",
  "danger/crystal/",
  "collectables/strawberry/normal",
  "objects/zipmover/",
  "objects/BumpBlockNew/fire00",
  "objects/BumpBlockNew/fire_center",
  "particles/snow",
];

const meta = parseMeta(await readFile(path.join(source, "Gameplay.meta")));
if (listPattern) {
  const matcher = new RegExp(listPattern, "i");
  for (const page of meta.pages) {
    for (const entry of page.entries)
      if (matcher.test(entry.name)) console.log(entry.name);
  }
  process.exit(0);
}
if (!destination)
  throw new Error("output directory is required unless --list is used");

const selected = [];
for (const page of meta.pages) {
  for (const entry of page.entries) {
    if (
      GAMEPLAY_PATHS.some(
        (prefix) => entry.name === prefix || entry.name.startsWith(prefix),
      )
    )
      selected.push({ ...entry, page: page.name });
  }
}

const decodedPages = new Map();
for (const pageName of new Set(selected.map((entry) => entry.page))) {
  decodedPages.set(
    pageName,
    decodeTexture(await readFile(path.join(source, `${pageName}.data`))),
  );
}

const sheetWidth = 1024;
const padding = 2;
let cursorX = padding;
let cursorY = padding;
let rowHeight = 0;
const placements = [];
for (const entry of selected) {
  if (cursorX + entry.width + padding > sheetWidth) {
    cursorX = padding;
    cursorY += rowHeight + padding;
    rowHeight = 0;
  }
  placements.push({ ...entry, sheetX: cursorX, sheetY: cursorY });
  cursorX += entry.width + padding;
  rowHeight = Math.max(rowHeight, entry.height);
}
const sheetHeight = cursorY + rowHeight + padding;
const composites = [];
for (const entry of placements) {
  const page = decodedPages.get(entry.page);
  const crop = await sharp(page.pixels, {
    raw: { width: page.width, height: page.height, channels: 4 },
  })
    .extract({
      left: entry.x,
      top: entry.y,
      width: entry.width,
      height: entry.height,
    })
    .png()
    .toBuffer();
  composites.push({ input: crop, left: entry.sheetX, top: entry.sheetY });
}

await mkdir(destination, { recursive: true });
await sharp({
  create: {
    width: sheetWidth,
    height: sheetHeight,
    channels: 4,
    background: { r: 0, g: 0, b: 0, alpha: 0 },
  },
})
  .composite(composites)
  .png({ compressionLevel: 9 })
  .toFile(path.join(destination, "gameplay-selected.png"));

const manifest = Object.fromEntries(
  placements.map((entry) => [
    entry.name,
    {
      x: entry.sheetX,
      y: entry.sheetY,
      width: entry.width,
      height: entry.height,
      drawOffsetX: -entry.trimX,
      drawOffsetY: -entry.trimY,
      frameWidth: entry.frameWidth,
      frameHeight: entry.frameHeight,
    },
  ]),
);
await writeFile(
  path.join(destination, "gameplay-selected.json"),
  `${JSON.stringify({ sourceHash: meta.sourceHash, entries: manifest }, null, 2)}\n`,
  "utf8",
);
console.log(
  `Extracted ${placements.length} original Gameplay textures to ${destination}`,
);
