import { expect, test } from "bun:test";
import {
  createRenderEffect,
  createRoot,
  createSignal,
  flush,
  type JSX,
} from "solid-js";
import { GRAPHIC_DATA } from "../protocol";
import { px, shadow } from "../style";
import { PathBuilder } from "../vector-path";
import {
  createElement,
  dispatchEvent,
  EVENT_CODE,
  GRAPHIC_SOURCE,
  insertNode,
  isDirectEvent,
  mount,
  OP,
  observeGlobalPointerEvent,
  reconcileControlledInputValues,
  runSweep,
  setProp,
  spread,
  type WabouNodeEvent,
  writer,
} from "./index";

test("global pointer observers run before target bubbling and can unsubscribe", () => {
  const node = createElement("button");
  const order: string[] = [];
  const stop = observeGlobalPointerEvent("pointerdown", (target, event) => {
    expect(target).toBe(node);
    expect(event.clientX).toBe(12);
    order.push("capture");
  });
  setProp(node, "onPointerDown", () => order.push("target"), undefined);

  dispatchEvent(
    node.id,
    EVENT_CODE.pointerdown,
    JSON.stringify({ clientX: 12, clientY: 8 }),
  );
  expect(order).toEqual(["capture", "target"]);

  stop();
  dispatchEvent(node.id, EVENT_CODE.pointerdown, "");
  expect(order).toEqual(["capture", "target", "target"]);
});

test("every declared pointer event decodes its numeric payload", () => {
  const node = createElement("view");
  const observed: Array<[string, number, number]> = [];
  for (const [prop, code] of [
    ["onPointerOver", EVENT_CODE.pointerover],
    ["onPointerEnter", EVENT_CODE.pointerenter],
    ["onPointerOut", EVENT_CODE.pointerout],
    ["onPointerLeave", EVENT_CODE.pointerleave],
    ["onPointerCancel", EVENT_CODE.pointercancel],
    ["onDblClick", EVENT_CODE.dblclick],
  ] as const) {
    setProp(
      node,
      prop,
      (event: { type: string; clientX: number; offsetY: number }) =>
        observed.push([event.type, event.clientX, event.offsetY]),
      undefined,
    );
    dispatchEvent(node.id, code, "", [12, 13, 2, 3, 0, 0, 0]);
  }

  expect(observed).toEqual([
    ["pointerover", 12, 3],
    ["pointerenter", 12, 3],
    ["pointerout", 12, 3],
    ["pointerleave", 12, 3],
    ["pointercancel", 12, 3],
    ["dblclick", 12, 3],
  ]);
});

test("pointer events retain native identity and pen measurements", () => {
  const node = createElement("view");
  let received: Record<string, unknown> | undefined;
  setProp(
    node,
    "onPointerMove",
    (event: Record<string, unknown>) => {
      received = event;
    },
    undefined,
  );
  dispatchEvent(
    node.id,
    EVENT_CODE.pointermove,
    "",
    [12, 13, 2, 3, 0, 1, 0, 0, 0, 0, 0, 0, 7, 9, 2, 1, 0.75, -0.2, 15, -8, 120],
  );
  expect(received).toMatchObject({
    pointerId: { lo: 7, hi: 9 },
    pointerType: "pen",
    primary: true,
    pressure: 0.75,
    tangentialPressure: -0.2,
    tiltX: 15,
    tiltY: -8,
    twist: 120,
  });
});

test("wheel events retain the native gesture phase", () => {
  const node = createElement("view");
  let phase: string | undefined;
  setProp(
    node,
    "onWheel",
    (event: { phase: string }) => {
      phase = event.phase;
    },
    undefined,
  );
  dispatchEvent(
    node.id,
    EVENT_CODE.wheel,
    "",
    [12, 13, 2, 3, 0, 0, 0, 4, -8, 0, 0, 2],
  );
  expect(phase).toBe("ended");
});

