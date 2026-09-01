import { renderComponent } from "@wabou/test/component";
import {
  MessageScroller,
  MessageScrollerContent,
  MessageScrollerViewport,
  ProjectionBoundary,
  Text,
} from "@wabou/ui";
import { expect, test } from "vitest";
import { AgentTerminalPanel } from "../../apps/pi-agent/ui/terminal-panel";

test("component tests observe the formal projection-boundary opcode", () => {
  const screen = renderComponent(() => (
    <ProjectionBoundary role="region" aria-label="Retained region" />
  ));
  expect(
    screen.getByRole("region", { name: "Retained region" }).projectionBoundary,
  ).toBe(true);
});

test("the pi transcript authors an explicit GPUI projection boundary", () => {
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

test("the retained terminal surface owns its GPUI projection boundary", () => {
  const screen = renderComponent(() => (
    <AgentTerminalPanel
      cwd="/tmp/wabou-agent"
      open
      close={() => {}}
      dispose={() => {}}
    />
  ));

  expect(
    screen.getByRole("region", { name: "Terminal panel" })
      .projectionBoundary,
  ).toBe(true);
});
