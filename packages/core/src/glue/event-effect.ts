import { type Accessor, createEffect, untrack } from "solid-js";

export interface EventEffectOptions<T> {
  /** A retained event feed. Items may be newest-first or oldest-first. */
  source: Accessor<readonly T[]>;
  /** A strictly increasing sequence assigned when the event is produced. */
  sequence: (event: T) => number;
  onEvent: (event: T) => unknown;
  /** Receives synchronous throws and asynchronous rejections from `onEvent`. */
  onError?: (error: unknown, event: T) => void;
  /** Consume retained history on mount. Defaults to false. */
  consumeInitial?: boolean;
}

/**
 * Consume every new event from a retained feed exactly once and in sequence
 * order. This avoids losing events when several feed updates are batched into
 * one reactive notification.
 */
export function createEventEffect<T>(options: EventEffectOptions<T>): void {
  const initial = untrack(options.source);
  let cursor = options.consumeInitial
    ? Number.NEGATIVE_INFINITY
    : latestSequence(initial, options.sequence);

  createEffect(options.source, (events) => {
    const pending = events
      .map((event) => ({ event, sequence: options.sequence(event) }))
      .filter((candidate) => candidate.sequence > cursor)
      .sort((left, right) => left.sequence - right.sequence);
    for (const candidate of pending) {
      // The sequence is the event identity. A retained feed containing the
      // same entry twice must not deliver it twice.
      if (candidate.sequence <= cursor) continue;
      // Advance before invoking application code so a synchronous feed update
      // cannot deliver the same event recursively.
      cursor = candidate.sequence;
      try {
        const result = options.onEvent(candidate.event);
        if (isPromiseLike(result))
          void Promise.resolve(result).catch((error) =>
            reportError(options, error, candidate.event, candidate.sequence),
          );
      } catch (error) {
        reportError(options, error, candidate.event, candidate.sequence);
      }
    }
  });
}

function isPromiseLike(value: unknown): value is PromiseLike<unknown> {
  return (
    (typeof value === "object" || typeof value === "function") &&
    value !== null &&
    typeof (value as { then?: unknown }).then === "function"
  );
}

function reportError<T>(
  options: EventEffectOptions<T>,
  error: unknown,
  event: T,
  sequence: number,
): void {
  if (options.onError) {
    try {
      options.onError(error, event);
      return;
    } catch (reportingError) {
      console.error(
        `[wabou-event-effect] onError failed for sequence ${sequence}`,
        reportingError,
      );
    }
  }
  console.error(
    `[wabou-event-effect] handler failed for sequence ${sequence}`,
    error,
  );
}

function latestSequence<T>(
  events: readonly T[],
  sequence: (event: T) => number,
): number {
  let latest = Number.NEGATIVE_INFINITY;
  for (const event of events) latest = Math.max(latest, sequence(event));
  return latest;
}
