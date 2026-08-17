export interface PollOptions {
  timeout?: number;
  interval?: number;
}

export interface PollResult<T> {
  matched: boolean;
  value: T;
}

export interface ResolvedPollOptions {
  timeout: number;
  interval: number;
}

function duration(value: number | undefined, fallback: number, name: string) {
  const resolved = value ?? fallback;
  if (!Number.isFinite(resolved) || resolved < 0) {
    throw new RangeError(`${name} must be a finite non-negative number`);
  }
  return resolved;
}

/** Resolve defaults before an operation is recorded so replay is deterministic. */
export function resolvePollOptions(
  options: PollOptions = {},
): ResolvedPollOptions {
  return {
    timeout: duration(options.timeout, 1_000, "timeout"),
    interval: duration(options.interval, 16, "interval"),
  };
}

/** Poll an observable value with one explicit clock and retry policy. */
export async function pollUntil<T>(
  read: () => T | Promise<T>,
  matches: (value: T) => boolean,
  options: PollOptions = {},
  beforeRead?: () => void | Promise<void>,
): Promise<PollResult<T>> {
  const { timeout, interval } = resolvePollOptions(options);
  const deadline = performance.now() + timeout;
  let value: T;

  for (;;) {
    await beforeRead?.();
    value = await read();
    if (matches(value)) return { matched: true, value };
    const remaining = deadline - performance.now();
    if (remaining <= 0) return { matched: false, value };
    await new Promise<void>((resolve) =>
      setTimeout(resolve, Math.min(interval, remaining)),
    );
  }
}
