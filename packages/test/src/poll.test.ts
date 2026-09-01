import { describe, expect, test } from "bun:test";
import { pollUntil, resolvePollOptions, retryUntilHandled } from "./poll";

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

  test("requires a continuous stable interval and resets it after a mismatch", async () => {
    let reads = 0;
    const result = await pollUntil(
      () => ++reads,
      (value) => value !== 3,
      { timeout: 100, interval: 1, stableFor: 5 },
    );

    expect(result.matched).toBe(true);
    expect(reads).toBeGreaterThan(3);
  });

  test("rejects ambiguous timing values", async () => {
    await expect(
      pollUntil(() => true, Boolean, { timeout: Number.NaN }),
    ).rejects.toThrow("timeout must be a finite non-negative number");
    await expect(
      pollUntil(() => true, Boolean, { interval: -1 }),
    ).rejects.toThrow("interval must be a finite non-negative number");
    await expect(
      pollUntil(() => true, Boolean, { timeout: 10, stableFor: 11 }),
    ).rejects.toThrow("stableFor cannot exceed timeout");
  });

  test("resolves stable defaults for recorded and replayed actions", () => {
    expect(resolvePollOptions()).toEqual({
      timeout: 1_000,
      interval: 16,
      stableFor: 0,
    });
    expect(
      resolvePollOptions({ timeout: 20, interval: 2, stableFor: 5 }),
    ).toEqual({
      timeout: 20,
      interval: 2,
      stableFor: 5,
    });
    expect(() => resolvePollOptions({ timeout: Number.NaN })).toThrow();
  });

  test("retries rejected host actions without repeating an accepted action", async () => {
    let attempts = 0;
    const handled = await retryUntilHandled(
      () => {
        attempts += 1;
        return attempts === 3;
      },
      { timeout: 100, interval: 0, stableFor: 50 },
    );

    expect(handled).toBe(true);
    expect(attempts).toBe(3);
  });
});
