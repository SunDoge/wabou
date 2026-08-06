import { type Accessor, createMemo, createSignal } from "solid-js";

export type TabKey = string | number;

export interface TabsOptions<T, K extends TabKey> {
  initialTabs?: readonly T[];
  key: (tab: T) => K;
  initialActiveKey?: K;
  onActiveChange?: (key: K | undefined) => void;
  orientation?: "horizontal" | "vertical";
}

export interface AddTabOptions {
  activate?: boolean;
  index?: number;
}

/** Minimal native capability needed for keyboard focus movement. */
export interface FocusTarget {
  focus(): void;
}

export interface TabsResult<T, K extends TabKey> {
  tabs: Accessor<readonly T[]>;
  activeKey: Accessor<K | undefined>;
  activeTab: Accessor<T | undefined>;
  select: (key: K) => boolean;
  selectNext: () => boolean;
  selectPrevious: () => boolean;
  selectFirst: () => boolean;
  selectLast: () => boolean;
  add: (tab: T, options?: AddTabOptions) => boolean;
  close: (key: K) => boolean;
  move: (key: K, index: number) => boolean;
  register: (key: K, node: FocusTarget) => void;
  focus: (key: K) => boolean;
  handleKeyDown: (key: K, event: TabKeyEvent) => boolean;
}

export interface TabKeyEvent {
  key: string;
  preventDefault?: () => void;
}

/**
 * Stateful tab collection with stable identity and deterministic activation.
 *
 * Closing the active tab selects its right-hand neighbour, or the previous
 * tab when the closed tab was last. Reordering never changes the active key.
 */
export function createTabs<T, K extends TabKey>(
  options: TabsOptions<T, K>,
): TabsResult<T, K> {
  const initialTabs = [...(options.initialTabs ?? [])];
  assertUniqueKeys(initialTabs, options.key);

  const initialActiveKey =
    options.initialActiveKey !== undefined &&
    initialTabs.some((tab) => options.key(tab) === options.initialActiveKey)
      ? options.initialActiveKey
      : initialTabs[0] === undefined
        ? undefined
        : options.key(initialTabs[0]);
  const [tabs, setTabs] = createSignal<readonly T[]>(initialTabs);
  const [activeKey, setActiveKey] = createSignal<K | undefined>(
    initialActiveKey,
  );
  const focusTargets = new Map<K, FocusTarget>();

  const commitActiveKey = (key: K | undefined) => {
    if (activeKey() === key) return false;
    setActiveKey(() => key);
    options.onActiveChange?.(key);
    return true;
  };
  const select = (key: K) => {
    if (!tabs().some((tab) => options.key(tab) === key)) return false;
    return commitActiveKey(key);
  };
  const selectAt = (index: number) => {
    const values = tabs();
    if (values.length === 0) return false;
    const normalized =
      ((index % values.length) + values.length) % values.length;
    return select(options.key(values[normalized]));
  };
  const activeIndex = () => {
    const current = activeKey();
    return current === undefined
      ? -1
      : tabs().findIndex((tab) => options.key(tab) === current);
  };
  const focus = (key: K) => {
    const target = focusTargets.get(key);
    if (target === undefined) return false;
    target.focus();
    return true;
  };
  const activateAt = (index: number, moveFocus: boolean) => {
    const values = tabs();
    if (values.length === 0) return false;
    const normalized =
      ((index % values.length) + values.length) % values.length;
    const key = options.key(values[normalized]);
    select(key);
    if (moveFocus) focus(key);
    return true;
  };

  return {
    tabs,
    activeKey,
    activeTab: createMemo(() => {
      const current = activeKey();
      return current === undefined
        ? undefined
        : tabs().find((tab) => options.key(tab) === current);
    }),
    select,
    selectNext: () => selectAt(activeIndex() + 1),
    selectPrevious: () =>
      selectAt(activeIndex() <= 0 ? tabs().length - 1 : activeIndex() - 1),
    selectFirst: () => selectAt(0),
    selectLast: () => selectAt(tabs().length - 1),
    add: (tab, addOptions = {}) => {
      const key = options.key(tab);
      const current = tabs();
      if (current.some((candidate) => options.key(candidate) === key))
        return false;
      const index = Math.max(
        0,
        Math.min(addOptions.index ?? current.length, current.length),
      );
      const next = [...current];
      next.splice(index, 0, tab);
      setTabs(next);
      if (activeKey() === undefined || addOptions.activate !== false) {
        commitActiveKey(key);
      }
      return true;
    },
    close: (key) => {
      const current = tabs();
      const index = current.findIndex((tab) => options.key(tab) === key);
      if (index < 0) return false;
      const next = current.filter(
        (_, candidateIndex) => candidateIndex !== index,
      );
      setTabs(next);
      focusTargets.delete(key);
      if (activeKey() === key) {
        const neighbour = next[Math.min(index, next.length - 1)];
        commitActiveKey(
          neighbour === undefined ? undefined : options.key(neighbour),
        );
      }
      return true;
    },
    move: (key, requestedIndex) => {
      const current = tabs();
      const index = current.findIndex((tab) => options.key(tab) === key);
      if (index < 0) return false;
      const target = Math.max(0, Math.min(requestedIndex, current.length - 1));
      if (target === index) return false;
      const next = [...current];
      const [tab] = next.splice(index, 1);
      next.splice(target, 0, tab);
      setTabs(next);
      return true;
    },
    register: (key, node) => {
      if (tabs().some((tab) => options.key(tab) === key)) {
        focusTargets.set(key, node);
      }
    },
    focus,
    handleKeyDown: (key, event) => {
      const index = tabs().findIndex((tab) => options.key(tab) === key);
      if (index < 0) return false;
      const horizontal = options.orientation !== "vertical";
      const target =
        event.key === "Home"
          ? 0
          : event.key === "End"
            ? tabs().length - 1
            : (horizontal && event.key === "ArrowRight") ||
                (!horizontal && event.key === "ArrowDown")
              ? index + 1
              : (horizontal && event.key === "ArrowLeft") ||
                  (!horizontal && event.key === "ArrowUp")
                ? index - 1
                : undefined;
      if (target === undefined) return false;
      event.preventDefault?.();
      return activateAt(target, true);
    },
  };
}

function assertUniqueKeys<T, K extends TabKey>(
  tabs: readonly T[],
  key: (tab: T) => K,
): void {
  const keys = new Set<K>();
  for (const tab of tabs) {
    const value = key(tab);
    if (keys.has(value)) {
      throw new Error(`Duplicate tab key: ${String(value)}`);
    }
    keys.add(value);
  }
}
