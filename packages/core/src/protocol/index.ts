// Wabou binary bridge protocol — single source of truth for the wire format.
// New records and handles must follow docs/runtime-contract.md.
//
// OP and EVENT_CODE here are the SOT. Rust constants are generated from them
// by `bun run gen` (scripts/gen-rust-op.ts →
// crates/wabou-runtime/src/gen/op.rs, include!d by
// src/protocol.rs). A drift-guard test in protocol.rs asserts every opcode
// decodes, so a stale regen surfaces as a test failure.

import type { NodeKey } from "./node-key";

export {
  formatNodeKey,
  isNodeKey,
  type NodeKey,
  NodeKeyAllocator,
  NodeKeyTable,
  nodeKey,
  nodeKeyEquals,
  nodeKeyFromSlotMapFfi,
  ROOT_NODE_KEY,
} from "./node-key";
export {
  createResourceKeyFamily,
  formatResourceKeyParts,
  isResourceKeyParts,
  type ResourceKey,
  type ResourceKeyFamily,
  type ResourceKeyParts,
  ResourceKeyTable,
  validateResourceKeyParts,
} from "./resource-key";

export const OP = {
  CreateElement: 0x01,
  CreateText: 0x02,
  AppendChild: 0x04,
  InsertBefore: 0x05,
  RemoveChild: 0x06,
  SetText: 0x08,
  SetAttribute: 0x09,
  RemoveAttribute: 0x0a,
  SetStyle: 0x0b,
  RemoveStyle: 0x0c,
  AddEventListener: 0x0d,
  RemoveEventListener: 0x0e,
  SetClassName: 0x0f,
  DropNode: 0x11,
  SetTransform2D: 0x12,
  FocusNode: 0x13,
  ScrollTo: 0x14,
  ScrollBy: 0x15,
  SetStyleValue: 0x16,
  SetShadows: 0x17,
  SetOverlayPlane: 0x18,
  SetScrollbarStyle: 0x19,
  SetWidgetConfig: 0x1a,
  RemoveWidgetConfig: 0x1b,
  SetTextBehavior: 0x1c,
  SetInteractionPolicy: 0x1d,
  SetGraphicSource: 0x1e,
  ClearGraphicSource: 0x1f,
  SetGraphicData: 0x20,
  ClearGraphicData: 0x21,
  SetTextMaxLines: 0x22,
} as const;

export type OpCode = (typeof OP)[keyof typeof OP];

export const TEXT_BEHAVIOR = {
  AggregateDirectText: 0x01,
  SingleLine: 0x02,
} as const;
const TEXT_BEHAVIOR_MASK =
  TEXT_BEHAVIOR.AggregateDirectText | TEXT_BEHAVIOR.SingleLine;

export const INTERACTION_POLICY = {
  Focusable: 0x01,
  BlockSubtree: 0x02,
  ContainFocus: 0x04,
} as const;
const INTERACTION_POLICY_MASK =
  INTERACTION_POLICY.Focusable |
  INTERACTION_POLICY.BlockSubtree |
  INTERACTION_POLICY.ContainFocus;

export const GRAPHIC_SOURCE = {
  Svg: 0x01,
  NetworkRaster: 0x02,
  FileRaster: 0x03,
} as const;

export const GRAPHIC_DATA = { VectorPath: 0x01 } as const;
const MAX_GRAPHIC_DATA_BYTES = 16 * 1024 * 1024;

function validGraphicSourceKind(kind: number): boolean {
  return (
    kind === GRAPHIC_SOURCE.Svg ||
    kind === GRAPHIC_SOURCE.NetworkRaster ||
    kind === GRAPHIC_SOURCE.FileRaster
  );
}

export const EVENT_CODE = {
  click: 1,
  input: 2,
  submit: 3,
  keydown: 4,
  keyup: 5,
  change: 6,
  pointerdown: 7,
  pointermove: 8,
  pointerup: 9,
  pointerenter: 10,
  pointerleave: 11,
  wheel: 12,
  focus: 13,
  blur: 14,
  imecommit: 15,
  pointercancel: 16,
  pointerover: 17,
  pointerout: 18,
  contextmenu: 19,
  dblclick: 20,
  focusin: 21,
  focusout: 22,
  scroll: 23,
  terminalexit: 24,
  terminalprogress: 25,
  terminalnotification: 26,
  terminaltitlechange: 27,
  terminalcwdchange: 28,
  terminalselectionchange: 29,
  textselectionchange: 30,
  terminalbell: 31,
  resourceready: 32,
  resourceerror: 33,
} as const;

