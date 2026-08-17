export class LocatorAmbiguousError extends Error {}

export interface NativeLocatorQuery<T> {
  matchCount: number;
  snapshot: T | null;
}

/** Decode and validate the request-scoped envelope without choosing a match. */
export function decodeNativeLocatorQuery<T>(
  raw: string | null | undefined,
): NativeLocatorQuery<T> | null {
  if (raw == null) return null;
  const query = JSON.parse(raw) as NativeLocatorQuery<T>;
  if (!Number.isSafeInteger(query.matchCount) || query.matchCount < 1) {
    throw new Error(`native locator query returned an invalid match count`);
  }
  return query;
}

/** Whether a locator occurrence is absent from this completed query frame. */
export function locatorQueryIsAbsent(
  raw: string | null | undefined,
  index?: number,
): boolean {
  const query = decodeNativeLocatorQuery(raw);
  return query === null || (index !== undefined && index >= query.matchCount);
}

/** Count matches without applying strict or indexed locator selection. */
export function locatorQueryMatchCount(raw: string | null | undefined): number {
  return decodeNativeLocatorQuery(raw)?.matchCount ?? 0;
}

/** Decode one request-scoped native query and enforce strict locator identity. */
export function decodeLocatorQuery<T>(
  raw: string | null | undefined,
  description: string,
  index?: number,
): T | null {
  const query = decodeNativeLocatorQuery<T>(raw);
  if (query === null) return null;
  if (index === undefined && query.matchCount !== 1) {
    throw new LocatorAmbiguousError(
      `found ${query.matchCount} matches for ${description}; expected exactly one`,
    );
  }
  if (index !== undefined && index >= query.matchCount) return null;
  if (query.snapshot === null) {
    throw new Error(`native locator query omitted the selected snapshot`);
  }
  return query.snapshot;
}
