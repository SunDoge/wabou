import {
  AdaptiveSplitPane,
  AdaptiveSplitPaneDetail,
  AdaptiveSplitPaneMain,
  Button,
  ButtonGroup,
  ContentState,
  ContextMenu,
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
import {
  createEffect,
  createSignal,
  For as ForValue,
  type JSX,
  Match,
  onCleanup,
  Show,
  Switch,
} from "solid-js";
import {
  FILE_PAGE_SIZE,
  type FileEntry,
  type SnapshotEntry,
  useRusticApi,
} from "./api";
import { BackupProgressStatus } from "./backup-progress";
import { FileDetails } from "./file-details";
import { BackupScheduleDialog } from "./schedule-dialog";
import { useTimestowSession } from "./session";
import { createSnapshotBrowserCache } from "./snapshot-browser-cache";
import { formatSnapshotTime, SnapshotDetails } from "./snapshot-details";
import { SnapshotDiffPanel } from "./snapshot-diff";
import { SnapshotFileTree } from "./snapshot-tree";
import { SortableTableHead } from "./sortable-table-head";
import { BackupSourcesDialog } from "./workspace-components";

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

export function snapshotAfterRefresh(
  snapshots: readonly SnapshotEntry[],
  currentId: string | undefined,
  selectNewest: boolean,
): SnapshotEntry | undefined {
  if (selectNewest) return snapshots[0];
  return currentId
    ? snapshots.find((snapshot) => snapshot.id === currentId)
    : undefined;
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
            {formatModified(entry().modified)}
          </TableCell>
        </TableRow>
      )}
    />
  );
}

export interface SnapshotWorkspaceHeaderProps {
  name: string;
  repositoryPath: string;
  sources: readonly string[];
  backingUp: boolean;
  showBackupAction?: boolean;
  scheduleControl?: JSX.Element;
  onSourcesChange(sources: string[]): void;
  onRefresh(): void;
  onBackup(): void;
}

export function SnapshotWorkspaceHeader(props: SnapshotWorkspaceHeaderProps) {
  return (
    <PageHeader
      stacked
      title={props.name}
      description={props.repositoryPath}
      actions={
        <View
          role="toolbar"
          aria-label="Backup workspace actions"
          class="w-full min-w-0 flex flex-row items-center justify-between gap-3"
        >
          <View class="min-w-0 flex flex-row items-center gap-2">
            <BackupSourcesDialog
              sources={props.sources}
              disabled={props.backingUp}
              onChange={props.onSourcesChange}
            />
            {props.scheduleControl}
          </View>
          <View class="flex-none flex flex-row items-center gap-2">
            <Button
              aria-label="Refresh snapshots"
              variant="outline"
              onClick={props.onRefresh}
            >
              <Icon source={refreshCw} size={14} /> Refresh
            </Button>
            <Show when={props.showBackupAction !== false}>
              <Button
                aria-label={props.backingUp ? "Backing up" : "Back up now"}
                disabled={props.sources.length === 0 || props.backingUp}
                onClick={props.onBackup}
              >
                {props.backingUp ? "Backing up…" : "Back up now"}
              </Button>
            </Show>
          </View>
        </View>
      }
    />
  );
}

export function SnapshotBrowserEmptyState(props: {
  loading: boolean;
  loadFailed: boolean;
  hasSnapshots: boolean;
  sourceCount: number;
  backingUp: boolean;
  onBackup(): void;
}) {
  return (
    <Switch
      fallback={
        <ContentState
          state="empty"
          title="Create your first snapshot"
          description={`Back up ${props.sourceCount} ${props.sourceCount === 1 ? "folder" : "folders"} to start the history.`}
          action={{ label: "Back up now", onAction: props.onBackup }}
          class="min-h-0 flex-1 border-0 shadow-none"
        />
      }
    >
      <Match when={props.loading}>
        <ContentState
          state="loading"
          title="Loading snapshots"
          description="Reading this backup’s history…"
          class="min-h-0 flex-1 border-0 shadow-none"
        />
      </Match>
      <Match when={props.loadFailed}>
        <ContentState
          state="error"
          title="Snapshot history unavailable"
          description="Resolve the error above or refresh to try again."
          class="min-h-0 flex-1 border-0 shadow-none"
        />
      </Match>
      <Match when={props.hasSnapshots}>
        <ContentState
          state="empty"
          title="Select a snapshot"
          description="Choose a point in time from the history to browse its files."
          class="min-h-0 flex-1 border-0 shadow-none"
        />
      </Match>
      <Match when={props.backingUp}>
        <ContentState
          state="loading"
          title="Creating your first snapshot"
          description="Timestow will open it here when the backup finishes."
          class="min-h-0 flex-1 border-0 shadow-none"
        />
      </Match>
      <Match when={props.sourceCount === 0}>
        <ContentState
          state="empty"
          title="Choose folders to back up"
          description="Add at least one folder using the folder control above."
          class="min-h-0 flex-1 border-0 shadow-none"
        />
      </Match>
    </Switch>
  );
}

