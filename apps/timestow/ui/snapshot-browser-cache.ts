import type { FileListing } from "./api";

export type CachedDirectoryListing = Pick<FileListing, "entries" | "total">;

export interface SnapshotBrowserCache {
  listing(snapshotId: string, path: string): CachedDirectoryListing | undefined;
  selectedSnapshot(profileId: string): string | undefined;
  lastPath(snapshotId: string): string;
  rememberSelection(profileId: string, snapshotId: string): void;
  remember(
    snapshotId: string,
    path: string,
    listing: CachedDirectoryListing,
  ): void;
  replaceSnapshot(
    profileId: string,
    previousSnapshotId: string,
    nextSnapshotId: string,
  ): void;
  clearListings(): void;
  clear(): void;
}

function cacheKey(snapshotId: string, path: string): string {
  return `${snapshotId}\u0000${path}`;
}

export function createSnapshotBrowserCache(): SnapshotBrowserCache {
  const listingsByPath = new Map<string, CachedDirectoryListing>();
  const selectedSnapshotByProfile = new Map<string, string>();
  const lastPathBySnapshot = new Map<string, string>();

  return {
    listing(snapshotId, path) {
      return listingsByPath.get(cacheKey(snapshotId, path));
    },
    selectedSnapshot(profileId) {
      return selectedSnapshotByProfile.get(profileId);
    },
    lastPath(snapshotId) {
      return lastPathBySnapshot.get(snapshotId) ?? "";
    },
    rememberSelection(profileId, snapshotId) {
      selectedSnapshotByProfile.set(profileId, snapshotId);
    },
    remember(snapshotId, path, listing) {
      listingsByPath.set(cacheKey(snapshotId, path), {
        entries: [...listing.entries],
        total: listing.total,
      });
      lastPathBySnapshot.set(snapshotId, path);
    },
    replaceSnapshot(profileId, previousSnapshotId, nextSnapshotId) {
      if (previousSnapshotId === nextSnapshotId) return;
      if (selectedSnapshotByProfile.get(profileId) === previousSnapshotId) {
        selectedSnapshotByProfile.set(profileId, nextSnapshotId);
      }
      const lastPath = lastPathBySnapshot.get(previousSnapshotId);
      if (lastPath !== undefined) {
        lastPathBySnapshot.set(nextSnapshotId, lastPath);
        lastPathBySnapshot.delete(previousSnapshotId);
      }
      const prefix = `${previousSnapshotId}\u0000`;
      for (const [key, listing] of listingsByPath) {
        if (!key.startsWith(prefix)) continue;
        listingsByPath.set(
          cacheKey(nextSnapshotId, key.slice(prefix.length)),
          listing,
        );
        listingsByPath.delete(key);
      }
    },
    clearListings() {
      listingsByPath.clear();
    },
    clear() {
      listingsByPath.clear();
      selectedSnapshotByProfile.clear();
      lastPathBySnapshot.clear();
    },
  };
}