export type EventType = keyof typeof EVENT_CODE;

export const EVENT_DATA_SLOT = {
  clientX: 0,
  clientY: 1,
  offsetX: 2,
  offsetY: 3,
  button: 4,
  buttons: 5,
  mods: 6,
  deltaX: 7,
  deltaY: 8,
  scrollX: 9,
  scrollY: 10,
} as const;

export const EVENT_DATA_LEN = Object.keys(EVENT_DATA_SLOT).length;
export type EventDataSlot = keyof typeof EVENT_DATA_SLOT;

/** Versioned Host → JS frame envelope. Keep in sync through `bun run gen`. */
export const HOST_FRAME = {
  Magic: 0x31464857,
  Version: 2,
  HeaderLen: 32,
} as const;

export const HOST_RECORD_KIND = {
  NodeEvent: 1,
  Resize: 2,
  ApplicationMessage: 3,
  Window: 4,
  Widget: 5,
} as const;

export const HOST_NODE_PAYLOAD = {
  None: 0,
  Numeric: 1,
  Json: 2,
} as const;

// Non-host fallback used by unit tests and standalone renderer tooling. A real
// Wabou runtime provides Rust-authoritative IDs through its private ABI.
const fallbackAtoms = new Map<string, number>();
function fallbackIntern(value: string): number {
  let id = fallbackAtoms.get(value);
  if (id === undefined) {
    id = fallbackAtoms.size + 1;
    fallbackAtoms.set(value, id);
  }
  return id;
}

// TextEncoder is installed by `@wabou/core` (backed by the host
// `__wabou_utf8_encode` fn), but this module evaluates before core does
// (protocol -> solid-renderer -> core load order). So we resolve it lazily on
// first use rather than capturing it at module top.
let encoder: TextEncoder | null | undefined;
const FLOAT_VIEW = new DataView(new ArrayBuffer(4));
function utf8Encode(s: string): Uint8Array {
  if (encoder === undefined) {
    encoder = typeof TextEncoder !== "undefined" ? new TextEncoder() : null;
  }
  // Fall back to a manual UTF-8 encoder only if no TextEncoder was ever
  // installed (e.g. running outside the wabou host). The host path is the
  // normal one.
  if (encoder) return encoder.encode(s);
  const out: number[] = [];
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c < 0x80) out.push(c);
    else if (c < 0x800) out.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f));
    else if (c >= 0xd800 && c <= 0xdbff) {
      const c2 = s.charCodeAt(++i);
      const cp = 0x10000 + ((c & 0x3ff) << 10) + (c2 & 0x3f);
      out.push(
        0xf0 | (cp >> 18),
        0x80 | ((cp >> 12) & 0x3f),
        0x80 | ((cp >> 6) & 0x3f),
        0x80 | (cp & 0x3f),
      );
    } else {
      out.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
    }
  }
  return new Uint8Array(out);
}

/**
 * Per-tick binary frame writer. Emits ops into an internal buffer; `flush()`
 * returns the complete frame (header + ops) or null if nothing was emitted.
 * The caller owns how the bytes cross the host bridge.
 */
export class Writer {
  private buf = new Uint8Array(4096);
  private cursor = 8; // Reserve first 8 bytes for header (seq u32 + count u32)
  private count = 0;
  private seq = 0;
  private atoms = new Map<string, number>();
  /** Strings already emitted in this frame (only values large enough to win). */
  private frameStrings = new Map<string, number>();
  private internHost: (value: string) => number;

  constructor(internHost?: (value: string) => number) {
    this.internHost =
      internHost ??
      ((value) => {
        const hostIntern = (
          globalThis as typeof globalThis & {
            __wabou_intern?: (value: string) => number;
          }
        ).__wabou_intern;
        return hostIntern ? hostIntern(value) : fallbackIntern(value);
      });
  }

