import {
  Badge,
  Button,
  ContentState,
  Icon,
  PageHeader,
  ProjectionBoundary,
  ScrollArea,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Text,
  useNavigate,
  View,
} from "@wabou/ui";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import file from "lucide-static/icons/file.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import refreshCw from "lucide-static/icons/refresh-cw.svg?raw";
import { createEffect, createSignal, For, Show } from "solid-js";
import { type FileEntry, type SnapshotEntry, useRusticApi } from "./api";
import { useRusticSession } from "./session";
import { BackupSourcesPanel } from "./workspace-components";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
}

function shortId(id: string): string {
  return id.slice(0, 8);
}

export function SnapshotsPage() {
  const api = useRusticApi();
  const session = useRusticSession();
  const navigate = useNavigate();
  const [snapshots, setSnapshots] = createSignal<SnapshotEntry[]>([]);
  const [selected, setSelected] = createSignal<SnapshotEntry>();
  const [files, setFiles] = createSignal<FileEntry[]>([]);
  const [currentPath, setCurrentPath] = createSignal("");
  const [loading, setLoading] = createSignal(true);
  const [backingUp, setBackingUp] = createSignal(false);
  const [error, setError] = createSignal<string>();

  async function loadSnapshots(selectNewest = false) {
    if (!session.status().connected) return;
    setLoading(true);
    setError(undefined);
    try {
      const next = await api.listSnapshots();
      setSnapshots(next);
      if (selectNewest && next[0]) await selectSnapshot(next[0]);
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

  async function loadFiles(snapshot: SnapshotEntry, path: string) {
    setError(undefined);
    try {
      const next = await api.listFiles({ snapshotId: snapshot.id, path });
      setCurrentPath(path);
      setFiles(next);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function selectSnapshot(snapshot: SnapshotEntry) {
    setSelected(snapshot);
    await loadFiles(snapshot, "");
  }

  async function saveSources(sources: string[]) {
    try {
      const status = await api.setSources({ sources });
      session.setStatus(status);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function runBackup() {
    if (backingUp() || session.status().sources.length === 0) return;
    setBackingUp(true);
    setError(undefined);
    try {
      await api.runBackup();
      await loadSnapshots(true);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBackingUp(false);
    }
  }

  function parentPath(path: string): string {
    const parts = path.split(/[\\/]/).filter(Boolean);
    parts.pop();
    return parts.join("/");
  }

  createEffect(
    () => session.status().connected,
    (connected) => {
      if (!connected) {
        void navigate({ to: "/" });
        return;
      }
      void loadSnapshots(true);
    },
  );

  return (
    <View class="w-full h-full min-w-0 min-h-0 flex flex-col">
      <View class="flex-none px-6 py-5 border-b border-subtle bg-surface">
        <PageHeader
          title="Backup workspace"
          description={
            session.status().repositoryPath ?? "No repository selected"
          }
          actions={
            <>
              <Button variant="outline" onClick={() => void loadSnapshots()}>
                <Icon source={refreshCw} size={14} /> Refresh
              </Button>
              <Button
                disabled={session.status().sources.length === 0 || backingUp()}
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
        <ProjectionBoundary id="rustic-sidebar">
          <View class="w-72 min-h-0 flex-none flex flex-col rounded-xl border border-subtle bg-surface shadow-sm overflow-hidden">
            <BackupSourcesPanel
              sources={session.status().sources}
              disabled={backingUp()}
              onChange={(sources) => void saveSources(sources)}
            />
            <View class="flex-none px-4 py-3 border-b border-subtle">
              <Text class="font-semibold">Snapshots</Text>
            </View>
            <ScrollArea
              class="min-h-0 flex-1"
              contentClass="flex flex-col py-2"
            >
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
                      onClick={() => void selectSnapshot(snapshot)}
                    >
                      <View class="min-w-0 flex-1 flex flex-col items-start gap-0.5">
                        <Text class="font-medium">{shortId(snapshot.id)}</Text>
                        <Text class="w-full truncate text-xs text-muted">
                          {snapshot.time}
                        </Text>
                      </View>
                    </Button>
                  )}
                </For>
              </Show>
            </ScrollArea>
          </View>
        </ProjectionBoundary>

        <ProjectionBoundary id="rustic-file-browser">
          <View class="min-w-0 min-h-0 flex-1 flex flex-col rounded-xl border border-subtle bg-surface shadow-sm overflow-hidden">
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
                  <View class="h-14 flex-none px-4 flex flex-row items-center gap-3 border-b border-subtle">
                    <Button
                      size="icon"
                      variant="ghost"
                      disabled={!currentPath()}
                      aria-label="Open parent folder"
                      onClick={() =>
                        void loadFiles(snapshot(), parentPath(currentPath()))
                      }
                    >
                      <Icon source={chevronLeft} size={15} />
                    </Button>
                    <View class="min-w-0 flex-1 flex flex-col">
                      <Text class="font-semibold">
                        Snapshot {shortId(snapshot().id)}
                      </Text>
                      <Text class="truncate text-xs text-muted">
                        /{currentPath() || ""}
                      </Text>
                    </View>
                    <Badge variant="secondary">{files().length} items</Badge>
                  </View>
                  <ScrollArea class="min-h-0 flex-1" contentClass="min-w-full">
                    <Table contentClass="min-w-2xl">
                      <TableHeader>
                        <TableRow class="bg-surface-muted">
                          <TableHead class="min-w-64 flex-3">Name</TableHead>
                          <TableHead class="min-w-28 flex-1">Size</TableHead>
                          <TableHead class="min-w-48 flex-2">
                            Modified
                          </TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        <For each={files()}>
                          {(entry) => (
                            <TableRow
                              class={
                                entry.kind === "directory"
                                  ? "cursor-pointer"
                                  : undefined
                              }
                              onClick={() =>
                                entry.kind === "directory"
                                  ? void loadFiles(snapshot(), entry.path)
                                  : undefined
                              }
                            >
                              <TableCell class="min-w-64 flex-3 gap-2">
                                <Icon
                                  source={
                                    entry.kind === "directory" ? folder : file
                                  }
                                  size={15}
                                  class="flex-none text-muted"
                                />
                                <Text class="min-w-0 flex-1 truncate">
                                  {entry.name}
                                </Text>
                              </TableCell>
                              <TableCell class="min-w-28 flex-1 text-muted">
                                {entry.kind === "directory"
                                  ? "—"
                                  : formatBytes(entry.size)}
                              </TableCell>
                              <TableCell class="min-w-48 flex-2 text-muted">
                                {entry.modified ?? "—"}
                              </TableCell>
                            </TableRow>
                          )}
                        </For>
                      </TableBody>
                    </Table>
                  </ScrollArea>
                </>
              )}
            </Show>
          </View>
        </ProjectionBoundary>
      </View>
    </View>
  );
}
