import {
  Badge,
  Button,
  ButtonGroup,
  ContentState,
  createTanStackDataTable,
  Icon,
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  PageHeader,
  ProjectionBoundary,
  ScrollArea,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  type TanStackDataTableColumn,
  Text,
  useNavigate,
  View,
} from "@wabou/ui";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import file from "lucide-static/icons/file.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import folderTree from "lucide-static/icons/folder-tree.svg?raw";
import gitCompare from "lucide-static/icons/git-compare-arrows.svg?raw";
import list from "lucide-static/icons/list.svg?raw";
import refreshCw from "lucide-static/icons/refresh-cw.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import shieldCheck from "lucide-static/icons/shield-check.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import { type FileEntry, type SnapshotEntry, useRusticApi } from "./api";
import { FileDetails } from "./file-details";
import { useTimestowSession } from "./session";
import { createSnapshotBrowserCache } from "./snapshot-browser-cache";
import { formatSnapshotTime, SnapshotDetails } from "./snapshot-details";
import { SnapshotDiffPanel } from "./snapshot-diff";
import { SnapshotFileTree } from "./snapshot-tree";
import { SortableTableHead } from "./sortable-table-head";
import { BackupSourcesPanel } from "./workspace-components";

const fileColumns: TanStackDataTableColumn<FileEntry>[] = [
  { accessorKey: "name", header: "Name" },
  { accessorKey: "size", header: "Size" },
  { accessorKey: "modified", header: "Modified" },
];

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
}

function shortId(id: string): string {
  return id.slice(0, 8);
}

export function formatModified(value?: string): string {
  if (!value) return "—";
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})/);
  return match
    ? `${match[1]}-${match[2]}-${match[3]} ${match[4]}:${match[5]}`
    : value;
}

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
    <TableRow
      aria-label={entry().name}
      selected={props.selected}
      class="cursor-pointer"
      onClick={select}
      onDblClick={() => {
        if (entry().kind !== "directory") return;
        cancelPendingSingleClick();
        props.onOpenDirectory(entry());
      }}
    >
      <TableCell class="min-w-64 flex-1 gap-2">
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
      <TableCell class="w-24 flex-none text-muted">
        {entry().kind === "directory" ? "—" : formatBytes(entry().size)}
      </TableCell>
      <TableCell class="w-36 flex-none text-muted">
        {formatModified(entry().modified)}
      </TableCell>
      <TableCell class="w-20 flex-none justify-end">
        <Show when={entry().kind === "directory"}>
          <Button
            size="sm"
            variant="ghost"
            onClick={(event) => {
              event.stopPropagation();
              props.onOpenDirectory(entry());
            }}
          >
            Open
          </Button>
        </Show>
      </TableCell>
    </TableRow>
  );
}

