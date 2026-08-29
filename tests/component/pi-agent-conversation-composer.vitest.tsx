import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import { ConversationComposer } from "../../apps/pi-agent/ui/conversation-composer";

const baseProps = {
  connection: "ready" as const,
  cwd: "/work/wabou",
  draft: "Inspect the renderer",
  images: [] as readonly string[],
  contextFiles: [] as readonly string[],
  deliveryMode: "followUp" as const,
  models: [
    {
      provider: "openai",
      id: "gpt-5",
      name: "GPT-5",
      contextWindow: 200_000,
    },
  ],
  modelProvider: "openai",
  modelId: "gpt-5",
  thinking: "medium" as const,
  thinkingLevels: ["low", "medium", "high"] as const,
  commands: [{ name: "review", source: "project" }],
  statuses: [],
  widgets: [],
  changeDraft: vi.fn(),
  changeImages: vi.fn(),
  changeContextFiles: vi.fn(),
  changeDeliveryMode: vi.fn(),
  chooseModel: vi.fn(),
  chooseThinking: vi.fn(),
  loadWorkspaceFiles: vi.fn(async () => []),
  submit: vi.fn(),
};

test("Pi Agent composer keeps the primary action and configuration discoverable", () => {
  const submit = vi.fn();
  const screen = renderComponent(() => (
    <ConversationComposer {...baseProps} submit={submit} />
  ));

  expect(
    screen.getByRole("textbox", {
      name: "Ask this agent to work in its repository…",
    }).value,
  ).toBe("Inspect the renderer");
  expect(screen.getByRole("combobox", { name: "Choose model" })).toBeTruthy();
  expect(screen.getByRole("combobox", { name: "Thinking level" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "Send" }).disabled).toBe(false);

  screen.getByRole("button", { name: "Send" }).click();
  expect(submit).toHaveBeenCalledOnce();
});

test("Pi Agent composer swaps configuration for delivery mode while running", () => {
  const screen = renderComponent(() => (
    <ConversationComposer {...baseProps} connection="running" />
  ));

  expect(screen.queryByRole("combobox", { name: "Choose model" })).toBeNull();
  expect(
    screen.getByRole("combobox", { name: "Message delivery" }),
  ).toBeTruthy();
  expect(screen.getByRole("button", { name: "Queue" })).toBeTruthy();
});
