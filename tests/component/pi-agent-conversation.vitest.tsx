import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  ConversationItem,
  ConversationList,
  formatTurnDuration,
  groupConversationItems,
  summarizeToolInput,
  ToolActivityGroup,
} from "../../apps/pi-agent/ui/conversation";

test("formats completed turn duration without false precision", () => {
  expect(formatTurnDuration(420)).toBe("<1s");
  expect(formatTurnDuration(12_200)).toBe("12s");
  expect(formatTurnDuration(84_000)).toBe("1m 24s");
});

test("Pi Agent renders assistant and user messages through Markdown", () => {
  const assistant = renderComponent(() => (
    <ConversationItem
      item={{
        id: "assistant-1",
        kind: "assistant",
        text: "## Change\n\n- Added `healthz`\n\n```sh\ncurl /healthz\n```",
      }}
    />
  ));
  expect(
    assistant.getByRole("region", { name: "Assistant response" }).text,
  ).toContain("Added healthz");
  expect(
    JSON.stringify(
      assistant.getByRole("region", { name: "Assistant response" }).snapshot(),
    ),
  ).toContain("text-sm leading-relaxed text-primary");
  expect(assistant.getByRole("group", { name: "Code block" }).text).toContain(
    "curl /healthz",
  );
  assistant.dispose();

  const user = renderComponent(() => (
    <ConversationItem
      item={{
        id: "user-1",
        kind: "user",
        text: "## Request\n\n**keep meaning** and run `cargo test`",
      }}
    />
  ));
  expect(user.queryByRole("region", { name: "Assistant response" })).toBeNull();
  const prompt = user.getByRole("region", { name: "User message" });
  expect(prompt.text).toContain("Requestkeep meaning and run cargo test");
  expect(prompt.text).not.toContain("**");
  expect(JSON.stringify(prompt.snapshot())).toContain(
    "text-sm font-semibold text-primary",
  );
});

test("Pi Agent exposes streaming progress without wrapping assistant prose in a card", () => {
  const screen = renderComponent(() => (
    <ConversationItem
      item={{
        id: "assistant-streaming",
        kind: "assistant",
        text: "## Working\n\nReading the repository…",
        streaming: true,
      }}
    />
  ));

  expect(screen.getByRole("status", { name: "Pi is writing" }).text).toBe(
    "Pi is writing",
  );
  const response = screen.getByRole("region", { name: "Assistant response" });
  expect(response.className).toContain("gap-2.5");
  expect(response.parent?.className).toContain("w-full px-2 pb-3");
  expect(
    screen.getByRole("button", { name: "Copy assistant response" }),
  ).toBeTruthy();
  expect(
    screen.getByRole("toolbar", { name: "Assistant response actions" }).text,
  ).toBe("");
});

test("Pi Agent messages enter with finite native motion", async () => {
  const screen = renderComponent(
    () => (
      <ConversationItem
        item={{ id: "assistant-motion", kind: "assistant", text: "Ready." }}
      />
    ),
    { clock: "fake" },
  );
  const message = screen.roots[0];
  expect(message?.transform?.[5]).toBe(5);
  expect(message?.style("opacity")).toEqual({ kind: 3, value: 0 });

  await screen.advanceTime(200);

  expect(message?.transform?.[5]).toBe(0);
  expect(message?.style("opacity")).toEqual({ kind: 3, value: 1 });
});

test("Pi Agent retains a streamed message instead of replaying its entrance", async () => {
  const [items, setItems] = createSignal([
    {
      id: "assistant-stream",
      kind: "assistant" as const,
      text: "First chunk",
      streaming: true,
    },
  ]);
  const screen = renderComponent(() => <ConversationList items={items()} />, {
    clock: "fake",
  });

  const message = screen.roots[0]?.children[0]?.children[0];
  expect(message?.style("opacity")).toBeNull();

  await screen.advanceTime(200);
  expect(message?.style("opacity")).toBeNull();

  setItems([
    {
      id: "assistant-stream",
      kind: "assistant",
      text: "First chunk and the next token",
      streaming: true,
    },
  ]);
  screen.flush();

  expect(message?.text).toContain("First chunk and the next token");
  expect(message?.style("opacity")).toBeNull();
});

test("Pi Agent animates newly appended messages without fading loaded history", async () => {
  const [items, setItems] = createSignal([
    { id: "history", kind: "assistant" as const, text: "Already loaded" },
  ]);
  const screen = renderComponent(() => <ConversationList items={items()} />, {
    clock: "fake",
  });

  const history = screen.roots[0]?.children[0]?.children[0];
  expect(history?.style("opacity")).toBeNull();

  setItems([
    { id: "history", kind: "assistant", text: "Already loaded" },
    { id: "new", kind: "assistant", text: "Just arrived" },
  ]);
  screen.flush();

  const appended = screen.roots[0]?.children[1]?.children[0];
  expect(appended?.style("opacity")).toEqual({ kind: 3, value: 0 });
  await screen.advanceTime(200);
  expect(appended?.style("opacity")).toEqual({ kind: 3, value: 1 });
});