export function snapshotMatchesQuery(
  snapshot: SnapshotEntry,
  query: string,
): boolean {
  const normalized = query.trim().toLocaleLowerCase();
  if (!normalized) return true;
  return [
    snapshot.label,
    snapshot.id,
    snapshot.hostname,
    snapshot.time,
    formatSnapshotTime(snapshot.time),
    ...snapshot.tags,
  ]
    .filter((value): value is string => Boolean(value))
    .some((value) => value.toLocaleLowerCase().includes(normalized));
}

export function SnapshotHistory(props: {
  loading: boolean;
  snapshots: readonly SnapshotEntry[];
  selectedId?: string;
  query: string;
  onQueryChange(query: string): void;
  onSelect(snapshot: SnapshotEntry): void;
}) {
  const filtered = () =>
    props.snapshots.filter((snapshot) =>
      snapshotMatchesQuery(snapshot, props.query),
    );
  const searchable = () => props.snapshots.length >= 8;
  return (
    <ProjectionBoundary
      id="rustic-sidebar"
      role="region"
      aria-label="Snapshot history"
      class="w-64 min-h-0 flex-none flex flex-col border-r border-subtle bg-surface-muted"
    >
      <View class="flex-none px-3 py-3 flex flex-col gap-2 border-b border-subtle">
        <Text class="px-1 text-xs font-semibold tracking-wide text-muted">
          Snapshot history
        </Text>
        <Show when={searchable()}>
          <InputGroup>
            <InputGroupAddon align="inline-start" class="px-2.5">
              <Icon source={search} size={13} class="text-muted" />
            </InputGroupAddon>
            <InputGroupInput
              aria-label="Filter snapshots"
              placeholder="Filter snapshots…"
              value={props.query}
              onInput={(event) => props.onQueryChange(event.currentTarget.value)}
            />
            <Show when={props.query.trim()}>
              <InputGroupAddon align="inline-end" class="px-1.5">
                <Button
                  size="icon"
                  variant="ghost"
                  class="w-6 h-6"
                  aria-label="Clear snapshot filter"
                  onClick={() => props.onQueryChange("")}
                >
                  <Icon source={x} size={12} />
                </Button>
              </InputGroupAddon>
            </Show>
          </InputGroup>
        </Show>
      </View>
      <ScrollArea
        class="min-h-0 flex-1"
        contentClass="flex flex-col gap-1 p-2"
      >
        <Show
          when={!props.loading && props.snapshots.length > 0}
          fallback={
            <ContentState
              state={props.loading ? "loading" : "empty"}
              title={props.loading ? "Loading snapshots" : "No snapshots yet"}
              description={
                props.loading
                  ? undefined
                  : "Run your first backup to create a snapshot."
              }
              class="border-0 shadow-none"
            />
          }
        >
          <Show
            when={filtered().length > 0}
            fallback={
              <ContentState
                state="empty"
                title="No matching snapshots"
                description="Try a label, date, host, tag, or snapshot ID."
                action={{
                  label: "Clear filter",
                  onAction: () => props.onQueryChange(""),
                }}
                class="border-0 shadow-none"
              />
            }
          >
            <ForValue each={filtered()}>
              {(snapshot) => (
                <Button
                  aria-label={`Open snapshot ${snapshot.label || formatSnapshotTime(snapshot.time)}`}
                  variant="ghost"
                  selected={props.selectedId === snapshot.id}
                  class="min-h-14 justify-start px-3"
                  onClick={() => props.onSelect(snapshot)}
                >
                  <View class="min-w-0 flex-1 flex flex-col items-start gap-0.5">
                    <Text class="w-full truncate font-medium">
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
                      <Text
                        class={
                          props.selectedId === snapshot.id
                            ? "min-w-0 flex-1 truncate text-xs text-secondary"
                            : "min-w-0 flex-1 truncate text-xs text-muted"
                        }
                      >
                        {snapshot.label
                          ? `${formatSnapshotTime(snapshot.time)} · `
                          : ""}
                        {shortId(snapshot.id)} · {snapshot.hostname || "Unknown host"}
                      </Text>
                    </View>
                  </View>
                </Button>
              )}
            </ForValue>
          </Show>
        </Show>
      </ScrollArea>
    </ProjectionBoundary>
  );
}

