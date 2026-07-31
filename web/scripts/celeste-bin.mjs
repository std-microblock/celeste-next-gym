// Build-time Celeste BinaryPacker (.bin) reader. Mirrors
// crates/celeste-physics/src/binary_packer.rs and is used by the mod theme
// extraction tool; it is not part of the web runtime.

class BinaryReader {
  constructor(bytes) {
    this.view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    this.pos = 0;
  }
  u8() {
    const value = this.view.getUint8(this.pos);
    this.pos += 1;
    return value;
  }
  i16() {
    const value = this.view.getInt16(this.pos, true);
    this.pos += 2;
    return value;
  }
  i32() {
    const value = this.view.getInt32(this.pos, true);
    this.pos += 4;
    return value;
  }
  f32() {
    const value = this.view.getFloat32(this.pos, true);
    this.pos += 4;
    return value;
  }
  string() {
    let length = 0;
    let shift = 0;
    for (;;) {
      const byte = this.u8();
      length |= (byte & 0x7f) << shift;
      if ((byte & 0x80) === 0) break;
      shift += 7;
    }
    const value = new TextDecoder().decode(
      new Uint8Array(this.view.buffer, this.view.byteOffset + this.pos, length),
    );
    this.pos += length;
    return value;
  }
}

function readElement(reader, table, depth) {
  if (depth > 256) return null;
  const name = table[reader.i16()];
  if (name === undefined) return null;
  const attributes = {};
  const attributeCount = reader.u8();
  for (let index = 0; index < attributeCount; index += 1) {
    const key = table[reader.i16()];
    if (key === undefined) return null;
    const type = reader.u8();
    switch (type) {
      case 0:
        attributes[key] = reader.u8() !== 0;
        break;
      case 1:
        attributes[key] = reader.u8();
        break;
      case 2:
        attributes[key] = reader.i16();
        break;
      case 3:
        attributes[key] = reader.i32();
        break;
      case 4:
        attributes[key] = reader.f32();
        break;
      case 5:
        attributes[key] = table[reader.i16()];
        break;
      case 6:
        attributes[key] = reader.string();
        break;
      case 7: {
        const length = reader.i16();
        let value = "";
        for (let offset = 0; offset < length; offset += 2) {
          const repeat = reader.u8();
          const charCode = reader.u8();
          value += String.fromCharCode(charCode).repeat(repeat);
        }
        attributes[key] = value;
        break;
      }
      default:
        return null;
    }
  }
  const childCount = reader.i16();
  if (childCount < 0) return null;
  const children = [];
  for (let index = 0; index < childCount; index += 1) {
    const child = readElement(reader, table, depth + 1);
    if (!child) return null;
    children.push(child);
  }
  return { name, attributes, children };
}

export function parseCelesteBin(bytes) {
  const reader = new BinaryReader(new Uint8Array(bytes));
  if (reader.string() !== "CELESTE MAP") return null;
  const packageName = reader.string();
  const count = reader.i16();
  if (count < 0 || count > 32767) return null;
  const table = [];
  for (let index = 0; index < count; index += 1) table.push(reader.string());
  const root = readElement(reader, table, 0);
  if (!root) return null;
  root.package = packageName;
  return root;
}

export function attrText(element, key) {
  const value = element?.attributes?.[key];
  return typeof value === "string" ? value : undefined;
}

export function attrNumber(element, key, fallback = 0) {
  const value = element?.attributes?.[key];
  if (typeof value === "boolean") return fallback;
  if (typeof value === "number") return Number.isFinite(value) ? value : fallback;
  if (typeof value === "string") {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : fallback;
  }
  return fallback;
}

export function attrBool(element, key, fallback = false) {
  const value = element?.attributes?.[key];
  if (typeof value === "boolean") return value;
  if (typeof value === "number") return value !== 0;
  if (typeof value === "string") return value === "true";
  return fallback;
}
