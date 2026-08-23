import { describe, expect, test } from "bun:test";
import { downloadProgressPercent } from "./workflow-state";

describe("model download progress", () => {
  test("clamps invalid and completed byte counts", () => {
    expect(
      downloadProgressPercent({
        state: "idle",
        downloadedBytes: 0,
        totalBytes: 0,
      }),
    ).toBe(0);
    expect(
      downloadProgressPercent({
        state: "downloading",
        downloadedBytes: 25,
        totalBytes: 100,
      }),
    ).toBe(25);
    expect(
      downloadProgressPercent({
        state: "complete",
        downloadedBytes: 120,
        totalBytes: 100,
      }),
    ).toBe(100);
  });
});