export function SnapshotsPage() {
  const api = useRusticApi();
  const session = useTimestowSession();
  const navigate = useNavigate();
  const [snapshots, setSnapshots] = createSignal<SnapshotEntry[]>([]);
  const [selected, setSelected] = createSignal<SnapshotEntry>();
  const [files, setFiles] = createSignal<FileEntry[]>([]);
  const [fileTotal, setFileTotal] = createSignal(0);
  const [selectedEntry, setSelectedEntry] = createSignal<FileEntry>();
  const [searchQuery, setSearchQuery] = createSignal("");
  const [snapshotQuery, setSnapshotQuery] = createSignal("");
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
  const [loadingMoreFiles, setLoadingMoreFiles] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const browserCache = createSnapshotBrowserCache();
  let fileRequestGeneration = 0;
  let searchRequestGeneration = 0;
  let snapshotRequestGeneration = 0;

  function clearSnapshotWorkspace(): void {
    fileRequestGeneration += 1;
    searchRequestGeneration += 1;
    setSelected(undefined);
    setFiles([]);
    setFileTotal(0);
    setSelectedEntry(undefined);
    setSearchQuery("");
    setSnapshotQuery("");
    setSearchResults([]);
    setSearchActive(false);
    setSearching(false);
    setCurrentPath("");
    setLoadingFiles(false);
    setLoadingMoreFiles(false);
  }

  async function loadSnapshots(profileId: string, selectNewest = false) {
    const generation = ++snapshotRequestGeneration;
    if (selectNewest) {
      browserCache.clear();
      clearSnapshotWorkspace();
      setSnapshots([]);
    }
    setLoading(true);
    setError(undefined);
    try {
      const next = await api.listSnapshots({ profileId });
      if (generation !== snapshotRequestGeneration) return false;
      setSnapshots(next);
      const refreshed = snapshotAfterRefresh(
        next,
        selected()?.id,
        selectNewest,
      );
      if (refreshed && selectNewest) selectSnapshot(profileId, refreshed);
      else if (refreshed) setSelected(refreshed);
      else if (selected()) {
        clearSnapshotWorkspace();
      }
      return true;
    } catch (cause) {
      if (generation === snapshotRequestGeneration) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
      return false;
    } finally {
      if (generation === snapshotRequestGeneration) setLoading(false);
    }
  }

  async function loadFiles(
    profileId: string,
    snapshot: SnapshotEntry,
    path: string,
  ) {
    const generation = ++fileRequestGeneration;
    searchRequestGeneration += 1;
    setCurrentPath(path);
    setFiles([]);
    setFileTotal(0);
    setSelectedEntry(undefined);
    setSearchActive(false);
    setSearchResults([]);
    setSearching(false);
    setLoadingMoreFiles(false);
    setError(undefined);
    const cached = browserCache.listing(snapshot.id, path);
    if (cached) {
      setFiles([...cached.entries]);
      setFileTotal(cached.total);
      setLoadingFiles(false);
      return;
    }
    setLoadingFiles(true);
    try {
      const next = await api.listFiles({
        profileId,
        snapshotId: snapshot.id,
        path,
        offset: 0,
        limit: FILE_PAGE_SIZE,
      });
      if (generation !== fileRequestGeneration) return;
      browserCache.remember(snapshot.id, path, next);
      setFiles(next.entries);
      setFileTotal(next.total);
    } catch (cause) {
      if (generation !== fileRequestGeneration) return;
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (generation === fileRequestGeneration) setLoadingFiles(false);
    }
  }

  async function loadMoreFiles() {
    const profile = session.activeProfile();
    const snapshot = selected();
    const path = currentPath();
    const offset = files().length;
    if (!profile || !snapshot || loadingMoreFiles() || offset >= fileTotal())
      return;
    const generation = fileRequestGeneration;
    setLoadingMoreFiles(true);
    setError(undefined);
    try {
      const next = await api.listFiles({
        profileId: profile.id,
        snapshotId: snapshot.id,
        path,
        offset,
        limit: FILE_PAGE_SIZE,
      });
      if (generation !== fileRequestGeneration) return;
      const combined = [...files(), ...next.entries];
      setFiles(combined);
      setFileTotal(next.total);
      browserCache.remember(snapshot.id, path, {
        entries: combined,
        total: next.total,
      });
    } catch (cause) {
      if (generation === fileRequestGeneration) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
    } finally {
      if (generation === fileRequestGeneration) setLoadingMoreFiles(false);
    }
  }

  async function runSearch() {
    const profile = session.activeProfile();
    const snapshot = selected();
    const query = searchQuery().trim();
    if (!profile || !snapshot || !query || searching()) return;
    const generation = ++searchRequestGeneration;
    setSearching(true);
    setError(undefined);
    try {
      const results = await api.searchFiles({
        profileId: profile.id,
        snapshotId: snapshot.id,
        query,
        limit: 200,
      });
      if (generation !== searchRequestGeneration) return;
      setSearchResults(results);
      setSearchActive(true);
      setSelectedEntry(undefined);
    } catch (cause) {
      if (generation === searchRequestGeneration) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
    } finally {
      if (generation === searchRequestGeneration) setSearching(false);
    }
  }

  function clearSearch() {
    searchRequestGeneration += 1;
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
    if (!(await loadSnapshots(profileId))) return;
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
    setError(undefined);
    try {
      await session.runBackup(profile.id);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  const backingUp = () => {
    const profile = session.activeProfile();
    return profile ? session.isBackingUp(profile.id) : false;
  };

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

  async function deleteSnapshot(snapshot: SnapshotEntry) {
    const profile = session.activeProfile();
    if (!profile) return;
    await api.deleteSnapshot({
      profileId: profile.id,
      snapshotId: snapshot.id,
    });
    browserCache.clear();
    await loadSnapshots(profile.id, true);
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

  createEffect(
    () => session.lastBackup(),
    (completed) => {
      const profile = session.activeProfile();
      if (!completed || completed.profileId !== profile?.id) return;
      void loadSnapshots(completed.profileId, true);
    },
  );

  const visibleFiles = () => (searchActive() ? searchResults() : files());
  const fileCountLabel = () => {
    if (loadingFiles()) return "Loading…";
    if (searchActive()) return `${visibleFiles().length} matches`;
    if (fileTotal() > files().length) {
      return `${files().length} of ${fileTotal()} items`;
    }
    return `${files().length} items`;
  };
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
      <View class="flex-none px-6 py-4 flex flex-col gap-3 border-b border-subtle bg-surface">
        <SnapshotWorkspaceHeader
          name={session.activeProfile()?.name ?? "Backup"}
          repositoryPath={session.activeProfile()?.repositoryPath ?? ""}
          sources={session.activeProfile()?.sources ?? []}
          backingUp={backingUp()}
          showBackupAction={snapshots().length > 0}
          scheduleControl={
            <Show when={session.activeProfile()}>
              {(profile) => (
                <BackupScheduleDialog
                  profile={profile()}
                  disabled={backingUp()}
                />
              )}
            </Show>
          }
          onSourcesChange={(sources) => void saveSources(sources)}
          onRefresh={() => {
            const profile = session.activeProfile();
            if (profile) void refreshSnapshots(profile.id);
          }}
          onBackup={() => void runBackup()}
        />
        <Show when={backingUp() && session.activeProfile()}>
          {(profile) => (
            <Show when={session.backupProgress(profile().id)}>
              {(progress) => <BackupProgressStatus progress={progress()} />}
            </Show>
          )}
        </Show>
      </View>
      <Show when={error()}>
        {(message) => (
          <View class="flex-none mx-6 mt-4 rounded-md border border-danger bg-danger-surface px-3 py-2">
            <Text class="text-sm text-danger-primary">{message()}</Text>
          </View>
        )}
      </Show>
      <View class="min-w-0 min-h-0 flex-1 flex flex-row bg-surface">
        <SnapshotHistory
          loading={loading()}
          snapshots={snapshots()}
          selectedId={selected()?.id}
          query={snapshotQuery()}
          onQueryChange={setSnapshotQuery}
          onSelect={(snapshot) => {
            const profile = session.activeProfile();
            if (profile) selectSnapshot(profile.id, snapshot);
          }}
        />

        <ProjectionBoundary
          id="rustic-file-browser"
          role="region"
          aria-label="Snapshot browser"
          class="min-w-0 min-h-0 flex-1 flex flex-col bg-surface overflow-hidden"
        >
          <Show
            when={selected()}
            fallback={
              <SnapshotBrowserEmptyState
                loading={loading()}
                loadFailed={error() !== undefined}
                hasSnapshots={snapshots().length > 0}
                sourceCount={session.activeProfile()?.sources.length ?? 0}
                backingUp={backingUp()}
                onBackup={() => void runBackup()}
              />
            }
          >
            {(snapshot) => (
              <>
                <View class="flex-none px-5 py-3 flex flex-col gap-3 border-b border-subtle">
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
                          : `/${currentPath() || ""}`}{" "}
                        · {fileCountLabel()}
                      </Text>
                    </View>
                  </View>
                  <View class="min-w-0 flex flex-row items-center justify-between gap-3">
                    <ButtonGroup
                      size="sm"
                      variant="ghost"
                      aria-label="Snapshot workspace"
                      class="flex-none"
                    >
                      <Button
                        size="sm"
                        variant="ghost"
                        selected={workspaceMode() === "browse"}
                        onClick={() => setWorkspaceMode("browse")}
                      >
                        <Icon source={folder} size={14} /> Browse
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        selected={workspaceMode() === "changes"}
                        onClick={() => setWorkspaceMode("changes")}
                      >
                        <Icon source={gitCompare} size={14} /> Changes
                      </Button>
                    </ButtonGroup>
                    <SnapshotDetails
                      snapshot={snapshot()}
                      onSave={(changes) => updateSnapshot(snapshot(), changes)}
                      onDelete={() => deleteSnapshot(snapshot())}
                    />
                    <Show when={workspaceMode() === "browse"}>
                      <ButtonGroup
                        size="sm"
                        variant="ghost"
                        aria-label="File view"
                        class="flex-none"
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
                    <AdaptiveSplitPane
                      compactAt={720}
                      aria-label="Snapshot file workspace"
                      class="min-w-0 min-h-0 flex-1"
                    >
                      <AdaptiveSplitPaneMain class="h-full">
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
                            <Table aria-label="Snapshot files">
                              <TableHeader>
                                <TableRow class="bg-surface-muted">
                                  <SortableTableHead
                                    label="Name"
                                    class="min-w-0 flex-1"
                                    direction={() => sortDirection("name")}
                                    onToggle={() =>
                                      fileTable.table
                                        .getColumn("name")
                                        ?.toggleSorting()
                                    }
                                  />
                                  <SortableTableHead
                                    label="Size"
                                    class="min-w-0 w-24 flex-none"
                                    direction={() => sortDirection("size")}
                                    onToggle={() =>
                                      fileTable.table
                                        .getColumn("size")
                                        ?.toggleSorting()
                                    }
                                  />
                                  <SortableTableHead
                                    label="Modified"
                                    class="min-w-0 w-36 flex-none"
                                    direction={() => sortDirection("modified")}
                                    onToggle={() =>
                                      fileTable.table
                                        .getColumn("modified")
                                        ?.toggleSorting()
                                    }
                                  />
                                </TableRow>
                              </TableHeader>
                              <TableBody>
                                <ForValue each={fileTable.rows()}>
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
                                </ForValue>
                              </TableBody>
                            </Table>
                            <Show
                              when={
                                !searchActive() && files().length < fileTotal()
                              }
                            >
                              <View class="w-full flex justify-center px-4 py-3">
                                <Button
                                  aria-label="Load more files"
                                  variant="outline"
                                  loading={loadingMoreFiles()}
                                  loadingLabel="Loading more…"
                                  onClick={() => void loadMoreFiles()}
                                >
                                  Load more files
                                </Button>
                              </View>
                            </Show>
                          </ScrollArea>
                        </Show>
                      </AdaptiveSplitPaneMain>
                      <Show when={selectedEntry()}>
                        <AdaptiveSplitPaneDetail
                          open={true}
                          aria-label="Selected file details"
                          class="w-72 h-full flex-none border-l border-subtle"
                          modalClass="w-96"
                          onOpenChange={(open) => {
                            if (!open) setSelectedEntry(undefined);
                          }}
                        >
                          <FileDetails
                            profileId={session.activeProfile()?.id ?? ""}
                            snapshotId={snapshot().id}
                            entry={selectedEntry()}
                          />
                        </AdaptiveSplitPaneDetail>
                      </Show>
                    </AdaptiveSplitPane>
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
