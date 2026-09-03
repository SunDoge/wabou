import { MessageScrollerNavigator } from "@wabou/ui";
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
}

/** A quiet, prompt-oriented rail for jumping through long conversations. */
export function ConversationNavigator(props: ConversationNavigatorProps) {
  const turns = () => conversationTurns(props.items);
  return (
    <MessageScrollerNavigator
      items={turns().map((turn) => ({ id: turn.id, label: turn.prompt }))}
      aria-label="Conversation turns"
      itemAriaLabel={(item, index) =>
        `Jump to turn ${index + 1}: ${item.label}`
      }
    />
  );
}
