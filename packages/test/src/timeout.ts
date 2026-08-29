export const MAX_TEST_TIMEOUT = 60_000;
export const DEFAULT_TEST_TIMEOUT = 5_000;
export const MAX_SUITE_TIMEOUT = 300_000;
export const SUITE_TIMEOUT_OVERHEAD = 5_000;

/**
 * Bound a complete scenario by the tests it actually registered.
 *
 * A fixed suite timeout makes a healthy, larger scenario fail merely because
 * it contains more independently bounded tests. The aggregate remains capped
 * so a broken runner cannot keep the native host alive indefinitely.
 */
export function suiteTimeout(testTimeouts: readonly number[]): number {
  let timeout = SUITE_TIMEOUT_OVERHEAD;
  for (const test of testTimeouts) {
    timeout += test;
    if (timeout >= MAX_SUITE_TIMEOUT) return MAX_SUITE_TIMEOUT;
  }
  return timeout;
}

export function testTimeout(value: number | undefined): number {
  const timeout = value ?? DEFAULT_TEST_TIMEOUT;
  if (!Number.isFinite(timeout) || timeout <= 0 || timeout > MAX_TEST_TIMEOUT) {
    throw new RangeError(
      `test timeout must be a finite number between 1 and ${MAX_TEST_TIMEOUT}ms`,
    );
  }
  return timeout;
}

/** Bound a combined replay by its authored waits without removing the hard cap. */
export function replayTimeout(actions: readonly object[]): number {
  let timeout = DEFAULT_TEST_TIMEOUT;
  for (const action of actions) {
    const wait = "wait" in action ? action.wait : undefined;
    timeout +=
      typeof wait === "object" &&
      wait !== null &&
      "timeout" in wait &&
      typeof wait.timeout === "number"
        ? wait.timeout
        : 100;
    if (timeout >= MAX_TEST_TIMEOUT) return MAX_TEST_TIMEOUT;
  }
  return timeout;
}

export class TestTimeoutError extends Error {}
export class SuiteTimeoutError extends Error {}

/** Bound one test body without allowing a timeout to continue the suite. */
export async function withTestTimeout<T>(
  name: string,
  timeout: number,
  operation: () => T | Promise<T>,
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const expired = new Promise<never>((_, reject) => {
    timer = setTimeout(
      () =>
        reject(
          new TestTimeoutError(
            `test ${JSON.stringify(name)} timed out after ${timeout}ms`,
          ),
        ),
      timeout,
    );
  });
  try {
    return await Promise.race([Promise.resolve().then(operation), expired]);
  } finally {
    if (timer !== undefined) clearTimeout(timer);
  }
}

/** Bound the complete scenario so native watchdogs remain a final fallback. */
export async function withSuiteTimeout<T>(
  timeout: number,
  operation: () => T | Promise<T>,
  activeTest: () => string | undefined = () => undefined,
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const expired = new Promise<never>((_, reject) => {
    timer = setTimeout(() => {
      const active = activeTest();
      reject(
        new SuiteTimeoutError(
          `test suite timed out after ${timeout}ms${active === undefined ? "" : ` while running ${JSON.stringify(active)}`}`,
        ),
      );
    }, timeout);
  });
  try {
    return await Promise.race([Promise.resolve().then(operation), expired]);
  } finally {
    if (timer !== undefined) clearTimeout(timer);
  }
}
