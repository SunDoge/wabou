import {
  Field,
  FieldDescription,
  FieldLabel,
  Select,
  Switch,
  View,
} from "@wabou/ui";
import type { AgentQueueMode, AgentViewState } from "./agent-state";
import { i18n, m } from "./i18n";

const queueModeOptions = () => [
  { value: "one-at-a-time", label: i18n.message(m.queue_mode_one, {}) },
  { value: "all", label: i18n.message(m.queue_mode_all, {}) },
];

export function SessionBehaviorSettings(props: {
  state: AgentViewState;
  setAutoCompaction(enabled: boolean): void;
  setSteeringMode(mode: AgentQueueMode): void;
  setFollowUpMode(mode: AgentQueueMode): void;
}) {
  const disabled = () => props.state.connection !== "ready";
  return (
    <View class="min-w-0 flex flex-col gap-4">
      <Field>
        <Switch
          aria-label={i18n.message(m.auto_compaction, {})}
          label={i18n.message(m.auto_compaction, {})}
          checked={props.state.autoCompactionEnabled ?? false}
          disabled={
            disabled() || props.state.autoCompactionEnabled === undefined
          }
          onCheckedChange={props.setAutoCompaction}
        />
        <FieldDescription>
          {i18n.message(m.auto_compaction_detail, {})}
        </FieldDescription>
      </Field>
      <Field>
        <FieldLabel>{i18n.message(m.steering_queue_mode, {})}</FieldLabel>
        <Select
          aria-label={i18n.message(m.steering_queue_mode, {})}
          options={queueModeOptions()}
          value={props.state.steeringMode}
          disabled={disabled() || props.state.steeringMode === undefined}
          onValueChange={(value) =>
            props.setSteeringMode(value as AgentQueueMode)
          }
        />
        <FieldDescription>
          {i18n.message(m.steering_queue_mode_detail, {})}
        </FieldDescription>
      </Field>
      <Field>
        <FieldLabel>{i18n.message(m.follow_up_queue_mode, {})}</FieldLabel>
        <Select
          aria-label={i18n.message(m.follow_up_queue_mode, {})}
          options={queueModeOptions()}
          value={props.state.followUpMode}
          disabled={disabled() || props.state.followUpMode === undefined}
          onValueChange={(value) =>
            props.setFollowUpMode(value as AgentQueueMode)
          }
        />
        <FieldDescription>
          {i18n.message(m.follow_up_queue_mode_detail, {})}
        </FieldDescription>
      </Field>
    </View>
  );
}
