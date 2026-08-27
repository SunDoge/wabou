import {
  Button,
  CodeBlock,
  CodeEditor,
  RichText,
  RichTextSpan,
  Separator,
  Text,
  TypographyBlockquote,
  TypographyH1,
  TypographyH2,
  TypographyH3,
  TypographyH4,
  View,
} from "@wabou/ui";
import { lexer, type Token, type Tokens } from "marked";
import { createMemo, createSignal, For, type JSX, onCleanup } from "solid-js";

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

function inlineText(tokens: Token[]): string {
  return tokens
    .map((token) => {
      if (token.type === "br") return "\n";
      if ("tokens" in token && Array.isArray(token.tokens)) {
        return inlineText(token.tokens);
      }
      return "text" in token && typeof token.text === "string"
        ? token.text
        : "";
    })
    .join("");
}

function InlineMarkdown(props: { tokens: Token[] }): JSX.Element {
  return (
    <RichText class="min-w-0 text-secondary whitespace-normal">
      <For each={props.tokens}>
        {(token) => {
          switch (token.type) {
            case "strong": {
              const strong = token as Tokens.Strong;
              return (
                <RichTextSpan class="font-semibold text-primary">
                  {inlineText(strong.tokens)}
                </RichTextSpan>
              );
            }
            case "em": {
              const emphasis = token as Tokens.Em;
              return (
                <RichTextSpan class="italic text-primary">
                  {inlineText(emphasis.tokens)}
                </RichTextSpan>
              );
            }
            case "codespan":
              return (
                <RichTextSpan class="font-mono text-sm font-normal text-primary">
                  {token.text}
                </RichTextSpan>
              );
            case "link": {
              const link = token as Tokens.Link;
              return (
                <RichTextSpan class="text-accent">
                  {inlineText(link.tokens)}
                </RichTextSpan>
              );
            }
            case "br":
              return <RichTextSpan>{"\n"}</RichTextSpan>;
            default:
              return (
                <RichTextSpan>
                  {"tokens" in token && Array.isArray(token.tokens)
                    ? inlineText(token.tokens)
                    : "text" in token && typeof token.text === "string"
                      ? token.text
                      : token.raw}
                </RichTextSpan>
              );
          }
        }}
      </For>
    </RichText>
  );
}

function Heading(props: { token: Tokens.Heading }): JSX.Element {
  const content = () => inlineText(props.token.tokens);
  switch (props.token.depth) {
    case 1:
      return <TypographyH1>{content()}</TypographyH1>;
    case 2:
      return <TypographyH2>{content()}</TypographyH2>;
    case 3:
      return <TypographyH3>{content()}</TypographyH3>;
    default:
      return <TypographyH4>{content()}</TypographyH4>;
  }
}

function MarkdownList(props: { token: Tokens.List }): JSX.Element {
  const start = typeof props.token.start === "number" ? props.token.start : 1;
  return (
    <View class="flex flex-col gap-2">
      <For each={props.token.items}>
        {(item, index) => (
          <View class="min-w-0 flex flex-row items-start gap-2">
            <Text aria-hidden="true" class="flex-none text-secondary">
              {props.token.ordered ? `${start + index()}.` : "•"}
            </Text>
            <View class="min-w-0 flex-1">
              <InlineMarkdown tokens={item.tokens} />
            </View>
          </View>
        )}
      </For>
    </View>
  );
}

function MarkdownBlock(props: { token: Token }): JSX.Element {
  const token = props.token;
  switch (token.type) {
    case "heading":
      return <Heading token={token as Tokens.Heading} />;
    case "paragraph":
      return <InlineMarkdown tokens={(token as Tokens.Paragraph).tokens} />;
    case "blockquote":
      return (
        <TypographyBlockquote>
          {inlineText((token as Tokens.Blockquote).tokens)}
        </TypographyBlockquote>
      );
    case "list":
      return <MarkdownList token={token as Tokens.List} />;
    case "code":
      return (
        <CodeBlock
          code={token.text}
          language={token.lang ?? "text"}
          copyable={false}
        />
      );
    case "hr":
      return <Separator />;
    case "space":
      return null;
    case "html":
      return (
        <Text class="text-sm text-muted whitespace-normal">
          HTML blocks are intentionally not rendered.
        </Text>
      );
    default:
      return "tokens" in token && Array.isArray(token.tokens) ? (
        <InlineMarkdown tokens={token.tokens} />
      ) : null;
  }
}

export function MarkdownPreview(props: { source: string }): JSX.Element {
  const tokens = createMemo(() => lexer(props.source, { gfm: true }));
  return (
    <View
      role="region"
      aria-label="Markdown preview"
      class="min-w-0 flex flex-col gap-4"
    >
      <For each={tokens()}>{(token) => <MarkdownBlock token={token} />}</For>
    </View>
  );
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
          <CodeEditor
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
