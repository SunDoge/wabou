import {
  type Accessor,
  createComponent,
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
  return createComponent(
    For as unknown as (props: {
      each: readonly T[] | undefined | null | false;
      keyed: (item: T) => K;
      fallback?: JSX.Element;
      children: (item: Accessor<T>, index: Accessor<number>) => JSX.Element;
    }) => JSX.Element,
    {
      get each() {
        return props.each;
      },
      keyed: props.by,
      get fallback() {
        return props.fallback;
      },
      children: (item, index) => {
        const entity = untrack(item);
        return props.children(entity, index);
      },
    },
  );
}
