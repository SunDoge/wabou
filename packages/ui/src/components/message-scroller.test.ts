import { describe, expect, test } from "bun:test";
import {
  activeMessageAnchor,
  isMessageScrollNearEnd,
  messageScrollRange,
  messageScrollRevealDelta,
} from "./message-scroller";

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

  test("tracks the last semantic anchor crossing the reading line", () => {
    const viewport = { x: 0, y: 100, width: 400, height: 300 };
    expect(
      activeMessageAnchor(viewport, [
        { id: "first", rect: { x: 20, y: 40, width: 360, height: 80 } },
        { id: "second", rect: { x: 20, y: 150, width: 360, height: 80 } },
        { id: "third", rect: { x: 20, y: 280, width: 360, height: 80 } },
      ]),
    ).toBe("second");
  });

  test("keeps the first anchor active before it reaches the reading line", () => {
    expect(
      activeMessageAnchor({ x: 0, y: 0, width: 400, height: 300 }, [
        {
          id: "first",
          rect: { x: 20, y: 180, width: 360, height: 80 },
        },
        {
          id: "second",
          rect: { x: 20, y: 300, width: 360, height: 80 },
        },
      ]),
    ).toBe("first");
  });
});

describe("messageScrollRevealDelta", () => {
  const viewport = { x: 20, y: 100, width: 300, height: 200 };

  test("does not move an already visible item", () => {
    expect(
      messageScrollRevealDelta(
        viewport,
        { x: 20, y: 140, width: 280, height: 40 },
        12,
      ),
    ).toBe(0);
  });

  test("reveals the nearest edge while preserving a margin", () => {
    expect(
      messageScrollRevealDelta(
        viewport,
        { x: 20, y: 70, width: 280, height: 30 },
        12,
      ),
    ).toBe(-42);
    expect(
      messageScrollRevealDelta(
        viewport,
        { x: 20, y: 290, width: 280, height: 40 },
        12,
      ),
    ).toBe(42);
  });
});
