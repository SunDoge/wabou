// DevTools UI model tests.
import { describe, expect, test } from "bun:test";
import {
  decode,
  EMPTY_OVERLAY_LAYERS,
  overlayEvidenceLabel,
  overlayStyle,
  screenshotPoint,
  toggleOverlayLayer,
} from "./model";

describe("DevTools view model", () => {
  test("decodes successful capability responses", () => {
    expect(decode<{ pid: number }>('{"ok":true,"value":{"pid":42}}')).toEqual({
      pid: 42,
    });
  });

  test("surfaces capability errors", () => {
    expect(() => decode('{"ok":false,"error":"target disconnected"}')).toThrow(
      "target disconnected",
    );
  });

  test("maps target layout coordinates onto the screenshot", () => {
    expect(
      overlayStyle({ x: 100, y: 50, width: 200, height: 100 }, 1000, 500),
    ).toEqual({ left: "10%", top: "10%", width: "20%", height: "20%" });
    expect(
      overlayStyle({ x: 0, y: 0, width: 1, height: 1 }, 0, 500),
    ).toBeUndefined();
  });

  test("maps stretched screenshot clicks back to inspected logical pixels", () => {
    expect(
      screenshotPoint(
        { x: 300, y: 150 },
        { width: 600, height: 300 },
        { width: 1_200, height: 900 },
      ),
    ).toEqual({ x: 600, y: 450 });
    expect(
      screenshotPoint(
        { x: 900, y: -20 },
        { width: 600, height: 300 },
        { width: 1_200, height: 900 },
      ),
    ).toEqual({ x: 1_200, y: 0 });
    expect(
      screenshotPoint(
        { x: 1, y: 1 },
        { width: 0, height: 300 },
        { width: 1_200, height: 900 },
      ),
    ).toBeUndefined();
  });

  test("toggles diagnostic layers independently", () => {
    const layout = toggleOverlayLayer(EMPTY_OVERLAY_LAYERS, "layout");
    const clips = toggleOverlayLayer(layout, "clips");

    expect(layout).toEqual({ layout: true, clips: false, hitTarget: false });
    expect(clips).toEqual({ layout: true, clips: true, hitTarget: false });
    expect(EMPTY_OVERLAY_LAYERS).toEqual({
      layout: false,
      clips: false,
      hitTarget: false,
    });
  });

  test("distinguishes requested overlays from completed native paint", () => {
    expect(overlayEvidenceLabel(undefined, undefined)).toBe(
      "overlay evidence unavailable",
    );
    expect(
      overlayEvidenceLabel(
        { layout: true, clips: false, hitTarget: false },
        {
          sequence: 4,
          enabled: false,
          layout_bounds: 0,
          clip_bounds: 0,
          highlights: 0,
        },
      ),
    ).toBe("overlay requested · awaiting native paint");
    expect(
      overlayEvidenceLabel(
        { layout: true, clips: true, hitTarget: false },
        {
          sequence: 5,
          enabled: true,
          layout_bounds: 42,
          clip_bounds: 3,
          highlights: 1,
        },
      ),
    ).toBe("overlay pass 5 · 42 bounds · 3 clips · 1 highlights");
  });
});
