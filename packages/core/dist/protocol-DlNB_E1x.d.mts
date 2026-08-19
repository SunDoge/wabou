//#region src/protocol/node-key.d.ts
declare const nodeKeyBrand: unique symbol;
/**
 * Full-width retained-node identity used on both sides of the native bridge.
 * `lo` selects a slot and `hi` identifies that slot's generation.
 */
interface NodeKey {
  readonly lo: number;
  readonly hi: number;
  readonly [nodeKeyBrand]: "NodeKey";
}
/** Structural form accepted when a key was deserialized from JSON. */
type NodeKeyParts = Pick<NodeKey, "lo" | "hi">;
declare const ROOT_NODE_KEY: NodeKey;
/** Construct a node key received from a trusted binary boundary. */
declare function nodeKey(lo: number, hi: number): NodeKey;
declare function isNodeKey(value: unknown): value is NodeKey;
declare function nodeKeyEquals(left: NodeKey | null | undefined, right: NodeKey | null | undefined): boolean;
/** Stable diagnostic form; do not use it on the binary hot path. */
declare function formatNodeKey(key: NodeKeyParts): string;
/**
 * Splits SlotMap's `KeyData::as_ffi()` representation without converting the
 * full value to an imprecise JavaScript number.
 */
declare function nodeKeyFromSlotMapFfi(lo: number, hi: number): NodeKey;
/**
 * Allocates full-width generational node keys. Exhausted generations retire a
 * slot instead of wrapping and making a stale key valid again.
 */
declare class NodeKeyAllocator {
  #private;
  constructor(firstSlot?: number);
  allocate(): NodeKey;
  release(key: NodeKey): boolean;
  isLive(key: NodeKey): boolean;
}
/**
 * Slot-indexed storage which always validates the complete generational key.
 * This keeps array lookup speed without allowing stale-key aliasing.
 */
declare class NodeKeyTable<T> {
  #private;
  set(key: NodeKey, value: T): this;
  get(key: NodeKey): T | undefined;
  has(key: NodeKey): boolean;
  delete(key: NodeKey): boolean;
  clear(): void;
}
//#endregion
//#region src/protocol/index.d.ts
declare const OP: {
  readonly CreateElement: 1;
  readonly CreateText: 2;
  readonly AppendChild: 4;
  readonly InsertBefore: 5;
  readonly RemoveChild: 6;
  readonly SetText: 8;
  readonly SetAttribute: 9;
  readonly RemoveAttribute: 10;
  readonly SetStyle: 11;
  readonly RemoveStyle: 12;
  readonly AddEventListener: 13;
  readonly RemoveEventListener: 14;
  readonly SetClassName: 15;
  readonly DropNode: 17;
  readonly SetTransform2D: 18;
  readonly FocusNode: 19;
  readonly ScrollTo: 20;
  readonly ScrollBy: 21;
  readonly SetStyleValue: 22;
  readonly SetShadows: 23;
  readonly SetOverlayPlane: 24;
  readonly SetScrollbarStyle: 25;
  readonly SetWidgetConfig: 26;
  readonly RemoveWidgetConfig: 27;
  readonly SetTextBehavior: 28;
  readonly SetInteractionPolicy: 29;
  readonly SetGraphicSource: 30;
  readonly ClearGraphicSource: 31;
};
type OpCode = (typeof OP)[keyof typeof OP];
declare const TEXT_BEHAVIOR: {
  readonly AggregateDirectText: 1;
  readonly SingleLine: 2;
};
declare const INTERACTION_POLICY: {
  readonly Focusable: 1;
  readonly BlockSubtree: 2;
  readonly ContainFocus: 4;
};
declare const GRAPHIC_SOURCE: {
  readonly Svg: 1;
  readonly NetworkRaster: 2;
};
declare const EVENT_CODE: {
  readonly click: 1;
  readonly input: 2;
  readonly submit: 3;
  readonly keydown: 4;
  readonly keyup: 5;
  readonly change: 6;
  readonly pointerdown: 7;
  readonly pointermove: 8;
  readonly pointerup: 9;
  readonly pointerenter: 10;
  readonly pointerleave: 11;
  readonly wheel: 12;
  readonly focus: 13;
  readonly blur: 14;
  readonly imecommit: 15;
  readonly pointercancel: 16;
  readonly pointerover: 17;
  readonly pointerout: 18;
  readonly contextmenu: 19;
  readonly dblclick: 20;
  readonly focusin: 21;
  readonly focusout: 22;
  readonly scroll: 23;
  readonly terminalexit: 24;
  readonly terminalprogress: 25;
  readonly terminalnotification: 26;
  readonly terminaltitlechange: 27;
  readonly terminalcwdchange: 28;
  readonly terminalselectionchange: 29;
  readonly textselectionchange: 30;
  readonly terminalbell: 31;
  readonly resourceready: 32;
  readonly resourceerror: 33;
};
type EventType = keyof typeof EVENT_CODE;
declare const EVENT_DATA_SLOT: {
  readonly clientX: 0;
  readonly clientY: 1;
  readonly offsetX: 2;
  readonly offsetY: 3;
  readonly button: 4;
  readonly buttons: 5;
  readonly mods: 6;
  readonly deltaX: 7;
  readonly deltaY: 8;
  readonly scrollX: 9;
  readonly scrollY: 10;
};
declare const EVENT_DATA_LEN: number;
type EventDataSlot = keyof typeof EVENT_DATA_SLOT;
/** Versioned Host → JS frame envelope. Keep in sync through `bun run gen`. */
declare const HOST_FRAME: {
  readonly Magic: 826689623;
  readonly Version: 2;
  readonly HeaderLen: 32;
};
declare const HOST_RECORD_KIND: {
  readonly NodeEvent: 1;
  readonly Resize: 2;
  readonly ApplicationMessage: 3;
  readonly Window: 4;
  readonly Widget: 5;
};
declare const HOST_NODE_PAYLOAD: {
  readonly None: 0;
  readonly Numeric: 1;
  readonly Json: 2;
};
/**
 * Per-tick binary frame writer. Emits ops into an internal buffer; `flush()`
 * returns the complete frame (header + ops) or null if nothing was emitted.
 * The caller owns how the bytes cross the host bridge.
 */
