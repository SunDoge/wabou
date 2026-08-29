import {
  Button,
  type Handle,
  Text,
  Tooltip,
  useMessageScroller,
  View,
} from "@wabou/ui";
import { For, Show } from "solid-js";
import type { AgentItem } from "./agent-state";

const PREVIEW_LIMIT = 72;

export interface ConversationTurn {
  id: string;
  index: number;
  prompt: string;
}

export function conversationTurns(
  items: readonly AgentItem[],
): readonly ConversationTurn[] {
  return items
    .flatMap((item) => {
      if (item.kind !== "user") return [];
      const prompt = item.text.replace(/\s+/g, " ").trim();
      return [
        {
          id: item.id,
          index: 0,
          prompt:
            prompt.length > PREVIEW_LIMIT
              ? `${prompt.slice(0, PREVIEW_LIMIT - 1).trimEnd()}…`
              : prompt || "Empty prompt",
        },
      ];
    })
    .map((turn, index) => ({ ...turn, index }));
}

export interface ConversationNavigatorProps {
  items: readonly AgentItem[];
  resolveItem(id: string): Handle | undefined;
}

/** A quiet, prompt-oriented rail for jumping through long conversations. */
export function ConversationNavigator(props: ConversationNavigatorProps) {
  const scroller = useMessageScroller();
  const turns = () => conversationTurns(props.items);

  const reveal = (id: string) => {
    const target = props.resolveItem(id);
    if (target) scroller.scrollIntoView(target, { margin: 24 });
  };

  return (
    <Show when={turns().length >= 2}>
      <View
        role="group"
        aria-label="Conversation turns"
        class="absolute z-20 right-2 top-4 bottom-14 w-8 flex flex-col items-center justify-center pointer-events-none"
      >
        <View class="max-h-full py-1 rounded-full border border-subtle bg-surface shadow-xs overflow-y-auto pointer-events-auto">
          <For each={turns()}>
            {(turn) => (
              <Tooltip
                placement="left"
                openDelay={240}
                contentClass="max-w-sm"
                trigger={(tooltip) => (
                  <Button
                    ref={tooltip.ref}
                    variant="ghost"
                    size="icon"
                    class="w-7 h-7 p-0 rounded-full"
                    aria-label={`Jump to turn ${turn.index + 1}: ${turn.prompt}`}
                    onPointerEnter={tooltip.onPointerEnter}
                    onPointerLeave={tooltip.onPointerLeave}
                    onFocus={tooltip.onFocus}
                    onBlur={tooltip.onBlur}
                    onKeyDown={tooltip.onKeyDown}
                    onClick={() => reveal(turn.id)}
                  >
                    <View
                      aria-hidden="true"
                      class="w-3 h-1 rounded-full bg-subtle"
                    />
                  </Button>
                )}
              >
                <Text class="text-xs text-primary whitespace-normal">
                  {turn.prompt}
                </Text>
              </Tooltip>
            )}
          </For>
        </View>
      </View>
    </Show>
  );
}
