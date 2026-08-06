import { expect, test } from "bun:test";
import { px } from "@wabou/style";
import type { JSX } from "solid-js";
import {
  createElement,
  dispatchEvent,
  EVENT_CODE,
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
