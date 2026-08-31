import {
  Alert,
  createKeyedAsyncAction,
  Field,
  FieldDescription,
  LabeledField,
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
  setAutoCompaction(enabled: boolean): void | Promise<void>;
  setSteeringMode(mode: AgentQueueMode): void | Promise<void>;
  setFollowUpMode(mode: AgentQueueMode): void | Promise<void>;
}) {
  const action = createKeyedAsyncAction(
    (id: BehaviorAction, _perform: () => void | Promise<void>) => id,
    (_id: BehaviorAction, perform: () => void | Promise<void>) => perform(),
  );
  const run = (id: BehaviorAction, perform: () => void | Promise<void>) =>
    action.run(id, perform);
  const error = () =>
    action.error("autoCompaction") ??
    action.error("steering") ??
    action.error("followUp");
  const disabled = () => props.state.connection !== "ready";
  return (
    <View class="min-w-0 flex flex-col gap-4">
      {error() ? (
        <Alert
          variant="destructive"
          title={i18n.message(m.session_behavior_update_failed, {})}
          class="p-3"
        >
          {String(error())}
        </Alert>
      ) : null}
      <Field>
        <Switch
          aria-label={i18n.message(m.auto_compaction, {})}
          label={i18n.message(m.auto_compaction, {})}
          checked={props.state.autoCompactionEnabled ?? false}
          disabled={
            disabled() ||
            props.state.autoCompactionEnabled === undefined ||
            action.pending("autoCompaction")
          }
          onCheckedChange={(enabled) =>
            void run("autoCompaction", () => props.setAutoCompaction(enabled))
          }
        />
        <FieldDescription class="text-secondary">
          {i18n.message(m.auto_compaction_detail, {})}
        </FieldDescription>
      </Field>
      <LabeledField
        label={i18n.message(m.steering_queue_mode, {})}
        description={i18n.message(m.steering_queue_mode_detail, {})}
        disabled={
          disabled() ||
          props.state.steeringMode === undefined ||
          action.pending("steering")
        }
        renderControl={(ref) => (
          <Select
            ref={ref}
            aria-label={i18n.message(m.steering_queue_mode, {})}
            options={queueModeOptions()}
            value={props.state.steeringMode}
            disabled={
              disabled() ||
              props.state.steeringMode === undefined ||
              action.pending("steering")
            }
            onValueChange={(value) => {
              const mode = value as AgentQueueMode;
              void run("steering", () => props.setSteeringMode(mode));
            }}
          />
        )}
      />
      <LabeledField
        label={i18n.message(m.follow_up_queue_mode, {})}
        description={i18n.message(m.follow_up_queue_mode_detail, {})}
        disabled={
          disabled() ||
          props.state.followUpMode === undefined ||
          action.pending("followUp")
        }
        renderControl={(ref) => (
          <Select
            ref={ref}
            aria-label={i18n.message(m.follow_up_queue_mode, {})}
            options={queueModeOptions()}
            value={props.state.followUpMode}
            disabled={
              disabled() ||
              props.state.followUpMode === undefined ||
              action.pending("followUp")
            }
            onValueChange={(value) => {
              const mode = value as AgentQueueMode;
              void run("followUp", () => props.setFollowUpMode(mode));
            }}
          />
        )}
      />
    </View>
  );
}

type BehaviorAction = "autoCompaction" | "steering" | "followUp";
