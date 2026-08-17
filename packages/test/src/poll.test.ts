import { describe, expect, test } from "bun:test";
import { pollUntil, resolvePollOptions } from "./poll";

describe("pollUntil", () => {
  test("supports async reads and runs the frame barrier for every attempt", async () => {
    let reads = 0;
    let barriers = 0;
    const result = await pollUntil(
      async () => ++reads,
      (value) => value === 3,
      { timeout: 100, interval: 0 },
      async () => {
        barriers += 1;
      },
    );

    expect(result).toEqual({ matched: true, value: 3 });
    expect(barriers).toBe(3);
  });

  test("reads once for a zero timeout and returns the last observation", async () => {
    let reads = 0;
    const result = await pollUntil(
      () => ++reads,
      () => false,
      { timeout: 0 },
    );

    expect(result).toEqual({ matched: false, value: 1 });
  });

  test("rejects ambiguous timing values", async () => {
    await expect(
      pollUntil(() => true, Boolean, { timeout: Number.NaN }),
    ).rejects.toThrow("timeout must be a finite non-negative number");
    await expect(
      pollUntil(() => true, Boolean, { interval: -1 }),
    ).rejects.toThrow("interval must be a finite non-negative number");
  });

  test("resolves stable defaults for recorded and replayed actions", () => {
    expect(resolvePollOptions()).toEqual({ timeout: 1_000, interval: 16 });
    expect(resolvePollOptions({ timeout: 20, interval: 2 })).toEqual({
      timeout: 20,
      interval: 2,
    });
    expect(() => resolvePollOptions({ timeout: Number.NaN })).toThrow();
  });
});
