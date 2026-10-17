import { expect, test } from "bun:test";
import { subscribeGesture } from "./gesture";
import { dispatchHostMessage } from "./host-messages";

test("decodes native gestures without DOM compatibility semantics", () => {
  const events: unknown[] = [];
  const unsubscribe = subscribeGesture((event) => events.push(event));
  dispatchHostMessage(
    "wabou:gesture",
    JSON.stringify({ type: "pan", deltaX: 12, deltaY: -4, phase: "changed" }),
  );
  dispatchHostMessage("wabou:gesture", JSON.stringify({ type: "double-tap" }));
  unsubscribe();
  expect(events).toEqual([
    { type: "pan", deltaX: 12, deltaY: -4, phase: "changed" },
    { type: "double-tap" },
  ]);
});

test("rejects malformed native gesture payloads", () => {
  const events: unknown[] = [];
  const unsubscribe = subscribeGesture((event) => events.push(event));
  dispatchHostMessage(
    "wabou:gesture",
    JSON.stringify({ type: "pinch", delta: 1, phase: "moving" }),
  );
  expect(events).toEqual([]);
  unsubscribe();
});
