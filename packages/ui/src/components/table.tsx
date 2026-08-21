import { type JSX, omit } from "solid-js";
import {
  createHover,
  Text,
  type TextProps,
  View,
  type ViewProps,
} from "../primitives";
import { join } from "./class-names";

export interface TableProps extends Omit<ViewProps, "class"> {
  class?: string;
  /** Minimum intrinsic width of the rows before horizontal scrolling begins. */
  contentClass?: string;
}

/**
 * A horizontally scrollable table surface.
 *
 * Wabou has no implicit HTML table layout. Columns align because every row
 * uses the same flex-cell anatomy; applications can override individual cell
 * widths with the usual flex and width utilities.
 */
export function Table(props: TableProps): JSX.Element {
  const rest = omit(props, "class", "contentClass", "children");
  return (
    <View
      {...rest}
      role="table"
      class={join(
        "relative w-full min-w-0 overflow-x-auto overflow-y-hidden",
        props.class,
      )}
    >
      <View
        class={join(
          "w-full min-w-full flex-none flex flex-col text-sm",
          props.contentClass,
        )}
      >
        {props.children}
      </View>
    </View>
  );
}

export function TableHeader(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role="group"
      aria-label={props["aria-label"] ?? "Table header"}
      class={join("w-full min-w-0 flex-none flex flex-col", props.class)}
    >
      {props.children}
    </View>
  );
}

export function TableBody(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role="group"
      aria-label={props["aria-label"] ?? "Table body"}
      class={join("w-full min-w-0 flex-none flex flex-col", props.class)}
    >
      {props.children}
    </View>
  );
}

export function TableFooter(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role="group"
      aria-label={props["aria-label"] ?? "Table footer"}
      class={join(
        "w-full min-w-0 flex-none flex flex-col border-t border-subtle bg-control",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export interface TableRowProps extends Omit<ViewProps, "class"> {
  class?: string;
  selected?: boolean;
}

export function TableRow(props: TableRowProps): JSX.Element {
  const hover = createHover();
  const rest = omit(
    props,
    "class",
    "selected",
    "children",
    "onPointerEnter",
    "onPointerLeave",
  );
  return (
    <View
      {...rest}
      role="row"
      aria-selected={props.selected}
      class={join(
        "w-full min-w-0 min-h-11 flex-none flex flex-row items-stretch border-b border-subtle",
        props.selected ? "bg-selected" : "bg-surface",
        hover.hovered() && !props.selected ? "bg-control-hover" : undefined,
        props.class,
      )}
      onPointerEnter={(event) => {
        hover.bindings.onPointerEnter();
        props.onPointerEnter?.(event);
      }}
      onPointerLeave={(event) => {
        hover.bindings.onPointerLeave();
        props.onPointerLeave?.(event);
      }}
    >
      {props.children}
    </View>
  );
}

export interface TableHeadProps extends Omit<TextProps, "class"> {
  class?: string;
}

export function TableHead(props: TableHeadProps): JSX.Element {
  return (
    <Text
      {...props}
      role="columnheader"
      class={join(
        "min-w-32 flex-1 px-3 flex items-center whitespace-nowrap text-xs font-medium text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export interface TableCellProps extends Omit<ViewProps, "class"> {
  class?: string;
}

export function TableCell(props: TableCellProps): JSX.Element {
  return (
    <View
      {...props}
      role="cell"
      class={join(
        "min-w-32 flex-1 px-3 flex items-center text-sm text-primary",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function TableCaption(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      role={props.role ?? "label"}
      class={join(
        "w-full min-w-0 flex-none px-3 py-3 whitespace-normal text-sm text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}
