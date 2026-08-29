import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import { ConversationComposer } from "../../apps/pi-agent/ui/conversation-composer";

const baseProps = {
  connection: "ready" as const,
  project: "Wabou",
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

test("Pi Agent composer completes an inline slash command without submitting", () => {
  const submit = vi.fn();
  const App = () => {
    const [draft, setDraft] = createSignal("");
    return (
      <ConversationComposer
        {...baseProps}
        draft={draft()}
        changeDraft={setDraft}
        commands={[
          { name: "review", source: "project", description: "Review changes" },
          { name: "compact", source: "extension" },
        ]}
        submit={submit}
      />
    );
  };
  const screen = renderComponent(App);
  const editor = screen.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });

  editor.input("/rev");
  expect(screen.getByRole("listbox", { name: "Commands" })).toBeTruthy();
  expect(screen.getByRole("option", { name: "/review" }).selected).toBe(true);

  editor.press("Enter");
  expect(editor.value).toBe("/review ");
  expect(submit).not.toHaveBeenCalled();
  expect(screen.queryByRole("listbox", { name: "Commands" })).toBeNull();
});

test("Pi Agent composer loads workspace files for inline at mentions", async () => {
  const changeContextFiles = vi.fn();
  const App = () => {
    const [draft, setDraft] = createSignal("");
    return (
      <ConversationComposer
        {...baseProps}
        draft={draft()}
        changeDraft={setDraft}
        changeContextFiles={changeContextFiles}
        loadWorkspaceFiles={vi.fn(async () => [
          "apps/pi-agent/ui/app.tsx",
          "README.md",
        ])}
      />
    );
  };
  const screen = renderComponent(App);
  const editor = screen.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });

  editor.input("Inspect @app");
  await screen.waitFor(() =>
    expect(
      screen.getByRole("option", { name: "apps/pi-agent/ui/app.tsx" }),
    ).toBeTruthy(),
  );
  screen.getByRole("option", { name: "apps/pi-agent/ui/app.tsx" }).click();
  expect(editor.value).toBe("Inspect @apps/pi-agent/ui/app.tsx ");
  expect(changeContextFiles).toHaveBeenCalledWith(["apps/pi-agent/ui/app.tsx"]);
});
