import {
  type Accessor,
  createContext,
  createMemo,
  For as ForValue,
  type JSX,
  omit,
  Show,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import { createMeasuredSize, View, type ViewProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";
import { Dialog } from "./dialog";

export type ResponsiveGridColumnCount = 1 | 2 | 3 | 4;

export interface ResponsiveGridState {
  columns: Accessor<ResponsiveGridColumnCount>;
  width: Accessor<number>;
  height: Accessor<number>;
}

const ResponsiveGridContext = createContext<ResponsiveGridState>();

/** Read the completed native size and active column count of the nearest grid. */
export function useResponsiveGrid(): ResponsiveGridState {
  const context = useContext(ResponsiveGridContext);
  if (!context) {
    throw new Error("useResponsiveGrid must be used inside ResponsiveGrid");
  }
  return context;
}

const responsiveGridColumnClass = (
  columns: ResponsiveGridColumnCount,
): string =>
  match(columns)
    .with(1, () => "grid-cols-1")
    .with(2, () => "grid-cols-2")
    .with(3, () => "grid-cols-3")
    .with(4, () => "grid-cols-4")
    .exhaustive();

export function responsiveGridColumnCount(options: {
  width: number;
  minColumnWidth: number;
  gap?: number;
  maxColumns?: ResponsiveGridColumnCount;
  initialColumns?: ResponsiveGridColumnCount;
  itemCount?: number;
  balanceLastRow?: boolean;
}): ResponsiveGridColumnCount {
  const maxColumns = options.maxColumns ?? 4;
  if (!Number.isFinite(options.width) || options.width <= 0) {
    return Math.min(
      options.initialColumns ?? 1,
      maxColumns,
    ) as ResponsiveGridColumnCount;
  }
  const gap = Math.max(0, options.gap ?? 16);
  const minColumnWidth = Math.max(1, options.minColumnWidth);
  const columns = Math.min(
    maxColumns,
    Math.max(1, Math.floor((options.width + gap) / (minColumnWidth + gap))),
  ) as ResponsiveGridColumnCount;
  const itemCount = Math.max(0, Math.floor(options.itemCount ?? 0));
  if (
    options.balanceLastRow &&
    itemCount > 1 &&
    columns > 1 &&
    itemCount % columns === 1
  ) {
    return (columns - 1) as ResponsiveGridColumnCount;
  }
  return columns;
}

export function responsiveGridRemainderCount(
  itemCount: number,
  columns: ResponsiveGridColumnCount,
): number {
  const remainder = Math.max(0, Math.floor(itemCount)) % columns;
  return remainder === 0 ? 0 : columns - remainder;
}

export interface ResponsiveGridProps
  extends Omit<ViewProps, "children" | "class" | "ref"> {
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
export function ResponsiveGrid(props: ResponsiveGridProps): JSX.Element {
  const measured = createMeasuredSize();
  const columns = createMemo(() =>
    responsiveGridColumnCount({
      width: measured.width(),
      minColumnWidth: props.minColumnWidth,
      gap: props.gap,
      maxColumns: props.maxColumns,
      initialColumns: props.initialColumns,
      itemCount: props.itemCount,
      balanceLastRow: props.balanceLastRow,
    }),
  );
  const rest = omit(
    props,
    "children",
    "minColumnWidth",
    "gap",
    "maxColumns",
    "initialColumns",
    "itemCount",
    "balanceLastRow",
    "class",
    "ref",
  );
  const state: ResponsiveGridState = {
    columns,
    width: measured.width,
    height: measured.height,
  };
  return (
    <ResponsiveGridContext value={state}>
      <View
        {...rest}
        ref={(node) => {
          measured.ref(node);
          props.ref?.(node);
        }}
        style={{ gap: props.gap ?? 16, ...props.style }}
        class={mergeClasses(
          "w-full min-w-0 grid",
          responsiveGridColumnClass(columns()),
          props.class,
        )}
      >
        {props.children}
      </View>
    </ResponsiveGridContext>
  );
}

/** Fill the unused cells in the final row using the grid's measured columns. */
export function ResponsiveGridRemainder(props: {
  itemCount: number;
  class?: string;
}): JSX.Element {
  const context = useResponsiveGrid();
  const cells = createMemo(() =>
    Array.from({
      length: responsiveGridRemainderCount(props.itemCount, context.columns()),
    }),
  );
  return (
    <ForValue each={cells()}>
      {() => <View aria-hidden class={mergeClasses("min-w-0", props.class)} />}
    </ForValue>
  );
}

/**
 * A horizontal primary/aside boundary with explicit flex shrink semantics.
 * Use `SplitPaneMain` for the elastic region and `SplitPaneAside` for a
 * class-sized fixed rail. Both regions clip at their own boundary, so content
 * cannot paint across the divider or a rounded parent clip.
 */
export function SplitPane(props: { children?: JSX.Element; class?: string }) {
  return (
    <View
      class={mergeClasses(
        "w-full min-w-0 flex flex-row overflow-hidden",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function SplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View class={mergeClasses("flex-1 min-w-0 overflow-hidden", props.class)}>
      {props.children}
    </View>
  );
}

export function SplitPaneAside(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View
      class={mergeClasses("flex-none min-w-0 overflow-hidden", props.class)}
    >
      {props.children}
    </View>
  );
}

interface AdaptiveSplitPaneContextValue {
  compact(): boolean;
}

const AdaptiveSplitPaneContext = createContext<AdaptiveSplitPaneContextValue>();

/**
 * Master/detail layout whose detail region can move from an inline rail to a
 * modal surface without changing the application's selection model.
 */
export function AdaptiveSplitPane(props: {
  children?: JSX.Element;
  compact: boolean;
  class?: string;
}) {
  const context: AdaptiveSplitPaneContextValue = {
    compact: () => props.compact,
  };
  return (
    <AdaptiveSplitPaneContext value={context}>
      <SplitPane class={props.class}>{props.children}</SplitPane>
    </AdaptiveSplitPaneContext>
  );
}

export function AdaptiveSplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return <SplitPaneMain class={props.class}>{props.children}</SplitPaneMain>;
}

export function AdaptiveSplitPaneDetail(props: {
  children?: JSX.Element;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  "aria-label": string;
  class?: string;
  modalClass?: string;
}) {
  const context = useContext(AdaptiveSplitPaneContext);
  return (
    <Show
      when={context.compact()}
      fallback={
        <SplitPaneAside class={props.class}>{props.children}</SplitPaneAside>
      }
    >
      <Dialog
        open={props.open}
        onOpenChange={(open) => props.onOpenChange(open)}
        aria-label={props["aria-label"]}
        contentClass={mergeClasses(
          "h-11/12 p-0 overflow-hidden",
          props.modalClass,
        )}
      >
        {props.children}
      </Dialog>
    </Show>
  );
}
