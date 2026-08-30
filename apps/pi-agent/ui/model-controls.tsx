import { Button, Combobox, Icon, Popover, Select, Text, View } from "@wabou/ui";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import { Show } from "solid-js";
import type { AgentModel, AgentThinkingLevel } from "./agent-state";
import { i18n, m } from "./i18n";

const modelValue = (model: Pick<AgentModel, "provider" | "id">) =>
  `${model.provider}\0${model.id}`;

const contextLabel = (tokens: number | undefined) => {
  if (tokens === undefined) return undefined;
  if (tokens >= 1_000_000) return `${tokens / 1_000_000}M context`;
  return `${Math.round(tokens / 1_000)}K context`;
};

export function ModelControls(props: {
  models: readonly AgentModel[];
  modelProvider?: string;
  modelId?: string;
  thinking?: AgentThinkingLevel;
  thinkingLevels: readonly AgentThinkingLevel[];
  disabled?: boolean;
  chooseModel(provider: string, modelId: string): void;
  chooseThinking(level: AgentThinkingLevel): void;
}) {
  const selectedModel = () =>
    props.modelProvider && props.modelId
      ? modelValue({ provider: props.modelProvider, id: props.modelId })
      : undefined;
  const modelOptions = () =>
    props.models.map((model) => ({
      id: modelValue(model),
      value: modelValue(model),
      label: model.name,
      description: [model.provider, contextLabel(model.contextWindow)]
        .filter(Boolean)
        .join(" · "),
      keywords: [model.provider, model.id],
    }));
  const thinkingOptions = () =>
    props.thinkingLevels.map((level) => ({ value: level, label: level }));

  return (
    <View class="min-w-0 flex-none flex flex-row items-center gap-1 overflow-hidden">
      <Combobox
        aria-label={i18n.message(m.choose_model, {})}
        triggerVariant="ghost"
        class="w-44 h-8 px-2"
        contentClass="w-80"
        options={modelOptions()}
        value={selectedModel()}
        placeholder={i18n.message(m.choose_model, {})}
        searchPlaceholder={i18n.message(m.search_models, {})}
        emptyText={i18n.message(m.no_models, {})}
        disabled={props.disabled || props.models.length === 0}
        onValueChange={(value) => {
          const separator = value.indexOf("\0");
          if (separator < 1) return;
          props.chooseModel(
            value.slice(0, separator),
            value.slice(separator + 1),
          );
        }}
      />
      <Select
        aria-label={i18n.message(m.choose_thinking, {})}
        triggerVariant="ghost"
        class="w-28 h-8 px-2"
        contentClass="w-40"
        options={thinkingOptions()}
        value={props.thinking}
        placeholder={i18n.message(m.choose_thinking, {})}
        disabled={props.disabled || props.thinkingLevels.length === 0}
        onValueChange={(value) =>
          props.chooseThinking(value as AgentThinkingLevel)
        }
      />
    </View>
  );
}

/**
 * Keeps low-frequency runtime configuration out of the primary composer row.
 * The current choice remains visible while the searchable controls expand in
 * a stable, unconstrained surface.
 */
export function ComposerModelControl(
  props: Parameters<typeof ModelControls>[0],
) {
  const selected = () =>
    props.models.find(
      (model) =>
        model.provider === props.modelProvider && model.id === props.modelId,
    );

  return (
    <Popover
      aria-label={i18n.message(m.choose_model, {})}
      placement="top-start"
      contentClass="w-80 p-3"
      trigger={(trigger) => (
        <Button
          {...trigger}
          variant="ghost"
          size="sm"
          aria-label={i18n.message(m.choose_model, {})}
          disabled={props.disabled || props.models.length === 0}
          class="min-w-0 max-w-56 h-8 px-2 gap-1.5"
        >
          <Text class="min-w-0 truncate text-xs font-medium">
            {selected()?.name ?? i18n.message(m.no_model, {})}
          </Text>
          <Show when={props.thinking}>
            {(level) => (
              <Text class="flex-none text-xs text-muted">· {level()}</Text>
            )}
          </Show>
          <Icon source={chevronDown} size={12} class="flex-none text-muted" />
        </Button>
      )}
    >
      <ModelControls {...props} />
    </Popover>
  );
}
