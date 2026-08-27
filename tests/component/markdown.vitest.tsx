import { renderComponent } from "@wabou/test/component";
import { Markdown } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  parseMarkdown,
  reconcileMarkdownBlocks,
} from "../../packages/ui/src/components/markdown-model";

test("normalizes parser tokens into nested Wabou inline styles", () => {
  const [paragraph] = parseMarkdown(
    "**bold and _italic_** plus [linked `code`](https://wabou.dev)",
  );

  expect(paragraph).toEqual({
    kind: "paragraph",
    runs: [
      { text: "bold and ", style: { strong: true } },
      {
        text: "italic",
        style: { strong: true, emphasis: true },
      },
      { text: " plus ", style: {} },
      { text: "linked ", style: { href: "https://wabou.dev" } },
      {
        text: "code",
        style: { href: "https://wabou.dev", code: true },
      },
    ],
  });
});

test("preserves nested blocks and task state instead of flattening them", () => {
  const parsed = parseMarkdown(
    "> First **paragraph**\n>\n> - [x] nested task\n> - [ ] pending",
  );

  expect(parsed).toMatchObject([
    {
      kind: "blockquote",
      blocks: [
        { kind: "paragraph" },
        {
          kind: "list",
          items: [{ checked: true }, { checked: false }],
        },
      ],
    },
  ]);
});

test("reuses settled blocks while a streaming tail grows", () => {
  const initial = parseMarkdown(
    "## Result\n\nFirst paragraph.\n\nPartial",
    true,
  );
  const next = parseMarkdown(
    "## Result\n\nFirst paragraph.\n\nPartial answer",
    true,
  );
  const reconciled = reconcileMarkdownBlocks(initial, next);

  expect(reconciled[0]).toBe(initial[0]);
  expect(reconciled[1]).toBe(initial[1]);
  expect(reconciled[2]).not.toBe(initial[2]);
  expect(reconcileMarkdownBlocks(reconciled, next)).toBe(reconciled);
});

test("renders reactive GFM as native semantic components", () => {
  const [source, setSource] = createSignal(
    "## Result\n\n- **Fast** updates\n\n```ts\nconst ready = true\n```",
  );
  const screen = renderComponent(() => (
    <Markdown source={source()} aria-label="Agent response" />
  ));

  const response = screen.getByRole("region", { name: "Agent response" });
  expect(response.text).toContain("Result");
  expect(response.text).toContain("Fast updates");
  expect(screen.getByRole("group", { name: "Code block" }).text).toContain(
    "const ready = true",
  );

  setSource("A streamed **answer**.");
  screen.flush();
  expect(screen.getByRole("region", { name: "Agent response" }).text).toContain(
    "A streamed answer.",
  );
  expect(screen.queryByRole("group", { name: "Code block" })).toBeNull();
});

test("renders compact conversation Markdown including GFM tables", () => {
  const screen = renderComponent(() => (
    <Markdown
      variant="conversation"
      aria-label="Compact response"
      source={
        "## Result\n\n| File | State |\n| --- | --- |\n| api.ts | Updated |"
      }
    />
  ));

  const response = screen.getByRole("region", { name: "Compact response" });
  expect(response.className).toContain("gap-3");
  expect(response.text).toContain("FileState");
  expect(response.text).toContain("api.tsUpdated");
  expect(response.snapshot()).toMatchObject({ role: "region" });
});

test("repairs incomplete inline Markdown only while streaming", () => {
  const [streaming, setStreaming] = createSignal(true);
  const screen = renderComponent(() => (
    <Markdown
      source="The **important answer"
      streaming={streaming()}
      aria-label="Streaming response"
    />
  ));

  const response = screen.getByRole("region", { name: "Streaming response" });
  expect(response.text).toBe("The important answer");

  setStreaming(false);
  screen.flush();
  expect(response.text).toContain("**important answer");
});
