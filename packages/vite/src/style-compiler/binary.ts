import type { StyleValue, WabouStyleSheet } from "./ir";

const MAGIC = 0x49534257; // "WBSI" in little endian
const WIRE_VERSION = 1;

class BinaryWriter {
  private bytes = new Uint8Array(4096);
  private view = new DataView(this.bytes.buffer);
  private cursor = 0;

  private ensure(length: number) {
    if (this.cursor + length <= this.bytes.length) return;
    let capacity = this.bytes.length;
    while (capacity < this.cursor + length) capacity *= 2;
    const next = new Uint8Array(capacity);
    next.set(this.bytes);
    this.bytes = next;
    this.view = new DataView(next.buffer);
  }

  u8(value: number) {
    this.ensure(1);
    this.view.setUint8(this.cursor, value);
    this.cursor += 1;
  }

  u16(value: number) {
    this.ensure(2);
    this.view.setUint16(this.cursor, value, true);
    this.cursor += 2;
  }

  u32(value: number) {
    this.ensure(4);
    this.view.setUint32(this.cursor, value, true);
    this.cursor += 4;
  }

  f32(value: number) {
    if (!Number.isFinite(value))
      throw new RangeError("Style IR numbers must be finite");
    this.ensure(4);
    this.view.setFloat32(this.cursor, value, true);
    this.cursor += 4;
  }

  raw(value: Uint8Array) {
    this.ensure(value.length);
    this.bytes.set(value, this.cursor);
    this.cursor += value.length;
  }

  finish() {
    return this.bytes.slice(0, this.cursor);
  }
}

function collectValueStrings(value: StyleValue, add: (value: string) => void) {
  switch (value.type) {
    case "keyword":
      add(value.value);
      break;
    case "color":
      if (value.value.kind === "token") add(value.value.name);
      break;
    case "list":
      for (const child of value.values) collectValueStrings(child, add);
      break;
    case "record":
      for (const [name, child] of Object.entries(value.fields)) {
        add(name);
        collectValueStrings(child, add);
      }
      break;
  }
}

