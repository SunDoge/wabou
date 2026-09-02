import {
  Badge,
  Checkbox,
  ContentState,
  createTanStackDataTable,
  Icon,
  ScrollArea,
  Select,
  Table,
  TableBody,
  TableCell,
  TableHeader,
  TableRow,
  type TanStackDataTableColumn,
  Text,
  View,
} from "@wabou/ui";
import file from "lucide-static/icons/file.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import type {
  SnapshotDiff,
  SnapshotDiffChange,
  SnapshotDiffEntry,
  SnapshotEntry,
} from "./api";
import { useRusticApi } from "./api";
import { formatSnapshotTime } from "./snapshot-details";
import { SortableTableHead } from "./sortable-table-head";

function formatBytes(bytes?: number): string {
  if (bytes === undefined) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
}

function formatModified(value?: string): string {
  if (!value) return "—";
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})/);
  return match
    ? `${match[1]}-${match[2]}-${match[3]} ${match[4]}:${match[5]}`
    : value;
}

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

export function SnapshotDiffPanel(props: {
  profileId: string;
  snapshot: SnapshotEntry;
  snapshots: readonly SnapshotEntry[];
}) {
  const api = useRusticApi();
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
  let requestGeneration = 0;

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
    }),
    (request) => {
      if (!request.baseSnapshotId) {
        setResult(undefined);
        setLoading(false);
        return;
      }
      const generation = ++requestGeneration;
      setLoading(true);
      setError(undefined);
      void Promise.resolve(
        api.diffSnapshots({
          ...request,
          baseSnapshotId: request.baseSnapshotId,
          path: "",
        }),
      )
        .then((next) => {
          if (generation === requestGeneration) setResult(next);
        })
        .catch((cause: unknown) => {
          if (generation === requestGeneration) {
            setError(cause instanceof Error ? cause.message : String(cause));
          }
        })
        .finally(() => {
          if (generation === requestGeneration) setLoading(false);
        });
    },
  );

  const baseSnapshot = () =>
    props.snapshots.find((snapshot) => snapshot.id === baseSnapshotId());
  const totalChanges = () => result()?.entries.length ?? 0;
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
      ? "min-w-64 flex-1"
      : columnId === "modified"
        ? "w-36 flex-none"
        : "w-28 flex-none";

  return (
    <View class="min-w-0 min-h-0 flex-1 flex flex-col">
      <View class="flex-none px-4 py-3 flex flex-row items-center gap-3 border-b border-subtle bg-surface-muted">
        <View class="min-w-0 flex-1 flex flex-col gap-0.5">
          <Text class="text-sm font-medium">Compare with</Text>
          <Text class="truncate text-xs text-muted">
            Current: {formatSnapshotTime(props.snapshot.time)}
          </Text>
        </View>
        <Select
          aria-label="Comparison snapshot"
          class="w-56"
          contentClass="w-72"
          value={baseSnapshotId()}
          placeholder="Choose a snapshot"
          options={candidates().map((snapshot) => ({
            value: snapshot.id,
            label: `${formatSnapshotTime(snapshot.time)} · ${snapshot.id.slice(0, 8)}`,
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
                class="min-h-0 flex-1 border-0 shadow-none"
              />
            }
          >
            <View class="flex-none px-4 py-2.5 flex flex-row items-center gap-2 border-b border-subtle">
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
              <Text class="ml-auto text-xs text-muted">
                {baseSnapshot()
                  ? `Since ${formatSnapshotTime(baseSnapshot()?.time ?? "")}`
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
              <ScrollArea
                class="min-w-0 min-h-0 flex-1"
                contentClass="min-w-full"
              >
                <Table aria-label="Snapshot changes">
                  <TableHeader>
                    <TableRow class="bg-surface-muted">
                      <For each={diffColumns}>
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
                      </For>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <For each={diffTable.rows()}>
                      {(row) => {
                        const entry = row.original;
                        const presentation = changePresentation[entry.change];
                        return (
                          <TableRow aria-label={entry.path}>
                            <TableCell class="min-w-64 flex-1 gap-2">
                              <Icon
                                source={
                                  entry.kind === "directory" ? folder : file
                                }
                                size={15}
                                class="flex-none text-muted"
                              />
                              <View class="min-w-0 flex-1 flex flex-col gap-0.5">
                                <Text class="w-full truncate">
                                  {entry.name}
                                </Text>
                                <Text class="w-full truncate text-xs text-muted">
                                  {entry.path}
                                </Text>
                              </View>
                            </TableCell>
                            <TableCell class="w-28 flex-none">
                              <Badge
                                variant={presentation.variant}
                                weight="normal"
                              >
                                {presentation.label}
                              </Badge>
                            </TableCell>
                            <TableCell class="w-28 flex-none text-muted">
                              {formatBytes(entry.previousSize)}
                            </TableCell>
                            <TableCell class="w-28 flex-none text-muted">
                              {formatBytes(entry.currentSize)}
                            </TableCell>
                            <TableCell class="w-36 flex-none text-muted">
                              {formatModified(
                                entry.currentModified ?? entry.previousModified,
                              )}
                            </TableCell>
                          </TableRow>
                        );
                      }}
                    </For>
                  </TableBody>
                </Table>
              </ScrollArea>
            </Show>
          </Show>
        </Show>
      </Show>
    </View>
  );
}
