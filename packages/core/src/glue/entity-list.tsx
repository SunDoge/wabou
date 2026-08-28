import {
  type Accessor,
  createComponent,
  createEffect,
  createMemo,
  For,
  type JSX,
  untrack,
} from "solid-js";

export type EntityKey = string | number;

export interface ForEntityProps<T, K extends EntityKey> {
  each: readonly T[] | undefined | null | false;
  by: (item: T) => K;
  fallback?: JSX.Element;
  children: (item: T, index: Accessor<number>) => JSX.Element;
}

export function validateEntityKeys<T, K extends EntityKey>(
  values: readonly T[],
  by: (item: T) => K,
): readonly T[] {
  const keys = new Set<K>();
  for (const entity of values) {
    const key = by(entity);
    if (keys.has(key)) {
      throw new Error(`ForEntity received duplicate key ${String(key)}`);
    }
    keys.add(key);
  }
  return values;
}

/**
 * Render stateful entities by a stable application key.
 *
 * The entity object itself is part of the identity contract: mutate its
 * internal signals/stores instead of replacing it with a new snapshot carrying
 * the same key. This keeps native widgets and other owned resources mounted.
 */
export function ForEntity<T, K extends EntityKey>(
  props: ForEntityProps<T, K>,
): JSX.Element {
  const by = untrack(() => props.by);
  const entities = createMemo(() => {
    const values = props.each;
    if (!values) return values;
    return validateEntityKeys(values, by);
  });
  return createComponent(
    For as unknown as (props: {
      each: readonly T[] | undefined | null | false;
      keyed: (item: T) => K;
      fallback?: JSX.Element;
      children: (item: Accessor<T>, index: Accessor<number>) => JSX.Element;
    }) => JSX.Element,
    {
      get each() {
        return entities();
      },
      keyed: by,
      get fallback() {
        return props.fallback;
      },
      children: (item, index) => {
        const entity = untrack(item);
        const key = by(entity);
        createEffect(item, (current) => {
          if (current !== entity) {
            throw new Error(
              `ForEntity key ${String(key)} replaced its entity object; keep the object stable and update its signals/store instead`,
            );
          }
        });
        return props.children(entity, index);
      },
    },
  );
}
