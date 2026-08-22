//#region src/protocol/resource-key.d.ts
declare const resourceKeyBrand: unique symbol;
/** Two-u32 representation shared by SlotMap-backed resource handles. */
interface ResourceKeyParts {
  readonly lo: number;
  readonly hi: number;
}
/**
 * Opaque generational resource identity. `Family` prevents image, font,
 * subscription, and other independently owned resources from being mixed.
 */
interface ResourceKey<Family extends string> extends ResourceKeyParts {
  readonly [resourceKeyBrand]: Family;
}
/** Validate the common two-u32 SlotMap wire representation. */
declare function validateResourceKeyParts(value: ResourceKeyParts, label?: string): ResourceKeyParts;
/** Structural check for a key arriving through JSON or another untyped edge. */
declare function isResourceKeyParts(value: unknown): value is ResourceKeyParts;
/** Stable diagnostic form; binary paths continue to write two u32 fields. */
declare function formatResourceKeyParts(value: ResourceKeyParts): string;
/** Slot-indexed storage that validates both the family and generation. */
declare class ResourceKeyTable<Family extends string, Value> {
  #private;
  constructor(family: ResourceKeyFamily<Family>);
  set(key: ResourceKey<Family>, value: Value): this;
  get(key: ResourceKey<Family>): Value | undefined;
  has(key: ResourceKey<Family>): boolean;
  delete(key: ResourceKey<Family>): boolean;
  clear(): void;
}
/** Operations bound to one resource family and its private runtime token. */
interface ResourceKeyFamily<Family extends string> {
  readonly name: Family;
  fromParts(lo: number, hi: number): ResourceKey<Family>;
  fromJSON(value: unknown): ResourceKey<Family>;
  is(value: unknown): value is ResourceKey<Family>;
  assert(value: unknown): asserts value is ResourceKey<Family>;
  equals(left: ResourceKey<Family> | null | undefined, right: ResourceKey<Family> | null | undefined): boolean;
  format(value: ResourceKeyParts): string;
  table<Value>(): ResourceKeyTable<Family, Value>;
}
/**
 * Define one opaque handle family. The private symbol token also catches
 * accidental cross-family casts at runtime; it is not serialized on the wire.
 */
declare function createResourceKeyFamily<const Family extends string>(name: Family, options?: {
  readonly runtimeBrand?: boolean;
}): ResourceKeyFamily<Family>;
//#endregion
//#region src/protocol/node-key.d.ts
/**
 * Full-width retained-node identity used on both sides of the native bridge.
 * `lo` selects a slot and `hi` identifies that slot's generation.
 */
type NodeKey = ResourceKey<"node">;
/** Structural form accepted when a key was deserialized from JSON. */
type NodeKeyParts = ResourceKeyParts;
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
declare class NodeKeyTable<T> extends ResourceKeyTable<"node", T> {
  constructor();
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
  readonly SetGraphicData: 32;
  readonly ClearGraphicData: 33;
  readonly SetTextMaxLines: 34;
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
  readonly FileRaster: 3;
};
declare const GRAPHIC_DATA: {
  readonly VectorPath: 1;
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
  setTextMaxLines(id: NodeKey, maxLines: number): void;
  setInteractionPolicy(id: NodeKey, flags: number, focusOrder: number): void;
  setGraphicSource(id: NodeKey, kind: number, source: string): void;
  clearGraphicSource(id: NodeKey, kind: number): void;
  setGraphicData(id: NodeKey, kind: number, data: Uint8Array): void;
  clearGraphicData(id: NodeKey, kind: number): void;
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
export { formatResourceKeyParts as A, nodeKeyEquals as C, ResourceKeyParts as D, ResourceKeyFamily as E, validateResourceKeyParts as M, ResourceKeyTable as O, nodeKey as S, ResourceKey as T, NodeKeyAllocator as _, EventType as a, formatNodeKey as b, HOST_FRAME as c, INTERACTION_POLICY as d, OP as f, NodeKey as g, Writer as h, EventDataSlot as i, isResourceKeyParts as j, createResourceKeyFamily as k, HOST_NODE_PAYLOAD as l, TEXT_BEHAVIOR as m, EVENT_DATA_LEN as n, GRAPHIC_DATA as o, OpCode as p, EVENT_DATA_SLOT as r, GRAPHIC_SOURCE as s, EVENT_CODE as t, HOST_RECORD_KIND as u, NodeKeyTable as v, nodeKeyFromSlotMapFfi as w, isNodeKey as x, ROOT_NODE_KEY as y };
//# sourceMappingURL=protocol-0b7PSaRx.d.mts.map