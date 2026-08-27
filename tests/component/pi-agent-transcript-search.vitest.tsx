import { renderComponent } from "@wabou/test/component";
import { MessageScroller } from "@wabou/ui";
import { expect, test } from "vitest";
import type { AgentItem } from "../../apps/pi-agent/ui/agent-state";
import {
  findTranscriptItems,
  TranscriptSearch,
} from "../../apps/pi-agent/ui/transcript-search";

const items: AgentItem[] = [
  { id: "user-1", kind: "user", text: "Please run the tests" },
  {
    id: "tool-1",
    kind: "tool",
    name: "bash",
    state: "success",
    input: '{"command":"cargo test"}',
    output: "all tests passed",
  },
  {
    id: "assistant-1",
    kind: "assistant",
    text: "Cargo finished successfully.",
    thinkingText: "Check the test output",
  },
];

test("finds transcript text across messages, reasoning and tool activity", () => {
  expect(findTranscriptItems(items, "cargo")).toEqual([
    "tool-1",
    "assistant-1",
  ]);
  expect(findTranscriptItems(items, "TEST OUTPUT")).toEqual(["assistant-1"]);
  expect(findTranscriptItems(items, "  ")).toEqual([]);
});

test("navigates matching transcript items and clears selection on close", () => {
  let active: string | undefined;
  let closed = false;
  const screen = renderComponent(() => (
    <MessageScroller>
      <TranscriptSearch
        items={items}
        resolveItem={() => undefined}
        activeChanged={(id) => {
          active = id;
        }}
        close={() => {
          closed = true;
        }}
      />
    </MessageScroller>
  ));

  screen.getByRole("textbox", { name: "Search conversation" }).input("cargo");
  expect(active).toBe("tool-1");
  expect(
    screen.getByRole("group", { name: "Search conversation" }).text,
  ).toContain("1 / 2");

  screen.getByRole("button", { name: "Next match" }).click();
  expect(active).toBe("assistant-1");
  screen.getByRole("button", { name: "Close search" }).click();
  expect(active).toBeUndefined();
  expect(closed).toBe(true);
});
