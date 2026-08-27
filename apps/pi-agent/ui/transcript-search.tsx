import {
  Button,
  type Handle,
  Icon,
  SearchField,
  Text,
  useMessageScroller,
  View,
} from "@wabou/ui";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import chevronUp from "lucide-static/icons/chevron-up.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { createEffect, createMemo, createSignal } from "solid-js";
import type { AgentItem } from "./agent-state";
import { i18n, m } from "./i18n";

function searchableText(item: AgentItem): string {
  switch (item.kind) {
    case "user":
      return `${item.text}\n${item.imageNames?.join("\n") ?? ""}\n${item.contextPaths?.join("\n") ?? ""}`;
    case "assistant":
      return `${item.text}\n${item.thinkingText ?? ""}`;
    case "tool":
      return `${item.name}\n${item.input}\n${item.output}`;
    case "notice":
      return item.text;
  }
}

export function findTranscriptItems(
  items: readonly AgentItem[],
  query: string,
): string[] {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return [];
  return items
    .filter((item) => searchableText(item).toLocaleLowerCase().includes(needle))
    .map((item) => item.id);
}

export interface TranscriptSearchProps {
  items: readonly AgentItem[];
  resolveItem(id: string): Handle | undefined;
  activeChanged(id: string | undefined): void;
  close(): void;
}

export function TranscriptSearch(props: TranscriptSearchProps) {
  const scroller = useMessageScroller();
  const [query, setQuery] = createSignal("");
  const [activeIndex, setActiveIndex] = createSignal(0);
  const matches = createMemo(() => findTranscriptItems(props.items, query()));

  const reveal = (index: number, found = matches()) => {
    if (found.length === 0) {
      setActiveIndex(0);
      props.activeChanged(undefined);
      return;
    }
    const normalized = ((index % found.length) + found.length) % found.length;
    const id = found[normalized];
    setActiveIndex(normalized);
    props.activeChanged(id);
    requestAnimationFrame(() => {
      const item = props.resolveItem(id);
      if (item) scroller.scrollIntoView(item, { margin: 18 });
    });
  };

  createEffect(matches, (found) => reveal(0, found));

  const close = () => {
    props.activeChanged(undefined);
    props.close();
  };

  return (
    <View
      role="group"
      aria-label={i18n.message(m.search_transcript, {})}
      class="flex-none px-4 py-2 border-b border-subtle bg-surface flex flex-row items-center gap-2"
    >
      <SearchField
        class="min-w-0 flex-1"
        aria-label={i18n.message(m.search_transcript, {})}
        placeholder={i18n.message(m.search_transcript, {})}
        value={query()}
        onValueChange={setQuery}
        clearLabel={i18n.message(m.clear_search, {})}
        onKeyDown={(event) => {
          if (event.key === "Escape") close();
          if (event.key === "Enter") {
            event.preventDefault();
            reveal(activeIndex() + ((event.mods & 1) === 0 ? 1 : -1));
          }
        }}
      />
      <Text class="w-12 flex-none text-center text-xs text-muted">
        {matches().length === 0
          ? "0 / 0"
          : `${activeIndex() + 1} / ${matches().length}`}
      </Text>
      <Button
        variant="ghost"
        size="icon"
        aria-label={i18n.message(m.previous_match, {})}
        disabled={matches().length === 0}
        onClick={() => reveal(activeIndex() - 1)}
      >
        <Icon source={chevronUp} size={14} />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        aria-label={i18n.message(m.next_match, {})}
        disabled={matches().length === 0}
        onClick={() => reveal(activeIndex() + 1)}
      >
        <Icon source={chevronDown} size={14} />
      </Button>
      <Button
        variant="ghost"
        size="icon"
        aria-label={i18n.message(m.close_search, {})}
        onClick={close}
      >
        <Icon source={x} size={14} />
      </Button>
    </View>
  );
}
