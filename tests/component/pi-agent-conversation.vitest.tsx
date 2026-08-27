import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { ConversationItem } from "../../apps/pi-agent/ui/conversation";

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
