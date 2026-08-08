export interface HistoryEntry {
  value: string;
  state?: unknown;
}

export interface HistoryUpdate extends HistoryEntry {
  replace?: boolean;
}

export type HistoryListener = (entry: HistoryEntry) => void;

export interface MemoryHistory {
  readonly back: () => void;
  readonly forward: () => void;
  readonly go: (delta: number) => void;
  readonly get: () => HistoryEntry;
  readonly listen: (listener: HistoryListener) => () => void;
  readonly set: (update: HistoryUpdate) => void;
  readonly canGoBack: () => boolean;
  readonly canGoForward: () => boolean;
}

export interface MemoryHistoryOptions {
  initialEntries?: readonly (string | HistoryEntry)[];
  initialIndex?: number;
}

function asEntry(entry: string | HistoryEntry): HistoryEntry {
  return typeof entry === "string"
    ? { value: entry }
    : { value: entry.value, state: entry.state };
}

/** A deterministic, window-independent navigation stack. */
export function createMemoryHistory(
  options: MemoryHistoryOptions = {},
): MemoryHistory {
  const initial = options.initialEntries?.length
    ? options.initialEntries.map(asEntry)
    : [{ value: "/" }];
  const entries = initial;
  let index = Math.max(
    0,
    Math.min(options.initialIndex ?? entries.length - 1, entries.length - 1),
  );
  const listeners = new Set<HistoryListener>();

  const current = () => entries[index];
  const publish = () => {
    const entry = current();
    for (const listener of listeners) listener(entry);
  };
  const go = (delta: number) => {
    if (!Number.isFinite(delta)) return;
    const next = Math.max(
      0,
      Math.min(index + Math.trunc(delta), entries.length - 1),
    );
    if (next === index) return;
    index = next;
    publish();
  };

  return {
    get: () => ({ ...current() }),
    set(update) {
      const next = asEntry(update);
      if (update.replace) {
        entries[index] = next;
      } else {
        entries.splice(index + 1, entries.length - index - 1, next);
        index += 1;
      }
      publish();
    },
    back: () => go(-1),
    forward: () => go(1),
    go,
    canGoBack: () => index > 0,
    canGoForward: () => index < entries.length - 1,
    listen(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
  };
}
