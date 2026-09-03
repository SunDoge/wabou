import {
  Button,
  Popover,
  PopoverDescription,
  PopoverHeader,
  PopoverTitle,
  ProgressFill,
  ProgressRoot,
  ProgressTrack,
  Text,
  View,
} from "@wabou/ui";
import { Show } from "solid-js";
import type { AgentSessionStats } from "./agent-state";
import { i18n, m } from "./i18n";

export function formatTokenCount(value: number): string {
  if (value < 1_000) return String(Math.round(value));
  if (value < 1_000_000) {
    return `${(value / 1_000).toFixed(value < 10_000 ? 1 : 0)}k`;
  }
  return `${(value / 1_000_000).toFixed(value < 10_000_000 ? 1 : 0)}m`;
}

function UsageRow(props: { label: string; value: string }) {
  return (
    <View class="min-w-0 flex flex-row items-center justify-between gap-4">
      <Text class="min-w-0 text-xs text-muted">{props.label}</Text>
      <Text class="flex-none text-xs font-medium text-primary">
        {props.value}
      </Text>
    </View>
  );
}

export function SessionUsage(props: { stats?: AgentSessionStats }) {
  const context = () => props.stats?.contextUsage;
  const percent = () => context()?.percent;
  return (
    <Show when={props.stats}>
      {(stats) => (
        <Popover
          aria-label={i18n.message(m.session_usage, {})}
          placement="top-end"
          contentClass="w-80"
          trigger={(trigger) => (
            <Button
              {...trigger}
              variant="ghost"
              size="sm"
              aria-label={i18n.message(m.session_usage, {})}
              class="min-w-0 h-7 px-2 gap-2 text-muted"
            >
              <Show
                when={
                  context() && percent() !== null && percent() !== undefined
                }
              >
                <Text class="flex-none text-xs text-muted">
                  {i18n.message(m.context_usage, {
                    percent: Math.round(percent() ?? 0),
                  })}
                </Text>
              </Show>
              <Text class="flex-none text-xs text-muted">
                {i18n.message(m.token_usage, {
                  tokens: formatTokenCount(stats().tokens.total),
                })}
              </Text>
              <Show when={stats().cost > 0}>
                <Text class="flex-none text-xs text-muted">
                  ${stats().cost.toFixed(3)}
                </Text>
              </Show>
            </Button>
          )}
        >
          <PopoverHeader>
            <PopoverTitle>{i18n.message(m.session_usage, {})}</PopoverTitle>
            <PopoverDescription>
              {i18n.message(m.session_usage_detail, {})}
            </PopoverDescription>
          </PopoverHeader>
          <Show
            when={context() && percent() !== null && percent() !== undefined}
          >
            <View class="min-w-0 flex flex-col gap-1.5">
              <View class="min-w-0 flex flex-row items-center justify-between gap-3">
                <Text class="text-xs font-medium text-primary">
                  {i18n.message(m.context_usage_label, {})}
                </Text>
                <Text class="flex-none text-xs text-muted">
                  {i18n.message(m.context_usage_amount, {
                    used: formatTokenCount(context()?.tokens ?? 0),
                    total: formatTokenCount(context()?.contextWindow ?? 0),
                    percent: Math.round(percent() ?? 0),
                  })}
                </Text>
              </View>
              <ProgressRoot
                label={i18n.message(m.context_usage_label, {})}
                value={percent() ?? 0}
              >
                <ProgressTrack class="h-1.5">
                  <ProgressFill />
                </ProgressTrack>
              </ProgressRoot>
            </View>
          </Show>
          <View class="min-w-0 flex flex-col gap-2 border-t border-subtle pt-3">
            <UsageRow
              label={i18n.message(m.input_tokens, {})}
              value={formatTokenCount(stats().tokens.input)}
            />
            <UsageRow
              label={i18n.message(m.output_tokens, {})}
              value={formatTokenCount(stats().tokens.output)}
            />
            <UsageRow
              label={i18n.message(m.cache_read_tokens, {})}
              value={formatTokenCount(stats().tokens.cacheRead)}
            />
            <UsageRow
              label={i18n.message(m.cache_write_tokens, {})}
              value={formatTokenCount(stats().tokens.cacheWrite)}
            />
          </View>
          <View class="min-w-0 flex flex-col gap-2 border-t border-subtle pt-3">
            <UsageRow
              label={i18n.message(m.session_messages, {})}
              value={String(stats().totalMessages)}
            />
            <UsageRow
              label={i18n.message(m.tool_calls, {})}
              value={String(stats().toolCalls)}
            />
            <UsageRow
              label={i18n.message(m.session_cost, {})}
              value={`$${stats().cost.toFixed(4)}`}
            />
          </View>
        </Popover>
      )}
    </Show>
  );
}
