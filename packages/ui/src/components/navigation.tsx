import {
  createComponent,
  createContext,
  For,
  type JSX,
  omit,
  useContext,
} from "solid-js";
import {
  Button as HeadlessButton,
  Icon,
  Text,
  type TextProps,
  View,
  type ViewProps,
} from "../primitives";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import ellipsis from "lucide-static/icons/ellipsis.svg?raw";
import { Button, type ButtonProps } from "./button";
import { mergeClasses } from "@wabou/core/style";
import {
  clampPage,
  createPaginationRange,
  normalizePageCount,
} from "./pagination-state";
import { createControllableState } from "../primitives/interactions";

export {
  clampPage,
  createPaginationRange,
  normalizePageCount,
  type PaginationRangeItem,
} from "./pagination-state";

export interface BreadcrumbProps extends Omit<ViewProps, "class" | "role"> {
  class?: string;
}

export function Breadcrumb(props: BreadcrumbProps): JSX.Element {
  const rest = omit(props, "class", "children");
  return (
    <View
      {...rest}
      role="group"
      aria-label={props["aria-label"] ?? "Breadcrumb"}
      class={mergeClasses("min-w-0", props.class)}
    >
      {props.children}
    </View>
  );
}

export function BreadcrumbList(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "min-w-0 flex flex-wrap items-center gap-1.5 text-sm text-muted",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function BreadcrumbItem(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses("min-w-0 flex items-center gap-1.5", props.class)}
    >
      {props.children}
    </View>
  );
}

export interface BreadcrumbLinkProps
  extends Omit<ButtonProps, "class" | "role" | "variant" | "size"> {
  class?: string;
}

export function BreadcrumbLink(props: BreadcrumbLinkProps): JSX.Element {
  return (
    <HeadlessButton
      {...props}
      unstyled
      role="link"
      class={(state) =>
        mergeClasses(
          "min-w-0 rounded-sm text-sm text-secondary",
          state.hovered && "text-primary",
          state.focusVisible && "border border-focus",
          props.class,
        )
      }
    />
  );
}

export interface BreadcrumbPageProps extends Omit<TextProps, "class" | "role"> {
  class?: string;
}

