import {
  Badge,
  Bubble,
  BubbleContent,
  Button,
  CollapsiblePresence,
  CopyButton,
  createKeyframeAnimation,
  type Handle,
  Icon,
  Markdown,
  Message,
  MessageActions,
  MessageContent,
  MessageGroup,
  MessageHeader,
  MessageScrollerItem,
  number,
  Pulse,
  Reasoning,
  ReasoningContent,
  ReasoningTrigger,
  Text,
  Tool,
  ToolContent,
  ToolHeader,
  ToolInput,
  ToolOutput,
  translate2d,
  useReducedMotion,
  View,
} from "@wabou/ui";
import check from "lucide-static/icons/check.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import copy from "lucide-static/icons/copy.svg?raw";
import fileCode from "lucide-static/icons/file-code-2.svg?raw";
import gitBranch from "lucide-static/icons/git-branch.svg?raw";
import image from "lucide-static/icons/image.svg?raw";
import terminal from "lucide-static/icons/terminal.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  For as ForValue,
  type JSX,
  Show,
  untrack,
} from "solid-js";
import { match } from "ts-pattern";
import type { AgentItem } from "./agent-state";
import { i18n, m } from "./i18n";

const TOOL_OUTPUT_PREVIEW_BYTES = 12_000;

type ToolItem = Extract<AgentItem, { kind: "tool" }>;
type ToolReasoning = { text: string; streaming: boolean };

export type ConversationEntry =
  | { id: string; kind: "item"; item: Exclude<AgentItem, ToolItem> }
  | {
      id: string;
      kind: "tools";
      items: readonly ToolItem[];
      reasoning?: ToolReasoning;
    };

function mergeReasoning(
  current: ToolReasoning | undefined,
  next: ToolReasoning | undefined,
): ToolReasoning | undefined {
  if (!next?.text.trim()) return current;
  if (!current?.text.trim()) return next;
  return {
    text: `${current.text.trimEnd()}\n\n${next.text.trimStart()}`,
    streaming: current.streaming || next.streaming,
  };
}

/** Fold adjacent tool events into the activity cluster produced by one turn. */
export function groupConversationItems(
  items: readonly AgentItem[],
): readonly ConversationEntry[] {
  const entries: ConversationEntry[] = [];
  let pendingReasoning: ToolReasoning | undefined;
  for (const [index, item] of items.entries()) {
    const previous = entries.at(-1);
    if (item.kind === "tool") {
      if (previous?.kind === "tools") {
        entries[entries.length - 1] = {
          ...previous,
          items: [...previous.items, item],
          reasoning: mergeReasoning(previous.reasoning, pendingReasoning),
        };
      } else {
        entries.push({
          id: `tools:${item.id}`,
          kind: "tools",
          items: [item],
          reasoning: pendingReasoning,
        });
      }
      pendingReasoning = undefined;
      continue;
    }
    if (item.kind === "assistant" && item.thinkingText) {
      const reasoning = {
        text: item.thinkingText,
        streaming: item.streaming === true,
      };
      if (previous?.kind === "tools") {
        entries[entries.length - 1] = {
          ...previous,
          reasoning: mergeReasoning(previous.reasoning, reasoning),
        };
        if (!item.text.trim()) continue;
        entries.push({
          id: item.id,
          kind: "item",
          item: { ...item, thinkingText: undefined },
        });
        continue;
      }
      if (items[index + 1]?.kind === "tool" && !item.text.trim()) {
        pendingReasoning = mergeReasoning(pendingReasoning, reasoning);
        continue;
      }
    }
    if (
      item.kind === "assistant" &&
      !item.text.trim() &&
      !item.thinkingText &&
      (previous?.kind === "tools" || items[index + 1]?.kind === "tool")
    ) {
      continue;
    }
    if (pendingReasoning) {
      entries.push({
        id: `reasoning:${item.id}`,
        kind: "item",
        item: {
          id: `reasoning:${item.id}`,
          kind: "assistant",
          text: "",
          thinkingText: pendingReasoning.text,
          streaming: pendingReasoning.streaming,
        },
      });
      pendingReasoning = undefined;
    }
    entries.push({ id: item.id, kind: "item", item });
  }
  if (pendingReasoning) {
    entries.push({
      id: "reasoning:tail",
      kind: "item",
      item: {
        id: "reasoning:tail",
        kind: "assistant",
        text: "",
        thinkingText: pendingReasoning.text,
        streaming: pendingReasoning.streaming,
      },
    });
  }
  return entries;
}

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

