export interface MenuStateItem {
  id: string;
  label: string;
  disabled?: boolean;
}

export type MenuMove = "first" | "last" | "next" | "previous";

/** Resolve one keyboard move without coupling menu state to rendering. */
export function moveMenuHighlight(
  items: readonly MenuStateItem[],
  current: string | undefined,
  move: MenuMove,
): string | undefined {
  const enabled = items.filter((item) => !item.disabled);
  if (enabled.length === 0) return undefined;
  if (move === "first") return enabled[0].id;
  if (move === "last") return enabled.at(-1)?.id;
  const index = enabled.findIndex((item) => item.id === current);
  if (move === "next") return enabled[(index + 1) % enabled.length].id;
  return enabled[(index <= 0 ? enabled.length : index) - 1].id;
}