/** Encode compiler Style IR into the bounded native wire format. */
export function encodeStyleSheet(sheet: WabouStyleSheet): Uint8Array {
  const strings: string[] = [];
  const ids = new Map<string, number>();
  const add = (value: string) => {
    if (!ids.has(value)) {
      ids.set(value, strings.length);
      strings.push(value);
    }
  };
  const addRecord = (record: Record<string, unknown>) => {
    for (const name of Object.keys(record)) add(name);
  };

  addRecord(sheet.theme.spacing);
  addRecord(sheet.theme.colors);
  if (sheet.colorThemes) {
    add(sheet.colorThemes.default);
    for (const [name, theme] of Object.entries(sheet.colorThemes.themes)) {
      add(name);
      addRecord(theme.colors);
    }
  }
  for (const value of sheet.diagnostics) add(value);
  for (const value of sheet.ignoredClassPatterns) add(value);
  for (const rule of sheet.rules) {
    add(rule.className);
    for (const declaration of rule.declarations) {
      add(declaration.property);
      collectValueStrings(declaration.value, add);
    }
  }

  const writer = new BinaryWriter();
  const encoder = new TextEncoder();
  const stringId = (value: string) => {
    const id = ids.get(value);
    if (id === undefined) throw new Error(`missing Style IR string: ${value}`);
    writer.u32(id);
  };
  const count = (value: number, label: string) => {
    if (!Number.isSafeInteger(value) || value < 0 || value > 0xffff_ffff)
      throw new RangeError(`${label} exceeds the Style IR wire limit`);
    writer.u32(value);
  };
  const entries = <T>(record: Record<string, T>) => Object.entries(record);
  const writeValue = (value: StyleValue, depth = 0): void => {
    if (depth > 32) throw new RangeError("Style IR value nesting exceeds 32");
    switch (value.type) {
      case "keyword":
        writer.u8(0);
        stringId(value.value);
        break;
      case "boolean":
        writer.u8(1);
        writer.u8(value.value ? 1 : 0);
        break;
      case "number":
        writer.u8(2);
        writer.f32(value.value);
        break;
      case "length":
        if (value.value.unit === "px") {
          writer.u8(3);
          writer.f32(value.value.value);
        } else if (value.value.unit === "percent") {
          writer.u8(4);
          writer.f32(value.value.value);
        } else {
          writer.u8(5);
        }
        break;
      case "color":
        if (value.value.kind === "literal") {
          writer.u8(6);
          writer.u32(value.value.rgba);
        } else {
          writer.u8(7);
          stringId(value.value.name);
        }
        break;
      case "list":
        writer.u8(8);
        count(value.values.length, "Style IR list");
        for (const child of value.values) writeValue(child, depth + 1);
        break;
      case "record": {
        writer.u8(9);
        const fields = entries(value.fields);
        count(fields.length, "Style IR record");
        for (const [name, child] of fields) {
          stringId(name);
          writeValue(child, depth + 1);
        }
        break;
      }
    }
  };

  writer.u32(MAGIC);
  writer.u16(WIRE_VERSION);
  writer.u16(sheet.version);
  count(strings.length, "Style IR string table");
  for (const value of strings) {
    const bytes = encoder.encode(value);
    count(bytes.length, "Style IR string");
    writer.raw(bytes);
  }
  const writeNumberRecord = (
    record: Record<string, number>,
    floats: boolean,
  ) => {
    const values = entries(record);
    count(values.length, "Style IR map");
    for (const [name, value] of values) {
      stringId(name);
      if (floats) writer.f32(value);
      else writer.u32(value);
    }
  };
  writeNumberRecord(sheet.theme.spacing, true);
  writeNumberRecord(sheet.theme.colors, false);
  if (sheet.colorThemes) {
    writer.u8(1);
    stringId(sheet.colorThemes.default);
    const themes = entries(sheet.colorThemes.themes);
    count(themes.length, "Style IR color themes");
    for (const [name, theme] of themes) {
      stringId(name);
      writer.u8(theme.appearance === "dark" ? 1 : 0);
      writeNumberRecord(theme.colors, false);
    }
  } else {
    writer.u8(0);
  }
  count(sheet.diagnostics.length, "Style IR diagnostics");
  for (const value of sheet.diagnostics) stringId(value);
  count(sheet.ignoredClassPatterns.length, "Style IR ignored class patterns");
  for (const value of sheet.ignoredClassPatterns) stringId(value);
  count(sheet.rules.length, "Style IR rules");
  for (const rule of sheet.rules) {
    stringId(rule.className);
    writer.u16(rule.specificity);
    writer.u32(rule.sourceOrder);
    count(rule.declarations.length, "Style IR declarations");
    for (const declaration of rule.declarations) {
      stringId(declaration.property);
      writer.u8(declaration.important ? 1 : 0);
      writeValue(declaration.value);
    }
  }
  return writer.finish();
}

/** Base64 is only an embedding format for the generated virtual JS module. */
export function styleSheetModuleSource(sheet: WabouStyleSheet): string {
  const base64 = Buffer.from(encodeStyleSheet(sheet)).toString("base64");
  return `(()=>{const s=${JSON.stringify(base64)},n=s.length,p=s.endsWith("==")?2:s.endsWith("=")?1:0,b=new Uint8Array((n>>2)*3-p),a="ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";let o=0;for(let i=0;i<n;i+=4){const x=(a.indexOf(s[i])<<18)|(a.indexOf(s[i+1])<<12)|((s[i+2]==="="?0:a.indexOf(s[i+2]))<<6)|(s[i+3]==="="?0:a.indexOf(s[i+3]));if(o<b.length)b[o++]=x>>16;if(o<b.length)b[o++]=x>>8;if(o<b.length)b[o++]=x}globalThis.__wabou_set_stylesheet?.(b)})();export {};`;
}
