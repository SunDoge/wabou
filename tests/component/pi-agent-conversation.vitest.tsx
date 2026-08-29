import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test, vi } from "vitest";
import {
  ConversationItem,
  ConversationList,
  groupConversationItems,
  summarizeToolInput,
  ToolActivityGroup,
} from "../../apps/pi-agent/ui/conversation";

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
      assistant
        .getByRole("region", { name: "Assistant response" })
        .snapshot(),
    ),
  ).toContain("text-base leading-relaxed text-primary");
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
  expect(response.className).toContain("gap-3");
  expect(response.parent?.className).toContain("w-full px-2 pb-3");
  expect(
    screen.getByRole("button", { name: "Copy assistant response" }),
  ).toBeTruthy();
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

  await screen.advanceTime(200);
  const message = screen.roots[0]?.children[0]?.children[0];
  expect(message?.style("opacity")).toEqual({ kind: 3, value: 1 });

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
  expect(message?.style("opacity")).toEqual({ kind: 3, value: 1 });
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
    name: "Worked, 2 tool calls",
  });
  expect(
    activity.parent?.children.filter((node) =>
      node.className.includes("h-px"),
    ),
  ).toHaveLength(2);
  expect(activity.expanded).toBe(false);
  expect(screen.queryByRole("button", { name: "read: README.md" })).toBeNull();
  activity.click();
  expect(activity.expanded).toBe(true);
  expect(screen.getByRole("button", { name: "Reasoning" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "read: README.md" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "bash: cargo test" })).toBeTruthy();
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
    screen.getByRole("button", { name: "Working, 1 tool call" }).expanded,
  ).toBe(true);

  setItems([{ ...items()[0]!, state: "success" }]);
  screen.flush();

  expect(
    screen.getByRole("button", { name: "Worked, 1 tool call" }).expanded,
  ).toBe(false);
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
  screen.getByRole("button", { name: "Fork from this message" }).click();
  expect(fork).toHaveBeenCalledOnce();
});
