import { number } from "@wabou/core/style";
import {
  Badge,
  Bubble,
  BubbleContent,
  Button,
  CodeBlock,
  createKeyframeAnimation,
  Icon,
  Markdown,
  Message,
  MessageAvatar,
  MessageContent,
  MessageHeader,
  Pulse,
  Text,
  translate2d,
  useReducedMotion,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import terminal from "lucide-static/icons/terminal.svg?raw";
import user from "lucide-static/icons/user.svg?raw";
import { createSignal, type JSX, Show } from "solid-js";
import { match } from "ts-pattern";
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
        <View class="relative w-5 h-5 flex-none flex items-center justify-center">
          <Icon source={terminal} size={14} />
          <Show when={props.item.state === "running"}>
            <Pulse
              aria-hidden="true"
              class="absolute right-0 bottom-0 w-1.5 h-1.5 rounded-full bg-accent"
              from={0.35}
              to={1}
              duration={0.9}
            />
          </Show>
        </View>
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

function MessageEntrance(props: { children: JSX.Element }) {
  const reducedMotion = useReducedMotion();
  const entrance = createKeyframeAnimation([0, 1], {
    duration: 0.18,
    ease: "easeOut",
    reducedMotion,
    reducedValue: 1,
  });
  const progress = () => entrance.value();
  return (
    <View
      data-motion="message-enter"
      style={{ opacity: number(progress()) }}
      transform={translate2d(0, (1 - progress()) * 5)}
    >
      {props.children}
    </View>
  );
}

export function ConversationItem(props: { item: AgentItem }) {
  const messageText = () => (props.item.kind === "tool" ? "" : props.item.text);
  const messageVariant = () =>
    match(props.item)
      .with({ kind: "assistant" }, () => "ghost" as const)
      .with({ kind: "user" }, () => "default" as const)
      .with({ kind: "notice", tone: "error" }, () => "destructive" as const)
      .otherwise(() => "outline" as const);
  return (
    <MessageEntrance>
      <Show
        when={props.item.kind !== "tool"}
        fallback={
          <ToolCall item={props.item as Extract<AgentItem, { kind: "tool" }>} />
        }
      >
        <Message align={props.item.kind === "user" ? "end" : "start"}>
          <Show when={props.item.kind === "user"}>
            <MessageAvatar class="self-start bg-selected">
              <Icon source={user} size={15} />
            </MessageAvatar>
          </Show>
          <MessageContent
            class={props.item.kind === "assistant" ? "gap-1" : undefined}
          >
            <Show
              when={props.item.kind === "assistant"}
              fallback={
                <MessageHeader>
                  {props.item.kind === "user" ? "You" : "System"}
                </MessageHeader>
              }
            >
              <View class="h-6 px-1 flex flex-row items-center gap-2">
                <View class="w-5 h-5 flex-none rounded-md bg-selected flex items-center justify-center text-accent">
                  <Icon source={bot} size={13} />
                </View>
                <Text class="text-xs font-semibold text-secondary">Pi</Text>
                <Show
                  when={props.item.kind === "assistant" && props.item.streaming}
                >
                  <View
                    role="status"
                    aria-label="Pi is writing"
                    class="flex flex-row items-center gap-1.5"
                  >
                    <Pulse
                      aria-hidden="true"
                      class="w-1.5 h-1.5 rounded-full bg-accent"
                      from={0.3}
                      to={1}
                      duration={0.8}
                    />
                    <Text class="text-xs text-muted">Writing</Text>
                  </View>
                </Show>
              </View>
            </Show>
            <Bubble variant={messageVariant()}>
              <BubbleContent>
                <Show
                  when={props.item.kind === "assistant"}
                  fallback={
                    <Text class="whitespace-normal text-sm">
                      {messageText()}
                    </Text>
                  }
                >
                  <Markdown
                    source={messageText()}
                    variant="conversation"
                    aria-label="Assistant response"
                  />
                </Show>
              </BubbleContent>
            </Bubble>
          </MessageContent>
        </Message>
      </Show>
    </MessageEntrance>
  );
}
