import { match } from "ts-pattern";

export type SelectionMode = "single" | "multiple";
export type Selection = string | readonly string[] | undefined;

export function toggleSelection(
  current: Selection,
  item: string,
  mode: SelectionMode,
  allowEmpty = false,
): Selection {
  return match(mode)
    .with("single", () => (current === item && allowEmpty ? undefined : item))
    .with("multiple", () => {
      const values = Array.isArray(current) ? current : [];
      return values.includes(item)
        ? values.filter((value) => value !== item)
        : [...values, item];
    })
    .exhaustive();
}

export function isSelected(selection: Selection, item: string): boolean {
  return Array.isArray(selection)
    ? selection.includes(item)
    : selection === item;
}
