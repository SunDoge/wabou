import { Badge, Icon, Pulse, Spinner, Text, View } from "@wabou/ui";
import circle from "lucide-static/icons/circle.svg?raw";
import circleAlert from "lucide-static/icons/circle-alert.svg?raw";
import { Match, Show, Switch } from "solid-js";
import type { AgentViewState } from "./agent-state";
import { i18n, m } from "./i18n";

export function agentActivityLabel(state: AgentViewState): string | undefined {
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
  const label = () => agentActivityLabel(props.state);
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

/** Quiet sidebar status: idle is silent, while exceptional states change shape. */
export function AgentSidebarStatus(props: { state: AgentViewState }) {
  const connection = () => props.state.connection;
  return (
    <Switch>
      <Match when={connection() === "running"}>
        <Spinner
          label={agentActivityLabel(props.state) ?? i18n.message(m.working, {})}
          class="w-3.5 h-3.5 text-accent"
        />
      </Match>
      <Match when={connection() === "failed"}>
        <View
          role="status"
          aria-label={i18n.message(m.agent_status_failed, {})}
          class="w-4 h-4 flex-none"
        >
          <Icon
            aria-hidden="true"
            source={circleAlert}
            size={14}
            class="text-danger-primary"
          />
        </View>
      </Match>
      <Match when={connection() === "stopped"}>
        <View
          role="status"
          aria-label={i18n.message(m.agent_status_stopped, {})}
          class="w-4 h-4 flex-none"
        >
          <Icon
            aria-hidden="true"
            source={circle}
            size={11}
            class="text-muted"
          />
        </View>
      </Match>
    </Switch>
  );
}
