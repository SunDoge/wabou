import { Combobox, Select, View } from "@wabou/ui";
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
