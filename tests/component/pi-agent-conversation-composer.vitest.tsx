import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  ConversationComposer,
  composerEditorHeightClass,
} from "../../apps/pi-agent/ui/conversation-composer";

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
  stats: {
    userMessages: 2,
    assistantMessages: 2,
    toolCalls: 1,
    totalMessages: 5,
    tokens: {
      input: 8_000,
      output: 2_000,
      cacheRead: 0,
      cacheWrite: 0,
      total: 10_000,
    },
    cost: 0.024,
    contextUsage: {
      tokens: 10_000,
      contextWindow: 200_000,
      percent: 5,
    },
  },
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

test("Pi Agent composer grows from a compact idle editor", () => {
  expect(composerEditorHeightClass("")).toBe("h-12");
  expect(composerEditorHeightClass("Short request")).toBe("h-12");
  expect(composerEditorHeightClass("First line\nSecond line")).toBe("h-16");
  expect(composerEditorHeightClass("One\nTwo\nThree")).toBe("h-20");
  expect(composerEditorHeightClass("1\n2\n3\n4\n5")).toBe("h-24");
});

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
  const usage = screen.getByRole("button", { name: "Session usage" });
  expect(usage.text).toContain("Context 5%");
  expect(usage.text).toContain("10k tokens");
  expect(usage.text).toContain("$0.024");
  expect(usage.className).toContain("h-7");
  expect(screen.getByRole("button", { name: "Send" }).disabled).toBe(false);

  screen.getByRole("button", { name: "Send" }).click();
  expect(submit).toHaveBeenCalledOnce();
});

test("keeps the composer mounted and exposes Pi connection failures", () => {
  const [connection, setConnection] = createSignal<"ready" | "failed">("ready");
  const screen = renderComponent(() => (
    <ConversationComposer
      {...baseProps}
      connection={connection()}
      error={
        connection() === "failed" ? "API authentication failed" : undefined
      }
      runtimeLog="Pi runtime warning"
    />
  ));
  const composer = screen.getByRole("group", {
    name: "Ask this agent to work in its repository…",
  });
  const identity = composer.identity;

  setConnection("failed");
  screen.flush();

  expect(
    screen.getByRole("group", {
      name: "Ask this agent to work in its repository…",
    }).identity,
  ).toEqual(identity);
  expect(screen.getByRole("alert").text).toContain("API authentication failed");
});

test("surfaces the latest Pi runtime diagnostic while the process is alive", () => {
  const screen = renderComponent(() => (
    <ConversationComposer
      {...baseProps}
      runtimeLog="Provider rejected credentials"
    />
  ));

  expect(
    screen.getByRole("status", { name: "Workspace status" }).text,
  ).toContain("Provider rejected credentials");
});

test("Pi Agent composer reflects an externally cleared controlled draft", () => {
  const App = () => {
    const [draft, setDraft] = createSignal("Send this request");
    return (
      <ConversationComposer
        {...baseProps}
        draft={draft()}
        changeDraft={setDraft}
        submit={() => setDraft("")}
      />
    );
  };
  const screen = renderComponent(App);
  const editor = screen.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });

  screen.getByRole("button", { name: "Send" }).click();
  expect(editor.value).toBe("");
});

test("Pi Agent composer preserves a native range selection", () => {
  const screen = renderComponent(() => <ConversationComposer {...baseProps} />);
  const editor = screen.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });

  editor.emit("textselectionchange", {
    anchor: 0,
    head: baseProps.draft.length,
    text: baseProps.draft,
    kind: "simple",
  });

  expect(editor.widgetConfig).toEqual({
    selection: { anchor: 0, head: baseProps.draft.length },
  });
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
