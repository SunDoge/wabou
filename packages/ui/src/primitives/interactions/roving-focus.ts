import { match, P } from "ts-pattern";
import { createSignal } from "solid-js";

export interface FocusTarget {
  focus(): void;
}
export interface RovingFocusItem {
  id: string;
  target: FocusTarget;
  disabled?: () => boolean;
}
export interface RovingFocusOptions {
  orientation?: () => "horizontal" | "vertical";
  loop?: boolean;
  onMove?: (id: string) => void;
  /** Prefer a selected/active item when focus has not entered the group yet. */
  preferred?: (id: string) => boolean;
}

export function createRovingFocus(options: RovingFocusOptions = {}) {
  const items: RovingFocusItem[] = [];
  // Registration is tied to rendered item ownership. Solid disposes those
  // items from an owned scope, so cleanup intentionally updates the shared
  // roving-focus registry rather than authored component state.
  const [activeId, setActiveId] = createSignal<string | undefined>(undefined, {
    ownedWrite: true,
  });
  const [registryVersion, setRegistryVersion] = createSignal(0, {
    ownedWrite: true,
  });
  const enabled = () => items.filter((item) => !item.disabled?.());
  const currentTabStop = () => {
    registryVersion();
    const candidates = enabled();
    const active = activeId();
    return (
      candidates.find((item) => item.id === active)?.id ??
      candidates.find((item) => options.preferred?.(item.id))?.id ??
      candidates[0]?.id
    );
  };
  const activate = (id: string) => {
    if (!enabled().some((item) => item.id === id)) return false;
    setActiveId(id);
    return true;
  };
  return {
    register(item: RovingFocusItem) {
      items.push(item);
      setRegistryVersion((version) => version + 1);
      return () => {
        const index = items.indexOf(item);
        if (index >= 0) items.splice(index, 1);
        if (activeId() === item.id) setActiveId(undefined);
        setRegistryVersion((version) => version + 1);
      };
    },
    activate,
    isTabStop(id: string) {
      return currentTabStop() === id;
    },
    move(current: string, key: string) {
      const orientation = options.orientation?.() ?? "horizontal";
      const direction = match({ orientation, key })
        .with({ key: "Home" }, () => "first" as const)
        .with({ key: "End" }, () => "last" as const)
        .with(
          P.union(
            { orientation: "horizontal", key: "ArrowRight" },
            { orientation: "vertical", key: "ArrowDown" },
          ),
          () => "next" as const,
        )
        .with(
          P.union(
            { orientation: "horizontal", key: "ArrowLeft" },
            { orientation: "vertical", key: "ArrowUp" },
          ),
          () => "previous" as const,
        )
        .otherwise(() => undefined);
      if (!direction) return false;
      const candidates = enabled();
      if (candidates.length === 0) return false;
      const index = candidates.findIndex((item) => item.id === current);
      const target = match(direction)
        .with("first", () => candidates[0])
        .with("last", () => candidates.at(-1))
        .with(
          "next",
          () =>
            candidates[index + 1] ??
            (options.loop === false ? undefined : candidates[0]),
        )
        .with(
          "previous",
          () =>
            candidates[index - 1] ??
            (options.loop === false ? undefined : candidates.at(-1)),
        )
        .exhaustive();
      if (!target) return false;
      activate(target.id);
      options.onMove?.(target.id);
      target.target.focus();
      return true;
    },
  };
}
