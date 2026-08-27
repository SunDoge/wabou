import { expect, test } from "bun:test";
import {
  EVENT_CODE,
  EVENT_DATA_LEN,
  EVENT_DATA_SLOT,
  HOST_FRAME,
  HOST_NODE_PAYLOAD,
  HOST_RECORD_KIND,
} from "../protocol";
import { createElement, setProp } from "../renderer";
import { createRenderEffect, createRoot, createSignal, flush } from "solid-js";
import { decodeAndDispatchHostFrame } from "./host-frame";
import { subscribeAll } from "./host-messages";

function applicationFrame(topic: string, value: string): Uint8Array {
  const encoder = new TextEncoder();
  const topicBytes = encoder.encode(topic);
  const valueBytes = encoder.encode(value);
  const recordLen = 8 + 2 + topicBytes.length + 1 + 2 + valueBytes.length;
  const bytes = new Uint8Array(HOST_FRAME.HeaderLen + recordLen);
  const view = new DataView(bytes.buffer);
  view.setUint32(0, HOST_FRAME.Magic, true);
  view.setUint16(4, HOST_FRAME.Version, true);
  view.setUint32(24, 1, true);
  view.setUint32(28, bytes.length, true);
  let offset = HOST_FRAME.HeaderLen;
  view.setUint8(offset, HOST_RECORD_KIND.ApplicationMessage);
  view.setUint32(offset + 4, recordLen, true);
  offset += 8;
  view.setUint16(offset, topicBytes.length, true);
  offset += 2;
  bytes.set(topicBytes, offset);
  offset += topicBytes.length;
  view.setUint8(offset++, 4);
  view.setUint16(offset, valueBytes.length, true);
  offset += 2;
  bytes.set(valueBytes, offset);
  return bytes;
}

function numericScrollFrame(
  target: { lo: number; hi: number },
  scrollY: number,
): Uint8Array {
  const numericLen = EVENT_DATA_SLOT.scrollY + 1;
  const recordLen = 8 + 16 + numericLen * 8;
  const bytes = new Uint8Array(HOST_FRAME.HeaderLen + recordLen);
  const view = new DataView(bytes.buffer);
  view.setUint32(0, HOST_FRAME.Magic, true);
  view.setUint16(4, HOST_FRAME.Version, true);
  view.setUint32(24, 1, true);
  view.setUint32(28, bytes.length, true);
  let offset = HOST_FRAME.HeaderLen;
  view.setUint8(offset, HOST_RECORD_KIND.NodeEvent);
  view.setUint32(offset + 4, recordLen, true);
  offset += 8;
  view.setUint32(offset, target.lo, true);
  view.setUint32(offset + 4, target.hi, true);
  view.setUint8(offset + 8, EVENT_CODE.scroll);
  view.setUint8(offset + 9, HOST_NODE_PAYLOAD.Numeric);
  view.setUint16(offset + 10, numericLen, true);
  offset += 16;
  view.setFloat64(offset + EVENT_DATA_SLOT.scrollY * 8, scrollY, true);
  return bytes;
}

test("unified HostEventFrame dispatches application records", () => {
  const received: Array<[string, unknown]> = [];
  const unsubscribe = subscribeAll((topic, payload) =>
    received.push([topic, payload]),
  );
  try {
    expect(
      decodeAndDispatchHostFrame(applicationFrame("status", "ready")),
    ).toEqual({ preventedEventIds: undefined, needsTick: true });
    expect(received).toEqual([["status", "ready"]]);
  } finally {
    unsubscribe();
  }
});

test("a complete host frame is one Solid reactive flush boundary", () => {
  let setValue!: (value: string) => string;
  let dispose!: () => void;
  let applied = "";
  createRoot((rootDispose) => {
    dispose = rootDispose;
    const [value, write] = createSignal("idle");
    setValue = write;
    createRenderEffect(value, (next) => {
      applied = next;
    });
  });
  flush();
  expect(applied).toBe("idle");

  const unsubscribe = subscribeAll((_topic, payload) => {
    setValue(String(payload));
    // Solid 2 keeps the previous applied state until the frame-level flush.
    expect(applied).toBe("idle");
  });
  try {
    decodeAndDispatchHostFrame(applicationFrame("status", "ready"));
    expect(applied).toBe("ready");
  } finally {
    unsubscribe();
    dispose();
  }
});

test("numeric host frames preserve extended scroll slots", () => {
  const node = createElement("div");
  let observed = 0;
  setProp(node, "onScroll", (event: { scrollY: number }) => {
    observed = event.scrollY;
  });

  decodeAndDispatchHostFrame(numericScrollFrame(node.id, 3_200));

  expect(observed).toBe(3_200);
});

test("numeric host frames carry only the event-specific slot prefix", () => {
  const frame = numericScrollFrame({ lo: 1, hi: 1 }, 42);
  const expectedRecordLen = 8 + 16 + (EVENT_DATA_SLOT.scrollY + 1) * 8;
  expect(frame.byteLength).toBe(HOST_FRAME.HeaderLen + expectedRecordLen);
  expect(new DataView(frame.buffer).getUint32(HOST_FRAME.HeaderLen + 4, true)).toBe(
    expectedRecordLen,
  );
});

test("malformed frames are rejected atomically", () => {
  const received: unknown[] = [];
  const unsubscribe = subscribeAll((_topic, payload) => received.push(payload));
  try {
    const frame = applicationFrame("status", "ready");
    const malformed = frame.slice(0, frame.length - 1);
    expect(() => decodeAndDispatchHostFrame(malformed)).toThrow();
    expect(received).toEqual([]);
  } finally {
    unsubscribe();
  }
});
