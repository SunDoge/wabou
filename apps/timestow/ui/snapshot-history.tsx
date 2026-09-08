import {
  Button,
  ContentState,
  Icon,
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  ProjectionBoundary,
  Text,
  View,
  VirtualList,
} from "@wabou/ui";
import search from "lucide-static/icons/search.svg?raw";
import shieldCheck from "lucide-static/icons/shield-check.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { createMemo, Match, Show, Switch } from "solid-js";
import type { SnapshotEntry } from "./api";
import { formatTimestamp } from "./format";

const SNAPSHOT_HISTORY_ROW_HEIGHT = 60;

function shortId(id: string): string {
  return id.slice(0, 8);
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
    formatTimestamp(snapshot.time),
    ...snapshot.tags,
  ]
    .filter((value): value is string => Boolean(value))
    .some((value) => value.toLocaleLowerCase().includes(normalized));
}

export function snapshotDisplayTitle(snapshot: SnapshotEntry): string {
  return snapshot.label.trim() || `Snapshot ${shortId(snapshot.id)}`;
}

export function snapshotHistoryMetadata(snapshot: SnapshotEntry): string {
  return `${formatTimestamp(snapshot.time)} · ${snapshot.hostname || "Unknown host"}`;
}

export function SnapshotHistory(props: {
  loading: boolean;
  loadFailed: boolean;
  snapshots: readonly SnapshotEntry[];
  selectedId?: string;
  query: string;
  onQueryChange(query: string): void;
  onSelect(snapshot: SnapshotEntry): void;
}) {
  const filtered = createMemo(() =>
    props.snapshots.filter((snapshot) =>
      snapshotMatchesQuery(snapshot, props.query),
    ),
  );
  const searchable = () => props.snapshots.length >= 8;
  const updateQuery = (query: string) => props.onQueryChange(query);

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
              onInput={(event) => updateQuery(event.currentTarget.value)}
            />
            <Show when={props.query.trim()}>
              <InputGroupAddon align="inline-end" class="px-1.5">
                <Button
                  size="icon"
                  variant="ghost"
                  class="w-6 h-6"
                  aria-label="Clear snapshot filter"
                  onClick={() => updateQuery("")}
                >
                  <Icon source={x} size={12} />
                </Button>
              </InputGroupAddon>
            </Show>
          </InputGroup>
        </Show>
      </View>
      <View class="min-h-0 flex-1 flex flex-col">
        <Switch>
          <Match when={props.loading}>
            <ContentState
              state="loading"
              title="Loading snapshots"
              description="Reading this backup’s history…"
              class="border-0 shadow-none"
            />
          </Match>
          <Match when={props.loadFailed}>
            <ContentState
              state="error"
              title="History unavailable"
              description="Refresh after resolving the repository error."
              class="border-0 shadow-none"
            />
          </Match>
          <Match when={props.snapshots.length === 0}>
            <ContentState
              state="empty"
              title="No snapshots yet"
              description="Run your first backup to create a snapshot."
              class="border-0 shadow-none"
            />
          </Match>
          <Match when={filtered().length === 0}>
            <ContentState
              state="empty"
              title="No matching snapshots"
              description="Try a label, date, host, tag, or snapshot ID."
              action={{
                label: "Clear filter",
                onAction: () => updateQuery(""),
              }}
              class="border-0 shadow-none"
            />
          </Match>
          <Match when>
            <VirtualList
              items={filtered}
              itemHeight={SNAPSHOT_HISTORY_ROW_HEIGHT}
              getItemKey={(snapshot) => snapshot.id}
              role="listbox"
              accessibilityLabel="Snapshots"
              class="min-h-0 flex-1 p-2"
            >
              {(snapshot) => {
                const title = createMemo(() =>
                  snapshotDisplayTitle(snapshot()),
                );
                const metadata = createMemo(() =>
                  snapshotHistoryMetadata(snapshot()),
                );
                const selected = createMemo(
                  () => props.selectedId === snapshot().id,
                );
                return (
                  <Button
                    aria-label={`Open snapshot ${title()}`}
                    variant="ghost"
                    selected={selected()}
                    class="w-full h-14 min-h-14 justify-start px-3"
                    onClick={() => props.onSelect(snapshot())}
                  >
                    <View class="min-w-0 flex-1 flex flex-col items-start gap-0.5">
                      <Text class="w-full truncate font-medium">{title()}</Text>
                      <View class="w-full min-w-0 flex flex-row items-center gap-1.5">
                        <Show when={snapshot().deleteProtected}>
                          <Icon
                            source={shieldCheck}
                            size={12}
                            class="flex-none text-success-primary"
                          />
                        </Show>
                        <Text
                          class={
                            selected()
                              ? "min-w-0 flex-1 truncate text-xs text-secondary"
                              : "min-w-0 flex-1 truncate text-xs text-muted"
                          }
                        >
                          {metadata()}
                        </Text>
                      </View>
                    </View>
                  </Button>
                );
              }}
            </VirtualList>
          </Match>
        </Switch>
      </View>
    </ProjectionBoundary>
  );
}