test("graphic sources use the typed resource protocol", () => {
  const svg = createElement("svg");
  const image = createElement("img");
  const sources: Array<[number, string]> = [];
  const cleared: number[] = [];
  const setGraphicSource = writer.setGraphicSource.bind(writer);
  const clearGraphicSource = writer.clearGraphicSource.bind(writer);
  writer.setGraphicSource = (_id, kind, source) => sources.push([kind, source]);
  writer.clearGraphicSource = (_id, kind) => cleared.push(kind);
  try {
    setProp(svg, "source", "<svg/>", undefined);
    setProp(image, "resource", { lo: 7, hi: 3 }, undefined);
    setProp(svg, "source", undefined, "<svg/>");
    setProp(image, "resource", undefined, { lo: 7, hi: 3 });
  } finally {
    writer.setGraphicSource = setGraphicSource;
    writer.clearGraphicSource = clearGraphicSource;
  }

  expect(sources).toEqual([
    [GRAPHIC_SOURCE.Svg, "<svg/>"],
    [GRAPHIC_SOURCE.ResourceRaster, "7:3"],
  ]);
  expect(cleared).toEqual([GRAPHIC_SOURCE.Svg, GRAPHIC_SOURCE.ResourceRaster]);
});

test("non-drawing vector paths clear stale native geometry", () => {
  const path = createElement("vector-path");
  const written: number[] = [];
  const cleared: number[] = [];
  const setGraphicData = writer.setGraphicData.bind(writer);
  const clearGraphicData = writer.clearGraphicData.bind(writer);
  writer.setGraphicData = (_id, kind) => written.push(kind);
  writer.clearGraphicData = (_id, kind) => cleared.push(kind);
  try {
    setProp(
      path,
      "source",
      new PathBuilder().moveTo(0, 0).lineTo(1, 1).build(),
      undefined,
    );
    setProp(path, "source", new PathBuilder().moveTo(0, 0).build(), undefined);
  } finally {
    writer.setGraphicData = setGraphicData;
    writer.clearGraphicData = clearGraphicData;
  }

  expect(written).toEqual([GRAPHIC_DATA.VectorPath]);
  expect(cleared).toEqual([GRAPHIC_DATA.VectorPath]);
});

test("mount manages the host root lifecycle", () => {
  const dispose = mount(() => createElement("main") as unknown as JSX.Element);
  writer.flush();

  expect(dispose).toBeInstanceOf(Function);

  dispose();
  runSweep();
  writer.flush();
});

test("Solid 2 static createElement props are emitted immediately", () => {
  writer.flush();
  createElement("div", { class: "flex flex-row" });
  const frame = writer.flush();
  expect(frame).not.toBeNull();
  expect(Array.from(frame!)).toContain(OP.SetClassName);
});

test("ARIA booleans preserve explicit false instead of removing state", () => {
  const option = createElement("div");
  const attributes: Array<[string, string]> = [];
  const removed: string[] = [];
  const setAttribute = writer.setAttribute.bind(writer);
  const removeAttribute = writer.removeAttribute.bind(writer);
  writer.setAttribute = (_id, name, value) => attributes.push([name, value]);
  writer.removeAttribute = (_id, name) => removed.push(name);
  try {
    setProp(option, "aria-selected", false, undefined);
    setProp(option, "aria-expanded", true, undefined);
  } finally {
    writer.setAttribute = setAttribute;
    writer.removeAttribute = removeAttribute;
  }
  expect(attributes).toEqual([
    ["aria-selected", "false"],
    ["aria-expanded", "true"],
  ]);
  expect(removed).toEqual([]);
});

test("input listeners receive a DOM-like currentTarget", () => {
  const input = createElement("input");
  let currentTarget: { id: typeof input.id; value: string } | undefined;
  setProp(
    input,
    "onInput",
    (event: { currentTarget: { id: typeof input.id; value: string } }) => {
      currentTarget = event.currentTarget;
    },
    undefined,
  );

  dispatchEvent(
    input.id,
    EVENT_CODE.input,
    JSON.stringify({ value: "typed text" }),
  );

  expect(currentTarget).toEqual({ id: input.id, value: "typed text" });
});

