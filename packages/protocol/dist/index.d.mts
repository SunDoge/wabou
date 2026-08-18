//#region src/index.d.ts
declare const OP: {
  readonly CreateElement: 1;
  readonly CreateText: 2;
  readonly CreateComment: 3;
  readonly AppendChild: 4;
  readonly InsertBefore: 5;
  readonly RemoveChild: 6;
  readonly ReplaceNode: 7;
  readonly SetText: 8;
  readonly SetAttribute: 9;
  readonly RemoveAttribute: 10;
  readonly SetStyle: 11;
  readonly RemoveStyle: 12;
  readonly AddEventListener: 13;
  readonly RemoveEventListener: 14;
  readonly SetClassName: 15;
  readonly FrameEnd: 16;
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
  readonly Version: 1;
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
  private f32;
  private str;
  private atom;
  private emit;
  createElement(id: number, tag: string): void;
  createText(id: number, text: string): void;
  createComment(id: number, text: string): void;
  appendChild(parent: number, child: number): void;
  insertBefore(parent: number, child: number, ref: number): void;
  removeChild(parent: number, child: number): void;
  replaceNode(parent: number, oldId: number, newId: number): void;
  setText(id: number, text: string): void;
  setAttribute(id: number, name: string, value: string): void;
  removeAttribute(id: number, name: string): void;
  setWidgetConfig(id: number, json: string): void;
  setTextBehavior(id: number, flags: number): void;
  setInteractionPolicy(id: number, flags: number, focusOrder: number): void;
  setGraphicSource(id: number, kind: number, source: string): void;
  clearGraphicSource(id: number, kind: number): void;
  removeWidgetConfig(id: number): void;
  setStyle(id: number, prop: string, value: string): void;
  setStyleValue(id: number, prop: string, kind: number, value: number): void;
  setShadows(id: number, shadows: readonly {
    offsetX: number;
    offsetY: number;
    spread: number;
    stdDev: number;
    color: number;
    radius?: number;
  }[]): void;
  setTransform2D(id: number, matrix: readonly [number, number, number, number, number, number]): void;
  setOverlayPlane(id: number, plane: number): void;
  setScrollbarStyle(id: number, style: {
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
  removeStyle(id: number, prop: string): void;
  addEventListener(id: number, eventCode: number): void;
  removeEventListener(id: number, eventCode: number): void;
  setClassName(id: number, value: string): void;
  frameEnd(): void;
  dropNode(id: number): void;
  focusNode(id: number): void;
  scrollTo(id: number, x: number, y: number): void;
  scrollBy(id: number, x: number, y: number): void;
  /** Drain the buffer into a frame, or null if no ops were emitted this tick. */
  flush(): Uint8Array | null;
}
//#endregion
export { EVENT_CODE, EVENT_DATA_LEN, EVENT_DATA_SLOT, EventDataSlot, EventType, GRAPHIC_SOURCE, HOST_FRAME, HOST_NODE_PAYLOAD, HOST_RECORD_KIND, INTERACTION_POLICY, OP, OpCode, TEXT_BEHAVIOR, Writer };
//# sourceMappingURL=index.d.mts.map