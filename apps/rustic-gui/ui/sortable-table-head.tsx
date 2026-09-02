import { Button, mergeClasses, Text } from "@wabou/ui";

export function SortableTableHead(props: {
  label: string;
  class?: string;
  direction?: () => "asc" | "desc" | undefined;
  onToggle: () => void;
}) {
  return (
    <Button
      unstyled
      role="columnheader"
      aria-label={`Sort by ${props.label}`}
      class={mergeClasses(
        "min-w-32 flex-1 px-3 flex flex-row items-center justify-start gap-2 whitespace-nowrap text-xs font-medium text-muted",
        props.class,
      )}
      onClick={props.onToggle}
    >
      <Text class="truncate text-xs font-medium text-muted">{props.label}</Text>
      <Text class="ml-auto text-xs text-muted">
        {props.direction?.() === "asc"
          ? "Asc"
          : props.direction?.() === "desc"
            ? "Desc"
            : ""}
      </Text>
    </Button>
  );
}
