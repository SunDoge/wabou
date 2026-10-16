import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/table.d.ts
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
export declare function Table(props: TableProps): JSX.Element;
export declare function TableHeader(props: ViewProps): JSX.Element;
export declare function TableBody(props: ViewProps): JSX.Element;
export declare function TableFooter(props: ViewProps): JSX.Element;
export interface TableRowProps extends Omit<ViewProps, "class"> {
  class?: string;
  selected?: boolean;
}
export declare function TableRow(props: TableRowProps): JSX.Element;
export interface TableHeadProps extends Omit<TextProps, "class"> {
  class?: string;
}
export declare function TableHead(props: TableHeadProps): JSX.Element;
export interface TableCellProps extends Omit<ViewProps, "class"> {
  class?: string;
}
export declare function TableCell(props: TableCellProps): JSX.Element;
export declare function TableCaption(props: TextProps): JSX.Element;
//#endregion
//# sourceMappingURL=table.d.mts.map