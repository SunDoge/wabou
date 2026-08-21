import { describe, expect, test } from "bun:test";
import { isMessageScrollNearEnd, messageScrollRange } from "./message-scroller";

describe("MessageScroller", () => {
  test("derives the native scroll range from measured logical sizes", () => {
    expect(messageScrollRange(900, 320)).toBe(580);
    expect(messageScrollRange(240, 320)).toBe(0);
  });

  test("keeps end following inside an explicit logical threshold", () => {
    expect(isMessageScrollNearEnd(550, 900, 320, 24)).toBe(false);
    expect(isMessageScrollNearEnd(556, 900, 320, 24)).toBe(true);
    expect(isMessageScrollNearEnd(580, 900, 320, 24)).toBe(true);
  });
});
