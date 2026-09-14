import {
  type ColumnDef,
  constructTable,
  columnFilteringFeature,
  columnVisibilityFeature,
  globalFilteringFeature,
  rowSelectionFeature,
  rowSortingFeature,
  filterFns,
  sortFns,
  tableFeatures,
  functionalUpdate,
  createCoreRowModel,
  createFilteredRowModel,
  createSortedRowModel,
  type Row,
  type RowSelectionState,
  type SortingState,
  type Table,
} from "@tanstack/table-core";
import { storeReactivityBindings } from "@tanstack/table-core/store-reactivity-bindings";
import {
  type Accessor,
  createMemo,
  createSignal,
  type Setter,
  untrack,
} from "solid-js";

const dataTableFeatures = tableFeatures({
  coreReactivityFeature: storeReactivityBindings(),
  columnFilteringFeature,
  columnVisibilityFeature,
  globalFilteringFeature,
  rowSelectionFeature,
  rowSortingFeature,
  coreRowModel: createCoreRowModel(),
  filteredRowModel: createFilteredRowModel(),
  sortedRowModel: createSortedRowModel(),
  filterFns,
  sortFns,
});

type DataTableFeatures = typeof dataTableFeatures;
export type TanStackDataTableRow<TData extends object> = Row<
  DataTableFeatures,
  TData
>;

export interface TanStackDataTableOptions<TData extends object> {
  /** Static data or a reactive accessor. */
  data: readonly TData[] | Accessor<readonly TData[]>;
  columns: readonly ColumnDef<DataTableFeatures, TData, unknown>[];
  getRowId?: (
    row: TData,
    index: number,
    parent?: TanStackDataTableRow<TData>,
  ) => string;
  enableRowSelection?:
    | boolean
    | ((row: TanStackDataTableRow<TData>) => boolean);
  initialSorting?: SortingState;
  initialGlobalFilter?: string;
  initialRowSelection?: RowSelectionState;
}

/** Column definition re-exported so ordinary consumers only import `@wabou/ui`. */
export type TanStackDataTableColumn<
  TData extends object,
  TValue = unknown,
> = ColumnDef<DataTableFeatures, TData, TValue>;

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

function access<T>(value: T | Accessor<T>): T {
  return typeof value === "function" ? (value as Accessor<T>)() : value;
}

/**
 * Solid's reactive ownership around TanStack Table's DOM-independent core.
 *
 * Wabou deliberately owns no duplicate sorting, filtering, or selection state
 * machine here. Applications retain the native renderer and component layer,
 * while TanStack owns the mature data model.
 */
export function createTanStackDataTable<TData extends object>(
  options: TanStackDataTableOptions<TData>,
): TanStackDataTable<TData> {
  const [sorting, setSorting] = createSignal<SortingState>(
    options.initialSorting ?? [],
  );
  const [globalFilter, setGlobalFilter] = createSignal(
    options.initialGlobalFilter ?? "",
  );
  const [rowSelection, setRowSelection] = createSignal<RowSelectionState>(
    options.initialRowSelection ?? {},
  );
  const table = constructTable<DataTableFeatures, TData>({
    features: dataTableFeatures,
    // TanStack needs an initial value before the reactive row memo is created.
    // Reading an accessor here would escape Solid's tracking scope in strict
    // mode; the memo below performs every reactive synchronization.
    data: [...untrack(() => access(options.data))],
    columns: [...options.columns],
    state: {},
    renderFallbackValue: "—",
    getRowId: options.getRowId,
    enableRowSelection: options.enableRowSelection,
    onSortingChange: (updater) =>
      setSorting((value) => functionalUpdate(updater, value)),
    onGlobalFilterChange: (updater) =>
      setGlobalFilter((value) => functionalUpdate(updater, value)),
    onRowSelectionChange: (updater) =>
      setRowSelection((value) => functionalUpdate(updater, value)),
  });
  const rows = createMemo(() => {
    table.setOptions((current) => ({
      ...current,
      data: [...access(options.data)],
      columns: [...options.columns],
      state: {
        ...table.initialState,
        sorting: sorting(),
        globalFilter: globalFilter(),
        rowSelection: rowSelection(),
      },
    }));
    return table.getRowModel().rows;
  });
  const selectedCount = createMemo(
    () => Object.values(rowSelection()).filter(Boolean).length,
  );

  return {
    table,
    rows,
    sorting,
    setSorting,
    globalFilter,
    setGlobalFilter,
    rowSelection,
    setRowSelection,
    selectedCount,
  };
}
