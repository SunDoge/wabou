import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ContentStateProps } from "./content-state.mjs";
import { SidebarProps } from "./sidebar.mjs";
import { workbenchClass, workbenchContentClass, workbenchContentColumnClass, workbenchFooterClass, workbenchHeaderClass, workbenchInspectorClass, workbenchInspectorContentClass, workbenchInspectorHeaderClass, workbenchMainClass, workbenchSidebarClass } from "./workbench-style.mjs";
import { JSX } from "solid-js";
//#region src/components/workbench.d.ts
/** Full-window desktop application boundary with explicit shrink semantics. */
export declare function Workbench(props: ViewProps): JSX.Element;
/** Fixed-width navigation rail paired with a {@link WorkbenchMain}. */
export declare function WorkbenchSidebar(props: SidebarProps): JSX.Element;
/** The resizable application column beside the navigation rail. */
export declare function WorkbenchMain(props: ViewProps): JSX.Element;
/** Shared 48px chrome row for both sidebar and content headers. */
export declare function WorkbenchHeader(props: ViewProps): JSX.Element;
/** Bounded application content. Add a ScrollArea inside when scrolling is needed. */
export declare function WorkbenchContent(props: ViewProps): JSX.Element;
/** A centered 896px desktop content column that still shrinks with its pane. */
export declare function WorkbenchContentColumn(props: ViewProps): JSX.Element;
/** Fixed chrome below the workbench content, such as a composer or status bar. */
export declare function WorkbenchFooter(props: ViewProps): JSX.Element;
/** Fixed-width auxiliary pane for file previews, diffs and contextual tools. */
export declare function WorkbenchInspector(props: ViewProps): JSX.Element;
/** Inspector title row with a stable height and bounded children. */
export declare function WorkbenchInspectorHeader(props: ViewProps): JSX.Element;
export interface WorkbenchInspectorTitlebarProps extends Omit<ViewProps, "children"> {
  title: string;
  description?: string;
  closeLabel?: string;
  onClose?: () => void;
  /** Lazily render an optional action before the standard close button. */
  renderAction?: () => JSX.Element;
}
/**
 * Consistent inspector chrome with a bounded title, optional description and
 * correctly sized trailing actions.
 */
export declare function WorkbenchInspectorTitlebar(props: WorkbenchInspectorTitlebarProps): JSX.Element;
/** Flexible, clipped inspector body. Add a ScrollArea inside when needed. */
export declare function WorkbenchInspectorContent(props: ViewProps): JSX.Element;
export interface WorkbenchInspectorStateProps extends ContentStateProps {}
/** Mutually exclusive centered state for a bounded inspector body. */
export declare function WorkbenchInspectorState(props: WorkbenchInspectorStateProps): JSX.Element;
//#endregion
export { workbenchClass, workbenchContentClass, workbenchContentColumnClass, workbenchFooterClass, workbenchHeaderClass, workbenchInspectorClass, workbenchInspectorContentClass, workbenchInspectorHeaderClass, workbenchMainClass, workbenchSidebarClass };
//# sourceMappingURL=workbench.d.mts.map