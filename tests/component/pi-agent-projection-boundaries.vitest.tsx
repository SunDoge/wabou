import { renderComponent } from "@wabou/test/component";
import {
  MessageScroller,
  MessageScrollerContent,
  MessageScrollerViewport,
  ProjectionBoundary,
  Text,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import { AgentTerminalPanel } from "../../apps/pi-agent/ui/terminal-panel";

test("component tests observe the formal projection-boundary opcode", () => {
  const screen = renderComponent(() => (
    <ProjectionBoundary role="region" aria-label="Retained region">
      <Text>Retained child</Text>
    </ProjectionBoundary>
  ));
  expect(
    screen.getByRole("region", { name: "Retained region" }).projectionBoundary,
  ).toBe(true);
  expect(
    screen.getByRole("region", { name: "Retained region" }).text,
  ).toContain("Retained child");
});

test("the pi transcript authors an explicit native projection boundary", () => {
  const screen = renderComponent(() => (
    <MessageScroller
      projectionBoundary
      role="region"
      aria-label="Conversation transcript boundary"
    >
      <MessageScrollerViewport>
        <MessageScrollerContent>
          <Text>Hello</Text>
        </MessageScrollerContent>
      </MessageScrollerViewport>
    </MessageScroller>
  ));

  expect(
    screen.getByRole("region", {
      name: "Conversation transcript boundary",
    }).projectionBoundary,
  ).toBe(true);
});

test("the retained terminal surface owns its native projection boundary", () => {
  const screen = renderComponent(() => (
    <AgentTerminalPanel
      cwd="/tmp/wabou-agent"
      open
      close={() => {}}
      dispose={() => {}}
    />
  ));

  expect(
    screen.getByRole("region", { name: "Terminal panel" }).projectionBoundary,
  ).toBe(true);
});

test("retained child text updates without replacing its region", () => {
  const [value, setValue] = createSignal("First");
  const screen = renderComponent(() => (
    <ProjectionBoundary role="region" aria-label="Live region">
      <Text maxLines={1}>{value()}</Text>
    </ProjectionBoundary>
  ));
  const region = screen.getByRole("region", { name: "Live region" });
  const identity = region.identity;
  expect(region.text).toContain("First");
  setValue("Second");
  screen.flush();
  const updated = screen.getByRole("region", { name: "Live region" });
  expect(updated.identity).toEqual(identity);
  expect(updated.text).toContain("Second");
  expect(updated.text).not.toContain("First");
});