export function SnapshotsPage() {
  const api = useRusticApi();
  const session = useTimestowSession();
  const navigate = useNavigate();
  const [snapshots, setSnapshots] = createSignal<SnapshotEntry[]>([]);
  const [selected, setSelected] = createSignal<SnapshotEntry>();
  const [files, setFiles] = createSignal<FileEntry[]>([]);
  const [selectedEntry, setSelectedEntry] = createSignal<FileEntry>();
  const [searchQuery, setSearchQuery] = createSignal("");
  const [searchResults, setSearchResults] = createSignal<FileEntry[]>([]);
  const [searchActive, setSearchActive] = createSignal(false);
  const [searching, setSearching] = createSignal(false);
  const [browserMode, setBrowserMode] = createSignal<"list" | "tree">("list");
  const [workspaceMode, setWorkspaceMode] = createSignal<"browse" | "changes">(
    "browse",
  );
  const [currentPath, setCurrentPath] = createSignal("");
  const [loading, setLoading] = createSignal(true);
  const [loadingFiles, setLoadingFiles] = createSignal(false);
  const [backingUp, setBackingUp] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const browserCache = createSnapshotBrowserCache();
  let fileRequestGeneration = 0;

  async function loadSnapshots(profileId: string, selectNewest = false) {
    setLoading(true);
    setError(undefined);
    try {
      const next = await api.listSnapshots({ profileId });
      setSnapshots(next);
      if (selectNewest && next[0]) selectSnapshot(profileId, next[0]);
      else if (selected()) {
        const refreshed = next.find((item) => item.id === selected()?.id);
        if (refreshed) setSelected(refreshed);
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  }

  async function loadFiles(
    profileId: string,
    snapshot: SnapshotEntry,
    path: string,
  ) {
    const generation = ++fileRequestGeneration;
    setCurrentPath(path);
    setFiles([]);
    setSelectedEntry(undefined);
    setSearchActive(false);
    setSearchResults([]);
    setError(undefined);
    const cached = browserCache.entries(snapshot.id, path);
    if (cached) {
      setFiles([...cached]);
      setLoadingFiles(false);
      return;
    }
    setLoadingFiles(true);
    try {
      const next = await api.listFiles({
        profileId,
        snapshotId: snapshot.id,
        path,
      });
      browserCache.remember(snapshot.id, path, next);
      if (generation !== fileRequestGeneration) return;
      setFiles(next);
    } catch (cause) {
      if (generation !== fileRequestGeneration) return;
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (generation === fileRequestGeneration) setLoadingFiles(false);
    }
  }

  async function runSearch() {
    const profile = session.activeProfile();
    const snapshot = selected();
    const query = searchQuery().trim();
    if (!profile || !snapshot || !query || searching()) return;
    setSearching(true);
    setError(undefined);
    try {
      setSearchResults(
        await api.searchFiles({
          profileId: profile.id,
          snapshotId: snapshot.id,
          query,
          limit: 200,
        }),
      );
      setSearchActive(true);
      setSelectedEntry(undefined);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSearching(false);
    }
  }

  function clearSearch() {
    setSearchQuery("");
    setSearchResults([]);
    setSearchActive(false);
    setSelectedEntry(undefined);
  }

  function selectSnapshot(profileId: string, snapshot: SnapshotEntry) {
    setSelected(snapshot);
    void loadFiles(profileId, snapshot, browserCache.lastPath(snapshot.id));
  }

  async function refreshSnapshots(profileId: string) {
    const selectedId = selected()?.id;
    const path = currentPath();
    browserCache.clear();
    await loadSnapshots(profileId);
    const refreshed = snapshots().find(
      (snapshot) => snapshot.id === selectedId,
    );
    if (refreshed) await loadFiles(profileId, refreshed, path);
  }

  async function saveSources(sources: string[]) {
    const profile = session.activeProfile();
    if (!profile) return;
    try {
      await session.updateSources(profile.id, sources);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function runBackup() {
    const profile = session.activeProfile();
    if (!profile || backingUp() || profile.sources.length === 0) return;
    setBackingUp(true);
    setError(undefined);
    try {
      await api.runBackup({ profileId: profile.id });
      await loadSnapshots(profile.id, true);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBackingUp(false);
    }
  }

  async function updateSnapshot(
    snapshot: SnapshotEntry,
    changes: Pick<
      SnapshotEntry,
      "label" | "description" | "tags" | "deleteProtected"
    >,
  ) {
    const profile = session.activeProfile();
    if (!profile) return;
    const updated = await api.updateSnapshot({
      profileId: profile.id,
      snapshotId: snapshot.id,
      label: changes.label,
      description: changes.description ?? "",
      tags: changes.tags,
      deleteProtected: changes.deleteProtected,
    });
    setSnapshots((items) =>
      items.map((item) => (item.id === snapshot.id ? updated : item)),
    );
    if (selected()?.id === snapshot.id) setSelected(updated);
  }

  function parentPath(path: string): string {
    const parts = path.split(/[\\/]/).filter(Boolean);
    parts.pop();
    return parts.join("/");
  }

  createEffect(
    () => {
      const profile = session.activeProfile();
      return profile &&
        session.runtime().unlockedProfileIds.includes(profile.id)
        ? profile.id
        : undefined;
    },
    (profileId) => {
      if (!profileId) {
        queueMicrotask(() => void navigate({ to: "/" }));
        return;
      }
      void loadSnapshots(profileId, true);
    },
  );

  const visibleFiles = () => (searchActive() ? searchResults() : files());
  const fileTable = createTanStackDataTable<FileEntry>({
    data: visibleFiles,
    columns: fileColumns,
    getRowId: (entry) => entry.path,
    initialSorting: [{ id: "name", desc: false }],
  });

  const sortDirection = (columnId: string) => {
    const sorting = fileTable.sorting().find(({ id }) => id === columnId);
    return sorting ? (sorting.desc ? "desc" : "asc") : undefined;
  };

  return (
    <View class="w-full h-full min-w-0 min-h-0 flex flex-col">
      <View class="flex-none px-6 py-5 border-b border-subtle bg-surface">
        <PageHeader
          title={session.activeProfile()?.name ?? "Backup"}
          description={session.activeProfile()?.repositoryPath ?? ""}
          actions={
            <>
              <Button
                variant="outline"
                onClick={() => {
                  const profile = session.activeProfile();
                  if (profile) void refreshSnapshots(profile.id);
                }}
              >
                <Icon source={refreshCw} size={14} /> Refresh
              </Button>
              <Button
                disabled={
                  !session.activeProfile()?.sources.length || backingUp()
                }
                onClick={() => void runBackup()}
              >
                {backingUp() ? "Backing up…" : "Back up now"}
              </Button>
            </>
          }
        />
      </View>
      <Show when={error()}>
        {(message) => (
          <View class="flex-none mx-6 mt-4 rounded-md border border-danger bg-danger-surface px-3 py-2">
            <Text class="text-sm text-danger-primary">{message()}</Text>
          </View>
        )}
      </Show>
      <View class="min-w-0 min-h-0 flex-1 flex flex-row p-4 gap-4">
        <ProjectionBoundary
          id="rustic-sidebar"
          class="w-72 min-h-0 flex-none flex flex-col rounded-xl border border-subtle bg-surface shadow-sm overflow-hidden"
        >
          <BackupSourcesPanel
            sources={session.activeProfile()?.sources ?? []}
            disabled={backingUp()}
            onChange={(sources) => void saveSources(sources)}
          />
          <View class="flex-none px-4 py-3 border-b border-subtle">
            <Text class="font-semibold">Snapshots</Text>
          </View>
          <ScrollArea class="min-h-0 flex-1" contentClass="flex flex-col py-2">
            <Show
              when={!loading() && snapshots().length > 0}
              fallback={
                <ContentState
                  state={loading() ? "loading" : "empty"}
                  title={loading() ? "Loading snapshots" : "No snapshots yet"}
                  description={
                    loading()
                      ? undefined
                      : "Add a folder and run your first backup."
                  }
                  class="border-0 shadow-none"
                />
              }
            >
              <For each={snapshots()}>
                {(snapshot) => (
                  <Button
                    variant="ghost"
                    selected={selected()?.id === snapshot.id}
                    class="mx-2 min-h-14 justify-start px-3"
                    onClick={() => {
                      const profile = session.activeProfile();
                      if (profile) selectSnapshot(profile.id, snapshot);
                    }}
                  >
                    <View class="min-w-0 flex-1 flex flex-col items-start gap-0.5">
                      <Text class="font-medium">
                        {snapshot.label || formatSnapshotTime(snapshot.time)}
                      </Text>
                      <View class="w-full min-w-0 flex flex-row items-center gap-1.5">
                        <Show when={snapshot.deleteProtected}>
                          <Icon
                            source={shieldCheck}
                            size={12}
                            class="flex-none text-success-primary"
                          />
                        </Show>
                        <Text class="min-w-0 flex-1 truncate text-xs text-muted">
                          {snapshot.label
                            ? `${formatSnapshotTime(snapshot.time)} · `
                            : ""}
                          {shortId(snapshot.id)} ·{" "}
                          {snapshot.hostname || "Unknown host"}
                        </Text>
                      </View>
                    </View>
                  </Button>
                )}
              </For>
            </Show>
          </ScrollArea>
        </ProjectionBoundary>

        <ProjectionBoundary
          id="rustic-file-browser"
          class="min-w-0 min-h-0 flex-1 flex flex-col rounded-xl border border-subtle bg-surface shadow-sm overflow-hidden"
        >
          <Show
            when={selected()}
            fallback={
              <ContentState
                state="empty"
                title="Select a snapshot"
                description="The files stored in that point in time will appear here."
                class="min-h-0 flex-1 border-0 shadow-none"
              />
            }
          >
            {(snapshot) => (
              <>
                <View class="flex-none px-4 py-3 flex flex-col gap-3 border-b border-subtle">
                  <View class="flex flex-row items-center gap-3">
                    <Button
                      size="icon"
                      variant="ghost"
                      disabled={!currentPath() || searchActive()}
                      aria-label="Open parent folder"
                      onClick={() =>
                        void loadFiles(
                          session.activeProfile()?.id ?? "",
                          snapshot(),
                          parentPath(currentPath()),
                        )
                      }
                    >
                      <Icon source={chevronLeft} size={15} />
                    </Button>
                    <View class="min-w-0 flex-1 flex flex-col">
                      <Text class="font-semibold">
                        Snapshot {shortId(snapshot().id)}
                      </Text>
                      <Text class="truncate text-xs text-muted">
                        {searchActive()
                          ? `Search results for “${searchQuery()}”`
                          : `/${currentPath() || ""}`}
                      </Text>
                    </View>
                    <ButtonGroup
                      size="sm"
                      variant="outline"
                      aria-label="Snapshot workspace"
                    >
                      <Button
                        size="sm"
                        variant="outline"
                        selected={workspaceMode() === "browse"}
                        onClick={() => setWorkspaceMode("browse")}
                      >
                        <Icon source={folder} size={14} /> Browse
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        selected={workspaceMode() === "changes"}
                        onClick={() => setWorkspaceMode("changes")}
                      >
                        <Icon source={gitCompare} size={14} /> Changes
                      </Button>
                    </ButtonGroup>
                    <SnapshotDetails
                      snapshot={snapshot()}
                      onSave={(changes) => updateSnapshot(snapshot(), changes)}
                    />
                    <Show when={workspaceMode() === "browse"}>
                      <ButtonGroup
                        size="sm"
                        variant="ghost"
                        aria-label="File view"
                      >
                        <Button
                          size="icon"
                          variant="ghost"
                          selected={browserMode() === "list"}
                          aria-label="List view"
                          onClick={() => setBrowserMode("list")}
                        >
                          <Icon source={list} size={14} />
                        </Button>
                        <Button
                          size="icon"
                          variant="ghost"
                          selected={browserMode() === "tree"}
                          aria-label="Tree view"
                          onClick={() => {
                            clearSearch();
                            setBrowserMode("tree");
                          }}
                        >
                          <Icon source={folderTree} size={14} />
                        </Button>
                      </ButtonGroup>
                      <Badge variant="secondary">
                        {loadingFiles()
                          ? "Loading…"
                          : `${visibleFiles().length} items`}
                      </Badge>
                    </Show>
                  </View>
                  <Show when={workspaceMode() === "browse"}>
                    <View class="flex flex-row items-center gap-2">
                      <InputGroup class="min-w-0 flex-1">
                        <InputGroupAddon align="inline-start" class="px-2.5">
                          <Icon source={search} size={14} class="text-muted" />
                        </InputGroupAddon>
                        <InputGroupInput
                          aria-label="Search snapshot"
                          placeholder="Search this snapshot…"
                          value={searchQuery()}
                          onInput={(event) =>
                            setSearchQuery(event.currentTarget.value)
                          }
                          onKeyDown={(event) => {
                            if (event.key === "Enter") {
                              event.preventDefault();
                              void runSearch();
                            }
                          }}
                        />
                      </InputGroup>
                      <Button
                        variant="outline"
                        disabled={!searchQuery().trim() || searching()}
                        loading={searching()}
                        loadingLabel="Searching…"
                        onClick={() => void runSearch()}
                      >
                        Search
                      </Button>
                      <Show when={searchActive()}>
                        <Button
                          size="icon"
                          variant="ghost"
                          aria-label="Clear search"
                          onClick={clearSearch}
                        >
                          <Icon source={x} size={14} />
                        </Button>
                      </Show>
                    </View>
                  </Show>
                </View>
                <Show
                  when={workspaceMode() === "browse"}
                  fallback={
                    <SnapshotDiffPanel
                      profileId={session.activeProfile()?.id ?? ""}
                      snapshot={snapshot()}
                      snapshots={snapshots()}
                    />
                  }
                >
                  <Show
                    when={!loadingFiles()}
                    fallback={
                      <ContentState
                        state="loading"
                        title="Loading snapshot files"
                        description="Reading this snapshot’s directory…"
                        class="min-h-0 flex-1 border-0 shadow-none"
                      />
                    }
                  >
                    <View class="min-w-0 min-h-0 flex-1 flex flex-row">
                      <Show
                        when={browserMode() === "list" || searchActive()}
                        fallback={
                          <ScrollArea
                            class="min-w-0 min-h-0 flex-1"
                            contentClass="min-w-full px-2 py-2"
                          >
                            <SnapshotFileTree
                              profileId={session.activeProfile()?.id ?? ""}
                              snapshotId={snapshot().id}
                              selectedPath={selectedEntry()?.path}
                              onSelect={setSelectedEntry}
                            />
                          </ScrollArea>
                        }
                      >
                        <ScrollArea
                          class="min-w-0 min-h-0 flex-1"
                          contentClass="min-w-full"
                        >
                          <Table>
                            <TableHeader>
                              <TableRow class="bg-surface-muted">
                                <SortableTableHead
                                  label="Name"
                                  class="min-w-64 flex-1"
                                  direction={() => sortDirection("name")}
                                  onToggle={() =>
                                    fileTable.table
                                      .getColumn("name")
                                      ?.toggleSorting()
                                  }
                                />
                                <SortableTableHead
                                  label="Size"
                                  class="w-24 flex-none"
                                  direction={() => sortDirection("size")}
                                  onToggle={() =>
                                    fileTable.table
                                      .getColumn("size")
                                      ?.toggleSorting()
                                  }
                                />
                                <SortableTableHead
                                  label="Modified"
                                  class="w-36 flex-none"
                                  direction={() => sortDirection("modified")}
                                  onToggle={() =>
                                    fileTable.table
                                      .getColumn("modified")
                                      ?.toggleSorting()
                                  }
                                />
                                <TableHead class="w-20 flex-none" />
                              </TableRow>
                            </TableHeader>
                            <TableBody>
                              <For each={fileTable.rows()}>
                                {(row) => (
                                  <SnapshotFileRow
                                    entry={row.original}
                                    selected={
                                      selectedEntry()?.path ===
                                      row.original.path
                                    }
                                    searchActive={searchActive()}
                                    onSelect={setSelectedEntry}
                                    onOpenDirectory={(directory) =>
                                      void loadFiles(
                                        session.activeProfile()?.id ?? "",
                                        snapshot(),
                                        directory.path,
                                      )
                                    }
                                  />
                                )}
                              </For>
                            </TableBody>
                          </Table>
                        </ScrollArea>
                      </Show>
                      <FileDetails
                        profileId={session.activeProfile()?.id ?? ""}
                        snapshotId={snapshot().id}
                        entry={selectedEntry()}
                      />
                    </View>
                  </Show>
                </Show>
              </>
            )}
          </Show>
        </ProjectionBoundary>
      </View>
    </View>
  );
}