export function formatTurnDuration(durationMs: number): string {
  const seconds = Math.max(0, Math.round(durationMs / 1_000));
  if (seconds < 1) return "<1s";
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  return remainder === 0 ? `${minutes}m` : `${minutes}m ${remainder}s`;
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
    <Tool
      open={open()}
      onOpenChange={setOpen}
      role="group"
      aria-label={`${props.item.name} tool call`}
    >
      <ToolHeader
        title={props.item.name}
        summary={summarizeToolInput(props.item.input)}
        status={props.item.state}
        icon={terminal}
        aria-label={`${props.item.name}: ${summarizeToolInput(props.item.input)}`}
      />
      <ToolContent>
        <ToolInput
          code={props.item.input}
          language="input"
          copyable={false}
          codeClass="border-0"
        />
        <Show when={props.item.output}>
          <View class="min-w-0 max-h-64 overflow-y-auto">
            <ToolOutput
              code={preview()}
              language="output"
              error={props.item.state === "failed"}
              codeClass="border-0"
            />
          </View>
        </Show>
      </ToolContent>
    </Tool>
  );
}

export function ToolActivityGroup(props: {
  items: readonly ToolItem[];
  reasoning?: { text: string; streaming: boolean };
}) {
  const running = () => props.items.some((item) => item.state === "running");
  const initiallyRunning = untrack(running);
  const [open, setOpen] = createSignal(initiallyRunning);
  let wasRunning = initiallyRunning;
  createEffect(running, (isRunning) => {
    if (isRunning) setOpen(true);
    else if (wasRunning) setOpen(false);
    wasRunning = isRunning;
  });
  const label = () => {
    const count = props.items.length;
    const tools = i18n.message(
      count === 1 ? m.tool_call_one : m.tool_call_many,
      { count },
    );
    if (running()) return i18n.message(m.tool_activity_working, { tools });
    const duration = props.items.findLast(
      (item) => item.turnDurationMs !== undefined,
    )?.turnDurationMs;
    return duration === undefined
      ? i18n.message(m.tool_activity_worked, { tools })
      : i18n.message(m.tool_activity_worked_duration, {
          duration: formatTurnDuration(duration),
          tools,
        });
  };
  return (
    <View class="w-full min-w-0 flex flex-col gap-1">
      <View class="w-full min-w-0 flex flex-row items-center">
        <Button
          variant="ghost"
          size="sm"
          class="h-7 min-w-0 max-w-full flex-none px-1.5 gap-1.5 text-secondary"
          aria-label={label()}
          aria-expanded={open()}
          onClick={() => setOpen((value) => !value)}
        >
          <Icon
            source={chevronRight}
            size={11}
            class={open() ? "rotate-90 text-muted" : "text-muted"}
          />
          <Show when={running()}>
            <Pulse
              aria-hidden="true"
              class="w-1.5 h-1.5 rounded-full bg-accent"
              from={0.3}
              to={1}
              duration={0.8}
            />
          </Show>
          <Text class="min-w-0 truncate text-xs font-medium text-secondary">
            {label()}
          </Text>
        </Button>
      </View>
      <CollapsiblePresence
        open={open()}
        duration={0.16}
        contentClass="min-w-0 ml-2 pl-3 pb-1 border-l border-subtle gap-2"
      >
        <Show when={props.reasoning}>
          {(reasoning) => (
            <ConversationReasoning
              text={reasoning().text}
              streaming={reasoning().streaming}
            />
          )}
        </Show>
        <ForValue each={props.items} keyed={(item) => item.id}>
          {(item) => <ToolCall item={item()} />}
        </ForValue>
      </CollapsiblePresence>
    </View>
  );
}

function ConversationEntryContent(props: {
  entry: ConversationEntry;
  animate?: boolean;
  fork?: (item: Extract<AgentItem, { kind: "user" }>) => void;
}) {
  const item = () => {
    const current = props.entry;
    if (current.kind !== "item") throw new Error("expected message entry");
    return current.item;
  };
  const tools = () => {
    const current = props.entry;
    if (current.kind !== "tools") throw new Error("expected tool entry");
    return current.items;
  };
  const reasoning = () => {
    const current = props.entry;
    return current.kind === "tools" ? current.reasoning : undefined;
  };
  const canFork = () => {
    const current = item();
    return current.kind === "user" && Boolean(current.entryId);
  };
  const fork = () => {
    const current = item();
    if (current.kind === "user" && current.entryId) props.fork?.(current);
  };
  return (
    <Show
      when={props.entry.kind === "tools"}
      fallback={
        <ConversationItem
          item={item()}
          animate={props.animate}
          fork={canFork() ? fork : undefined}
        />
      }
    >
      <ToolActivityGroup items={tools()} reasoning={reasoning()} />
    </Show>
  );
}

function ConversationReasoning(props: { text: string; streaming: boolean }) {
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
    <Reasoning open={open()} onOpenChange={setOpen}>
      <ReasoningTrigger streaming={props.streaming} />
      <ReasoningContent>
        <Markdown
          source={props.text}
          variant="conversation"
          aria-label="Model reasoning"
          class="gap-2"
        />
      </ReasoningContent>
    </Reasoning>
  );
}

