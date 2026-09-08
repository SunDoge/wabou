import {
  Button,
  ContextMenu,
  createTanStackDataTable,
  Icon,
  Table,
  TableCell,
  TableHeader,
  TableRow,
  Text,
  View,
  VirtualList,
  type VirtualListController,
} from "@wabou/ui";
import file from "lucide-static/icons/file.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import { onCleanup, Show } from "solid-js";
import type { FileEntry } from "./api";
import { formatBytes, formatOptionalTimestamp } from "./format";
import { SortableTableHead } from "./sortable-table-head";

const fileColumns = [
  { accessorKey: "name", header: "Name" },
  { accessorKey: "size", header: "Size" },
  { accessorKey: "modified", header: "Modified" },
] as const;

export function SnapshotFileRow(props: {
  entry: FileEntry;
  selected: boolean;
  searchActive: boolean;
  onSelect: (entry: FileEntry) => void;
  onOpenDirectory: (entry: FileEntry) => void;
}) {
  const entry = () => props.entry;
  let pendingSingleClick: ReturnType<typeof setTimeout> | undefined;
  const cancelPendingSingleClick = () => {
    if (pendingSingleClick === undefined) return;
    clearTimeout(pendingSingleClick);
    pendingSingleClick = undefined;
  };
  const select = () => {
    if (entry().kind !== "directory") {
      props.onSelect(entry());
      return;
    }
    cancelPendingSingleClick();
    pendingSingleClick = setTimeout(() => {
      pendingSingleClick = undefined;
      props.onSelect(entry());
    }, 410);
  };
  onCleanup(cancelPendingSingleClick);
  return (
    <ContextMenu
      aria-label={`${entry().name} actions`}
      items={
        entry().kind === "directory"
          ? [
              { id: "open", label: "Open folder" },
              { id: "details", label: "Show details" },
            ]
          : [{ id: "details", label: "Show details" }]
      }
      onAction={(action) => {
        if (action === "open") props.onOpenDirectory(entry());
        if (action === "details") props.onSelect(entry());
      }}
      trigger={(contextMenu) => (
        <TableRow
          ref={contextMenu.ref}
          id={entry().path}
          aria-label={entry().name}
          aria-haspopup={contextMenu["aria-haspopup"]}
          aria-expanded={contextMenu["aria-expanded"]}
          selected={props.selected}
          class="cursor-pointer"
          onClick={select}
          onContextMenu={(event) => {
            cancelPendingSingleClick();
            props.onSelect(entry());
            contextMenu.onContextMenu(event);
          }}
          onKeyDown={contextMenu.onKeyDown}
          onDblClick={() => {
            if (entry().kind !== "directory") return;
            cancelPendingSingleClick();
            props.onOpenDirectory(entry());
          }}
        >
          <TableCell class="min-w-0 flex-1 gap-2">
            <Icon
              source={entry().kind === "directory" ? folder : file}
              size={15}
              class="flex-none text-muted"
            />
            <View class="min-w-0 flex-1 flex flex-col gap-0.5">
              <Text class="w-full truncate">{entry().name}</Text>
              <Show when={props.searchActive}>
                <Text class="w-full truncate text-xs text-muted">
                  {entry().path}
                </Text>
              </Show>
            </View>
          </TableCell>
          <TableCell class="min-w-0 w-24 flex-none text-muted">
            {entry().kind === "directory" ? "—" : formatBytes(entry().size)}
          </TableCell>
          <TableCell class="min-w-0 w-36 flex-none text-muted">
            {formatOptionalTimestamp(entry().modified)}
          </TableCell>
        </TableRow>
      )}
    />
  );
}

