import {
  Badge,
  Checkbox,
  ContentState,
  createContainerMatch,
  createTanStackDataTable,
  Icon,
  Select,
  Table,
  TableCell,
  TableHeader,
  TableRow,
  type TanStackDataTableColumn,
  Text,
  View,
  VirtualList,
} from "@wabou/ui";
import file from "lucide-static/icons/file.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  For as ForValue,
  onCleanup,
  Show,
} from "solid-js";
import type {
  SnapshotDiff,
  SnapshotDiffChange,
  SnapshotDiffEntry,
  SnapshotEntry,
} from "./api";
import { useRusticApi } from "./api";
import { createAsyncRequestGate } from "./async-request";
import {
  formatBytes,
  formatOptionalTimestamp,
  formatTimestamp,
} from "./format";
import { SortableTableHead } from "./sortable-table-head";

const changePresentation: Record<
  SnapshotDiffChange,
  {
    label: string;
    variant: "success" | "destructive" | "secondary" | "outline";
  }
> = {
  added: { label: "Added", variant: "success" },
  removed: { label: "Removed", variant: "destructive" },
  modified: { label: "Modified", variant: "secondary" },
  metadata: { label: "Metadata", variant: "outline" },
  typeChanged: { label: "Type changed", variant: "outline" },
};

const diffColumns: TanStackDataTableColumn<SnapshotDiffEntry>[] = [
  { id: "path", accessorKey: "path", header: "Path" },
  { id: "change", accessorKey: "change", header: "Change" },
  { id: "previousSize", accessorKey: "previousSize", header: "Before" },
  { id: "currentSize", accessorKey: "currentSize", header: "After" },
  {
    id: "modified",
    header: "Modified",
    accessorFn: (entry) =>
      entry.currentModified ?? entry.previousModified ?? "",
  },
];

const DIFF_ENTRY_LIMIT = 250;

export function snapshotComparisonLabel(snapshot: SnapshotEntry): string {
  const title = snapshot.label.trim() || `Snapshot ${snapshot.id.slice(0, 8)}`;
  return `${title} · ${formatTimestamp(snapshot.time)}`;
}

function compactEntryDetails(entry: SnapshotDiffEntry): string {
  let sizes: string | undefined;
  if (entry.previousSize !== undefined && entry.currentSize !== undefined) {
    sizes = `${formatBytes(entry.previousSize)} → ${formatBytes(entry.currentSize)}`;
  } else if (entry.currentSize !== undefined) {
    sizes = formatBytes(entry.currentSize);
  } else if (entry.previousSize !== undefined) {
    sizes = formatBytes(entry.previousSize);
  }
  const modified = formatOptionalTimestamp(
    entry.currentModified ?? entry.previousModified,
  );
  return [sizes, modified === "—" ? undefined : modified]
    .filter(Boolean)
    .join(" · ");
}

