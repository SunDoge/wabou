import type { Row } from "@tanstack/table-core";
import { For as ForValue, type JSX, Show } from "solid-js";
import type { TanStackDataTable } from "../integrations";
import { Button as PrimitiveButton, Text, View } from "../primitives";
import { Table, TableBody, TableCell, TableHeader, TableRow } from "./table";

export interface DataTableProps<TData> {
  model: TanStackDataTable<TData>;
  "aria-label": string;
  emptyMessage?: string;
  selectable?: boolean;
  renderCell?: (options: {
    value: unknown;
    columnId: string;
    row: Row<TData>;
  }) => JSX.Element;
}

/** Shadcn-style table anatomy backed by the framework-agnostic TanStack core. */
export function DataTable<TData>(props: DataTableProps<TData>): JSX.Element {
  return (
    <Table aria-label={props["aria-label"]}>
      <TableHeader>
        <ForValue each={props.model.table.getHeaderGroups()}>
          {(headerGroup) => (
            <TableRow class="h-11 border-strong bg-control">
              <ForValue each={headerGroup.headers}>
                {(header) => {
                  const sortable = () => header.column.getCanSort();
                  const direction = () =>
                    props.model
                      .sorting()
                      .find(({ id }) => id === header.column.id)?.desc
                      ? "desc"
                      : props.model
                            .sorting()
                            .some(({ id }) => id === header.column.id)
                        ? "asc"
                        : false;
                  const label = () =>
                    String(header.column.columnDef.header ?? header.id);
                  return (
                    <PrimitiveButton
                      unstyled
                      role="columnheader"
                      aria-label={sortable() ? `Sort by ${label()}` : label()}
                      class="flex-1 min-w-0 px-4 justify-start text-xs font-semibold text-secondary"
                      disabled={!sortable()}
                      onClick={() => header.column.toggleSorting()}
                    >
                      {label()}
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
              </ForValue>
            </TableRow>
          )}
        </ForValue>
      </TableHeader>
      <TableBody>
        <ForValue each={props.model.rows()}>
          {(row) => {
            const cells = () => (
              <ForValue each={row.getVisibleCells()}>
                {(cell) => (
                  <TableCell class="px-4">
                    {props.renderCell?.({
                      value: cell.getValue(),
                      columnId: cell.column.id,
                      row,
                    }) ?? (
                      <Text class="w-full truncate text-sm text-primary">
                        {String(cell.getValue() ?? "")}
                      </Text>
                    )}
                  </TableCell>
                )}
              </ForValue>
            );
            return (
              <Show
                when={props.selectable}
                fallback={
                  <TableRow aria-label={`Row ${row.id}`}>{cells()}</TableRow>
                }
              >
                <PrimitiveButton
                  unstyled
                  role="row"
                  aria-label={`Select row ${row.id}`}
                  aria-selected={props.model.rowSelection()[row.id] === true}
                  selected={props.model.rowSelection()[row.id] === true}
                  class={
                    props.model.rowSelection()[row.id] === true
                      ? "h-12 w-full flex border-b border-subtle bg-selected"
                      : "h-12 w-full flex border-b border-subtle bg-surface"
                  }
                  onClick={() => row.toggleSelected()}
                >
                  {cells()}
                </PrimitiveButton>
              </Show>
            );
          }}
        </ForValue>
        <Show when={props.model.rows().length === 0}>
          <View class="h-24 flex items-center justify-center">
            <Text class="text-sm text-muted">
              {props.emptyMessage ?? "No results"}
            </Text>
          </View>
        </Show>
      </TableBody>
    </Table>
  );
}
