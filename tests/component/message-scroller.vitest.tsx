import { renderComponent } from "@wabou/test/component";
import { MessageScroller, Text, useMessageScroller, View } from "@wabou/ui";
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
