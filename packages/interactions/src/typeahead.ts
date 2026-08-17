import type { CollectionItem } from "./collection";

export interface TypeaheadOptions {
  timeout?: number;
  locale?: string;
}

export interface Typeahead<T extends CollectionItem> {
  search(items: readonly T[], key: string, activeId?: string): T | undefined;
  reset(): void;
}

export function createTypeahead<T extends CollectionItem>(
  options: TypeaheadOptions = {},
): Typeahead<T> {
  let keys = "";
  let timer: ReturnType<typeof setTimeout> | undefined;
  const collator =
    typeof Intl === "undefined" || typeof Intl.Collator !== "function"
      ? undefined
      : new Intl.Collator(options.locale, {
          usage: "search",
          sensitivity: "base",
        });
  const reset = () => {
    keys = "";
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
  };
  return {
    search(items, key, activeId) {
      if (key.length !== 1) return undefined;
      keys += key;
      if (timer !== undefined) clearTimeout(timer);
      timer = setTimeout(reset, options.timeout ?? 350);
      const repeated =
        keys.length > 1 && [...keys].every((value) => value === keys[0]);
      const query = repeated ? keys[0] : keys;
      const enabled = items.filter((item) => !item.disabled && item.textValue);
      const active = enabled.findIndex((item) => item.id === activeId);
      const ordered = [
        ...enabled.slice(active + 1),
        ...enabled.slice(0, active + 1),
      ];
      return ordered.find((item) => {
        const prefix = item.textValue?.slice(0, query.length) ?? "";
        return collator
          ? collator.compare(prefix, query) === 0
          : prefix.toLowerCase() === query.toLowerCase();
      });
    },
    reset,
  };
}
