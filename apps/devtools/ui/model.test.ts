// DevTools UI model tests.
import { describe, expect, test } from "bun:test";
import {
  decode,
  EMPTY_OVERLAY_LAYERS,
  overlayStyle,
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
});
