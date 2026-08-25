import { describe, expect, test } from "vitest";
import {
  appendFrameSample,
  frameDuration,
  frameStages,
} from "./overview-metrics";

const stats = {
  js_tick_ms: 2,
  build_frame_ms: 4,
  scene_ms: 1,
  present_ms: 3,
  node_count: 420,
  viewport_w: 1280,
  viewport_h: 800,
};

describe("overview native frame metrics", () => {
  test("derives totals and stages from the host sample", () => {
    expect(frameDuration(stats)).toBe(8);
    expect(frameStages(stats)).toEqual([
      { label: "JS", value: 2, width: expect.closeTo(11.9976, 3) },
      { label: "Host", value: 2, width: expect.closeTo(11.9976, 3) },
      { label: "Scene", value: 1, width: expect.closeTo(5.9988, 3) },
      { label: "Present", value: 3, width: expect.closeTo(17.9964, 3) },
    ]);
  });

  test("keeps bounded history and rejects invalid samples", () => {
    expect(appendFrameSample([1, 2, 3], 4, 3)).toEqual([2, 3, 4]);
    expect(appendFrameSample([1, 2], Number.NaN)).toEqual([1, 2]);
    expect(appendFrameSample([1, 2], -1)).toEqual([1, 2]);
  });
});