export function BreadcrumbPage(props: BreadcrumbPageProps): JSX.Element {
  const rest = omit(props, "class", "children");
  return (
    <Text
      {...rest}
      role="link"
      aria-disabled="true"
      aria-current="page"
      class={mergeClasses(
        "min-w-0 text-sm font-medium text-primary",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export interface BreadcrumbSeparatorProps
  extends Omit<ViewProps, "class" | "role"> {
  class?: string;
}

export function BreadcrumbSeparator(
  props: BreadcrumbSeparatorProps,
): JSX.Element {
  const rest = omit(props, "class", "children");
  return (
    <View
      {...rest}
      role="presentation"
      aria-hidden="true"
      class={mergeClasses(
        "w-4 h-4 flex-none flex items-center justify-center text-muted",
        props.class,
      )}
    >
      {props.children ?? <Icon source={chevronRight} size={14} />}
    </View>
  );
}

export interface BreadcrumbEllipsisProps
  extends Omit<ViewProps, "class" | "role" | "children"> {
  class?: string;
}

export function BreadcrumbEllipsis(
  props: BreadcrumbEllipsisProps,
): JSX.Element {
  const rest = omit(props, "class");
  return (
    <View
      {...rest}
      role="presentation"
      aria-hidden="true"
      class={mergeClasses(
        "w-8 h-8 flex-none flex items-center justify-center text-muted",
        props.class,
      )}
    >
      <Icon source={ellipsis} size={16} />
    </View>
  );
}

interface PaginationContextValue {
  managed: boolean;
  count(): number;
  page(): number;
  disabled(): boolean;
  select(page: number): void;
}

// A real sentinel keeps the legacy composition-only controls usable outside
// managed pagination. Solid 2 treats `undefined` as no context default.
const PaginationContext = createContext<PaginationContextValue>({
  managed: false,
  count: () => 1,
  page: () => 1,
  disabled: () => false,
  select: () => {},
});

export interface PaginationProps {
  children?: JSX.Element;
  class?: string;
  "aria-label"?: string;
  /** Enables managed pagination. Omit it to retain the composition-only API. */
  count?: number;
  page?: number;
  defaultPage?: number;
  disabled?: boolean;
  onPageChange?: (page: number) => void;
}

export function Pagination(props: PaginationProps): JSX.Element {
  const state = createControllableState({
    value: () => props.page,
    defaultValue: props.defaultPage ?? 1,
    onChange: props.onPageChange,
  });
  const count = () => normalizePageCount(props.count ?? 1);
  const page = () => clampPage(state.value(), count());
  const context: PaginationContextValue = {
    managed: true,
    count,
    page,
    disabled: () => props.disabled ?? false,
    select: (next) => {
      if (props.count === undefined || context.disabled()) return;
      state.set(clampPage(next, count()));
    },
  };
  const content = () => (
    <View
      role="group"
      aria-label={props["aria-label"] ?? "Pagination"}
      aria-disabled={context.disabled() || undefined}
      class={mergeClasses("flex items-center", props.class)}
    >
      {props.children}
    </View>
  );
  return props.count === undefined
    ? content()
    : createComponent(PaginationContext, {
        value: context,
        get children() {
          return content();
        },
      });
}

export function PaginationContent(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses("flex items-center gap-1", props.class)}
    />
  );
}

export function PaginationItem(props: ViewProps): JSX.Element {
  return <View {...props} class={mergeClasses("flex-none", props.class)} />;
}

export interface PaginationLinkProps
  extends Omit<ButtonProps, "variant" | "size"> {
  active?: boolean;
  /** Selects this page when used inside a managed Pagination. */
  page?: number;
}

export function PaginationLink(props: PaginationLinkProps): JSX.Element {
  const context = useContext(PaginationContext);
  const forwarded = omit(props, "active", "page");
  const active = () =>
    props.active ??
    (props.page !== undefined &&
      context.managed &&
      context.page() === props.page);
  return (
    <Button
      {...forwarded}
      role="link"
      size="icon"
      variant={active() ? "outline" : "ghost"}
      selected={active()}
      aria-current={active() ? "page" : undefined}
      aria-label={
        props["aria-label"] ??
        (props.page === undefined ? undefined : `Page ${props.page}`)
      }
      disabled={props.disabled ?? (context.managed && context.disabled())}
      onClick={(event) => {
        props.onClick?.(event);
        if (!event.defaultPrevented && props.page !== undefined)
          context.select(props.page);
      }}
    />
  );
}

export function PaginationEllipsis(props: { class?: string }): JSX.Element {
  return (
    <Text
      aria-hidden
      class={mergeClasses("w-8 text-center text-muted", props.class)}
    >
      ...
    </Text>
  );
}

export function PaginationItems(props: {
  siblingCount?: number;
  boundaryCount?: number;
  renderItem?: (page: number) => JSX.Element;
  renderEllipsis?: (side: "start" | "end") => JSX.Element;
}): JSX.Element {
  const context = useContext(PaginationContext);
  if (!context.managed)
    throw new Error("PaginationItems must be used inside a managed Pagination");
  const items = () =>
    createPaginationRange({
      count: context.count(),
      page: context.page(),
      siblingCount: props.siblingCount,
      boundaryCount: props.boundaryCount,
    });
  return (
    <For each={items()}>
      {(item) =>
        typeof item === "number"
          ? (props.renderItem?.(item) ?? (
              <PaginationItem>
                <PaginationLink page={item}>{String(item)}</PaginationLink>
              </PaginationItem>
            ))
          : (props.renderEllipsis?.(
              item === "ellipsis-start" ? "start" : "end",
            ) ?? <PaginationEllipsis />)
      }
    </For>
  );
}

export function PaginationPrevious(
  props: Omit<ButtonProps, "variant" | "size">,
): JSX.Element {
  const context = useContext(PaginationContext);
  return (
    <Button
      {...props}
      variant="ghost"
      size="sm"
      disabled={
        props.disabled ??
        (context.managed ? context.disabled() || context.page() <= 1 : false)
      }
      onClick={(event) => {
        props.onClick?.(event);
        if (!event.defaultPrevented && context.managed)
          context.select(context.page() - 1);
      }}
    >
      {props.children ?? "Previous"}
    </Button>
  );
}

export function PaginationNext(
  props: Omit<ButtonProps, "variant" | "size">,
): JSX.Element {
  const context = useContext(PaginationContext);
  return (
    <Button
      {...props}
      variant="ghost"
      size="sm"
      disabled={
        props.disabled ??
        (context.managed
          ? context.disabled() || context.page() >= context.count()
          : false)
      }
      onClick={(event) => {
        props.onClick?.(event);
        if (!event.defaultPrevented && context.managed)
          context.select(context.page() + 1);
      }}
    >
      {props.children ?? "Next"}
    </Button>
  );
}
