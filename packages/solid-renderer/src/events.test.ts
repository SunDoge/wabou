import { expect, test } from "bun:test";
import { px, shadow } from "@wabou/style";
import type { JSX } from "solid-js";
import {
  createElement,
  dispatchEvent,
  EVENT_CODE,
  OP,
  mount,
  runSweep,
  setProp,
  writer,
} from "./index";

test("mount manages the host root lifecycle", () => {
  const dispose = mount(() => createElement("main") as unknown as JSX.Element);
  writer.flush();

  expect(dispose).toBeInstanceOf(Function);

  dispose();
  runSweep();
  writer.flush();
});

test("input listeners receive a DOM-like currentTarget", () => {
  const input = createElement("input");
  let currentTarget: { id: number; value: string } | undefined;
  setProp(
    input,
    "onInput",
    (event: { currentTarget: { id: number; value: string } }) => {
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

test("dispatch reports preventDefault to the Host", () => {
  const anchor = createElement("a");
  setProp(anchor, "href", "https://example.com/story", undefined);
  expect(dispatchEvent(anchor.id, EVENT_CODE.click, "")).toBe(false);
  setProp(
    anchor,
    "onClick",
    (event: { preventDefault(): void }) => event.preventDefault(),
    undefined,
  );
  expect(dispatchEvent(anchor.id, EVENT_CODE.click, "")).toBe(true);
});

test("handles expose imperative native focus through the bridge", () => {
  writer.flush();
  const button = createElement("button");
  writer.flush();

  button.focus();
  const frame = writer.flush()!;
  const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

  expect(frame.byteLength).toBe(13);
  expect(frame[8]).toBe(0x13);
  expect(view.getUint32(9, true)).toBe(button.id);
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
  expect(frame[13]).toBe(1);
  expect(bytes.getFloat32(14, true)).toBe(14);
  expect(bytes.getUint32(30, true)).toBe(0x11182788);
  expect(bytes.getUint32(34, true)).toBe(0x38bdf8ff);
});

test("typed inline style bypasses UTF-8 serialization", () => {
  writer.flush();
  const viewNode = createElement("view");
  writer.flush();

  setProp(viewNode, "style", { width: px(24.5) }, undefined);
  const frame = writer.flush()!;
  const view = new DataView(frame.buffer, frame.byteOffset, frame.byteLength);

  expect(frame[8]).toBe(0x16);
  expect(view.getUint32(9, true)).toBe(viewNode.id);
  expect(frame[17]).toBe(1);
  expect(view.getFloat32(18, true)).toBe(24.5);
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
  expect(view.getUint32(9, true)).toBe(viewNode.id);
  expect(view.getUint16(13, true)).toBe(1);
  expect(view.getFloat32(19, true)).toBe(4);
  expect(view.getFloat32(23, true)).toBe(-1);
  expect(view.getFloat32(27, true)).toBe(3);
  expect(view.getUint32(31, true)).toBe(0x00000066);
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
