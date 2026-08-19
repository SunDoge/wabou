import {
  type ColumnDef,
  createTable,
  functionalUpdate,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  type RowSelectionState,
  type SortingState,
} from "@tanstack/table-core";
import { Badge, Button, Input, PrimitiveButton, Text, View } from "@wabou/ui";
import { createMemo, createSignal, For, Show } from "solid-js";

interface Project {
  id: string;
  name: string;
  owner: string;
  status: "Active" | "Review" | "Paused";
  score: number;
}

const data: Project[] = [
  { id: "router", name: "Router", owner: "Mina", status: "Active", score: 98 },
  {
    id: "cache",
    name: "Resource cache",
    owner: "Arun",
    status: "Review",
    score: 94,
  },
  {
    id: "table",
    name: "Data table",
    owner: "Iris",
    status: "Active",
    score: 91,
  },
  {
    id: "virtual",
    name: "Virtual lists",
    owner: "Noah",
    status: "Active",
    score: 88,
  },
  {
    id: "a11y",
    name: "Accessibility",
    owner: "Mina",
    status: "Review",
    score: 84,
  },
  {
    id: "editor",
    name: "Editor widget",
    owner: "Arun",
    status: "Paused",
    score: 76,
  },
];

const columns: ColumnDef<Project>[] = [
  { accessorKey: "name", header: "Project" },
  { accessorKey: "owner", header: "Owner" },
  { accessorKey: "status", header: "Status" },
  { accessorKey: "score", header: "Score" },
];

function statusVariant(status: Project["status"]) {
  if (status === "Active") return "success" as const;
  if (status === "Paused") return "destructive" as const;
  return "secondary" as const;
}

export function DataTablePage() {
  const [sorting, setSorting] = createSignal<SortingState>([]);
  const [globalFilter, setGlobalFilter] = createSignal("");
  const [rowSelection, setRowSelection] = createSignal<RowSelectionState>({});

  const table = createTable<Project>({
    data,
    columns,
    state: {},
    onStateChange: () => {},
    renderFallbackValue: "—",
    getRowId: (row) => row.id,
    enableRowSelection: true,
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

  const model = createMemo(() => {
    table.setOptions((options) => ({
      ...options,
      state: {
        ...table.initialState,
        sorting: sorting(),
        globalFilter: globalFilter(),
        rowSelection: rowSelection(),
      },
    }));
    return table.getRowModel();
  });

  const selectedCount = () =>
    Object.values(rowSelection()).filter(Boolean).length;

  return (
    <View class="flex flex-col gap-5">
      <View class="rounded-xl border border-subtle bg-surface overflow-hidden">
        <View class="p-4 flex items-center gap-3 border-b border-subtle">
          <Input
            aria-label="Filter projects"
            class="w-72"
            placeholder="Filter projects…"
            value={globalFilter()}
            onInput={(event) => setGlobalFilter(event.currentTarget.value)}
          />
          <Text
            role="status"
            aria-label="Visible project count"
            class="text-sm text-muted"
          >
            {model().rows.length} visible
          </Text>
          <Show when={selectedCount() > 0}>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => setRowSelection({})}
            >
              Clear {selectedCount()} selected
            </Button>
          </Show>
        </View>

        <View role="table" aria-label="Project table" class="w-full">
          <For each={table.getHeaderGroups()}>
            {(headerGroup) => (
              <View
                role="row"
                class="h-11 flex border-b border-strong bg-control"
              >
                <For each={headerGroup.headers}>
                  {(header) => {
                    const sortable = () => header.column.getCanSort();
                    const direction = () => header.column.getIsSorted();
                    return (
                      <PrimitiveButton
                        unstyled
                        role="columnheader"
                        aria-label={`Sort by ${String(header.column.columnDef.header)}`}
                        class="flex-1 min-w-0 px-4 justify-start text-xs font-semibold text-secondary"
                        disabled={!sortable()}
                        onClick={() => header.column.toggleSorting()}
                      >
                        {String(header.column.columnDef.header)}
                        <Text class="ml-auto text-xs text-muted">
                          {direction() === "asc"
                            ? "Asc"
                            : direction() === "desc"
                              ? "Desc"
                              : ""}
                        </Text>
                      </PrimitiveButton>
                    );
                  }}
                </For>
              </View>
            )}
          </For>

          <For each={model().rows}>
            {(row) => (
              <PrimitiveButton
                unstyled
                role="row"
                aria-label={`Select ${row.original.name}`}
                aria-selected={row.getIsSelected()}
                selected={row.getIsSelected()}
                class={
                  row.getIsSelected()
                    ? "h-12 w-full flex border-b border-subtle bg-selected"
                    : "h-12 w-full flex border-b border-subtle bg-surface"
                }
                onClick={() => row.toggleSelected()}
              >
                <For each={row.getVisibleCells()}>
                  {(cell) => (
                    <View
                      role="cell"
                      class="flex-1 min-w-0 px-4 flex items-center"
                    >
                      <Show
                        when={cell.column.id === "status"}
                        fallback={
                          <Text class="w-full truncate text-sm text-primary">
                            {String(cell.getValue())}
                          </Text>
                        }
                      >
                        <Badge
                          variant={statusVariant(
                            cell.getValue() as Project["status"],
                          )}
                        >
                          {String(cell.getValue())}
                        </Badge>
                      </Show>
                    </View>
                  )}
                </For>
              </PrimitiveButton>
            )}
          </For>

          <Show when={model().rows.length === 0}>
            <View class="h-24 flex items-center justify-center">
              <Text class="text-sm text-muted">No matching projects</Text>
            </View>
          </Show>
        </View>
      </View>

      <View class="rounded-lg border border-subtle bg-surface-muted p-4 flex flex-col gap-2">
        <Text class="text-sm font-semibold text-primary">
          Experiment boundary
        </Text>
        <Text class="whitespace-normal text-sm text-secondary">
          TanStack Table owns row models, sorting, filtering, and selection.
          Wabou owns rendering, semantics, input routing, and styling. This page
          uses table-core directly without a Solid adapter.
        </Text>
      </View>
    </View>
  );
}
