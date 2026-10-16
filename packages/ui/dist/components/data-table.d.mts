import { TanStackDataTable, TanStackDataTableRow } from "../integrations/tanstack-table.mjs";
import "../integrations/index.mjs";
import { JSX } from "solid-js";
//#region src/components/data-table.d.ts
export interface DataTableProps<TData extends object> {
  model: TanStackDataTable<TData>;
  "aria-label": string;
  emptyMessage?: string;
  selectable?: boolean;
  renderCell?: (options: {
    value: unknown;
    columnId: string;
    row: TanStackDataTableRow<TData>;
  }) => JSX.Element;
}
export type TableSortDirection = "asc" | "desc" | false | undefined;
export declare function TableSortIndicator(props: {
  direction: TableSortDirection | (() => TableSortDirection);
}): JSX.Element;
/** Shadcn-style table anatomy backed by the framework-agnostic TanStack core. */
export declare function DataTable<TData extends object>(props: DataTableProps<TData>): JSX.Element;
//#endregion
//# sourceMappingURL=data-table.d.mts.map