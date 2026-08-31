import { Button, Editor, Markdown, Text, View } from "@wabou/ui";
import { createSignal, type JSX, onCleanup } from "solid-js";

const initialMarkdown = `# Wabou Markdown

Edit this document and the native preview updates immediately.

## Why this example matters

- **Marked** parses Markdown without a DOM.
- Solid updates only the affected preview nodes.
- Wabou owns layout, text, scrolling, and native rendering.

> JavaScript libraries can provide algorithms without bringing a browser engine.

### A small example

Use \`createMemo\` to avoid parsing unrelated reactive updates:

\`\`\`ts
const tokens = createMemo(() => lexer(source()));
\`\`\`

Read the [Wabou repository](https://github.com/SunDoge/wabou) for more details.
`;

const streamingChunks = [
  "# Streaming Markdown\n\n",
  "Tokens can arrive from an LLM, ",
  "a background Rust task, ",
  "or any asynchronous JavaScript source.\n\n",
  "- Parse in JavaScript\n",
  "- Diff with Solid\n",
  "- Render natively with Wabou\n\n",
  "```ts\n",
  "for await (const chunk of response) {\n",
  "  setMarkdown((current) => current + chunk);\n",
  "}\n```\n",
] as const;

export function MarkdownPreview(props: { source: string }): JSX.Element {
  return <Markdown source={props.source} aria-label="Markdown preview" />;
}

export function MarkdownPage(): JSX.Element {
  const [source, setSource] = createSignal(initialMarkdown);
  const [streaming, setStreaming] = createSignal(false);
  let streamTimer: ReturnType<typeof setInterval> | undefined;

  const stopStreaming = () => {
    if (streamTimer !== undefined) clearInterval(streamTimer);
    streamTimer = undefined;
    setStreaming(false);
  };
  const startStreaming = () => {
    stopStreaming();
    setSource("");
    setStreaming(true);
    let index = 0;
    streamTimer = setInterval(() => {
      const chunk = streamingChunks[index++];
      if (chunk !== undefined) setSource((current) => current + chunk);
      if (index >= streamingChunks.length) stopStreaming();
    }, 80);
  };
  onCleanup(stopStreaming);

  return (
    <View class="min-w-0 flex flex-col gap-4">
      <View class="flex flex-row flex-wrap items-center justify-between gap-3">
        <Text class="text-sm text-secondary whitespace-normal">
          Type directly or replay an asynchronous Markdown stream.
        </Text>
        <Button
          size="sm"
          variant="outline"
          onClick={streaming() ? stopStreaming : startStreaming}
        >
          {streaming() ? "Stop streaming" : "Stream example"}
        </Button>
      </View>
      <View class="flex flex-row flex-wrap items-stretch gap-4">
        <View class="min-w-72 flex-1 h-[560px] flex flex-col overflow-hidden rounded-lg border border-subtle bg-control">
          <View class="h-10 flex-none px-4 flex items-center border-b border-subtle">
            <Text class="text-xs font-medium text-muted">MARKDOWN</Text>
          </View>
          <Editor
            aria-label="Markdown source"
            class="w-full min-h-0 flex-1"
            value={source()}
            onInput={(event) => setSource(event.currentTarget.value)}
          />
        </View>
        <View class="min-w-72 flex-1 h-[560px] flex flex-col overflow-hidden rounded-lg border border-subtle bg-surface">
          <View class="h-10 flex-none px-4 flex items-center border-b border-subtle">
            <Text class="text-xs font-medium text-muted">NATIVE PREVIEW</Text>
          </View>
          <View class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto p-6">
            <MarkdownPreview source={source()} />
          </View>
        </View>
      </View>
      <Text class="text-sm text-muted whitespace-normal">
        Marked provides the parser only. The preview maps its tokens to Wabou
        typography, links, lists, separators, and code blocks without creating
        HTML or relying on DOM APIs.
      </Text>
    </View>
  );
}
