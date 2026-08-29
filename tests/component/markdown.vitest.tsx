import { renderComponent } from "@wabou/test/component";
import { Markdown } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  type MarkdownBlock,
  MarkdownDocument,
  type MarkdownRun,
  parseMarkdown,
  reconcileMarkdownBlocks,
} from "../../packages/ui/src/components/markdown-model";

function mergeAdjacentRuns(runs: readonly MarkdownRun[]): MarkdownRun[] {
  const merged: MarkdownRun[] = [];
  for (const run of runs) {
    const previous = merged.at(-1);
    if (
      previous &&
      JSON.stringify(previous.style) === JSON.stringify(run.style)
    ) {
      previous.text += run.text;
    } else {
      merged.push({ text: run.text, style: { ...run.style } });
    }
  }
  return merged;
}

function semanticBlocks(blocks: readonly MarkdownBlock[]): MarkdownBlock[] {
  return blocks.map((block) => {
    switch (block.kind) {
      case "heading":
      case "paragraph":
        return { ...block, runs: mergeAdjacentRuns(block.runs) };
      case "blockquote":
        return { ...block, blocks: semanticBlocks(block.blocks) };
      case "list":
        return {
          ...block,
          items: block.items.map((item) => ({
            ...item,
            blocks: semanticBlocks(item.blocks),
          })),
        };
      case "table":
        return {
          ...block,
          header: block.header.map(mergeAdjacentRuns),
          rows: block.rows.map((row) => row.map(mergeAdjacentRuns)),
        };
      default:
        return block;
    }
  });
}

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
  if (initial[2]?.kind !== "paragraph" || reconciled[2]?.kind !== "paragraph")
    throw new Error("expected paragraph tail");
  expect(reconciled[2].runs[0]).toBe(initial[2].runs[0]);
  expect(reconciled[2].runs[1]?.text).toBe(" answer");
  expect(reconcileMarkdownBlocks(reconciled, next)).toBe(reconciled);
});

test("incremental document retains the stable prefix across streaming appends", () => {
  const document = new MarkdownDocument();
  const initial = document.setSource(
    "# Result\n\nStable one.\n\nStable two.\n\nPartial",
    true,
  );
  const next = document.setSource(
    "# Result\n\nStable one.\n\nStable two.\n\nPartial answer",
    true,
  );

  expect(next[0]).toBe(initial[0]);
  expect(next[1]).toBe(initial[1]);
  expect(next[2]).toBe(initial[2]);
  expect(next.at(-1)).not.toBe(initial.at(-1));
  expect(document.lastParse.incremental).toBe(true);
  expect(document.lastParse.parsedBytes).toBeLessThan(
    document.lastParse.sourceBytes,
  );
});

test("incremental document reparses non-local reference definitions", () => {
  const document = new MarkdownDocument();
  document.setSource("[Wabou][site]\n\nA stable paragraph.", true);
  const next = document.setSource(
    "[Wabou][site]\n\nA stable paragraph.\n\n[site]: https://wabou.dev",
    true,
  );

  expect(next[0]).toMatchObject({
    kind: "paragraph",
    runs: [{ text: "Wabou", style: { href: "https://wabou.dev" } }],
  });
  expect(document.lastParse.incremental).toBe(false);
});

test.each([
  [
    "ordinary blocks",
    "# Heading\n\nA paragraph with **bold**.\n\n- one\n- two\n\n```js\nlet x = 1;\n```\n\nTail.",
  ],
  [
    "GFM tables",
    "**After** — a table:\n\n| State | Fill | Icon |\n|---|---|---|\n| Rest | white | circle |\n| Hover | blue | arrow |\n\nDone.",
  ],
  [
    "inline images",
    "before ![a shot](https://example.com/x.png) after, and more prose.",
  ],
])("incremental %s appends agree with a fresh streaming parse", (_name, source) => {
  for (const chunkSize of [1, 2, 3, 7, 17, 64]) {
    const document = new MarkdownDocument();
    let accumulated = "";
    const characters = Array.from(source);
    for (let index = 0; index < characters.length; index += chunkSize) {
      accumulated += characters.slice(index, index + chunkSize).join("");
      expect(semanticBlocks(document.setSource(accumulated, true))).toEqual(
        semanticBlocks(parseMarkdown(accumulated, true)),
      );
    }
  }
});

test("marks only text appended after mount for a streaming reveal", () => {
  const [source, setSource] = createSignal("Settled text");
  const screen = renderComponent(() => (
    <Markdown source={source()} streaming aria-label="Streaming reveal" />
  ));
  const response = screen.getByRole("region", { name: "Streaming reveal" });
  expect(JSON.stringify(response.snapshot())).not.toContain('"opacity":"0.72"');

  setSource("Settled text arrives");
  screen.flush();
  expect(JSON.stringify(response.snapshot())).toContain('"opacity":"0.72"');
});

test("retains the streaming tail block while appending text", () => {
  const [source, setSource] = createSignal("Partial");
  const screen = renderComponent(() => (
    <Markdown source={source()} streaming aria-label="Retained stream" />
  ));
  const response = screen.getByRole("region", { name: "Retained stream" });
  const paragraph = response.children[0];
  expect(paragraph?.text).toBe("Partial");

  setSource("Partial answer");
  screen.flush();

  expect(paragraph?.text).toBe("Partial answer");
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
  expect(screen.getByRole("button", { name: "Copy code" }).text).toBe("");

  setSource("A streamed **answer**.");
  screen.flush();
  expect(screen.getByRole("region", { name: "Agent response" }).text).toContain(
    "A streamed answer.",
  );
  expect(screen.queryByRole("group", { name: "Code block" })).toBeNull();
});

test("renders readable conversation Markdown including GFM tables", () => {
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
  expect(response.className).toContain("w-full");
  expect(response.className).toContain("gap-2.5");
  expect(JSON.stringify(response.snapshot())).toContain(
    "text-base leading-relaxed text-primary",
  );
  expect(JSON.stringify(response.snapshot())).toContain(
    "text-lg font-semibold tracking-tight",
  );
  expect(response.text).toContain("FileState");
  expect(response.text).toContain("api.tsUpdated");
  expect(response.snapshot()).toMatchObject({ role: "region" });
});

test("keeps prompt Markdown readable inside a message bubble", () => {
  const screen = renderComponent(() => (
    <Markdown
      variant="prompt"
      aria-label="User prompt"
      source={"# Request\n\n- inspect `src/main.rs`\n- run **tests**"}
    />
  ));

  const prompt = screen.getByRole("region", { name: "User prompt" });
  expect(prompt.className).not.toContain("w-full");
  expect(prompt.className).toContain("gap-2");
  expect(JSON.stringify(prompt.snapshot())).toContain("text-base");
  expect(prompt.text).toContain("inspect src/main.rs");
  expect(prompt.text).not.toContain("**");
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
