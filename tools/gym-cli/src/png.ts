/**
 * Minimal PNG encoder (8-bit RGBA, non-interlaced) on top of Node's zlib.
 *
 * Skia's encoder — `canvas.toBuffer("image/png")` — costs ~11 ms for a 640x360
 * game frame and silently ignores `compressionLevel`, which dominates a PNG
 * sequence render (`cg ... --render frames/`). Filtering the rows ourselves and
 * handing them to `zlib.deflateSync` is ~1.5x faster and produces slightly
 * smaller files (measured on a Forsaken City frame: 24.5 KB vs 27.1 KB).
 *
 * The Up filter wins here because consecutive rows of a game frame are nearly
 * identical (static backdrop, tiled terrain): level 6 + Up beats level 1 + Sub
 * on size *and* time.
 */
import { deflate, deflateSync } from "node:zlib";

const SIGNATURE = Uint8Array.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

const CRC_TABLE = new Uint32Array(256);
for (let n = 0; n < 256; n += 1) {
  let c = n;
  for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  CRC_TABLE[n] = c >>> 0;
}

function crc32(bytes: Uint8Array): number {
  let c = 0xffffffff;
  for (let i = 0; i < bytes.length; i += 1) c = CRC_TABLE[(c ^ bytes[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type: string, data: Uint8Array): Buffer {
  const out = Buffer.allocUnsafe(12 + data.length);
  out.writeUInt32BE(data.length, 0);
  out.write(type, 4, "latin1");
  Buffer.from(data.buffer, data.byteOffset, data.length).copy(out, 8);
  out.writeUInt32BE(crc32(out.subarray(4, 8 + data.length)), 8 + data.length);
  return out;
}

export interface PngEncodeOptions {
  /** zlib level (0-9). 6 matches Skia's size at 1.5x the speed. */
  level?: number;
}

/** Filter RGBA rows into the PNG scanline stream (one Up-filter byte per row). */
export function filterRows(
  pixels: Uint8Array | Uint8ClampedArray,
  width: number,
  height: number,
): Buffer {
  const stride = width * 4;
  if (pixels.length < stride * height) {
    throw new Error(`encodePng: expected ${stride * height} bytes for ${width}x${height}, got ${pixels.length}`);
  }
  const raw = Buffer.allocUnsafe((stride + 1) * height);
  for (let y = 0; y < height; y += 1) {
    const to = y * (stride + 1);
    const from = y * stride;
    raw[to] = 2;
    if (y === 0) {
      for (let x = 0; x < stride; x += 1) raw[to + 1 + x] = pixels[from + x];
    } else {
      const above = from - stride;
      for (let x = 0; x < stride; x += 1) raw[to + 1 + x] = (pixels[from + x] - pixels[above + x]) & 0xff;
    }
  }
  return raw;
}

function assemblePng(filtered: Buffer, width: number, height: number, idat: Buffer): Buffer {
  const ihdr = Buffer.allocUnsafe(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // colour type: RGBA
  ihdr[10] = 0; // deflate
  ihdr[11] = 0; // adaptive filtering
  ihdr[12] = 0; // no interlace
  return Buffer.concat([SIGNATURE, chunk("IHDR", ihdr), chunk("IDAT", idat), chunk("IEND", new Uint8Array(0))]);
}

/** Encode straight (non-premultiplied) RGBA8 pixels as a PNG. */
export function encodePng(
  pixels: Uint8Array | Uint8ClampedArray,
  width: number,
  height: number,
  options: PngEncodeOptions = {},
): Buffer {
  const level = options.level ?? 6;
  const filtered = filterRows(pixels, width, height);
  return assemblePng(filtered, width, height, deflateSync(filtered, { level: level as 9 }));
}

/**
 * Same, but the deflate runs on libuv's thread pool. A PNG sequence can keep
 * several of these in flight, so compressing frame N overlaps drawing frame
 * N+1 instead of serialising them.
 */
export function encodePngAsync(
  pixels: Uint8Array | Uint8ClampedArray,
  width: number,
  height: number,
  options: PngEncodeOptions = {},
): Promise<Buffer> {
  const level = options.level ?? 6;
  const filtered = filterRows(pixels, width, height);
  return new Promise((resolvePromise, reject) => {
    deflate(filtered, { level: level as 9 }, (error, idat) => {
      if (error) reject(error);
      else resolvePromise(assemblePng(filtered, width, height, idat));
    });
  });
}
