import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/content-state.d.ts
export type ContentStateKind = "empty" | "loading" | "error";
export interface ContentStateProps extends Omit<ViewProps, "children" | "role"> {
  state: ContentStateKind;
  title: string;
  description?: string;
  /** Standard compact recovery or next-step action. */
  action?: {
    label: string;
    onAction(): void;
  };
  /** Lazily render media inside the state's reactive owner. */
  renderMedia?: () => JSX.Element;
  /** Lazily render custom actions inside the state's reactive owner. */
  renderAction?: () => JSX.Element;
}
/** Mutually exclusive loading, empty, or error state for a bounded region. */
export declare function ContentState(props: ContentStateProps): JSX.Element;
export interface ResourceBoundaryProps extends Omit<ViewProps, "children" | "role"> {
  /** True while the resource is performing its initial load or refreshing. */
  loading: boolean;
  /** A rejected resource value. `undefined` and `null` mean no error. */
  error?: unknown;
  /** Whether a usable resource value currently exists, including an empty collection. */
  hasContent: boolean;
  loadingTitle: string;
  errorTitle: string;
  emptyTitle: string;
  loadingDescription?: string;
  emptyDescription?: string;
  retryLabel?: string;
  onRetry?: () => void;
  renderContent(): JSX.Element;
  renderEmptyMedia?: () => JSX.Element;
  renderErrorMedia?: () => JSX.Element;
}
/**
 * Mutually exclusive async resource boundary.
 *
 * Existing content remains mounted during a background refresh. This avoids
 * replacing a useful inspector or list with a loading spinner every time its
 * resource is refreshed.
 */
export declare function ResourceBoundary(props: ResourceBoundaryProps): JSX.Element;
//#endregion
//# sourceMappingURL=content-state.d.mts.map