export function SnapshotFileList(props: {
  entries: readonly FileEntry[];
  selectedPath?: string;
  searchActive: boolean;
  total: number;
  loadingMore: boolean;
  loadMoreError?: string;
  canOpenParent: boolean;
  onSelect(entry: FileEntry): void;
  onOpenDirectory(entry: FileEntry): void;
  onOpenParent(): void;
  onLoadMore(): void;
}) {
  const table = createTanStackDataTable<FileEntry>({
    data: () => props.entries,
    columns: fileColumns,
    getRowId: (entry) => entry.path,
    initialSorting: [{ id: "name", desc: false }],
  });
  let listController: VirtualListController | undefined;
  const rows = table.rows;
  const selectedIndex = () =>
    rows().findIndex((row) => row.original.path === props.selectedPath);
  const activeDescendant = () =>
    selectedIndex() >= 0 ? props.selectedPath : undefined;
  const selectAt = (index: number) => {
    const entry = rows()[index]?.original;
    if (!entry) return false;
    props.onSelect(entry);
    listController?.scrollToIndex(index);
    return true;
  };
  const handleKey = (event: { key: string; preventDefault(): void }) => {
    const current = selectedIndex();
    const last = rows().length - 1;
    const next =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? last
          : event.key === "ArrowDown"
            ? Math.min(last, Math.max(0, current + 1))
            : event.key === "ArrowUp"
              ? Math.max(0, current < 0 ? last : current - 1)
              : undefined;
    if (next !== undefined && selectAt(next)) {
      event.preventDefault();
      return;
    }
    if (event.key === "Enter") {
      const entry = rows()[current]?.original;
      if (entry?.kind === "directory") {
        event.preventDefault();
        props.onOpenDirectory(entry);
      }
      return;
    }
    if (event.key === "Backspace" && props.canOpenParent) {
      event.preventDefault();
      props.onOpenParent();
    }
  };
  const sortDirection = (columnId: string) => {
    const sorting = table.sorting().find(({ id }) => id === columnId);
    return sorting ? (sorting.desc ? "desc" : "asc") : undefined;
  };

  return (
    <View class="w-full h-full min-w-0 min-h-0 flex flex-col">
      <Table
        aria-label="Snapshot files"
        class="min-h-0 flex-1"
        contentClass="h-full min-h-0"
      >
        <TableHeader>
          <TableRow class="bg-surface-muted">
            <SortableTableHead
              label="Name"
              class="min-w-0 flex-1"
              direction={() => sortDirection("name")}
              onToggle={() => table.table.getColumn("name")?.toggleSorting()}
            />
            <SortableTableHead
              label="Size"
              class="min-w-0 w-24 flex-none"
              direction={() => sortDirection("size")}
              onToggle={() => table.table.getColumn("size")?.toggleSorting()}
            />
            <SortableTableHead
              label="Modified"
              class="min-w-0 w-36 flex-none"
              direction={() => sortDirection("modified")}
              onToggle={() =>
                table.table.getColumn("modified")?.toggleSorting()
              }
            />
          </TableRow>
        </TableHeader>
        <VirtualList
          items={rows}
          itemHeight={44}
          getItemKey={(row) => row.id}
          role="group"
          accessibilityLabel="Snapshot file rows"
          focusOrder={0}
          aria-activedescendant={activeDescendant()}
          onKeyDown={handleKey}
          controllerRef={(controller) => {
            listController = controller;
          }}
          class="min-h-0 flex-1"
          onVisibleRangeChange={({ end }) => {
            if (
              !props.searchActive &&
              end >= props.entries.length - 4 &&
              props.entries.length < props.total &&
              !props.loadMoreError
            ) {
              props.onLoadMore();
            }
          }}
        >
          {(row) => (
            <SnapshotFileRow
              entry={row().original}
              selected={props.selectedPath === row().original.path}
              searchActive={props.searchActive}
              onSelect={props.onSelect}
              onOpenDirectory={props.onOpenDirectory}
            />
          )}
        </VirtualList>
      </Table>
      <Show when={props.loadingMore}>
        <View
          role="status"
          aria-label="Loading more files"
          class="w-full flex-none flex justify-center px-4 py-2"
        >
          <Text class="text-xs text-muted">Loading more files…</Text>
        </View>
      </Show>
      <Show when={props.loadMoreError}>
        {(message) => (
          <View class="w-full flex-none flex flex-row items-center justify-between gap-3 border-t border-subtle px-4 py-2">
            <Text
              role="alert"
              class="min-w-0 flex-1 truncate text-xs text-danger-primary"
            >
              {message()}
            </Text>
            <Button
              size="sm"
              variant="outline"
              aria-label="Retry loading more files"
              onClick={props.onLoadMore}
            >
              Retry
            </Button>
          </View>
        )}
      </Show>
    </View>
  );
}
