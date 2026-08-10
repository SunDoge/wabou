import { match, P } from "ts-pattern";

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
}

export function createRovingFocus(options: RovingFocusOptions = {}) {
  const items: RovingFocusItem[] = [];
  const enabled = () => items.filter((item) => !item.disabled?.());
  return {
    register(item: RovingFocusItem) {
      items.push(item);
      return () => {
        const index = items.indexOf(item);
        if (index >= 0) items.splice(index, 1);
      };
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
      options.onMove?.(target.id);
      target.target.focus();
      return true;
    },
  };
}
