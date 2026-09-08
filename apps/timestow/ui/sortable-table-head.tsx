import { mergeClasses, TableSortIndicator, Text } from "@wabou/ui";
import { Button as PrimitiveButton } from "@wabou/ui/primitives";

export function SortableTableHead(props: {
  label: string;
  class?: string;
  direction?: () => "asc" | "desc" | undefined;
  onToggle: () => void;
}) {
  const accessibleLabel = () => {
    const direction = props.direction?.();
    return direction
      ? `${props.label}, sorted ${direction === "asc" ? "ascending" : "descending"}`
      : `Sort by ${props.label}`;
  };
  return (
    <PrimitiveButton
      unstyled
      role="columnheader"
      aria-label={accessibleLabel()}
      class={(state) =>
        mergeClasses(
          "min-w-32 flex-1 px-3 flex flex-row items-center justify-start gap-2 whitespace-nowrap bg-transparent text-xs font-medium text-secondary",
          state.hovered ? "bg-control-hover text-primary" : undefined,
          props.class,
        )
      }
      onClick={props.onToggle}
    >
      <Text class="truncate text-xs font-medium">{props.label}</Text>
      <TableSortIndicator direction={() => props.direction?.()} />
    </PrimitiveButton>
  );
}
