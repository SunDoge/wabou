import type { FileEntry } from "./api";

export interface SnapshotBrowserCache {
  entries(snapshotId: string, path: string): readonly FileEntry[] | undefined;
  lastPath(snapshotId: string): string;
  remember(
    snapshotId: string,
    path: string,
    entries: readonly FileEntry[],
  ): void;
  clear(): void;
}

function cacheKey(snapshotId: string, path: string): string {
  return `${snapshotId}\u0000${path}`;
}

export function createSnapshotBrowserCache(): SnapshotBrowserCache {
  const entriesByPath = new Map<string, readonly FileEntry[]>();
  const lastPathBySnapshot = new Map<string, string>();

  return {
    entries(snapshotId, path) {
      return entriesByPath.get(cacheKey(snapshotId, path));
    },
    lastPath(snapshotId) {
      return lastPathBySnapshot.get(snapshotId) ?? "";
    },
    remember(snapshotId, path, entries) {
      entriesByPath.set(cacheKey(snapshotId, path), [...entries]);
      lastPathBySnapshot.set(snapshotId, path);
    },
    clear() {
      entriesByPath.clear();
      lastPathBySnapshot.clear();
    },
  };
}
