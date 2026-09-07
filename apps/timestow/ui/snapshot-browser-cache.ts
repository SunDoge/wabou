import type { FileListing } from "./api";

export type CachedDirectoryListing = Pick<FileListing, "entries" | "total">;

export interface SnapshotBrowserCache {
  listing(snapshotId: string, path: string): CachedDirectoryListing | undefined;
  lastPath(snapshotId: string): string;
  remember(
    snapshotId: string,
    path: string,
    listing: CachedDirectoryListing,
  ): void;
  clear(): void;
}

function cacheKey(snapshotId: string, path: string): string {
  return `${snapshotId}\u0000${path}`;
}

export function createSnapshotBrowserCache(): SnapshotBrowserCache {
  const listingsByPath = new Map<string, CachedDirectoryListing>();
  const lastPathBySnapshot = new Map<string, string>();

  return {
    listing(snapshotId, path) {
      return listingsByPath.get(cacheKey(snapshotId, path));
    },
    lastPath(snapshotId) {
      return lastPathBySnapshot.get(snapshotId) ?? "";
    },
    remember(snapshotId, path, listing) {
      listingsByPath.set(cacheKey(snapshotId, path), {
        entries: [...listing.entries],
        total: listing.total,
      });
      lastPathBySnapshot.set(snapshotId, path);
    },
    clear() {
      listingsByPath.clear();
      lastPathBySnapshot.clear();
    },
  };
}
