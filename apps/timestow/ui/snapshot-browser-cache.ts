import type { FileListing } from "./api";

export type CachedDirectoryListing = Pick<FileListing, "entries" | "total">;

export interface SnapshotBrowserCacheOptions {
  /** Maximum number of directory listings retained across all profiles. */
  maxListings?: number;
}

export interface SnapshotBrowserCache {
  listing(
    profileId: string,
    snapshotId: string,
    path: string,
  ): CachedDirectoryListing | undefined;
  selectedSnapshot(profileId: string): string | undefined;
  lastPath(profileId: string, snapshotId: string): string;
  rememberSelection(profileId: string, snapshotId: string): void;
  remember(
    profileId: string,
    snapshotId: string,
    path: string,
    listing: CachedDirectoryListing,
  ): void;
  replaceSnapshot(
    profileId: string,
    previousSnapshotId: string,
    nextSnapshotId: string,
  ): void;
  removeSnapshot(profileId: string, snapshotId: string): void;
  clearListings(profileId: string): void;
  clear(): void;
}

function snapshotKey(profileId: string, snapshotId: string): string {
  return `${profileId}\u0000${snapshotId}`;
}

function cacheKey(profileId: string, snapshotId: string, path: string): string {
  return `${snapshotKey(profileId, snapshotId)}\u0000${path}`;
}

export function createSnapshotBrowserCache(
  options: SnapshotBrowserCacheOptions = {},
): SnapshotBrowserCache {
  const maxListings = options.maxListings ?? 64;
  if (!Number.isInteger(maxListings) || maxListings < 1) {
    throw new Error(
      "snapshot browser cache maxListings must be a positive integer",
    );
  }
  const listingsByPath = new Map<string, CachedDirectoryListing>();
  const selectedSnapshotByProfile = new Map<string, string>();
  const lastPathBySnapshot = new Map<string, string>();

  function retainListing(key: string, listing: CachedDirectoryListing): void {
    listingsByPath.delete(key);
    listingsByPath.set(key, listing);
    while (listingsByPath.size > maxListings) {
      const oldest = listingsByPath.keys().next().value;
      if (oldest === undefined) break;
      listingsByPath.delete(oldest);
    }
  }

  return {
    listing(profileId, snapshotId, path) {
      const key = cacheKey(profileId, snapshotId, path);
      const listing = listingsByPath.get(key);
      if (listing) retainListing(key, listing);
      return listing;
    },
    selectedSnapshot(profileId) {
      return selectedSnapshotByProfile.get(profileId);
    },
    lastPath(profileId, snapshotId) {
      return lastPathBySnapshot.get(snapshotKey(profileId, snapshotId)) ?? "";
    },
    rememberSelection(profileId, snapshotId) {
      selectedSnapshotByProfile.set(profileId, snapshotId);
    },
    remember(profileId, snapshotId, path, listing) {
      retainListing(cacheKey(profileId, snapshotId, path), {
        entries: [...listing.entries],
        total: listing.total,
      });
      lastPathBySnapshot.set(snapshotKey(profileId, snapshotId), path);
    },
    replaceSnapshot(profileId, previousSnapshotId, nextSnapshotId) {
      if (previousSnapshotId === nextSnapshotId) return;
      if (selectedSnapshotByProfile.get(profileId) === previousSnapshotId) {
        selectedSnapshotByProfile.set(profileId, nextSnapshotId);
      }
      const previousKey = snapshotKey(profileId, previousSnapshotId);
      const nextKey = snapshotKey(profileId, nextSnapshotId);
      const lastPath = lastPathBySnapshot.get(previousKey);
      if (lastPath !== undefined) {
        lastPathBySnapshot.set(nextKey, lastPath);
        lastPathBySnapshot.delete(previousKey);
      }
      const prefix = `${previousKey}\u0000`;
      for (const [key, listing] of listingsByPath) {
        if (!key.startsWith(prefix)) continue;
        retainListing(
          cacheKey(profileId, nextSnapshotId, key.slice(prefix.length)),
          listing,
        );
        listingsByPath.delete(key);
      }
    },
    removeSnapshot(profileId, snapshotId) {
      if (selectedSnapshotByProfile.get(profileId) === snapshotId) {
        selectedSnapshotByProfile.delete(profileId);
      }
      const key = snapshotKey(profileId, snapshotId);
      lastPathBySnapshot.delete(key);
      const prefix = `${key}\u0000`;
      for (const key of listingsByPath.keys()) {
        if (key.startsWith(prefix)) listingsByPath.delete(key);
      }
    },
    clearListings(profileId) {
      const prefix = `${profileId}\u0000`;
      for (const key of listingsByPath.keys()) {
        if (key.startsWith(prefix)) listingsByPath.delete(key);
      }
    },
    clear() {
      listingsByPath.clear();
      selectedSnapshotByProfile.clear();
      lastPathBySnapshot.clear();
    },
  };
}
