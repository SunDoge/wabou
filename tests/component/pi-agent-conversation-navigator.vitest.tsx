import {
  MessageScroller,
  MessageScrollerContent,
  MessageScrollerViewport,
  View,
} from "@wabou/ui";
import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
import type { AgentItem } from "../../apps/pi-agent/ui/agent-state";
import {
  ConversationNavigator,
  conversationTurns,
} from "../../apps/pi-agent/ui/conversation-navigator";

const items: readonly AgentItem[] = [
  { id: "user-1", kind: "user", text: "  Inspect   the router\nfirst. " },
  { id: "assistant-1", kind: "assistant", text: "I will inspect it." },
  {
    id: "tool-1",
    kind: "tool",
    name: "read",
    state: "success",
    input: "",
    output: "",
  },
  {
    id: "user-2",
    kind: "user",
    text: "Explain why this long prompt should remain concise in the conversation navigator preview without losing its identity.",
  },
];

function NavigatorHarness(props: {
  source?: readonly AgentItem[];
  resolveItem(id: string): undefined;
}) {
  return (
    <MessageScroller>
      <MessageScrollerViewport>
        <MessageScrollerContent>
          <View class="h-96" />
        </MessageScrollerContent>
      </MessageScrollerViewport>
      <ConversationNavigator
        items={props.source ?? items}
        resolveItem={props.resolveItem}
      />
    </MessageScroller>
  );
}

test("Pi Agent derives one compact navigation target per user turn", () => {
  const turns = conversationTurns(items);
  expect(turns).toHaveLength(2);
  expect(turns[0]).toEqual({
    id: "user-1",
    index: 0,
    prompt: "Inspect the router first.",
  });
  expect(turns[1]?.prompt.endsWith("…")).toBe(true);
  expect(turns[1]?.prompt.length).toBeLessThanOrEqual(72);
});

test("Pi Agent exposes a quiet, accessible turn rail for long conversations", () => {
  const resolveItem = vi.fn(() => undefined);
  const screen = renderComponent(() => (
    <NavigatorHarness resolveItem={resolveItem} />
  ));

  const rail = screen.getByRole("group", { name: "Conversation turns" });
  expect(rail.className).toContain("pointer-events-none");
  const first = screen.getByRole("button", {
    name: "Jump to turn 1: Inspect the router first.",
  });
  expect(screen.getAllByRole("button").map((button) => button.name)).toEqual([
    "Jump to turn 1: Inspect the router first.",
    "Jump to turn 2: Explain why this long prompt should remain concise in the conversation…",
  ]);

  first.click();
  expect(resolveItem).toHaveBeenCalledWith("user-1");
});

test("Pi Agent omits turn navigation until there is something to navigate", () => {
  const screen = renderComponent(() => (
    <NavigatorHarness
      source={[{ id: "user-1", kind: "user", text: "Only turn" }]}
      resolveItem={() => undefined}
    />
  ));
  expect(
    screen.queryByRole("group", { name: "Conversation turns" }),
  ).toBeNull();
});