  private ensure(n: number): void {
    if (this.cursor + n <= this.buf.length) return;
    let cap = this.buf.length;
    while (cap < this.cursor + n) cap *= 2;
    const next = new Uint8Array(cap);
    next.set(this.buf);
    this.buf = next;
  }

  private u8(v: number): void {
    this.ensure(1);
    this.buf[this.cursor++] = v & 0xff;
  }
  private u16(v: number): void {
    this.ensure(2);
    const c = this.cursor;
    this.buf[c] = v & 0xff;
    this.buf[c + 1] = (v >> 8) & 0xff;
    this.cursor += 2;
  }
  private u32(v: number): void {
    this.ensure(4);
    const c = this.cursor;
    this.buf[c] = v & 0xff;
    this.buf[c + 1] = (v >> 8) & 0xff;
    this.buf[c + 2] = (v >> 16) & 0xff;
    this.buf[c + 3] = (v >> 24) & 0xff;
    this.cursor += 4;
  }
  private key(value: NodeKey): void {
    this.u32(value.lo);
    this.u32(value.hi);
  }
  private f32(v: number): void {
    FLOAT_VIEW.setFloat32(0, v, true);
    this.u32(FLOAT_VIEW.getUint32(0, true));
  }
  private str(s: string): void {
    const existing = this.frameStrings.get(s);
    if (existing !== undefined) {
      this.u16(0xffff);
      this.u16(existing);
      return;
    }
    const bytes = utf8Encode(s);
    if (bytes.length >= 0xffff) {
      throw new RangeError(
        `protocol string is ${bytes.length} bytes; maximum is 65534`,
      );
    }
    this.u16(bytes.length);
    this.ensure(bytes.length);
    this.buf.set(bytes, this.cursor);
    this.cursor += bytes.length;
    // A reference occupies four bytes (marker + index), so indexing shorter
    // values cannot reduce frame size. The index is frame-local and discarded
    // on flush, avoiding unbounded retention of dynamic UI text.
    if (bytes.length >= 4 && this.frameStrings.size <= 0xffff) {
      this.frameStrings.set(s, this.frameStrings.size);
    }
  }

  private atom(value: string): void {
    let id = this.atoms.get(value);
    if (id === undefined) {
      id = this.internHost(value);
      if (!Number.isInteger(id) || id <= 0 || id > 0xffff_ffff) {
        throw new RangeError(`invalid Atom ID ${id}`);
      }
      this.atoms.set(value, id);
    }
    this.u32(id);
  }

  private emit(op: OpCode): void {
    if (this.count === 0xffff_ffff) {
      throw new RangeError(
        "protocol frame cannot contain more than 2^32-1 ops",
      );
    }
    this.u8(op);
    this.count++;
  }

