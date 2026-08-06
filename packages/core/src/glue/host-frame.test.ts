import { expect, test } from "bun:test";
import { HOST_FRAME, HOST_RECORD_KIND } from "@wabou/protocol";
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
