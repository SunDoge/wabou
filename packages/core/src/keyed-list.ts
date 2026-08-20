export interface KeyedListPatch<T, Key> {
  readonly upserted: readonly T[];
  readonly removed: readonly Key[];
  readonly order: readonly Key[];
}

/**
 * Reconcile a host-owned keyed list while validating its complete order.
 * Returns `undefined` for duplicate, missing, or unaccounted-for keys so the
 * caller can request a full snapshot instead of accepting divergent state.
 */
export function reconcileKeyedList<T, Key>(
  current: readonly T[],
  patch: KeyedListPatch<T, Key>,
  keyOf: (value: T) => Key,
): T[] | undefined {
  const values = new Map(current.map((value) => [keyOf(value), value]));
  for (const key of patch.removed) values.delete(key);
  for (const value of patch.upserted) values.set(keyOf(value), value);
  if (patch.order.length !== values.size) return undefined;

  const seen = new Set<Key>();
  const ordered: T[] = [];
  for (const key of patch.order) {
    if (seen.has(key)) return undefined;
    const value = values.get(key);
    if (value === undefined) return undefined;
    seen.add(key);
    ordered.push(value);
  }
  return ordered;
}
