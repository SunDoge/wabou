import { type JSX, omit } from "solid-js";
import { View, type ViewProps } from "../primitives";
import { Sidebar, type SidebarProps } from "./sidebar";
import {
  workbenchClass,
  workbenchContentClass,
  workbenchFooterClass,
  workbenchHeaderClass,
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

/** Fixed chrome below the workbench content, such as a composer or status bar. */
export function WorkbenchFooter(props: ViewProps): JSX.Element {
  const forwarded = omit(props, "class");
  return <View {...forwarded} class={workbenchFooterClass(props.class)} />;
}
