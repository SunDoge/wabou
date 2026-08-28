import {
  Badge,
  Bubble,
  BubbleContent,
  Button,
  CodeBlock,
  CollapsiblePresence,
  CopyButton,
  createKeyframeAnimation,
  type Handle,
  Icon,
  Markdown,
  Message,
  MessageAvatar,
  MessageContent,
  MessageGroup,
  MessageHeader,
  MessageScrollerItem,
  number,
  Pulse,
  Text,
  translate2d,
  useReducedMotion,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import fileCode from "lucide-static/icons/file-code-2.svg?raw";
import gitBranch from "lucide-static/icons/git-branch.svg?raw";
import image from "lucide-static/icons/image.svg?raw";
import terminal from "lucide-static/icons/terminal.svg?raw";
import user from "lucide-static/icons/user.svg?raw";
import {
  createEffect,
  createSignal,
  For as ForValue,
  type JSX,
  Show,
  untrack,
} from "solid-js";
import { match } from "ts-pattern";
import type { AgentItem } from "./agent-state";

const TOOL_OUTPUT_PREVIEW_BYTES = 12_000;

export function summarizeToolInput(input: string): string {
  try {
    const value = JSON.parse(input) as Record<string, unknown>;
    for (const key of [
      "command",
      "path",
      "file_path",
      "filename",
      "query",
      "url",
    ]) {
      const candidate = value[key];
      if (typeof candidate === "string" && candidate.trim()) {
        const firstLine = candidate.trim().split("\n", 1)[0] ?? "";
        return firstLine.length > 88 ? `${firstLine.slice(0, 85)}…` : firstLine;
      }
    }
  } catch {
    // The raw input remains available when a third-party tool does not use JSON.
  }
  const compact = input.trim().replace(/\s+/g, " ");
  return compact.length > 88 ? `${compact.slice(0, 85)}…` : compact;
}

function ToolCall(props: { item: Extract<AgentItem, { kind: "tool" }> }) {
  const initiallyRunning = untrack(() => props.item.state === "running");
  const [open, setOpen] = createSignal(initiallyRunning);
  let wasRunning = initiallyRunning;
  createEffect(
    () => props.item.state === "running",
    (running) => {
      if (running) setOpen(true);
      else if (wasRunning) setOpen(false);
      wasRunning = running;
    },
  );
  const preview = () => {
    const value = props.item.output;
    return value.length > TOOL_OUTPUT_PREVIEW_BYTES
      ? `… output truncated in UI …\n${value.slice(-TOOL_OUTPUT_PREVIEW_BYTES)}`
      : value;
  };
  return (
    <View class="w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface shadow-xs">
      <View class="min-h-10 px-3 flex items-center gap-2 bg-control">
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
          class="flex-1 min-w-0 justify-start gap-2"
          aria-label={`${props.item.name}: ${summarizeToolInput(props.item.input)}`}
          aria-expanded={open()}
          onClick={() => setOpen((value) => !value)}
        >
          <Icon
            source={chevronRight}
            size={12}
            class={open() ? "rotate-90 text-muted" : "text-muted"}
          />
          <Text class="flex-none text-sm font-semibold text-primary">
            {props.item.name}
          </Text>
          <Text class="min-w-0 flex-1 truncate text-left text-xs text-muted">
            {summarizeToolInput(props.item.input)}
          </Text>
        </Button>
        <Badge
          variant={props.item.state === "failed" ? "destructive" : "secondary"}
        >
          {props.item.state}
        </Badge>
      </View>
      <CollapsiblePresence
        open={open()}
        duration={0.16}
        contentClass="min-w-0 border-t border-subtle"
      >
        <CodeBlock
          code={props.item.input}
          language="input"
          copyable={false}
          class="border-0 rounded-none"
        />
        <Show when={props.item.output}>
          <View class="min-w-0 max-h-64 overflow-y-auto border-t border-subtle">
            <CodeBlock
              code={preview()}
              language="output"
              class="border-0 rounded-none"
            />
          </View>
        </Show>
      </CollapsiblePresence>
    </View>
  );
}

function Reasoning(props: { text: string; streaming: boolean }) {
  const initiallyStreaming = untrack(() => props.streaming);
  const [open, setOpen] = createSignal(initiallyStreaming);
  let wasStreaming = initiallyStreaming;
  createEffect(
    () => props.streaming,
    (streaming) => {
      if (streaming) setOpen(true);
      else if (wasStreaming) setOpen(false);
      wasStreaming = streaming;
    },
  );
  return (
    <View class="w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface-muted">
      <Button
        variant="ghost"
        size="sm"
        class="w-full min-w-0 justify-start gap-2 px-3"
        aria-expanded={open()}
        onClick={() => setOpen((value) => !value)}
      >
        <Icon
          source={chevronRight}
          size={13}
          class={open() ? "rotate-90 text-muted" : "text-muted"}
        />
        <Text class="min-w-0 flex-1 text-left text-xs font-medium text-secondary">
          {props.streaming ? "Thinking" : "Reasoning"}
        </Text>
        <Show when={props.streaming}>
          <Pulse
            aria-hidden="true"
            class="w-1.5 h-1.5 rounded-full bg-accent"
            from={0.3}
            to={1}
            duration={0.8}
          />
        </Show>
      </Button>
      <CollapsiblePresence
        open={open()}
        duration={0.16}
        contentClass="min-w-0 border-t border-subtle px-3 py-2"
      >
        <Markdown
          source={props.text}
          variant="conversation"
          aria-label="Model reasoning"
          class="gap-2"
        />
      </CollapsiblePresence>
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

export function ConversationItem(props: {
  item: AgentItem;
  fork?: () => void;
}) {
  const messageText = () => (props.item.kind === "tool" ? "" : props.item.text);
  const messageStreaming = () =>
    props.item.kind === "assistant" && props.item.streaming === true;
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
            class={props.item.kind === "assistant" ? "gap-2" : undefined}
          >
            <Show
              when={props.item.kind === "assistant"}
              fallback={
                <View class="flex flex-row items-center justify-between gap-2">
                  <View class="flex flex-row items-center gap-2">
                    <MessageHeader>
                      {props.item.kind === "user" ? "You" : "System"}
                    </MessageHeader>
                    <Show
                      when={props.item.kind === "user" && props.item.queued}
                    >
                      <View role="status" aria-label="Queued follow-up">
                        <Badge variant="secondary">Queued</Badge>
                      </View>
                    </Show>
                  </View>
                  <CopyButton
                    value={messageText()}
                    variant="ghost"
                    size="sm"
                    idleLabel="Copy"
                    copiedLabel="Copied"
                    aria-label="Copy user message"
                  />
                  <Show when={props.item.kind === "user" && props.fork}>
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label="Fork from this message"
                      onClick={() => props.fork?.()}
                    >
                      <Icon source={gitBranch} size={13} />
                    </Button>
                  </Show>
                </View>
              }
            >
              <View class="h-7 w-full px-2 flex flex-row items-center justify-between gap-2">
                <View class="min-w-0 flex flex-row items-center gap-2">
                  <View class="w-5 h-5 flex-none rounded-md bg-selected flex items-center justify-center text-accent">
                    <Icon source={bot} size={13} />
                  </View>
                  <Text class="text-xs font-semibold text-secondary">Pi</Text>
                  <Show
                    when={
                      props.item.kind === "assistant" && props.item.streaming
                    }
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
                <CopyButton
                  value={messageText()}
                  variant="ghost"
                  size="sm"
                  idleLabel="Copy"
                  copiedLabel="Copied"
                  aria-label="Copy assistant response"
                />
              </View>
            </Show>
            <Show
              when={props.item.kind === "assistant" && props.item.thinkingText}
            >
              <Reasoning
                text={
                  props.item.kind === "assistant"
                    ? (props.item.thinkingText ?? "")
                    : ""
                }
                streaming={
                  props.item.kind === "assistant" &&
                  props.item.streaming === true
                }
              />
            </Show>
            <Bubble variant={messageVariant()}>
              <BubbleContent
                class={
                  props.item.kind === "assistant"
                    ? "w-full px-2 pb-3"
                    : undefined
                }
              >
                <Show
                  when={
                    props.item.kind === "user" && props.item.imageNames?.length
                  }
                >
                  <View
                    role="group"
                    aria-label="Attached images"
                    class="mb-2 flex flex-row flex-wrap gap-1.5"
                  >
                    <ForValue
                      each={
                        props.item.kind === "user" ? props.item.imageNames : []
                      }
                    >
                      {(name) => (
                        <Badge variant="secondary">
                          <Icon source={image} size={12} /> {name}
                        </Badge>
                      )}
                    </ForValue>
                  </View>
                </Show>
                <Show
                  when={
                    props.item.kind === "user" &&
                    props.item.contextPaths?.length
                  }
                >
                  <View
                    role="group"
                    aria-label="Context files"
                    class="mb-2 flex flex-row flex-wrap gap-1.5"
                  >
                    <ForValue
                      each={
                        props.item.kind === "user"
                          ? props.item.contextPaths
                          : []
                      }
                    >
                      {(path) => (
                        <Badge variant="outline">
                          <Icon source={fileCode} size={12} /> {path}
                        </Badge>
                      )}
                    </ForValue>
                  </View>
                </Show>
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
                    streaming={messageStreaming()}
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

/** Keep streamed message components mounted by semantic item id. */
export function ConversationList(props: {
  items: readonly AgentItem[];
  activeSearchItem?: string;
  registerItem?: (id: string, node: Handle) => void;
  fork?: (item: Extract<AgentItem, { kind: "user" }>) => void;
}) {
  return (
    <MessageGroup class="gap-5">
      <ForValue each={props.items} keyed={(item) => item.id}>
        {(item) => {
          const canFork = () => {
            const current = item();
            return current.kind === "user" && Boolean(current.entryId);
          };
          const fork = () => {
            const current = item();
            if (current.kind === "user" && current.entryId) {
              props.fork?.(current);
            }
          };
          return (
            <MessageScrollerItem
              ref={(node) => props.registerItem?.(item().id, node)}
              class={
                props.activeSearchItem === item().id
                  ? "rounded-lg bg-selected"
                  : undefined
              }
            >
              <ConversationItem
                item={item()}
                fork={canFork() ? fork : undefined}
              />
            </MessageScrollerItem>
          );
        }}
      </ForValue>
    </MessageGroup>
  );
}
