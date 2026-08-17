import { describe, expect, test } from "bun:test";
import {
  DEFAULT_TEST_TIMEOUT,
  MAX_TEST_TIMEOUT,
  replayTimeout,
  SUITE_TIMEOUT,
  SuiteTimeoutError,
  TestTimeoutError,
  testTimeout,
  withSuiteTimeout,
  withTestTimeout,
} from "./timeout";

describe("test timeout", () => {
  test("uses a bounded explicit default", () => {
    expect(testTimeout(undefined)).toBe(DEFAULT_TEST_TIMEOUT);
    expect(testTimeout(25)).toBe(25);
    expect(() => testTimeout(0)).toThrow();
    expect(() => testTimeout(MAX_TEST_TIMEOUT + 1)).toThrow();
    expect(() => testTimeout(Number.NaN)).toThrow();
  });

  test("gives combined replay actions their cumulative bounded wait budget", () => {
    expect(replayTimeout([])).toBe(DEFAULT_TEST_TIMEOUT);
    expect(
      replayTimeout([
        { wait: { timeout: 2_000 } },
        { wait: { timeout: 1_500 } },
        {},
      ]),
    ).toBe(8_600);
    expect(
      replayTimeout(
        Array.from({ length: 100 }, () => ({
          wait: { timeout: 1_000 },
        })),
      ),
    ).toBe(MAX_TEST_TIMEOUT);
  });

  test("returns completed work and reports the stalled test name", async () => {
    expect(await withTestTimeout("quick", 100, async () => 42)).toBe(42);
    await expect(
      withTestTimeout("stalled", 1, () => new Promise(() => {})),
    ).rejects.toEqual(
      new TestTimeoutError('test "stalled" timed out after 1ms'),
    );
  });

  test("reports the active test when the complete suite budget expires", async () => {
    expect(SUITE_TIMEOUT).toBe(MAX_TEST_TIMEOUT);
    await expect(
      withSuiteTimeout(
        1,
        () => new Promise(() => {}),
        () => "slow test",
      ),
    ).rejects.toEqual(
      new SuiteTimeoutError(
        'test suite timed out after 1ms while running "slow test"',
      ),
    );
  });
});