function MessageEntrance(props: { animate: boolean; children: JSX.Element }) {
  if (!untrack(() => props.animate)) {
    return <View>{props.children}</View>;
  }
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
  animate?: boolean;
  fork?: () => void;
}) {
  const messageText = () => (props.item.kind === "tool" ? "" : props.item.text);
  const messageStreaming = () =>
    props.item.kind === "assistant" && props.item.streaming === true;
  const messageVariant = () =>
    match(props.item)
      .with({ kind: "assistant" }, () => "ghost" as const)
      .with({ kind: "user" }, () => "secondary" as const)
      .with({ kind: "notice", tone: "error" }, () => "destructive" as const)
      .otherwise(() => "outline" as const);
  return (
    <MessageEntrance animate={props.animate !== false}>
      <Show
        when={props.item.kind !== "tool"}
        fallback={
          <ToolCall item={props.item as Extract<AgentItem, { kind: "tool" }>} />
        }
      >
        <Message align={props.item.kind === "user" ? "end" : "start"}>
          <MessageContent
            class={
              props.item.kind === "assistant"
                ? "items-stretch gap-2"
                : undefined
            }
          >
            <Show when={props.item.kind === "notice"}>
              <MessageHeader>System</MessageHeader>
            </Show>
            <Show when={props.item.kind === "user" && props.item.queued}>
              <MessageHeader>
                <View role="status" aria-label="Queued follow-up">
                  <Badge variant="secondary">Queued</Badge>
                </View>
              </MessageHeader>
            </Show>
            <Show
              when={props.item.kind === "assistant" && props.item.streaming}
            >
              <View
                role="status"
                aria-label="Pi is writing"
                class="h-6 px-1 flex flex-row items-center gap-1.5"
              >
                <Pulse
                  aria-hidden="true"
                  class="w-1.5 h-1.5 rounded-full bg-accent"
                  from={0.3}
                  to={1}
                  duration={0.8}
                />
                <Text class="text-xs text-muted">Pi is writing</Text>
              </View>
            </Show>
            <Show
              when={props.item.kind === "assistant" && props.item.thinkingText}
            >
              <ConversationReasoning
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
            <Bubble
              variant={messageVariant()}
              class={props.item.kind === "assistant" ? "w-full" : undefined}
            >
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
                    <Show
                      when={props.item.kind === "user"}
                      fallback={
                        <Text class="whitespace-normal text-sm">
                          {messageText()}
                        </Text>
                      }
                    >
                      <Markdown
                        source={messageText()}
                        variant="prompt"
                        aria-label="User message"
                      />
                    </Show>
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
            <Show
              when={
                messageText() &&
                (props.item.kind === "assistant" || props.item.kind === "user")
              }
            >
              <MessageActions
                align="end"
                visibility="interaction"
                aria-label={
                  props.item.kind === "user"
                    ? "User message actions"
                    : "Assistant response actions"
                }
              >
                <CopyButton
                  value={messageText()}
                  variant="ghost"
                  size="icon"
                  class="w-7 h-7 text-muted"
                  idleLabel="Copy"
                  copiedLabel="Copied"
                  aria-label={
                    props.item.kind === "user"
                      ? "Copy user message"
                      : "Copy assistant response"
                  }
                  copiedChildren={<Icon source={check} size={13} />}
                >
                  <Icon source={copy} size={13} />
                </CopyButton>
                <Show when={props.item.kind === "user" && props.fork}>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="w-7 h-7 text-muted"
                    aria-label="Fork from this message"
                    onClick={() => props.fork?.()}
                  >
                    <Icon source={gitBranch} size={13} />
                  </Button>
                </Show>
              </MessageActions>
            </Show>
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
  const entries = createMemo(() => groupConversationItems(props.items));
  const initialEntryIds = new Set(
    untrack(() => entries().map((entry) => entry.id)),
  );
  return (
    <MessageGroup class="gap-5">
      <ForValue each={entries()} keyed={(entry) => entry.id}>
        {(entry) => {
          const register = (node: Handle) => {
            const current = entry();
            if (current.kind === "tools") {
              for (const item of current.items) {
                props.registerItem?.(item.id, node);
              }
            } else {
              props.registerItem?.(current.item.id, node);
            }
          };
          const highlighted = () => {
            const current = entry();
            return current.kind === "tools"
              ? current.items.some((item) => item.id === props.activeSearchItem)
              : current.item.id === props.activeSearchItem;
          };
          const anchor = () => {
            const current = entry();
            return current.kind !== "tools" && current.item.kind === "user"
              ? current.item.id
              : undefined;
          };
          return (
            <MessageScrollerItem
              ref={register}
              anchor={anchor()}
              class={highlighted() ? "rounded-lg bg-selected" : undefined}
            >
              <ConversationEntryContent
                entry={entry()}
                animate={!initialEntryIds.has(entry().id)}
                fork={props.fork}
              />
            </MessageScrollerItem>
          );
        }}
      </ForValue>
    </MessageGroup>
  );
}
