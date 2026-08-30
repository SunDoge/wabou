type AcceptCallback = (module: unknown) => void;
type DisposeCallback = (data: Record<string, unknown>) => void;

declare function __wabou_vite_update_style(id: string, css: string): void;
declare function __wabou_vite_remove_style(id: string): void;

interface HotContext {
  data: Record<string, unknown>;
  accepted: AcceptCallback[];
  disposed: DisposeCallback[];
  invalidated: boolean;
  accept(callback?: AcceptCallback): void;
  dispose(callback: DisposeCallback): void;
  decline(): void;
  invalidate(): void;
  on(): void;
  send(): void;
  prune(): void;
}

interface HotRecord {
  data: Record<string, unknown>;
  current: HotContext | null;
  next: HotContext | null;
  loading: boolean;
}

type WabouGlobal = typeof globalThis & {
  __wabou_hmr_records?: Map<string, HotRecord>;
  __wabou_apply_hmr?: (
    path: string,
    acceptedPath: string,
    timestamp: number,
  ) => Promise<boolean>;
  /** Start a reversible hot-record replacement for an in-process full reload. */
  __wabou_hmr_begin_full_reload?: () => void;
  /** Keep the records registered by the successfully imported graph. */
  __wabou_hmr_commit_full_reload?: () => void;
  /** Restore the last-good records after a failed entry import. */
  __wabou_hmr_rollback_full_reload?: () => void;
};

const wabouGlobal = globalThis as WabouGlobal;
const existingRecords = wabouGlobal.__wabou_hmr_records;
const records = existingRecords ?? new Map<string, HotRecord>();
let fullReloadSnapshot: Map<string, HotRecord> | null = null;
if (!existingRecords) {
  wabouGlobal.__wabou_hmr_records = records;
}

function nextContext(record: HotRecord): HotContext | null {
  return record.next;
}

export function createHotContext(ownerPath: string): HotContext {
  let record = records.get(ownerPath);
  if (!record) {
    record = { data: {}, current: null, next: null, loading: false };
    records.set(ownerPath, record);
  }

  const context: HotContext = {
    data: record.data,
    accepted: [],
    disposed: [],
    invalidated: false,
    accept(callback) {
      if (typeof callback === "function") this.accepted.push(callback);
    },
    dispose(callback) {
      this.disposed.push(callback);
    },
    decline() {
      this.invalidated = true;
    },
    invalidate() {
      this.invalidated = true;
    },
    on() {},
    send() {},
    prune() {},
  };

  if (record.loading) record.next = context;
  else record.current = context;
  return context;
}

// Vite-generated CSS modules call these hooks during initial evaluation.
// Keep the native document stylesheet in Rust because QuickJS has no browser
// style element for Vite to mutate.
export function updateStyle(id: string, css: string): void {
  __wabou_vite_update_style(id, css);
}

export function removeStyle(id: string): void {
  __wabou_vite_remove_style(id);
}

/**
 * Apply one Vite JS HMR update. Returns:
 * - `true` — Solid refresh (or another accept handler) took the update
 * - `false` — no hot context / declined / invalidate / import error → host
 *   should full-reload the entry rather than leave a half-applied tree
 */
wabouGlobal.__wabou_apply_hmr = async (path, acceptedPath, timestamp) => {
  const record = records.get(path);
  if (!record?.current) {
    console.warn(
      `[wabou-hmr] no hot context for ${path}; host will full-reload`,
    );
    return false;
  }

  const previous = record.current;
  for (const dispose of previous.disposed) {
    try {
      dispose(record.data);
    } catch (error) {
      console.error(`[wabou-hmr] dispose failed for ${path}`, error);
    }
  }
  record.loading = true;
  record.next = null;

  try {
    const separator = acceptedPath.includes("?") ? "&" : "?";
    const module = await import(`${acceptedPath}${separator}t=${timestamp}`);
    const next = nextContext(record);
    if (!next || next.invalidated) {
      console.warn(
        `[wabou-hmr] update for ${path} was invalidated/declined; host will full-reload`,
      );
      return false;
    }
    record.current = next;
    for (const accept of previous.accepted) {
      try {
        accept(module);
      } catch (error) {
        console.error(`[wabou-hmr] accept handler failed for ${path}`, error);
        return false;
      }
    }
    if (previous.invalidated) {
      console.warn(
        `[wabou-hmr] ${path} invalidated during accept; host will full-reload`,
      );
      return false;
    }
    return true;
  } catch (error) {
    console.error(`[wabou-hmr] failed to import ${acceptedPath}`, error);
    return false;
  } finally {
    record.loading = false;
    record.next = null;
  }
};

/**
 * Replace hot records transactionally during an entry reload. Vite can report
 * a reload while a saved file still fails to transform; retaining the previous
 * records lets the next valid save use HMR instead of leaving the runtime in a
 * permanently degraded state.
 */
export function beginFullReload(): void {
  if (fullReloadSnapshot) return;
  fullReloadSnapshot = new Map(records);
  records.clear();
}

export function commitFullReload(): void {
  fullReloadSnapshot = null;
}

export function rollbackFullReload(): void {
  if (!fullReloadSnapshot) return;
  records.clear();
  for (const [path, record] of fullReloadSnapshot) records.set(path, record);
  fullReloadSnapshot = null;
}

wabouGlobal.__wabou_hmr_begin_full_reload = beginFullReload;
wabouGlobal.__wabou_hmr_commit_full_reload = commitFullReload;
wabouGlobal.__wabou_hmr_rollback_full_reload = rollbackFullReload;
