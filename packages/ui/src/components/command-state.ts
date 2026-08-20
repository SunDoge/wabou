import { type MenuStateItem, moveMenuHighlight } from "./menu-state";

export interface CommandStateItem extends MenuStateItem {
  keywords?: readonly string[];
}

export function filterCommandItems<T extends CommandStateItem>(
  items: readonly T[],
  query: string,
): T[] {
  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  if (terms.length === 0) return [...items];
  return items.filter((item) => {
    const haystack = [item.label, ...(item.keywords ?? [])]
      .join(" ")
      .toLocaleLowerCase();
    return terms.every((term) => haystack.includes(term));
  });
}

export function reconcileCommandHighlight<T extends CommandStateItem>(
  items: readonly T[],
  highlighted: string | undefined,
): string | undefined {
  if (items.some((item) => item.id === highlighted && !item.disabled)) {
    return highlighted;
  }
  return moveMenuHighlight(items, undefined, "first");
}
