import { Accessor, Setter } from "solid-js";
import { ColumnDef, Row, RowSelectionState, SortingState, Table } from "@tanstack/table-core";
//#region src/integrations/tanstack-table.d.ts
declare const dataTableFeatures: {
  coreReactivityFeature: import("@tanstack/table-core/reactivity").TableReactivityBindings;
  columnFilteringFeature: import("@tanstack/table-core").TableFeature;
  columnVisibilityFeature: import("@tanstack/table-core").TableFeature;
  globalFilteringFeature: import("@tanstack/table-core").TableFeature;
  rowSelectionFeature: import("@tanstack/table-core").TableFeature;
  rowSortingFeature: import("@tanstack/table-core").TableFeature;
  coreRowModel: (table: Table<any, any>) => () => import("@tanstack/table-core").RowModel<any, any>;
  filteredRowModel: (table: Table<any, any>) => () => import("@tanstack/table-core").RowModel<any, any>;
  sortedRowModel: (table: Table<any, any>) => () => import("@tanstack/table-core").RowModel<any, any>;
  filterFns: {
    arrIncludes: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    arrIncludesAll: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    arrHas: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    arrIncludesSome: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    between: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    betweenInclusive: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    empty: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    endsWith: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    equals: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    equalsString: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    equalsStringSensitive: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    inDateRange: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    inNumberRange: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    includesString: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    includesStringSensitive: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    notEmpty: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    startsWith: import("@tanstack/table-core").CreatedFilterFn<any, any>;
    weakEquals: import("@tanstack/table-core").CreatedFilterFn<any, any>;
  };
  sortFns: {
    alphanumeric: import("@tanstack/table-core").CreatedSortFn<any, any>;
    alphanumericCaseSensitive: import("@tanstack/table-core").CreatedSortFn<any, any>;
    basic: import("@tanstack/table-core").CreatedSortFn<any, any>;
    datetime: import("@tanstack/table-core").CreatedSortFn<any, any>;
    text: import("@tanstack/table-core").CreatedSortFn<any, any>;
    textCaseSensitive: import("@tanstack/table-core").CreatedSortFn<any, any>;
  };
};
type DataTableFeatures = typeof dataTableFeatures;
export type TanStackDataTableRow<TData extends object> = Row<DataTableFeatures, TData>;
export interface TanStackDataTableOptions<TData extends object> {
  /** Static data or a reactive accessor. */
  data: readonly TData[] | Accessor<readonly TData[]>;
  columns: readonly ColumnDef<DataTableFeatures, TData, unknown>[];
  getRowId?: (row: TData, index: number, parent?: TanStackDataTableRow<TData>) => string;
  enableRowSelection?: boolean | ((row: TanStackDataTableRow<TData>) => boolean);
  initialSorting?: SortingState;
  initialGlobalFilter?: string;
  initialRowSelection?: RowSelectionState;
}
/** Column definition re-exported so ordinary consumers only import `@wabou/ui`. */
export type TanStackDataTableColumn<TData extends object, TValue = unknown> = ColumnDef<DataTableFeatures, TData, TValue>;
export interface TanStackDataTable<TData extends object> {
  /** TanStack v9 instance with filtering, sorting, visibility, and selection. */
  readonly table: Table<DataTableFeatures, TData>;
  /** Reactive rows after filtering and sorting. */
  readonly rows: Accessor<readonly TanStackDataTableRow<TData>[]>;
  readonly sorting: Accessor<SortingState>;
  readonly setSorting: Setter<SortingState>;
  readonly globalFilter: Accessor<string>;
  readonly setGlobalFilter: Setter<string>;
  readonly rowSelection: Accessor<RowSelectionState>;
  readonly setRowSelection: Setter<RowSelectionState>;
  readonly selectedCount: Accessor<number>;
}
/**
 * Solid's reactive ownership around TanStack Table's DOM-independent core.
 *
 * Wabou deliberately owns no duplicate sorting, filtering, or selection state
 * machine here. Applications retain the native renderer and component layer,
 * while TanStack owns the mature data model.
 */
export declare function createTanStackDataTable<TData extends object>(options: TanStackDataTableOptions<TData>): TanStackDataTable<TData>;
//#endregion
export {};
//# sourceMappingURL=tanstack-table.d.mts.map