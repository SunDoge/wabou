import { Icon, Text, type TreeNode, TreeView, View } from "@wabou/ui";
import file from "lucide-static/icons/file.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  Show,
  untrack,
} from "solid-js";
import { FILE_PAGE_SIZE, type FileEntry, useRusticApi } from "./api";

const ROOT_ID = "snapshot-root";
const LOAD_MORE_PREFIX = "timestow-load-more:";

interface DirectoryListing {
  entries: readonly FileEntry[];
  total: number;
}

export function SnapshotFileTree(props: {
  profileId: string;
  snapshotId: string;
  selectedPath?: string;
  onSelect: (entry: FileEntry | undefined) => void;
}) {
  const api = useRusticApi();
  const [directories, setDirectories] = createSignal<
    Readonly<Record<string, DirectoryListing>>
  >({});
  const [expandedIds, setExpandedIds] = createSignal<readonly string[]>([
    ROOT_ID,
  ]);
  const [loadingPaths, setLoadingPaths] = createSignal<readonly string[]>([]);
  const [error, setError] = createSignal<string>();
  let generation = 0;

  const pathForId = (id: string) => (id === ROOT_ID ? "" : id);
  const idForPath = (path: string) => (path ? path : ROOT_ID);
  const loaded = (path: string) => Object.hasOwn(directories(), path);

  async function load(path: string, append = false) {
    if ((!append && loaded(path)) || loadingPaths().includes(path)) return;
    const requestGeneration = generation;
    const offset = append ? (directories()[path]?.entries.length ?? 0) : 0;
    setLoadingPaths((current) => [...current, path]);
    setError(undefined);
    try {
      const listing = await api.listFiles({
        profileId: props.profileId,
        snapshotId: props.snapshotId,
        path,
        offset,
        limit: FILE_PAGE_SIZE,
      });
      if (requestGeneration !== generation) return;
      setDirectories((current) => ({
        ...current,
        [path]: {
          entries: append
            ? [...(current[path]?.entries ?? []), ...listing.entries]
            : listing.entries,
          total: listing.total,
        },
      }));
    } catch (cause) {
      if (requestGeneration === generation) {
        setError(cause instanceof Error ? cause.message : String(cause));
      }
    } finally {
      if (requestGeneration === generation) {
        setLoadingPaths((current) => current.filter((item) => item !== path));
      }
    }
  }

  const nodes = createMemo<readonly TreeNode[]>(() => {
    const build = (path: string): readonly TreeNode[] | undefined => {
      const listing = directories()[path];
      if (listing === undefined) return undefined;
      const entries: TreeNode[] = listing.entries.map((entry) => ({
        id: entry.path,
        label: entry.name,
        hasChildren: entry.kind === "directory",
        children: entry.kind === "directory" ? build(entry.path) : undefined,
      }));
      if (listing.entries.length < listing.total) {
        entries.push({
          id: `${LOAD_MORE_PREFIX}${encodeURIComponent(path)}`,
          label: `Load more (${listing.total - listing.entries.length} remaining)`,
          hasChildren: false,
        });
      }
      return entries;
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
    for (const listing of Object.values(directories())) {
      for (const entry of listing.entries) result.set(entry.path, entry);
    }
    return result;
  });

  createEffect(
    () => `${props.profileId}\u0000${props.snapshotId}`,
    () => {
      generation += 1;
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

  function iconForNode(id: string): string {
    if (id.startsWith(LOAD_MORE_PREFIX)) return plus;
    if (id === ROOT_ID || entriesByPath().get(id)?.kind === "directory") {
      return folder;
    }
    return file;
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
          if (id?.startsWith(LOAD_MORE_PREFIX)) {
            void load(
              decodeURIComponent(id.slice(LOAD_MORE_PREFIX.length)),
              true,
            );
            return;
          }
          props.onSelect(
            id && id !== ROOT_ID ? entriesByPath().get(id) : undefined,
          );
        }}
        renderItem={(node) => (
          <View class="min-w-0 flex-1 flex flex-row items-center gap-2">
            <Icon
              source={iconForNode(node.id)}
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
