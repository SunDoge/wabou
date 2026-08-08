import { expect, test } from "bun:test";

const calls: unknown[][] = [];
Object.assign(globalThis, {
  __wabou_open_url: (url: string) => {
    calls.push(["open", url]);
    return url.startsWith("https://");
  },
  __wabou_load_font: (path: string) => {
    calls.push(["font", path]);
    return path.endsWith(".ttf");
  },
  __wabou_frame_stats: () =>
    JSON.stringify({
      build_frame_ms: 1,
      js_tick_ms: 2,
      scene_ms: 3,
      present_ms: 4,
      node_count: 5,
      viewport_w: 640,
      viewport_h: 480,
    }),
  __wabou_layout_snapshot: (ids: Uint32Array) =>
    JSON.stringify({
      revision: 7,
      viewport: { x: 0, y: 0, width: 640, height: 480 },
      nodes: Array.from(ids, (id) => ({
        id,
        rect: { x: id, y: 2, width: 3, height: 4 },
        clip: { x: 0, y: 0, width: 640, height: 480 },
      })),
    }),
});

const { defaultHost } = await import("./host");

test("defaultHost adapts system, font, and diagnostics ABI", () => {
  expect(defaultHost.system.openUrl("https://wabou.dev")).toBe(true);
  expect(defaultHost.fonts.load("Inter.ttf")).toBe(true);
  expect(calls).toEqual([
    ["open", "https://wabou.dev"],
    ["font", "Inter.ttf"],
  ]);
  expect(defaultHost.diagnostics.frameStats()).toEqual({
    build_frame_ms: 1,
    js_tick_ms: 2,
    scene_ms: 3,
    present_ms: 4,
    node_count: 5,
    viewport_w: 640,
    viewport_h: 480,
  });
});

test("defaultHost adapts typed layout targets and convenience methods", () => {
  expect(
    defaultHost.layout.snapshot([3, { id: 9 }]).nodes.map(({ id }) => id),
  ).toEqual([3, 9]);
  expect(defaultHost.layout.measure({ id: 4 })).toEqual({
    x: 4,
    y: 2,
    width: 3,
    height: 4,
  });
  expect(defaultHost.layout.clippingRect(4)).toEqual({
    x: 0,
    y: 0,
    width: 640,
    height: 480,
  });
  expect(defaultHost.layout.viewport()).toEqual({
    x: 0,
    y: 0,
    width: 640,
    height: 480,
  });
});