test("Pi Agent keeps reasoning in an independently collapsible process detail", () => {
  const screen = renderComponent(() => (
    <ConversationItem
      item={{
        id: "assistant-reasoning",
        kind: "assistant",
        thinkingText: "I should inspect `src/main.rs` first.",
        text: "The issue is fixed.",
        streaming: false,
      }}
    />
  ));

  const toggle = screen.getByRole("button", { name: "Reasoning" });
  expect(toggle.expanded).toBe(false);
  expect(
    screen.getByRole("region", { name: "Assistant response" }).text,
  ).toContain("The issue is fixed.");
  expect(screen.queryByRole("region", { name: "Model reasoning" })).toBeNull();

  toggle.click();
  expect(toggle.expanded).toBe(true);
  expect(
    screen.getByRole("region", { name: "Model reasoning" }).text,
  ).toContain("I should inspect src/main.rs first.");
});

test("Pi Agent tool activity starts expanded and summarizes common arguments", () => {
  expect(
    summarizeToolInput(
      JSON.stringify({ command: "cargo test -p wabou-runtime\nnext line" }),
    ),
  ).toBe("cargo test -p wabou-runtime");

  const screen = renderComponent(() => (
    <ConversationItem
      item={{
        id: "tool-1",
        kind: "tool",
        name: "bash",
        state: "running",
        input: JSON.stringify({ command: "cargo test -p wabou-runtime" }),
        output: "running tests",
      }}
    />
  ));
  const toggle = screen.getByRole("button", {
    name: "bash: cargo test -p wabou-runtime",
  });
  expect(toggle.expanded).toBe(true);
  expect(toggle.text).toContain("cargo test -p wabou-runtime");
  expect(screen.getAllByRole("group", { name: "Code block" })).toHaveLength(2);

  toggle.click();
  expect(toggle.expanded).toBe(false);
});

test("Pi Agent folds adjacent completed tools into one turn activity group", () => {
  const items = [
    { id: "user-1", kind: "user" as const, text: "Inspect it" },
    {
      id: "tool-1",
      kind: "tool" as const,
      name: "read",
      state: "success" as const,
      input: JSON.stringify({ path: "README.md" }),
      output: "readme",
    },
    {
      id: "tool-2",
      kind: "tool" as const,
      name: "bash",
      state: "success" as const,
      input: JSON.stringify({ command: "cargo test" }),
      output: "ok",
    },
    {
      id: "assistant-1",
      kind: "assistant" as const,
      thinkingText: "I inspected the project.",
      text: "Done.",
    },
  ];
  const entries = groupConversationItems(items);
  expect(entries.map((entry) => entry.kind)).toEqual(["item", "tools", "item"]);
  expect(entries[1]?.kind === "tools" && entries[1].items).toHaveLength(2);
  expect(entries[1]?.kind === "tools" && entries[1].reasoning?.text).toBe(
    "I inspected the project.",
  );
  expect(
    entries[2]?.kind === "item" &&
      entries[2].item.kind === "assistant" &&
      entries[2].item.thinkingText,
  ).toBeUndefined();

  const screen = renderComponent(() => <ConversationList items={items} />);
  const activity = screen.getByRole("button", {
    name: "Worked · 2 tool calls",
  });
  expect(activity.className).toContain("text-secondary");
  expect(activity.className).toContain("self-start");
  expect(activity.parent?.className).toContain("w-full");
  expect(
    activity.parent?.children.filter((node) => node.className.includes("h-px")),
  ).toHaveLength(0);
  expect(activity.expanded).toBe(false);
  expect(screen.queryByRole("button", { name: "read: README.md" })).toBeNull();
  expect(
    screen.queryByRole("list", { name: "Recent tool activity" }),
  ).toBeNull();
  activity.click();
  expect(activity.expanded).toBe(true);
  expect(
    screen.queryByRole("list", { name: "Recent tool activity" }),
  ).toBeNull();
  expect(screen.getByRole("button", { name: "Reasoning" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "read: README.md" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "bash: cargo test" })).toBeTruthy();
});

