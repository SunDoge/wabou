import { mergeClasses, Text } from "@wabou/ui";
import { Button as PrimitiveButton } from "@wabou/ui/primitives";

export function SortableTableHead(props: {
  label: string;
  class?: string;
  direction?: () => "asc" | "desc" | undefined;
  onToggle: () => void;
}) {
  return (
    <PrimitiveButton
      unstyled
      role="columnheader"
      aria-label={`Sort by ${props.label}`}
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
      <Text class="ml-auto text-xs">
        {props.direction?.() === "asc"
          ? "Asc"
          : props.direction?.() === "desc"
            ? "Desc"
            : ""}
      </Text>
    </PrimitiveButton>
  );
}
