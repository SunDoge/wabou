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

export type MarkdownVariant = "document" | "conversation";

function InlineMarkdown(props: {
  tokens: Token[];
  variant: MarkdownVariant;
}): JSX.Element {
  return (
    <RichText
      class={mergeClasses(
        "min-w-0 whitespace-normal",
        props.variant === "conversation"
          ? "text-sm leading-relaxed text-primary"
          : "text-base leading-relaxed text-secondary",
      )}
    >
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

function Heading(props: {
  token: Tokens.Heading;
  variant: MarkdownVariant;
}): JSX.Element {
  const text = () => inlineText(props.token.tokens);
  if (props.variant === "conversation") {
    const className = () => {
      switch (props.token.depth) {
        case 1:
          return "text-2xl font-semibold text-primary whitespace-normal";
        case 2:
          return "text-xl font-semibold text-primary whitespace-normal";
        case 3:
          return "text-lg font-semibold text-primary whitespace-normal";
        default:
          return "text-base font-semibold text-primary whitespace-normal";
      }
    };
    return <Text class={className()}>{text()}</Text>;
  }
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

function MarkdownList(props: {
  token: Tokens.List;
  variant: MarkdownVariant;
}): JSX.Element {
  const start = typeof props.token.start === "number" ? props.token.start : 1;
  return (
    <View
      class={
        props.variant === "conversation"
          ? "flex flex-col gap-1.5"
          : "flex flex-col gap-2"
      }
    >
      <For each={props.token.items}>
        {(item, index) => (
          <View class="min-w-0 flex flex-row items-start gap-2">
            <Text aria-hidden="true" class="flex-none text-secondary">
              {props.token.ordered ? `${start + index()}.` : "•"}
            </Text>
            <View class="min-w-0 flex-1">
              <InlineMarkdown tokens={item.tokens} variant={props.variant} />
            </View>
          </View>
        )}
      </For>
    </View>
  );
}

function MarkdownTable(props: {
  token: Tokens.Table;
  variant: MarkdownVariant;
}): JSX.Element {
  const rows = () => [props.token.header, ...props.token.rows];
  return (
    <View class="min-w-0 overflow-hidden rounded-lg border border-subtle">
      <For each={rows()}>
        {(row, rowIndex) => (
          <View
            class={mergeClasses(
              "min-w-0 flex flex-row border-b border-subtle",
              rowIndex() === 0 ? "bg-control" : "bg-surface",
            )}
          >
            <For each={row}>
              {(cell) => (
                <View class="min-w-0 flex-1 px-3 py-2 border-r border-subtle">
                  <InlineMarkdown
                    tokens={cell.tokens}
                    variant={props.variant}
                  />
                </View>
              )}
            </For>
          </View>
        )}
      </For>
    </View>
  );
}

function MarkdownBlock(props: {
  token: Token;
  variant: MarkdownVariant;
}): JSX.Element {
  const token = props.token;
  switch (token.type) {
    case "heading":
      return (
        <Heading token={token as Tokens.Heading} variant={props.variant} />
      );
    case "paragraph":
      return (
        <InlineMarkdown
          tokens={(token as Tokens.Paragraph).tokens}
          variant={props.variant}
        />
      );
    case "blockquote":
      return (
        <TypographyBlockquote
          class={props.variant === "conversation" ? "text-sm" : undefined}
        >
          {inlineText((token as Tokens.Blockquote).tokens)}
        </TypographyBlockquote>
      );
    case "list":
      return (
        <MarkdownList token={token as Tokens.List} variant={props.variant} />
      );
    case "table":
      return (
        <MarkdownTable token={token as Tokens.Table} variant={props.variant} />
      );
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
        <InlineMarkdown tokens={token.tokens} variant={props.variant} />
      ) : null;
  }
}

export interface MarkdownProps {
  source: string;
  /** Document typography by default; conversation keeps agent replies compact. */
  variant?: MarkdownVariant;
  class?: string;
  "aria-label"?: string;
}

/** Parses GFM in JavaScript and renders native Wabou components, without HTML or a DOM. */
export function Markdown(props: MarkdownProps): JSX.Element {
  const tokens = createMemo(() => lexer(props.source, { gfm: true }));
  const variant = () => props.variant ?? "document";
  return (
    <View
      role="region"
      aria-label={props["aria-label"] ?? "Markdown"}
      class={mergeClasses(
        "min-w-0 flex flex-col",
        variant() === "conversation" ? "gap-2.5" : "gap-4",
        props.class,
      )}
    >
      <For each={tokens()}>
        {(token) => <MarkdownBlock token={token} variant={variant()} />}
      </For>
    </View>
  );
}
