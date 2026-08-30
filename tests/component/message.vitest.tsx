import { renderComponent } from "@wabou/test/component";
import { Button, Message, MessageActions, MessageContent } from "@wabou/ui";
import { expect, test } from "vitest";

test("interaction message actions reveal on hover without shifting layout", () => {
  const screen = renderComponent(() => (
    <Message aria-label="Assistant response">
      <MessageContent>
        <MessageActions
          visibility="interaction"
          aria-label="Response actions"
        >
          <Button aria-label="Copy response" />
        </MessageActions>
      </MessageContent>
    </Message>
  ));

  const message = screen.getByRole("group", { name: "Assistant response" });
  const actions = screen.getByRole("toolbar", { name: "Response actions" });
  expect(actions.className).toContain("opacity-0");
  expect(actions.className).toContain("h-7");

  message.hover();
  expect(actions.className).toContain("opacity-100");
  expect(actions.className).not.toContain("pointer-events-none");

  message.unhover();
  expect(actions.className).toContain("opacity-0");
});

test("interaction message actions remain available to keyboard focus", () => {
  const screen = renderComponent(() => (
    <Message aria-label="Assistant response">
      <MessageContent>
        <MessageActions
          visibility="interaction"
          aria-label="Response actions"
        >
          <Button aria-label="Copy response" />
        </MessageActions>
      </MessageContent>
    </Message>
  ));

  const actions = screen.getByRole("toolbar", { name: "Response actions" });
  screen.getByRole("button", { name: "Copy response" }).focus();
  expect(actions.className).toContain("opacity-100");
});