declare class Writer {
  private buf;
  private cursor;
  private count;
  private seq;
  private atoms;
  /** Strings already emitted in this frame (only values large enough to win). */
  private frameStrings;
  private internHost;
  constructor(internHost?: (value: string) => number);
  private ensure;
  private u8;
  private u16;
  private u32;
  private key;
  private f32;
  private str;
  private atom;
  private emit;
  createElement(id: NodeKey, tag: string): void;
  createText(id: NodeKey, text: string): void;
  appendChild(parent: NodeKey, child: NodeKey): void;
  insertBefore(parent: NodeKey, child: NodeKey, ref: NodeKey): void;
  removeChild(parent: NodeKey, child: NodeKey): void;
  setText(id: NodeKey, text: string): void;
  setAttribute(id: NodeKey, name: string, value: string): void;
  removeAttribute(id: NodeKey, name: string): void;
  setWidgetConfig(id: NodeKey, json: string): void;
  setTextBehavior(id: NodeKey, flags: number): void;
  setInteractionPolicy(id: NodeKey, flags: number, focusOrder: number): void;
  setGraphicSource(id: NodeKey, kind: number, source: string): void;
  clearGraphicSource(id: NodeKey, kind: number): void;
  removeWidgetConfig(id: NodeKey): void;
  setStyle(id: NodeKey, prop: string, value: string): void;
  setStyleValue(id: NodeKey, prop: string, kind: number, value: number): void;
  setShadows(id: NodeKey, shadows: readonly {
    offsetX: number;
    offsetY: number;
    spread: number;
    stdDev: number;
    color: number;
    radius?: number;
  }[]): void;
  setTransform2D(id: NodeKey, matrix: readonly [number, number, number, number, number, number]): void;
  setOverlayPlane(id: NodeKey, plane: number): void;
  setScrollbarStyle(id: NodeKey, style: {
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
  }): void;
  removeStyle(id: NodeKey, prop: string): void;
  addEventListener(id: NodeKey, eventCode: number): void;
  removeEventListener(id: NodeKey, eventCode: number): void;
  setClassName(id: NodeKey, value: string): void;
  dropNode(id: NodeKey): void;
  focusNode(id: NodeKey): void;
  scrollTo(id: NodeKey, x: number, y: number): void;
  scrollBy(id: NodeKey, x: number, y: number): void;
  /** Drain the buffer into a frame, or null if no ops were emitted this tick. */
  flush(): Uint8Array | null;
}
//#endregion
export { nodeKeyFromSlotMapFfi as C, nodeKeyEquals as S, NodeKeyTable as _, EventType as a, isNodeKey as b, HOST_NODE_PAYLOAD as c, OP as d, OpCode as f, NodeKeyAllocator as g, NodeKey as h, EventDataSlot as i, HOST_RECORD_KIND as l, Writer as m, EVENT_DATA_LEN as n, GRAPHIC_SOURCE as o, TEXT_BEHAVIOR as p, EVENT_DATA_SLOT as r, HOST_FRAME as s, EVENT_CODE as t, INTERACTION_POLICY as u, ROOT_NODE_KEY as v, nodeKey as x, formatNodeKey as y };
//# sourceMappingURL=protocol-DlNB_E1x.d.mts.map