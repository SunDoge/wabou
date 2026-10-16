import {
  CommandList,
  createCommandListNavigation,
  type Handle,
  Icon,
  Input,
  Row,
  ScrollArea,
  Text,
  View,
} from "@wabou/ui";
import appWindow from "lucide-static/icons/app-window.svg?raw";
import arrowUpDown from "lucide-static/icons/arrow-up-down.svg?raw";
import cornerDownLeft from "lucide-static/icons/corner-down-left.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import { createMemo, createSignal, type JSX } from "solid-js";
import type { LauncherEntry } from "./generated/rust-bindings";
import { rankLauncherEntries } from "./ranking";

export interface LauncherSurfaceProps {
  entries: readonly LauncherEntry[];
  loading?: boolean;
  error?: unknown;
  busyId?: string;
  onLaunch: (id: string) => void;
  onDismiss: () => void;
  onRetry?: () => void;
}

export function LauncherSurface(props: LauncherSurfaceProps): JSX.Element {
  const [query, setQuery] = createSignal("");
  let input: Handle | undefined;
  const results = createMemo(() => rankLauncherEntries(props.entries, query()));
  const commands = createMemo(() =>
    results().map((entry) => ({
      ...entry,
      disabled: props.busyId !== undefined,
      shortcut: props.busyId === entry.id ? "Opening…" : undefined,
    })),
  );
  const navigation = createCommandListNavigation(commands, {
    onAction: props.onLaunch,
    onDismiss: props.onDismiss,
  });

  queueMicrotask(() => input?.focus());

  return (
    <View class="w-full h-full p-3 bg-transparent text-primary">
      <View
        role="dialog"
        aria-label="Application launcher"
        class="w-full h-full min-w-0 overflow-hidden flex flex-col rounded-2xl border border-strong bg-canvas shadow-xl"
      >
        <Row class="h-20 flex-none items-center gap-4 px-5 border-b border-subtle bg-surface">
          <View class="w-10 h-10 flex-none flex items-center justify-center rounded-xl border border-strong bg-control text-accent shadow-sm">
            <Icon source={search} size={21} />
          </View>
          <Input
            ref={(node) => {
              input = node;
            }}
            aria-label="Search applications"
            chrome="none"
            class="h-12 min-w-0 flex-1 px-0 text-xl font-medium bg-transparent"
            value={query()}
            placeholder="Search applications"
            onInput={(event) => setQuery(event.currentTarget.value)}
            onKeyDown={navigation.handleKeyDown}
          />
          <Text class="flex-none rounded-md border border-subtle bg-control px-2 py-1 text-xs text-muted">
            esc
          </Text>
        </Row>

        <Row class="h-10 flex-none items-center px-5">
          <Text class="text-xs font-semibold text-muted tracking-wide">
            {query().trim() ? "BEST MATCHES" : "APPLICATIONS"}
          </Text>
          <Text class="ml-auto text-xs text-muted">
            {props.loading
              ? "Indexing…"
              : `${results().length} shown · ${props.entries.length} indexed`}
          </Text>
        </Row>

        <ScrollArea
          aria-label="Launcher results viewport"
          class="min-h-0 flex-1 px-3"
          contentClass="pb-2"
          scrollbar={{ visibility: "auto", thickness: 6, margin: 2 }}
        >
          <CommandList
            aria-label="Application results"
            items={commands()}
            highlighted={navigation.highlighted()}
            loading={props.loading}
            loadingText="Indexing installed applications…"
            error={props.error}
            errorText="Could not read installed applications"
            retryLabel="Try again"
            onRetry={props.onRetry}
            emptyText={
              query().trim()
                ? `No applications match “${query().trim()}”.`
                : "No applications were found on this system."
            }
            class="gap-1.5"
            itemClass="min-h-12 px-3 py-2 rounded-xl"
            onHighlightChange={navigation.setHighlighted}
            onAction={navigation.select}
            renderLeading={() => (
              <View class="w-9 h-9 flex-none flex items-center justify-center rounded-lg border border-subtle bg-control text-secondary">
                <Icon source={appWindow} size={17} />
              </View>
            )}
          />
        </ScrollArea>

        <Row class="h-11 flex-none items-center gap-4 px-5 border-t border-subtle bg-surface-muted text-xs text-muted">
          <Row class="items-center gap-1.5">
            <Icon source={arrowUpDown} size={13} />
            <Text>Navigate</Text>
          </Row>
          <Row class="items-center gap-1.5">
            <Icon source={cornerDownLeft} size={13} />
            <Text>Open</Text>
          </Row>
          <Text class="ml-auto">Super Shift Space</Text>
        </Row>
      </View>
    </View>
  );
}
