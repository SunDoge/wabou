import {
  type ColumnDef,
  createTable,
  functionalUpdate,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  type Row,
  type RowSelectionState,
  type SortingState,
  type Table,
} from "@tanstack/table-core";
import {
  type Accessor,
  createMemo,
  createSignal,
  type Setter,
  untrack,
} from "solid-js";

export interface TanStackDataTableOptions<TData> {
  /** Static data or a reactive accessor. */
  data: readonly TData[] | Accessor<readonly TData[]>;
  columns: readonly ColumnDef<TData, unknown>[];
  getRowId?: (row: TData, index: number, parent?: Row<TData>) => string;
  enableRowSelection?: boolean | ((row: Row<TData>) => boolean);
  initialSorting?: SortingState;
  initialGlobalFilter?: string;
  initialRowSelection?: RowSelectionState;
}

/** Column definition re-exported so ordinary consumers only import `@wabou/ui`. */
export type TanStackDataTableColumn<TData, TValue = unknown> = ColumnDef<
  TData,
  TValue
>;

export interface TanStackDataTable<TData> {
  /** The framework-agnostic TanStack instance for advanced capabilities. */
  readonly table: Table<TData>;
  /** Reactive rows after filtering and sorting. */
  readonly rows: Accessor<readonly Row<TData>[]>;
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
export function createTanStackDataTable<TData>(
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
  const table = createTable<TData>({
    // TanStack needs an initial value before the reactive row memo is created.
    // Reading an accessor here would escape Solid's tracking scope in strict
    // mode; the memo below performs every reactive synchronization.
    data: [...untrack(() => access(options.data))],
    columns: [...options.columns],
    state: {},
    onStateChange: () => {},
    renderFallbackValue: "—",
    getRowId: options.getRowId,
    enableRowSelection: options.enableRowSelection,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getSortedRowModel: getSortedRowModel(),
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
