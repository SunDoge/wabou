import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { Text, View, type ViewProps } from "../primitives";
import { Spinner } from "./display";
import { Sidebar, type SidebarProps } from "./sidebar";
import {
  workbenchClass,
  workbenchContentClass,
  workbenchContentColumnClass,
  workbenchFooterClass,
  workbenchHeaderClass,
  workbenchInspectorClass,
  workbenchInspectorContentClass,
  workbenchInspectorHeaderClass,
  workbenchMainClass,
  workbenchSidebarClass,
} from "./workbench-style";

export * from "./workbench-style";

/** Full-window desktop application boundary with explicit shrink semantics. */
export function Workbench(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return <View {...forwarded} class={workbenchClass(props.class)} />;
}

/** Fixed-width navigation rail paired with a {@link WorkbenchMain}. */
export function WorkbenchSidebar(props: SidebarProps): JSX.Element {
  const forwarded = omit(props, "class");
  return <Sidebar {...forwarded} class={workbenchSidebarClass(props.class)} />;
}

/** The resizable application column beside the navigation rail. */
export function WorkbenchMain(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return <View {...forwarded} class={workbenchMainClass(props.class)} />;
}

/** Shared 48px chrome row for both sidebar and content headers. */
export function WorkbenchHeader(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return <View {...forwarded} class={workbenchHeaderClass(props.class)} />;
}

/** Bounded application content. Add a ScrollArea inside when scrolling is needed. */
export function WorkbenchContent(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return <View {...forwarded} class={workbenchContentClass(props.class)} />;
}

/** A centered 896px desktop content column that still shrinks with its pane. */
export function WorkbenchContentColumn(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return (
    <View {...forwarded} class={workbenchContentColumnClass(props.class)} />
  );
}

/** Fixed chrome below the workbench content, such as a composer or status bar. */
export function WorkbenchFooter(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return <View {...forwarded} class={workbenchFooterClass(props.class)} />;
}

/** Fixed-width auxiliary pane for file previews, diffs and contextual tools. */
export function WorkbenchInspector(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return <View {...forwarded} class={workbenchInspectorClass(props.class)} />;
}

/** Inspector title row with a stable height and bounded children. */
export function WorkbenchInspectorHeader(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return (
    <View {...forwarded} class={workbenchInspectorHeaderClass(props.class)} />
  );
}

/** Flexible, clipped inspector body. Add a ScrollArea inside when needed. */
export function WorkbenchInspectorContent(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return (
    <View {...forwarded} class={workbenchInspectorContentClass(props.class)} />
  );
}

export type WorkbenchInspectorStateKind = "empty" | "loading" | "error";

export interface WorkbenchInspectorStateProps
  extends Omit<ViewProps, "children" | "role"> {
  state: WorkbenchInspectorStateKind;
  title: string;
  description?: string;
  /** Lazily render media inside the inspector state's reactive owner. */
  renderMedia?: () => JSX.Element;
  /** Lazily render actions inside the inspector state's reactive owner. */
  renderAction?: () => JSX.Element;
}

/** Mutually exclusive centered state for a bounded inspector body. */
export function WorkbenchInspectorState(
  props: WorkbenchInspectorStateProps,
): JSX.Element {
  const forwarded = omit(
    props,
    "state",
    "title",
    "description",
    "renderMedia",
    "renderAction",
    "class",
  );
  const error = () => props.state === "error";
  return (
    <View
      {...forwarded}
      role={error() ? "alert" : "status"}
      aria-label={props["aria-label"] ?? props.title}
      class={mergeClasses(
        "w-full h-full min-w-0 min-h-0 flex-1 p-6 flex flex-col items-center justify-center gap-3 text-center",
        error() ? "text-danger-primary" : "text-secondary",
        props.class,
      )}
    >
      {props.renderMedia?.() ??
        (props.state === "loading" ? <Spinner decorative /> : null)}
      <View class="w-full max-w-sm min-w-0 flex flex-col items-center gap-1">
        <Text
          class={mergeClasses(
            "w-full min-w-0 whitespace-normal text-sm font-medium",
            error() ? "text-danger-primary" : "text-primary",
          )}
        >
          {props.title}
        </Text>
        {props.description === undefined ? null : (
          <Text
            class={mergeClasses(
              "w-full min-w-0 whitespace-normal text-xs",
              error() ? "text-danger-primary" : "text-muted",
            )}
          >
            {props.description}
          </Text>
        )}
      </View>
      {props.renderAction?.()}
    </View>
  );
}
