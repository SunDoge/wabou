import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import {
  ConversationItem,
  summarizeToolInput,
} from "../../apps/pi-agent/ui/conversation";

test("Pi Agent renders assistant Markdown but preserves user source text", () => {
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
  expect(assistant.getByRole("group", { name: "Code block" }).text).toContain(
    "curl /healthz",
  );
  assistant.dispose();

  const user = renderComponent(() => (
    <ConversationItem
      item={{ id: "user-1", kind: "user", text: "**keep source**" }}
    />
  ));
  expect(user.queryByRole("region", { name: "Assistant response" })).toBeNull();
  expect(user.roots[0]?.text).toContain("**keep source**");
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

  expect(screen.getByRole("status", { name: "Pi is writing" }).text).toContain(
    "Writing",
  );
  const response = screen.getByRole("region", { name: "Assistant response" });
  expect(response.className).toContain("gap-2.5");
  expect(response.parent?.className).toBe(
    "max-w-full min-w-0 overflow-hidden rounded-xl border p-0 border-transparent bg-transparent text-primary",
  );
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
