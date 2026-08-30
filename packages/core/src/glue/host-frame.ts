import { flush } from "solid-js";
import {
  EVENT_DATA_LEN,
  HOST_FRAME,
  HOST_NODE_PAYLOAD,
  HOST_RECORD_KIND,
  type NodeKey,
  nodeKey,
} from "../protocol";
import {
  dispatchEvent,
  reconcileControlledInputValues,
  writer,
} from "../renderer";
import { dispatchHostMessage } from "./host-messages";
import { dispatchResizeObservation } from "./resize-observer";

const RECORD_HEADER_LEN = 8;
const FLAG_CANCELLABLE = 1;
const textDecoder = new TextDecoder();

export interface HostFrameDisposition {
  preventedEventIds?: Uint32Array;
  needsTick: boolean;
  protocolFrame?: Uint8Array;
}

type DecodedRecord =
  | {
      kind: "node";
      flags: number;
      target: NodeKey;
      eventCode: number;
      eventId: number;
      json: string;
      numeric?: Float64Array;
    }
  | { kind: "resize"; target: NodeKey; width: number; height: number }
  | { kind: "message"; topic: string; payload: unknown }
  | { kind: "unknown" };

function viewOf(input: ArrayBufferView | ArrayBuffer): {
  bytes: Uint8Array;
  view: DataView;
} {
  const bytes =
    input instanceof ArrayBuffer
      ? new Uint8Array(input)
      : new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  return {
    bytes,
    view: new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength),
  };
}

/**
 * Decode the complete frame before dispatching any record. Malformed frames
 * are atomic: no listener, observer or application subscriber is called.
 */
