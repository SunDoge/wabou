import { createMemo, createSignal, untrack } from "solid-js";
import { columnFilteringFeature, columnVisibilityFeature, constructTable, createCoreRowModel, createFilteredRowModel, createSortedRowModel, filterFns, functionalUpdate, globalFilteringFeature, rowSelectionFeature, rowSortingFeature, sortFns, tableFeatures } from "@tanstack/table-core";
import { storeReactivityBindings } from "@tanstack/table-core/store-reactivity-bindings";
//#region src/integrations/tanstack-table.ts
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
	sortFns
});
function access(value) {
	return typeof value === "function" ? value() : value;
}
/**
* Solid's reactive ownership around TanStack Table's DOM-independent core.
*
* Wabou deliberately owns no duplicate sorting, filtering, or selection state
* machine here. Applications retain the native renderer and component layer,
* while TanStack owns the mature data model.
*/
function createTanStackDataTable(options) {
	const [sorting, setSorting] = createSignal(options.initialSorting ?? []);
	const [globalFilter, setGlobalFilter] = createSignal(options.initialGlobalFilter ?? "");
	const [rowSelection, setRowSelection] = createSignal(options.initialRowSelection ?? {});
	const table = constructTable({
		features: dataTableFeatures,
		data: [...untrack(() => access(options.data))],
		columns: [...options.columns],
		state: {},
		renderFallbackValue: "—",
		getRowId: options.getRowId,
		enableRowSelection: options.enableRowSelection,
		onSortingChange: (updater) => setSorting((value) => functionalUpdate(updater, value)),
		onGlobalFilterChange: (updater) => setGlobalFilter((value) => functionalUpdate(updater, value)),
		onRowSelectionChange: (updater) => setRowSelection((value) => functionalUpdate(updater, value))
	});
	return {
		table,
		rows: createMemo(() => {
			table.setOptions((current) => ({
				...current,
				data: [...access(options.data)],
				columns: [...options.columns],
				state: {
					...table.initialState,
					sorting: sorting(),
					globalFilter: globalFilter(),
					rowSelection: rowSelection()
				}
			}));
			return table.getRowModel().rows;
		}),
		sorting,
		setSorting,
		globalFilter,
		setGlobalFilter,
		rowSelection,
		setRowSelection,
		selectedCount: createMemo(() => Object.values(rowSelection()).filter(Boolean).length)
	};
}
//#endregion
export { createTanStackDataTable };

//# sourceMappingURL=tanstack-table.mjs.map