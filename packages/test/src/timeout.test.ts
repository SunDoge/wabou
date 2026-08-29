import { describe, expect, test } from "bun:test";
import {
  DEFAULT_TEST_TIMEOUT,
  MAX_SUITE_TIMEOUT,
  MAX_TEST_TIMEOUT,
  replayTimeout,
  suiteTimeout,
  SUITE_TIMEOUT_OVERHEAD,
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

  test("derives the suite budget from independently bounded tests", () => {
    expect(suiteTimeout([])).toBe(SUITE_TIMEOUT_OVERHEAD);
    expect(suiteTimeout([5_000, 2_000, 7_500])).toBe(19_500);
    expect(suiteTimeout(Array.from({ length: 100 }, () => 5_000))).toBe(
      MAX_SUITE_TIMEOUT,
    );
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