export function decodeAndDispatchHostFrame(
  input: ArrayBufferView | ArrayBuffer,
): HostFrameDisposition {
  const { bytes, view } = viewOf(input);
  if (view.byteLength < HOST_FRAME.HeaderLen) {
    throw new TypeError("short HostEventFrame header");
  }
  if (view.getUint32(0, true) !== HOST_FRAME.Magic) {
    throw new TypeError("invalid HostEventFrame magic");
  }
  if (view.getUint16(4, true) !== HOST_FRAME.Version) {
    throw new TypeError("unsupported HostEventFrame version");
  }
  const count = view.getUint32(24, true);
  const byteLen = view.getUint32(28, true);
  if (byteLen !== view.byteLength) {
    throw new TypeError("HostEventFrame byte length mismatch");
  }

  let offset: number = HOST_FRAME.HeaderLen;
  const records: DecodedRecord[] = [];
  const requireBytes = (length: number, end: number): void => {
    if (length < 0 || offset + length > end) {
      throw new TypeError("truncated HostEventFrame record");
    }
  };

  for (let index = 0; index < count; index++) {
    if (offset + RECORD_HEADER_LEN > byteLen) {
      throw new TypeError("truncated HostEventFrame record header");
    }
    const kind = view.getUint8(offset);
    const flags = view.getUint8(offset + 1);
    const recordLen = view.getUint32(offset + 4, true);
    if (recordLen < RECORD_HEADER_LEN || offset + recordLen > byteLen) {
      throw new TypeError("invalid HostEventFrame record length");
    }
    const end = offset + recordLen;
    offset += RECORD_HEADER_LEN;

    if (kind === HOST_RECORD_KIND.NodeEvent) {
      requireBytes(16, end);
      const target = nodeKey(
        view.getUint32(offset, true),
        view.getUint32(offset + 4, true),
      );
      const eventCode = view.getUint8(offset + 8);
      const payloadKind = view.getUint8(offset + 9);
      const numericLen = view.getUint16(offset + 10, true);
      const eventId = view.getUint32(offset + 12, true);
      offset += 16;
      if (payloadKind === HOST_NODE_PAYLOAD.None) {
        records.push({
          kind: "node",
          flags,
          target,
          eventCode,
          eventId,
          json: "",
        });
      } else if (payloadKind === HOST_NODE_PAYLOAD.Numeric) {
        if (numericLen > EVENT_DATA_LEN) {
          throw new TypeError(
            "HostEventFrame numeric payload exceeds ABI slots",
          );
        }
        requireBytes(8 * numericLen, end);
        const absoluteOffset = bytes.byteOffset + offset;
        let numeric: Float64Array;
        if (absoluteOffset % Float64Array.BYTES_PER_ELEMENT === 0) {
          numeric = new Float64Array(bytes.buffer, absoluteOffset, numericLen);
        } else {
          // ArrayBufferView callers may provide an arbitrary byteOffset. Rust
          // host frames are aligned, but retain a safe fallback for tooling.
          numeric = new Float64Array(numericLen);
          for (let slot = 0; slot < numeric.length; slot++) {
            numeric[slot] = view.getFloat64(offset + slot * 8, true);
          }
        }
        offset += 8 * numericLen;
        records.push({
          kind: "node",
          flags,
          target,
          eventCode,
          eventId,
          json: "",
          numeric,
        });
      } else if (payloadKind === HOST_NODE_PAYLOAD.Json) {
        requireBytes(4, end);
        const len = view.getUint32(offset, true);
        offset += 4;
        requireBytes(len, end);
        const json = textDecoder.decode(bytes.subarray(offset, offset + len));
        offset += len;
        records.push({
          kind: "node",
          flags,
          target,
          eventCode,
          eventId,
          json,
        });
      } else {
        throw new TypeError(`unknown node payload kind ${payloadKind}`);
      }
    } else if (kind === HOST_RECORD_KIND.Resize) {
      requireBytes(16, end);
      records.push({
        kind: "resize",
        target: nodeKey(
          view.getUint32(offset, true),
          view.getUint32(offset + 4, true),
        ),
        width: view.getFloat32(offset + 8, true),
        height: view.getFloat32(offset + 12, true),
      });
      offset += 16;
    } else if (kind === HOST_RECORD_KIND.ApplicationMessage) {
      requireBytes(2, end);
      const topicLen = view.getUint16(offset, true);
      offset += 2;
      requireBytes(topicLen + 1, end);
      const topic = textDecoder.decode(
        bytes.subarray(offset, offset + topicLen),
      );
      offset += topicLen;
      const payloadKind = view.getUint8(offset++);
      let payload: unknown;
      if (payloadKind === 0) payload = null;
      else if (payloadKind === 1) {
        requireBytes(1, end);
        payload = view.getUint8(offset++) !== 0;
      } else if (payloadKind === 2) {
        requireBytes(4, end);
        payload = view.getInt32(offset, true);
        offset += 4;
      } else if (payloadKind === 3) {
        requireBytes(8, end);
        payload = view.getFloat64(offset, true);
        offset += 8;
      } else if (payloadKind === 4) {
        requireBytes(2, end);
        const len = view.getUint16(offset, true);
        offset += 2;
        requireBytes(len, end);
        payload = textDecoder.decode(bytes.subarray(offset, offset + len));
        offset += len;
      } else if (payloadKind === 5) {
        requireBytes(4, end);
        const len = view.getUint32(offset, true);
        offset += 4;
        requireBytes(len, end);
        payload = bytes.subarray(offset, offset + len).slice();
        offset += len;
      } else {
        throw new TypeError(`unknown application payload kind ${payloadKind}`);
      }
      records.push({ kind: "message", topic, payload });
    } else {
      records.push({ kind: "unknown" });
    }

    // Known records must consume their payload exactly. Unknown records are
    // skipped by record_len for forward-compatible additions.
    if (offset > end) throw new TypeError("HostEventFrame record overflow");
    offset = end;
  }
  if (offset !== byteLen) throw new TypeError("trailing HostEventFrame bytes");

  const prevented: number[] = [];
  let needsTick = false;
  // Solid 2's synchronous boundary must enclose the writes, not merely drain
  // them afterwards. This also guarantees that one host frame cannot expose
  // partially applied node/message/resize state to an effect.
  flush(() => {
    for (const record of records) {
      if (record.kind === "node") {
        const defaultPrevented = dispatchEvent(
          record.target,
          record.eventCode,
          record.json,
          record.numeric,
        );
        if (
          defaultPrevented &&
          (record.flags & FLAG_CANCELLABLE) !== 0 &&
          record.eventId !== 0
        ) {
          prevented.push(record.eventId);
        }
        needsTick = true;
      } else if (record.kind === "resize") {
        dispatchResizeObservation(record.target, record.width, record.height);
        needsTick = true;
      } else if (record.kind === "message") {
        dispatchHostMessage(record.topic, record.payload);
        needsTick = true;
      }
    }
  });
  reconcileControlledInputValues();
  // Host events and the renderer writes they cause form one transaction. In
  // particular, a controlled native input must receive its reconciled value
  // before another native event can observe the widget. Waiting for the next
  // rAF leaves the JS and native values split during rapid input/click bursts.
  const protocolFrame = writer.flush() ?? undefined;

  return {
    preventedEventIds:
      prevented.length > 0 ? Uint32Array.from(prevented) : undefined,
    needsTick,
    protocolFrame,
  };
}

function __wabou_dispatch_host_frame(frame: Uint8Array): HostFrameDisposition {
  return decodeAndDispatchHostFrame(frame);
}

(globalThis as unknown as Record<string, unknown>).__wabou_dispatch_host_frame =
  __wabou_dispatch_host_frame;
