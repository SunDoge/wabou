import {
  Badge,
  Bubble,
  BubbleContent,
  Button,
  CodeBlock,
  Icon,
  Message,
  MessageAvatar,
  MessageContent,
  MessageHeader,
  Text,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import terminal from "lucide-static/icons/terminal.svg?raw";
import user from "lucide-static/icons/user.svg?raw";
import { createSignal, Show } from "solid-js";
import type { AgentItem } from "./agent-state";

const TOOL_OUTPUT_PREVIEW_BYTES = 12_000;

function ToolCall(props: { item: Extract<AgentItem, { kind: "tool" }> }) {
  const [open, setOpen] = createSignal(props.item.state === "running");
  const preview = () => {
    const value = props.item.output;
    return value.length > TOOL_OUTPUT_PREVIEW_BYTES
      ? `… output truncated in UI …\n${value.slice(-TOOL_OUTPUT_PREVIEW_BYTES)}`
      : value;
  };
  return (
    <View class="w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface">
      <View class="h-9 px-3 flex items-center gap-2 bg-control">
        <Icon source={terminal} size={14} />
        <Button
          variant="ghost"
          size="sm"
          class="flex-1 min-w-0 justify-start"
          onClick={() => setOpen((value) => !value)}
        >
          <Text class="min-w-0 text-sm font-medium">{props.item.name}</Text>
        </Button>
        <Badge
          variant={props.item.state === "failed" ? "destructive" : "secondary"}
        >
          {props.item.state}
        </Badge>
      </View>
      <Show when={open()}>
        <CodeBlock
          code={props.item.input}
          language="json"
          class="border-0 rounded-none"
        />
      </Show>
      <Show when={open() && props.item.output}>
        <CodeBlock
          code={preview()}
          language="output"
          class="border-0 rounded-none border-t border-subtle"
        />
      </Show>
    </View>
  );
}

export function ConversationItem(props: { item: AgentItem }) {
  const messageText = () => (props.item.kind === "tool" ? "" : props.item.text);
  return (
    <Show
      when={props.item.kind !== "tool"}
      fallback={
        <ToolCall item={props.item as Extract<AgentItem, { kind: "tool" }>} />
      }
    >
      <Message align={props.item.kind === "user" ? "end" : "start"}>
        <MessageAvatar>
          <Icon source={props.item.kind === "user" ? user : bot} size={15} />
        </MessageAvatar>
        <MessageContent>
          <MessageHeader>
            {props.item.kind === "user"
              ? "You"
              : props.item.kind === "assistant"
                ? "Pi"
                : "System"}
          </MessageHeader>
          <Bubble
            variant={
              props.item.kind === "user"
                ? "default"
                : props.item.kind === "notice" && props.item.tone === "error"
                  ? "destructive"
                  : "outline"
            }
          >
            <BubbleContent>
              <Text class="whitespace-normal text-sm">{messageText()}</Text>
            </BubbleContent>
          </Bubble>
        </MessageContent>
      </Message>
    </Show>
  );
}
