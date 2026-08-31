import { renderComponent } from "@wabou/test/component";
import {
  MessageScroller,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerViewport,
  View,
} from "@wabou/ui";
import { expect, test } from "vitest";
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

function NavigatorHarness(props: { source?: readonly AgentItem[] }) {
  return (
    <MessageScroller>
      <MessageScrollerViewport>
        <MessageScrollerContent>
          <MessageScrollerItem anchor="user-1">
            <View class="h-96" />
          </MessageScrollerItem>
          <MessageScrollerItem anchor="user-2">
            <View class="h-96" />
          </MessageScrollerItem>
        </MessageScrollerContent>
      </MessageScrollerViewport>
      <ConversationNavigator items={props.source ?? items} />
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
  const screen = renderComponent(() => <NavigatorHarness />);

  const rail = screen.getByRole("toolbar", { name: "Conversation turns" });
  expect(rail.orientation).toBe("vertical");
  const first = screen.getByRole("button", {
    name: "Jump to turn 1: Inspect the router first.",
  });
  expect(screen.getAllByRole("button").map((button) => button.name)).toEqual([
    "Jump to turn 1: Inspect the router first.",
    "Jump to turn 2: Explain why this long prompt should remain concise in the conversation…",
  ]);

  expect(() => first.click()).not.toThrow();
});

test("Pi Agent omits turn navigation until there is something to navigate", () => {
  const screen = renderComponent(() => (
    <NavigatorHarness
      source={[{ id: "user-1", kind: "user", text: "Only turn" }]}
    />
  ));
  expect(
    screen.queryByRole("toolbar", { name: "Conversation turns" }),
  ).toBeNull();
});
