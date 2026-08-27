import { mergeClasses } from "@wabou/core/style";
import { lexer, type Token, type Tokens } from "marked";
import { createMemo, For, type JSX } from "solid-js";
import { RichText, RichTextSpan, Text, View } from "../primitives";
import { CodeBlock } from "./code-block";
import { Separator } from "./separator";
import {
  TypographyBlockquote,
  TypographyH1,
  TypographyH2,
  TypographyH3,
  TypographyH4,
} from "./typography";

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
            case "strong":
              return (
                <RichTextSpan class="font-semibold text-primary">
                  {inlineText((token as Tokens.Strong).tokens)}
                </RichTextSpan>
              );
            case "em":
              return (
                <RichTextSpan class="italic text-primary">
                  {inlineText((token as Tokens.Em).tokens)}
                </RichTextSpan>
              );
            case "codespan":
              return (
                <RichTextSpan class="font-mono text-sm font-normal text-primary">
                  {token.text}
                </RichTextSpan>
              );
            case "link":
              return (
                <RichTextSpan class="text-accent">
                  {inlineText((token as Tokens.Link).tokens)}
                </RichTextSpan>
              );
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
  const text = () => inlineText(props.token.tokens);
  switch (props.token.depth) {
    case 1:
      return <TypographyH1>{text()}</TypographyH1>;
    case 2:
      return <TypographyH2>{text()}</TypographyH2>;
    case 3:
      return <TypographyH3>{text()}</TypographyH3>;
    default:
      return <TypographyH4>{text()}</TypographyH4>;
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

export interface MarkdownProps {
  source: string;
  class?: string;
  "aria-label"?: string;
}

/** Parses GFM in JavaScript and renders native Wabou components, without HTML or a DOM. */
export function Markdown(props: MarkdownProps): JSX.Element {
  const tokens = createMemo(() => lexer(props.source, { gfm: true }));
  return (
    <View
      role="region"
      aria-label={props["aria-label"] ?? "Markdown"}
      class={mergeClasses("min-w-0 flex flex-col gap-4", props.class)}
    >
      <For each={tokens()}>{(token) => <MarkdownBlock token={token} />}</For>
    </View>
  );
}