  createElement(id: NodeKey, tag: string): void {
    this.emit(OP.CreateElement);
    this.key(id);
    this.atom(tag);
  }
  createText(id: NodeKey, text: string): void {
    this.emit(OP.CreateText);
    this.key(id);
    this.str(text);
  }
  appendChild(parent: NodeKey, child: NodeKey): void {
    this.emit(OP.AppendChild);
    this.key(parent);
    this.key(child);
  }
  insertBefore(parent: NodeKey, child: NodeKey, ref: NodeKey): void {
    this.emit(OP.InsertBefore);
    this.key(parent);
    this.key(child);
    this.key(ref);
  }
  removeChild(parent: NodeKey, child: NodeKey): void {
    this.emit(OP.RemoveChild);
    this.key(parent);
    this.key(child);
  }
  setText(id: NodeKey, text: string): void {
    this.emit(OP.SetText);
    this.key(id);
    this.str(text);
  }
  setAttribute(id: NodeKey, name: string, value: string): void {
    this.emit(OP.SetAttribute);
    this.key(id);
    this.atom(name);
    this.str(value);
  }
  removeAttribute(id: NodeKey, name: string): void {
    this.emit(OP.RemoveAttribute);
    this.key(id);
    this.atom(name);
  }
  setWidgetConfig(id: NodeKey, json: string): void {
    this.emit(OP.SetWidgetConfig);
    this.key(id);
    this.str(json);
  }
  setTextBehavior(id: NodeKey, flags: number): void {
    if (
      !Number.isInteger(flags) ||
      flags < 0 ||
      (flags & ~TEXT_BEHAVIOR_MASK) !== 0
    ) {
      throw new RangeError(`invalid text behavior flags ${flags}`);
    }
    this.emit(OP.SetTextBehavior);
    this.key(id);
    this.u8(flags);
  }
  setTextMaxLines(id: NodeKey, maxLines: number): void {
    if (!Number.isInteger(maxLines) || maxLines < 0 || maxLines > 0xffffffff) {
      throw new RangeError(`invalid text max lines ${maxLines}`);
    }
    this.emit(OP.SetTextMaxLines);
    this.key(id);
    this.u32(maxLines);
  }
  setInteractionPolicy(id: NodeKey, flags: number, focusOrder: number): void {
    if (
      !Number.isInteger(flags) ||
      flags < 0 ||
      (flags & ~INTERACTION_POLICY_MASK) !== 0
    ) {
      throw new RangeError(`invalid interaction policy flags ${flags}`);
    }
    if (
      !Number.isInteger(focusOrder) ||
      focusOrder < -0x80000000 ||
      focusOrder > 0x7fffffff
    ) {
      throw new RangeError(`invalid focus order ${focusOrder}`);
    }
    if ((flags & INTERACTION_POLICY.Focusable) === 0 && focusOrder !== 0) {
      throw new RangeError("a non-focusable policy must encode focus order 0");
    }
    this.emit(OP.SetInteractionPolicy);
    this.key(id);
    this.u8(flags);
    this.u32(focusOrder >>> 0);
  }
  setGraphicSource(id: NodeKey, kind: number, source: string): void {
    if (!validGraphicSourceKind(kind)) {
      throw new RangeError(`invalid graphic source kind ${kind}`);
    }
    this.emit(OP.SetGraphicSource);
    this.key(id);
    this.u8(kind);
    this.str(source);
  }
  clearGraphicSource(id: NodeKey, kind: number): void {
    if (!validGraphicSourceKind(kind)) {
      throw new RangeError(`invalid graphic source kind ${kind}`);
    }
    this.emit(OP.ClearGraphicSource);
    this.key(id);
    this.u8(kind);
  }
  setGraphicData(id: NodeKey, kind: number, data: Uint8Array): void {
    if (kind !== GRAPHIC_DATA.VectorPath)
      throw new RangeError(`invalid graphic data kind ${kind}`);
    if (data.byteLength > MAX_GRAPHIC_DATA_BYTES)
      throw new RangeError("graphic data exceeds the 16 MiB protocol limit");
    this.emit(OP.SetGraphicData);
    this.key(id);
    this.u8(kind);
    this.u32(data.byteLength);
    this.ensure(data.byteLength);
    this.buf.set(data, this.cursor);
    this.cursor += data.byteLength;
  }
  clearGraphicData(id: NodeKey, kind: number): void {
    if (kind !== GRAPHIC_DATA.VectorPath)
      throw new RangeError(`invalid graphic data kind ${kind}`);
    this.emit(OP.ClearGraphicData);
    this.key(id);
    this.u8(kind);
  }
  removeWidgetConfig(id: NodeKey): void {
    this.emit(OP.RemoveWidgetConfig);
    this.key(id);
  }
  setStyle(id: NodeKey, prop: string, value: string): void {
    this.emit(OP.SetStyle);
    this.key(id);
    this.atom(prop);
    this.str(value);
  }
  setStyleValue(id: NodeKey, prop: string, kind: number, value: number): void {
    this.emit(OP.SetStyleValue);
    this.key(id);
    this.atom(prop);
    this.u8(kind);
    if (kind !== 6) {
      if (kind === 5) this.u32(value >>> 0);
      else if (kind === 4) this.u8(value ? 1 : 0);
      else this.f32(value);
    }
  }
  setShadows(
    id: NodeKey,
    shadows: readonly {
      offsetX: number;
      offsetY: number;
      spread: number;
      stdDev: number;
      color: number;
      radius?: number;
    }[],
  ): void {
    if (shadows.length > 0xffff) {
      throw new RangeError("a node cannot have more than 65535 shadow layers");
    }
    this.emit(OP.SetShadows);
    this.key(id);
    this.u16(shadows.length);
    for (const shadow of shadows) {
      this.f32(shadow.offsetX);
      this.f32(shadow.offsetY);
      this.f32(shadow.spread);
      this.f32(shadow.stdDev);
      this.u32(shadow.color >>> 0);
      this.f32(shadow.radius ?? Number.NaN);
    }
  }
  setTransform2D(
    id: NodeKey,
    matrix: readonly [number, number, number, number, number, number],
  ): void {
    this.emit(OP.SetTransform2D);
    this.key(id);
    for (const part of matrix) this.f32(part);
  }
  setOverlayPlane(id: NodeKey, plane: number): void {
    this.emit(OP.SetOverlayPlane);
    this.key(id);
    this.u8(plane);
  }
  setScrollbarStyle(
    id: NodeKey,
    style: {
      visibility: number;
      hideDelay: number;
      fadeDuration: number;
      thickness: number;
      margin: number;
      minThumbLength: number;
      radius: number;
      trackColor: number;
      thumbColor: number;
      hoverColor: number;
      activeColor: number;
    },
  ): void {
    this.emit(OP.SetScrollbarStyle);
    this.key(id);
    this.u8(style.visibility);
    this.f32(style.hideDelay);
    this.f32(style.fadeDuration);
    this.f32(style.thickness);
    this.f32(style.margin);
    this.f32(style.minThumbLength);
    this.f32(style.radius);
    this.u32(style.trackColor >>> 0);
    this.u32(style.thumbColor >>> 0);
    this.u32(style.hoverColor >>> 0);
    this.u32(style.activeColor >>> 0);
  }
  removeStyle(id: NodeKey, prop: string): void {
    this.emit(OP.RemoveStyle);
    this.key(id);
    this.atom(prop);
  }
  addEventListener(id: NodeKey, eventCode: number): void {
    this.emit(OP.AddEventListener);
    this.key(id);
    this.u8(eventCode);
  }
  removeEventListener(id: NodeKey, eventCode: number): void {
    this.emit(OP.RemoveEventListener);
    this.key(id);
    this.u8(eventCode);
  }
  setClassName(id: NodeKey, value: string): void {
    this.emit(OP.SetClassName);
    this.key(id);
    const classes = value.split(/\s+/).filter(Boolean);
    if (classes.length > 0xffff) {
      throw new RangeError("class list cannot contain more than 65535 tokens");
    }
    this.u16(classes.length);
    for (const className of classes) this.atom(className);
  }
  dropNode(id: NodeKey): void {
    this.emit(OP.DropNode);
    this.key(id);
  }
  focusNode(id: NodeKey): void {
    this.emit(OP.FocusNode);
    this.key(id);
  }
  scrollTo(id: NodeKey, x: number, y: number): void {
    this.emit(OP.ScrollTo);
    this.key(id);
    this.f32(x);
    this.f32(y);
  }
  scrollBy(id: NodeKey, x: number, y: number): void {
    this.emit(OP.ScrollBy);
    this.key(id);
    this.f32(x);
    this.f32(y);
  }

  /** Drain the buffer into a frame, or null if no ops were emitted this tick. */
  flush(): Uint8Array | null {
    if (this.count === 0) return null;
    this.seq++;
    const s = this.seq;
    this.buf[0] = s & 0xff;
    this.buf[1] = (s >> 8) & 0xff;
    this.buf[2] = (s >> 16) & 0xff;
    this.buf[3] = (s >> 24) & 0xff;
    this.buf[4] = this.count & 0xff;
    this.buf[5] = (this.count >> 8) & 0xff;
    this.buf[6] = (this.count >> 16) & 0xff;
    this.buf[7] = (this.count >> 24) & 0xff;
    const out = this.buf.subarray(0, this.cursor);
    this.cursor = 8;
    this.count = 0;
    this.frameStrings.clear();
    return out;
  }
}