test("host click is dispatched exactly once after pointerup", () => {
  const button = createElement("button");
  let clicks = 0;
  setProp(button, "onClick", () => clicks++, undefined);

  dispatchEvent(button.id, EVENT_CODE.pointerup, "");
  expect(clicks).toBe(0);

  dispatchEvent(button.id, EVENT_CODE.click, "");
  expect(clicks).toBe(1);
});

test("context-menu listeners receive compact pointer coordinates", () => {
  writer.flush();
  const target = createElement("view");
  let received:
    | { clientX: number; clientY: number; button: number }
    | undefined;
  setProp(
    target,
    "onContextMenu",
    (event: { clientX: number; clientY: number; button: number }) => {
      received = event;
    },
    undefined,
  );

  dispatchEvent(target.id, EVENT_CODE.contextmenu, "", [42, 24, 2, 3, 2, 0, 0]);

  expect(received).toMatchObject({ clientX: 42, clientY: 24, button: 2 });
  writer.flush();
});

test("event handler failures retain the event context and JavaScript stack", () => {
  const button = createElement("button");
  const messages: string[] = [];
  const original = globalThis.__wabou_log;
  globalThis.__wabou_log = (_level, message) => messages.push(message);
  try {
    setProp(
      button,
      "onClick",
      () => {
        throw new Error("navigation exploded");
      },
      undefined,
    );
    dispatchEvent(button.id, EVENT_CODE.click, "");
  } finally {
    globalThis.__wabou_log = original;
  }

  expect(messages).toHaveLength(1);
  expect(messages[0]).toContain("[wabou-event] click handler failed");
  expect(messages[0]).toContain(`target ${button.id.lo}v${button.id.hi}`);
  expect(messages[0]).toContain("Error: navigation exploded");
  expect(messages[0]).toContain("events.test.ts");
});

test("async event handler failures retain the event context", async () => {
  const button = createElement("button");
  const messages: string[] = [];
  const original = globalThis.__wabou_log;
  globalThis.__wabou_log = (_level, message) => messages.push(message);
  try {
    setProp(
      button,
      "onClick",
      async () => {
        await Promise.resolve();
        throw new Error("async navigation exploded");
      },
      undefined,
    );
    dispatchEvent(button.id, EVENT_CODE.click, "");
    await Promise.resolve();
    await Promise.resolve();
  } finally {
    globalThis.__wabou_log = original;
  }

  expect(messages).toHaveLength(1);
  expect(messages[0]).toContain("[wabou-event] click handler failed");
  expect(messages[0]).toContain(`target ${button.id.lo}v${button.id.hi}`);
  expect(messages[0]).toContain("Error: async navigation exploded");
});

test("dispatch reports preventDefault to the Host", () => {
  const target = createElement("view");
  expect(dispatchEvent(target.id, EVENT_CODE.click, "")).toBe(false);
  setProp(
    target,
    "onClick",
    (event: { preventDefault(): void }) => event.preventDefault(),
    undefined,
  );
  expect(dispatchEvent(target.id, EVENT_CODE.click, "")).toBe(true);
});

test("native events expose their actual target and payload contract", () => {
  const target = createElement("view");
  let observed:
    | {
        target: { id: typeof target.id };
        currentTarget: { id: typeof target.id };
        payload: { value: string };
        propagationStopped: boolean;
        stopImmediatePropagation(): void;
      }
    | undefined;
  setProp(
    target,
    "onInput",
    (event: NonNullable<typeof observed>) => {
      observed = event;
      event.stopImmediatePropagation();
    },
    undefined,
  );
  dispatchEvent(target.id, EVENT_CODE.input, JSON.stringify({ value: "next" }));
  expect(observed?.target.id).toEqual(target.id);
  expect(observed?.currentTarget.id).toEqual(target.id);
  expect(observed?.payload).toEqual({ value: "next" });
  expect(observed?.propagationStopped).toBe(true);
});

