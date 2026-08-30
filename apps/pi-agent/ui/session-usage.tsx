import {
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

export function SessionUsage(props: { stats?: AgentSessionStats }) {
  const context = () => props.stats?.contextUsage;
  const percent = () => context()?.percent;
  return (
    <Show when={props.stats}>
      {(stats) => (
        <View
          role="status"
          aria-label={i18n.message(m.session_usage, {})}
          class="min-w-0 h-7 px-2 rounded-md bg-control flex flex-row items-center gap-2"
        >
          <Show
            when={context() && percent() !== null && percent() !== undefined}
          >
            <Text class="flex-none text-xs text-muted">
              {i18n.message(m.context_usage, {
                percent: Math.round(percent() ?? 0),
              })}
            </Text>
            <ProgressRoot
              class="w-16 flex-none"
              label={i18n.message(m.context_usage_label, {})}
              value={percent() ?? 0}
            >
              <ProgressTrack class="h-1">
                <ProgressFill />
              </ProgressTrack>
            </ProgressRoot>
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
        </View>
      )}
    </Show>
  );
}
