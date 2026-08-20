export interface PollOptions {
  timeout?: number;
  interval?: number;
  /** The predicate must remain true continuously for this many milliseconds. */
  stableFor?: number;
}

export interface PollResult<T> {
  matched: boolean;
  value: T;
}

export interface ResolvedPollOptions {
  timeout: number;
  interval: number;
  stableFor: number;
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
  const resolved = {
    timeout: duration(options.timeout, 1_000, "timeout"),
    interval: duration(options.interval, 16, "interval"),
    stableFor: duration(options.stableFor, 0, "stableFor"),
  };
  if (resolved.stableFor > resolved.timeout) {
    throw new RangeError("stableFor cannot exceed timeout");
  }
  return resolved;
}

/** Poll an observable value with one explicit clock and retry policy. */
export async function pollUntil<T>(
  read: () => T | Promise<T>,
  matches: (value: T) => boolean,
  options: PollOptions = {},
  beforeRead?: () => void | Promise<void>,
): Promise<PollResult<T>> {
  const { timeout, interval, stableFor } = resolvePollOptions(options);
  const deadline = performance.now() + timeout;
  let matchedSince: number | undefined;
  let value: T;

  for (;;) {
    await beforeRead?.();
    value = await read();
    const now = performance.now();
    if (matches(value)) {
      matchedSince ??= now;
      if (now - matchedSince >= stableFor) return { matched: true, value };
    } else {
      matchedSince = undefined;
    }
    const remaining = deadline - now;
    if (remaining <= 0) return { matched: false, value };
    const stabilityRemaining =
      matchedSince === undefined
        ? remaining
        : Math.max(0, stableFor - (now - matchedSince));
    await new Promise<void>((resolve) =>
      setTimeout(resolve, Math.min(interval, remaining, stabilityRemaining)),
    );
  }
}