test("controlled input reconciliation survives a batched input and submit", () => {
  const input = createElement("input");
  const submit = createElement("button");
  const [draft, setDraft] = createSignal("");
  let previous = "";
  let dispose!: () => void;
  createRoot((rootDispose) => {
    dispose = rootDispose;
    createRenderEffect(draft, (next) => {
      setProp(input, "value", next, previous);
      previous = next;
    });
  });
  setProp(
    input,
    "onInput",
    (event: { currentTarget: { value: string } }) =>
      setDraft(event.currentTarget.value),
    undefined,
  );
  setProp(submit, "onClick", () => setDraft(""), undefined);
  flush();
  writer.flush();

  const original = writer.setAttribute.bind(writer);
  const values: string[] = [];
  writer.setAttribute = (id, name, value) => {
    if (id === input.id && name === "value") values.push(value);
    original(id, name, value);
  };
  try {
    flush(() => {
      dispatchEvent(
        input.id,
        EVENT_CODE.input,
        JSON.stringify({ value: "typed" }),
      );
      dispatchEvent(submit.id, EVENT_CODE.click, "");
    });
    reconcileControlledInputValues();
    expect(draft()).toBe("");
    expect(values).toEqual([""]);
  } finally {
    dispose();
    writer.setAttribute = original;
    writer.flush();
  }
});

test("direct-event checks distinguish a hit node from its bubbling parent", () => {
  const parent = createElement("view");
  const child = createElement("button");
  insertNode(parent, child, undefined);
  const observed: boolean[] = [];
  setProp(
    child,
    "onClick",
    (event: WabouNodeEvent) => {
      observed.push(isDirectEvent(event));
    },
    undefined,
  );
  setProp(
    parent,
    "onClick",
    (event: WabouNodeEvent) => {
      observed.push(isDirectEvent(event));
    },
    undefined,
  );
  dispatchEvent(child.id, EVENT_CODE.click, "");
  expect(observed).toEqual([true, false]);
});

test("native scroll observations expose authoritative offsets", () => {
  const view = createElement("div");
  let observed: { scrollX: number; scrollY: number } | undefined;
  setProp(
    view,
    "onScroll",
    (event: { scrollX: number; scrollY: number }) => {
      observed = { scrollX: event.scrollX, scrollY: event.scrollY };
    },
    undefined,
  );

  dispatchEvent(
    view.id,
    EVENT_CODE.scroll,
    "",
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 12.5, 320],
  );

  expect(observed).toEqual({ scrollX: 12.5, scrollY: 320 });
});

test("handles expose imperative native focus through the bridge", () => {
  writer.flush();
  const button = createElement("button");
  writer.flush();

  button.focus();
  const frame = writer.flush()!;
  const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

  expect(frame.byteLength).toBe(17);
  expect(frame[8]).toBe(0x13);
  expect(view.getUint32(9, true)).toBe(button.id.lo);
  expect(view.getUint32(13, true)).toBe(button.id.hi);
});

test("focus and subtree blocking compose as one typed policy", () => {
  writer.flush();
  const view = createElement("view");
  writer.flush();

  setProp(view, "focusOrder", 2, undefined);
  let frame = writer.flush()!;
  let bytes = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);
  expect(frame[8]).toBe(OP.SetInteractionPolicy);
  expect(frame[17]).toBe(0x01);
  expect(bytes.getInt32(18, true)).toBe(2);

  setProp(view, "interactionBlocked", true, undefined);
  frame = writer.flush()!;
  bytes = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);
  expect(frame[17]).toBe(0x03);
  expect(bytes.getInt32(18, true)).toBe(2);

  setProp(view, "focusContained", true, undefined);
  frame = writer.flush()!;
  bytes = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);
  expect(frame[17]).toBe(0x07);
  expect(bytes.getInt32(18, true)).toBe(2);

  setProp(view, "focusOrder", undefined, 2);
  frame = writer.flush()!;
  bytes = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);
  expect(frame[17]).toBe(0x06);
  expect(bytes.getInt32(18, true)).toBe(0);
});