export function SnapshotDiffPanel(props: {
  profileId: string;
  snapshot: SnapshotEntry;
  snapshots: readonly SnapshotEntry[];
}) {
  const api = useRusticApi();
  const compactTable = createContainerMatch({ maxWidth: 600 });
  const candidates = createMemo(() =>
    props.snapshots.filter((snapshot) => snapshot.id !== props.snapshot.id),
  );
  const preferredBase = () => {
    const recordedParent = candidates().find(
      (snapshot) => snapshot.id === props.snapshot.parentId,
    );
    return (
      recordedParent?.id ??
      props.snapshots[
        props.snapshots.findIndex((item) => item.id === props.snapshot.id) + 1
      ]?.id ??
      candidates()[0]?.id
    );
  };
  const [baseSnapshotId, setBaseSnapshotId] = createSignal<string>();
  const [includeMetadata, setIncludeMetadata] = createSignal(false);
  const [result, setResult] = createSignal<SnapshotDiff>();
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const [retryRevision, setRetryRevision] = createSignal(0);
  const requests = createAsyncRequestGate();
  onCleanup(() => requests.invalidate());

  createEffect(
    () => ({
      key: `${props.profileId}\u0000${props.snapshot.id}`,
      preferredBase: preferredBase(),
    }),
    ({ preferredBase }) => {
      setBaseSnapshotId(preferredBase);
    },
  );

  createEffect(
    () => ({
      profileId: props.profileId,
      snapshotId: props.snapshot.id,
      baseSnapshotId: baseSnapshotId(),
      includeMetadata: includeMetadata(),
      retryRevision: retryRevision(),
    }),
    (request) => {
      const requestToken = requests.begin();
      if (!request.baseSnapshotId) {
        setResult(undefined);
        setLoading(false);
        setError(undefined);
        return;
      }
      setLoading(true);
      setError(undefined);
      void Promise.resolve(
        api.diffSnapshots({
          profileId: request.profileId,
          snapshotId: request.snapshotId,
          baseSnapshotId: request.baseSnapshotId,
          includeMetadata: request.includeMetadata,
          path: "",
          limit: DIFF_ENTRY_LIMIT,
        }),
      )
        .then((next) => {
          if (requests.isCurrent(requestToken)) setResult(next);
        })
        .catch((cause: unknown) => {
          if (requests.isCurrent(requestToken)) {
            setError(cause instanceof Error ? cause.message : String(cause));
          }
        })
        .finally(() => {
          if (requests.isCurrent(requestToken)) setLoading(false);
        });
    },
  );

  const baseSnapshot = () =>
    props.snapshots.find((snapshot) => snapshot.id === baseSnapshotId());
  const totalChanges = () => result()?.totalEntries ?? 0;
  const renderedChanges = () => result()?.entries.length ?? 0;
  const diffTable = createTanStackDataTable<SnapshotDiffEntry>({
    data: () => result()?.entries ?? [],
    columns: diffColumns,
    getRowId: (entry) => entry.path,
    initialSorting: [{ id: "path", desc: false }],
  });
  const sortDirection = (columnId: string) => {
    const sorting = diffTable.sorting().find(({ id }) => id === columnId);
    return sorting ? (sorting.desc ? "desc" : "asc") : undefined;
  };

  const columnClass = (columnId: string) =>
    columnId === "path"
      ? "min-w-0 flex-1"
      : columnId === "modified"
        ? "min-w-0 w-36 flex-none"
        : "min-w-0 w-28 flex-none";
  const visibleColumns = () =>
    compactTable.matches() ? diffColumns.slice(0, 2) : diffColumns;

  return (
    <View
      ref={compactTable.ref}
      class="w-full h-full min-w-0 min-h-0 flex flex-col"
    >
      <View class="flex-none px-4 py-3 flex flex-row flex-wrap items-center gap-3 border-b border-subtle bg-surface-muted">
        <View class="min-w-40 flex-1 flex flex-col gap-0.5">
          <Text class="text-sm font-medium">Compare with</Text>
          <Text class="truncate text-xs text-muted">
            Current: {formatTimestamp(props.snapshot.time)}
          </Text>
        </View>
        <Select
          aria-label="Comparison snapshot"
          class="min-w-48 w-64 flex-none"
          contentClass="w-72"
          value={baseSnapshotId()}
          placeholder="Choose a snapshot"
          options={candidates().map((snapshot) => ({
            value: snapshot.id,
            label: snapshotComparisonLabel(snapshot),
          }))}
          onValueChange={setBaseSnapshotId}
        />
        <Checkbox
          size="sm"
          label="Metadata changes"
          checked={includeMetadata()}
          onCheckedChange={setIncludeMetadata}
        />
      </View>
      <Show
        when={candidates().length > 0}
        fallback={
          <ContentState
            state="empty"
            title="Create another snapshot to compare"
            description="Diff becomes available after this backup has at least two snapshots."
            class="min-h-0 flex-1 border-0 shadow-none"
          />
        }
      >
        <Show
          when={!loading()}
          fallback={
            <ContentState
              state="loading"
              title="Comparing snapshots"
              description="Reading both directory trees and matching changed paths…"
              class="min-h-0 flex-1 border-0 shadow-none"
            />
          }
        >
          <Show
            when={!error()}
            fallback={
              <ContentState
                state="error"
                title="Could not compare snapshots"
                description={error()}
                action={{
                  label: "Retry comparison",
                  onAction: () => setRetryRevision((current) => current + 1),
                }}
                class="min-h-0 flex-1 border-0 shadow-none"
              />
            }
          >
            <View class="flex-none px-4 py-2.5 flex flex-row flex-wrap items-center gap-2 border-b border-subtle">
              <Badge variant="success" weight="normal">
                +{result()?.summary.added ?? 0}
              </Badge>
              <Badge variant="destructive" weight="normal">
                −{result()?.summary.removed ?? 0}
              </Badge>
              <Badge variant="secondary" weight="normal">
                {result()?.summary.modified ?? 0} modified
              </Badge>
              <Show when={includeMetadata()}>
                <Badge variant="outline" weight="normal">
                  {result()?.summary.metadata ?? 0} metadata
                </Badge>
              </Show>
              <Text class="min-w-48 flex-1 text-right text-xs text-muted">
                {result()?.truncated
                  ? `Showing first ${renderedChanges()} changes`
                  : baseSnapshot()
                    ? `Since ${formatTimestamp(baseSnapshot()?.time ?? "")}`
                    : `${totalChanges()} changes`}
              </Text>
            </View>
            <Show
              when={totalChanges() > 0}
              fallback={
                <ContentState
                  state="empty"
                  title="No changes"
                  description="These snapshots contain the same files for the selected comparison."
                  class="min-h-0 flex-1 border-0 shadow-none"
                />
              }
            >
              <View class="w-full h-full min-w-0 min-h-0 flex flex-col">
                <Table
                  aria-label="Snapshot changes"
                  class="min-h-0 flex-1"
                  contentClass={
                    compactTable.matches()
                      ? "h-full min-h-0"
                      : "h-full min-h-0 min-w-[46rem]"
                  }
                >
                  <TableHeader>
                    <TableRow class="bg-surface-muted">
                      <ForValue each={visibleColumns()}>
                        {(column) => {
                          const id = String(column.id);
                          return (
                            <SortableTableHead
                              label={String(column.header)}
                              class={columnClass(id)}
                              direction={() => sortDirection(id)}
                              onToggle={() =>
                                diffTable.table.getColumn(id)?.toggleSorting()
                              }
                            />
                          );
                        }}
                      </ForValue>
                    </TableRow>
                  </TableHeader>
                  <VirtualList
                    items={diffTable.rows}
                    itemHeight={64}
                    getItemKey={(row) => row.id}
                    role="group"
                    accessibilityLabel="Snapshot change rows"
                    class="min-h-0 flex-1"
                  >
                    {(row) => {
                      const entry = () => row().original;
                      const presentation = () =>
                        changePresentation[entry().change];
                      return (
                        <TableRow aria-label={entry().path} class="h-16">
                          <TableCell class="min-w-0 flex-1 gap-2">
                            <Icon
                              source={
                                entry().kind === "directory" ? folder : file
                              }
                              size={15}
                              class="flex-none text-muted"
                            />
                            <View class="min-w-0 flex-1 flex flex-col gap-0.5">
                              <Text class="w-full truncate">
                                {entry().name}
                              </Text>
                              <Text class="w-full truncate text-xs text-muted">
                                {entry().path}
                              </Text>
                              <Show when={compactTable.matches()}>
                                <Text class="w-full truncate text-xs text-muted">
                                  {compactEntryDetails(entry())}
                                </Text>
                              </Show>
                            </View>
                          </TableCell>
                          <TableCell class="min-w-0 w-28 flex-none">
                            <Badge
                              variant={presentation().variant}
                              weight="normal"
                            >
                              {presentation().label}
                            </Badge>
                          </TableCell>
                          <Show when={!compactTable.matches()}>
                            <TableCell class="min-w-0 w-28 flex-none text-muted">
                              {formatBytes(entry().previousSize)}
                            </TableCell>
                            <TableCell class="min-w-0 w-28 flex-none text-muted">
                              {formatBytes(entry().currentSize)}
                            </TableCell>
                            <TableCell class="min-w-0 w-36 flex-none text-muted">
                              {formatOptionalTimestamp(
                                entry().currentModified ??
                                  entry().previousModified,
                              )}
                            </TableCell>
                          </Show>
                        </TableRow>
                      );
                    }}
                  </VirtualList>
                </Table>
              </View>
            </Show>
          </Show>
        </Show>
      </Show>
    </View>
  );
}
