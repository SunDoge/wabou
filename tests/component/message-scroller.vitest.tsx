import { renderComponent } from "@wabou/test/component";
import {
  MessageScroller,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerNavigator,
  MessageScrollerViewport,
  Text,
  useMessageScroller,
  View,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

function FollowState() {
  const scroller = useMessageScroller();
  return (
    <View role="status" aria-label="Follow state">
      <Text>{scroller.followingEnd() ? "following" : "free"}</Text>
    </View>
  );
}

test("MessageScroller follows a reactive followEnd contract", () => {
  const [follow, setFollow] = createSignal(false);
  const screen = renderComponent(() => (
    <MessageScroller followEnd={follow()}>
      <FollowState />
    </MessageScroller>
  ));
  const state = screen.getByRole("status", { name: "Follow state" });

  expect(state.text).toBe("free");
  setFollow(true);
  screen.flush();
  expect(state.text).toBe("following");
  setFollow(false);
  screen.flush();
  expect(state.text).toBe("free");
});

test("MessageScrollerNavigator exposes retained anchors without app-owned chrome", () => {
  const screen = renderComponent(() => (
    <MessageScroller>
      <MessageScrollerViewport>
        <MessageScrollerContent>
          <MessageScrollerItem anchor="request-1">
            <View class="h-48" />
          </MessageScrollerItem>
          <MessageScrollerItem anchor="request-2">
            <View class="h-48" />
          </MessageScrollerItem>
        </MessageScrollerContent>
      </MessageScrollerViewport>
      <MessageScrollerNavigator
        items={[
          { id: "request-1", label: "Inspect the renderer" },
          { id: "request-2", label: "Run focused tests" },
        ]}
        aria-label="Request navigator"
        itemAriaLabel={(item, index) =>
          `Jump to request ${index + 1}: ${item.label}`
        }
      />
    </MessageScroller>
  ));

  const navigator = screen.getByRole("toolbar", {
    name: "Request navigator",
  });
  expect(navigator.orientation).toBe("vertical");
  const first = screen.getByRole("button", {
    name: "Jump to request 1: Inspect the renderer",
  });
  const second = screen.getByRole("button", {
    name: "Jump to request 2: Run focused tests",
  });
  expect(first.focusOrder).toBe(0);
  expect(second.focusOrder).toBe(-1);
  first.focus();
  first.press("ArrowDown");
  expect(second.focused).toBe(true);
  expect(() => second.click()).not.toThrow();
});