test("native scrollbar customization uses one typed host operation", () => {
  writer.flush();
  const view = createElement("view");
  writer.flush();
  setProp(
    view,
    "scrollbar",
    {
      visibility: "always",
      hideDelay: 700,
      fadeDuration: 160,
      thickness: 14,
      margin: 3,
      minThumbLength: 40,
      radius: 5,
      trackColor: 0x11182788,
      thumbColor: 0x38bdf8ff,
    },
    undefined,
  );
  const frame = writer.flush()!;
  const bytes = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

  expect(frame[8]).toBe(OP.SetScrollbarStyle);
  expect(frame[17]).toBe(1);
  expect(bytes.getFloat32(18, true)).toBe(700);
  expect(bytes.getFloat32(22, true)).toBe(160);
  expect(bytes.getFloat32(26, true)).toBe(14);
  expect(bytes.getUint32(42, true)).toBe(0x11182788);
  expect(bytes.getUint32(46, true)).toBe(0x38bdf8ff);
});

test("typed inline style bypasses UTF-8 serialization", () => {
  writer.flush();
  const viewNode = createElement("view");
  writer.flush();

  setProp(viewNode, "style", { width: px(24.5) }, undefined);
  const frame = writer.flush()!;
  const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

  expect(frame[8]).toBe(0x16);
  expect(view.getUint32(9, true)).toBe(viewNode.id.lo);
  expect(view.getUint32(13, true)).toBe(viewNode.id.hi);
  expect(frame[21]).toBe(1);
  expect(view.getFloat32(22, true)).toBe(24.5);
});

test("Vello shadows bypass string style serialization", () => {
  writer.flush();
  const viewNode = createElement("view");
  writer.flush();

  setProp(
    viewNode,
    "shadows",
    [shadow({ offsetY: 4, stdDev: 3, spread: -1, color: 0x00000066 })],
    undefined,
  );
  const frame = writer.flush()!;
  const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

  expect(frame[8]).toBe(0x17);
  expect(view.getUint32(9, true)).toBe(viewNode.id.lo);
  expect(view.getUint32(13, true)).toBe(viewNode.id.hi);
  expect(view.getUint16(17, true)).toBe(1);
  expect(view.getFloat32(23, true)).toBe(4);
  expect(view.getFloat32(27, true)).toBe(-1);
  expect(view.getFloat32(31, true)).toBe(3);
  expect(view.getUint32(35, true)).toBe(0x00000066);
});

test("classList explicitly merges interaction classes with static class", () => {
  writer.flush();
  const viewNode = createElement("view");
  writer.flush();

  setProp(viewNode, "class", "flex bg-slate-900", undefined);
  setProp(
    viewNode,
    "classList",
    { "bg-slate-700": true, "opacity-50": false },
    undefined,
  );
  const first = writer.flush();
  expect(first).not.toBeNull();

  setProp(
    viewNode,
    "classList",
    { "bg-slate-700": false, "opacity-50": true },
    { "bg-slate-700": true, "opacity-50": false },
  );
  const second = writer.flush();
  expect(second).not.toBeNull();
});

test("spread tracks reactive class getters across a host flush", () => {
  createRoot((dispose) => {
    writer.flush();
    const node = createElement("view");
    const [compact, setCompact] = createSignal(false, { ownedWrite: true });
    spread(
      node,
      {
        get class() {
          return compact() ? "grid grid-cols-2" : "flex h-40";
        },
      },
      false,
    );
    flush();
    writer.flush();

    flush(() => setCompact(true));
    const frame = writer.flush();
    expect(frame).not.toBeNull();
    if (!frame) throw new Error("reactive class update emitted no frame");
    expect(Array.from(frame)).toContain(OP.SetClassName);
    dispose();
  });
});

test("spread tracks reactive semantic boolean getters across a host flush", () => {
  createRoot((dispose) => {
    writer.flush();
    const node = createElement("button");
    const [selected, setSelected] = createSignal(false, { ownedWrite: true });
    const attributes: Array<[string, string]> = [];
    const setAttribute = writer.setAttribute.bind(writer);
    writer.setAttribute = (_id, name, value) => attributes.push([name, value]);
    try {
      spread(
        node,
        {
          get "aria-selected"() {
            return selected();
          },
        },
        false,
      );
      flush();
      expect(attributes.at(-1)).toEqual(["aria-selected", "false"]);

      flush(() => setSelected(true));
      expect(attributes.at(-1)).toEqual(["aria-selected", "true"]);
    } finally {
      writer.setAttribute = setAttribute;
      dispose();
    }
  });
});
