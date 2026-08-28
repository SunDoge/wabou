import { Badge, Pulse, Text, View } from "@wabou/ui";
import { Show } from "solid-js";
import type { AgentViewState } from "./agent-state";
import { i18n, m } from "./i18n";

function activityLabel(state: AgentViewState): string | undefined {
  switch (state.activity?.kind) {
    case "responding":
      return i18n.message(m.activity_responding, {});
    case "compacting":
      return i18n.message(m.activity_compacting, {});
    case "summarizing":
      return i18n.message(m.activity_summarizing, {});
    case "retrying":
      return state.activity.attempt !== undefined &&
        state.activity.maxAttempts !== undefined
        ? i18n.message(m.activity_retrying_attempt, {
            attempt: state.activity.attempt,
            maximum: state.activity.maxAttempts,
          })
        : i18n.message(m.activity_retrying, {});
    default:
      return undefined;
  }
}

export function AgentActivityStatus(props: { state: AgentViewState }) {
  const label = () => activityLabel(props.state);
  const queued = () => props.state.queue.steering + props.state.queue.followUp;
  return (
    <View class="min-w-0 flex flex-row items-center gap-2">
      <Show when={label()}>
        {(text) => (
          <View
            role="status"
            class="min-w-0 flex flex-row items-center gap-1.5"
          >
            <Pulse
              aria-hidden="true"
              class="w-1.5 h-1.5 flex-none rounded-full bg-accent"
              from={0.3}
              to={1}
              duration={0.8}
            />
            <Text class="min-w-0 truncate text-xs text-muted">{text()}</Text>
          </View>
        )}
      </Show>
      <Show when={queued() > 0}>
        <Badge variant="secondary">
          {i18n.message(m.activity_queued, { count: queued() })}
        </Badge>
      </Show>
    </View>
  );
}
