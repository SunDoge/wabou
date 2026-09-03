import { Icon, Text, type TreeNode, TreeView, View } from "@wabou/ui";
import folder from "lucide-static/icons/folder.svg?raw";
import file from "lucide-static/icons/file.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  Show,
  untrack,
} from "solid-js";
import { type FileEntry, useRusticApi } from "./api";

const ROOT_ID = "snapshot-root";

export function SnapshotFileTree(props: {
  profileId: string;
  snapshotId: string;
  selectedPath?: string;
  onSelect: (entry: FileEntry | undefined) => void;
}) {
  const api = useRusticApi();
  const [directories, setDirectories] = createSignal<
    Readonly<Record<string, readonly FileEntry[]>>
  >({});
  const [expandedIds, setExpandedIds] = createSignal<readonly string[]>([
    ROOT_ID,
  ]);
  const [loadingPaths, setLoadingPaths] = createSignal<readonly string[]>([]);
  const [error, setError] = createSignal<string>();

  const pathForId = (id: string) => (id === ROOT_ID ? "" : id);
  const idForPath = (path: string) => (path ? path : ROOT_ID);
  const loaded = (path: string) =>
    Object.prototype.hasOwnProperty.call(directories(), path);

  async function load(path: string) {
    if (loaded(path) || loadingPaths().includes(path)) return;
    setLoadingPaths((current) => [...current, path]);
    setError(undefined);
    try {
      const entries = await api.listFiles({
        profileId: props.profileId,
        snapshotId: props.snapshotId,
        path,
      });
      setDirectories((current) => ({
        ...current,
        [path]: entries,
      }));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoadingPaths((current) => current.filter((item) => item !== path));
    }
  }

  const nodes = createMemo<readonly TreeNode[]>(() => {
    const build = (path: string): readonly TreeNode[] | undefined => {
      const entries = directories()[path];
      if (entries === undefined) return undefined;
      return entries.map((entry) => ({
        id: entry.path,
        label: entry.name,
        hasChildren: entry.kind === "directory",
        children: entry.kind === "directory" ? build(entry.path) : undefined,
      }));
    };
    return [
      {
        id: ROOT_ID,
        label: "Snapshot root",
        hasChildren: true,
        children: build(""),
      },
    ];
  });

  const entriesByPath = createMemo(() => {
    const result = new Map<string, FileEntry>();
    for (const entries of Object.values(directories())) {
      for (const entry of entries) result.set(entry.path, entry);
    }
    return result;
  });

  createEffect(
    () => `${props.profileId}\u0000${props.snapshotId}`,
    () => {
      setDirectories({});
      setExpandedIds([ROOT_ID]);
      setLoadingPaths([]);
      setError(undefined);
      untrack(() => void load(""));
    },
  );

  function updateExpanded(next: readonly string[]) {
    const previous = expandedIds();
    setExpandedIds(next);
    for (const id of next) {
      if (!previous.includes(id)) void load(pathForId(id));
    }
  }

  return (
    <View class="min-w-0 flex flex-col gap-2">
      <View class="flex-none px-2 py-1 flex flex-row items-center justify-between">
        <Text class="text-xs font-medium text-muted">Snapshot files</Text>
        <Show when={loadingPaths().length > 0}>
          <Text class="text-xs text-muted">Loading…</Text>
        </Show>
      </View>
      <Show when={error()}>
        {(message) => (
          <Text class="px-4 pb-2 whitespace-normal text-xs text-danger-primary">
            {message()}
          </Text>
        )}
      </Show>
      <TreeView
        items={nodes()}
        aria-label="Snapshot files"
        expandedIds={expandedIds()}
        selectedId={props.selectedPath ? idForPath(props.selectedPath) : null}
        onExpandedChange={updateExpanded}
        onSelectedChange={(id) => {
          props.onSelect(
            id && id !== ROOT_ID ? entriesByPath().get(id) : undefined,
          );
        }}
        renderItem={(node) => (
          <View class="min-w-0 flex-1 flex flex-row items-center gap-2">
            <Icon
              source={
                node.id === ROOT_ID ||
                entriesByPath().get(node.id)?.kind === "directory"
                  ? folder
                  : file
              }
              size={14}
              class="flex-none text-muted"
            />
            <Text class="min-w-0 flex-1 truncate">{node.label}</Text>
          </View>
        )}
      />
    </View>
  );
}
