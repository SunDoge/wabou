import { renderComponent } from "@wabou/test/component";
import { Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import type { AgentThinkingLevel } from "../../apps/pi-agent/ui/agent-state";
import {
  ComposerModelControl,
  ModelControls,
} from "../../apps/pi-agent/ui/model-controls";

const models = [
  {
    provider: "anthropic",
    id: "claude-sonnet-4-5",
    name: "Claude Sonnet 4.5",
    reasoning: true,
    contextWindow: 200_000,
  },
  {
    provider: "openai",
    id: "gpt-5.2-codex",
    name: "GPT-5.2 Codex",
    reasoning: true,
    contextWindow: 400_000,
  },
] as const;

test("searches Pi models and reports the exact provider and model id", () => {
  const [selection, setSelection] = createSignal("");
  const screen = renderComponent(() => (
    <View>
      <ModelControls
        models={models}
        modelProvider="anthropic"
        modelId="claude-sonnet-4-5"
        thinking="medium"
        thinkingLevels={["off", "medium", "high"]}
        chooseModel={(provider, modelId) =>
          setSelection(`${provider}/${modelId}`)
        }
        chooseThinking={() => {}}
      />
      <Text role="status">{selection()}</Text>
    </View>
  ));

  const model = screen.getByRole("combobox", { name: "Choose model" });
  expect(model.text).toContain("Claude Sonnet 4.5");
  expect(model.className).toContain("bg-transparent");
  expect(model.className).toContain("h-8");
  model.hover();
  expect(model.className).toContain("bg-control-hover");
  model.click();
  screen.getByRole("textbox", { name: "Choose model search" }).input("openai");
  screen.getByRole("option", { name: "GPT-5.2 Codex" }).click();

  expect(screen.getByRole("status").text).toBe("openai/gpt-5.2-codex");
});

test("selects an explicit Pi thinking level", () => {
  const [thinking, setThinking] = createSignal<AgentThinkingLevel>("medium");
  const screen = renderComponent(() => (
    <ModelControls
      models={models}
      modelProvider="anthropic"
      modelId="claude-sonnet-4-5"
      thinking={thinking()}
      thinkingLevels={["off", "medium", "high"]}
      chooseModel={() => {}}
      chooseThinking={setThinking}
    />
  ));

  const trigger = screen.getByRole("combobox", { name: "Thinking level" });
  trigger.click();
  screen.getByRole("option", { name: "high" }).click();
  expect(thinking()).toBe("high");
  expect(trigger.text).toContain("high");
});

test("disables unavailable runtime controls", () => {
  const screen = renderComponent(() => (
    <ModelControls
      models={[]}
      thinkingLevels={[]}
      chooseModel={() => {}}
      chooseThinking={() => {}}
    />
  ));

  expect(screen.getByRole("combobox", { name: "Choose model" }).disabled).toBe(
    true,
  );
  expect(
    screen.getByRole("combobox", { name: "Thinking level" }).disabled,
  ).toBe(true);
});

test("composer summarizes model configuration until it is requested", () => {
  const screen = renderComponent(() => (
    <ComposerModelControl
      models={models}
      modelProvider="anthropic"
      modelId="claude-sonnet-4-5"
      thinking="medium"
      thinkingLevels={["off", "medium", "high"]}
      chooseModel={() => {}}
      chooseThinking={() => {}}
    />
  ));

  const trigger = screen.getByRole("button", { name: "Choose model" });
  expect(trigger.text).toContain("Claude Sonnet 4.5");
  expect(trigger.text).toContain("medium");
  expect(screen.queryByRole("combobox", { name: "Thinking level" })).toBeNull();

  trigger.click();
  expect(screen.getByRole("combobox", { name: "Choose model" }).text).toContain(
    "Claude Sonnet 4.5",
  );
  expect(
    screen.getByRole("combobox", { name: "Thinking level" }).text,
  ).toContain("medium");
});
