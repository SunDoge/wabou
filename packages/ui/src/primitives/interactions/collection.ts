export interface CollectionItem {
  id: string;
  disabled?: boolean;
  textValue?: string;
}

export interface Collection<T extends CollectionItem> {
  items(): readonly T[];
  find(id: string): T | undefined;
  indexOf(id: string): number;
  first(): T | undefined;
  last(): T | undefined;
  next(id: string | undefined, loop?: boolean): T | undefined;
  previous(id: string | undefined, loop?: boolean): T | undefined;
}

export function createCollection<T extends CollectionItem>(
  source: () => readonly T[],
): Collection<T> {
  const enabled = () => source().filter((item) => !item.disabled);
  const adjacent = (id: string | undefined, delta: -1 | 1, loop: boolean) => {
    const items = enabled();
    if (items.length === 0) return undefined;
    const index =
      id === undefined ? -1 : items.findIndex((item) => item.id === id);
    const next = index + delta;
    if (next >= 0 && next < items.length) return items[next];
    if (!loop) return undefined;
    return delta === 1 ? items[0] : items[items.length - 1];
  };
  return {
    items: source,
    find: (id) => source().find((item) => item.id === id),
    indexOf: (id) => source().findIndex((item) => item.id === id),
    first: () => enabled()[0],
    last: () => enabled().at(-1),
    next: (id, loop = false) => adjacent(id, 1, loop),
    previous: (id, loop = false) => adjacent(id, -1, loop),
  };
}
