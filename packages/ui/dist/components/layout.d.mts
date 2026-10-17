import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { Accessor, JSX } from "solid-js";
//#region src/components/layout.d.ts
export type ResponsiveGridColumnCount = 1 | 2 | 3 | 4;
export interface ResponsiveGridState {
  columns: Accessor<ResponsiveGridColumnCount>;
  width: Accessor<number>;
  height: Accessor<number>;
}
/** Read the completed native size and active column count of the nearest grid. */
export declare function useResponsiveGrid(): ResponsiveGridState;
export declare function responsiveGridColumnCount(options: {
  width: number;
  minColumnWidth: number;
  gap?: number;
  maxColumns?: ResponsiveGridColumnCount;
  initialColumns?: ResponsiveGridColumnCount;
  itemCount?: number;
  balanceLastRow?: boolean;
}): ResponsiveGridColumnCount;
export declare function responsiveGridRemainderCount(itemCount: number, columns: ResponsiveGridColumnCount): number;
export interface ResponsiveGridProps extends Omit<ViewProps, "children" | "class" | "ref"> {
  children?: JSX.Element;
  /** Minimum usable content width for one item, in logical pixels. */
  minColumnWidth: number;
  /** Native row/column gap in logical pixels; also used to select the column count. */
  gap?: number;
  maxColumns?: ResponsiveGridColumnCount;
  /** Safe column count used until the native container has been measured. */
  initialColumns?: ResponsiveGridColumnCount;
  /** Number of rendered cells, used by optional last-row balancing. */
  itemCount?: number;
  /** Reduce the column count when it would leave one orphaned final cell. */
  balanceLastRow?: boolean;
  class?: string;
  ref?: ViewProps["ref"];
}
/**
 * A grid that responds to its own native content box instead of the window.
 *
 * This is important inside sidebars, split panes and dialogs: window media
 * queries do not know how much width the component actually receives.
 */
export declare function ResponsiveGrid(props: ResponsiveGridProps): JSX.Element;
/** Fill the unused cells in the final row using the grid's measured columns. */
export declare function ResponsiveGridRemainder(props: {
  itemCount: number;
  class?: string;
}): JSX.Element;
/**
 * A horizontal primary/aside boundary with explicit flex shrink semantics.
 * Use `SplitPaneMain` for the elastic region and `SplitPaneAside` for a
 * class-sized fixed rail. Both regions clip at their own boundary, so content
 * cannot paint across the divider or a rounded parent clip.
 */
export declare function SplitPane(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function SplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function SplitPaneAside(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
/**
 * Master/detail layout whose detail region can move from an inline rail to a
 * modal surface without changing the application's selection model.
 */
export declare function AdaptiveSplitPane(props: {
  children?: JSX.Element;
  /** Controlled compact mode. Omit it to measure this pane natively. */
  compact?: boolean;
  /** Inclusive native content width that activates compact mode. */
  compactAt?: number;
  onCompactChange?: (compact: boolean) => void;
  "aria-label"?: string;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function AdaptiveSplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
export declare function AdaptiveSplitPaneDetail(props: {
  children?: JSX.Element;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  "aria-label": string;
  class?: string;
  modalClass?: string;
}): import("@wabou/core/jsx-runtime").JSX.Element;
//#endregion
//# sourceMappingURL=layout.d.mts.map