test("Pi Agent folds reasoning-separated tool calls into one turn activity group", () => {
  const tool = (id: string, name: string) => ({
    id,
    kind: "tool" as const,
    name,
    state: "success" as const,
    input: JSON.stringify({ path: `${name}.ts` }),
    output: "ok",
  });
  const entries = groupConversationItems([
    {
      id: "thinking-before",
      kind: "assistant",
      text: "",
      thinkingText: "Inspect the workspace.",
    },
    tool("tool-1", "read"),
    {
      id: "thinking-between-1",
      kind: "assistant",
      text: "",
      thinkingText: "Check the manifest.",
    },
    tool("tool-2", "read-package"),
    {
      id: "empty-lifecycle-message",
      kind: "assistant",
      text: "",
    },
    tool("tool-3", "read-cargo"),
    {
      id: "answer",
      kind: "assistant",
      text: "Wabou is a retained desktop UI runtime.",
      thinkingText: "Synthesize the answer.",
    },
  ]);

  expect(entries).toHaveLength(2);
  expect(entries[0]).toMatchObject({
    kind: "tools",
    items: [{ id: "tool-1" }, { id: "tool-2" }, { id: "tool-3" }],
    reasoning: {
      text: "Inspect the workspace.\n\nCheck the manifest.\n\nSynthesize the answer.",
      streaming: false,
    },
  });
  expect(entries[1]).toMatchObject({
    kind: "item",
    item: {
      id: "answer",
      text: "Wabou is a retained desktop UI runtime.",
      thinkingText: undefined,
    },
  });

  const screen = renderComponent(() => (
    <ConversationList
      items={[
        tool("tool-1", "read"),
        {
          id: "thinking-between",
          kind: "assistant",
          text: "",
          thinkingText: "Continue inspecting.",
        },
        tool("tool-2", "read-package"),
      ]}
    />
  ));
  expect(
    screen.getByRole("button", { name: "Worked · 2 tool calls" }),
  ).toBeTruthy();
  expect(
    screen.queryByRole("button", { name: "Worked · 1 tool call" }),
  ).toBeNull();
});

test("Pi Agent keeps live tool activity open and folds it after completion", () => {
  const [items, setItems] = createSignal([
    {
      id: "tool-live",
      kind: "tool" as const,
      name: "bash",
      state: "running" as const | "success",
      input: JSON.stringify({ command: "cargo test" }),
      output: "",
    },
  ]);
  const screen = renderComponent(() => <ToolActivityGroup items={items()} />);
  expect(
    screen.getByRole("button", { name: "Working · 1 tool call" }).expanded,
  ).toBe(true);

  const liveItem = items()[0];
  if (!liveItem) throw new Error("expected live tool item");
  setItems([{ ...liveItem, state: "success" }]);
  screen.flush();

  expect(
    screen.getByRole("button", { name: "Worked · 1 tool call" }).expanded,
  ).toBe(false);
});

test("Pi Agent reports the measured duration of a completed tool turn", () => {
  const screen = renderComponent(() => (
    <ToolActivityGroup
      items={[
        {
          id: "tool-measured",
          kind: "tool",
          name: "read",
          state: "success",
          input: "{}",
          output: "ok",
          turnDurationMs: 12_200,
        },
      ]}
    />
  ));

  expect(
    screen.getByRole("button", {
      name: "Worked for 12s · 1 tool call",
    }),
  ).toBeTruthy();
});

test("Pi Agent distinguishes a queued follow-up from a sent message", () => {
  const screen = renderComponent(() => (
    <ConversationItem
      item={{
        id: "queued-1",
        kind: "user",
        text: "After that, run the integration tests.",
        queued: true,
      }}
    />
  ));

  expect(screen.getByRole("status", { name: "Queued follow-up" }).text).toBe(
    "Queued",
  );
});

test("Pi Agent keeps submitted image names visible and user messages copyable", () => {
  const fork = vi.fn();
  const screen = renderComponent(() => (
    <ConversationItem
      item={{
        id: "image-1",
        kind: "user",
        text: "Explain this screenshot",
        imageNames: ["layout.png", "error.png"],
        contextPaths: ["src/layout.rs"],
      }}
      fork={fork}
    />
  ));

  const attachments = screen.getByRole("group", { name: "Attached images" });
  expect(attachments.text).toContain("layout.png");
  expect(attachments.text).toContain("error.png");
  expect(screen.getByRole("group", { name: "Context files" }).text).toContain(
    "src/layout.rs",
  );
  expect(
    screen.getByRole("button", { name: "Copy user message" }),
  ).toBeTruthy();
  expect(
    screen.getByRole("toolbar", { name: "User message actions" }).text,
  ).toBe("");
  screen.getByRole("button", { name: "Fork from this message" }).click();
  expect(fork).toHaveBeenCalledOnce();
});
