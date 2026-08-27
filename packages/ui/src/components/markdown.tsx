import { mergeClasses } from "@wabou/core/style";
import { createMemo, For, type JSX } from "solid-js";
import { RichText, RichTextSpan, Text, View } from "../primitives";
import { CodeBlock } from "./code-block";
import {
  type MarkdownBlock as MarkdownBlockModel,
  type MarkdownRun,
  parseMarkdown,
} from "./markdown-model";
import { Separator } from "./separator";
import {
  TypographyH1,
  TypographyH2,
  TypographyH3,
  TypographyH4,
} from "./typography";

export type MarkdownVariant = "document" | "conversation";

function runClass(run: MarkdownRun): string | undefined {
  return mergeClasses(
    run.style.strong && "font-semibold text-primary",
    run.style.emphasis && "italic text-primary",
    run.style.code && "font-mono text-sm font-normal text-primary",
    run.style.deleted && "text-muted",
    run.style.href && "text-accent",
  );
}

function InlineMarkdown(props: {
  runs: MarkdownRun[];
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
      <For each={props.runs}>
        {(run) => <RichTextSpan class={runClass(run)}>{run.text}</RichTextSpan>}
      </For>
    </RichText>
  );
}

function Heading(props: {
  block: Extract<MarkdownBlockModel, { kind: "heading" }>;
  variant: MarkdownVariant;
}): JSX.Element {
  const text = () => props.block.runs.map((run) => run.text).join("");
  if (props.variant === "conversation") {
    const className = () => {
      switch (props.block.depth) {
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
  switch (props.block.depth) {
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

function MarkdownBlocks(props: {
  blocks: MarkdownBlockModel[];
  variant: MarkdownVariant;
}): JSX.Element {
  return (
    <For each={props.blocks}>
      {(block) => <MarkdownBlock block={block} variant={props.variant} />}
    </For>
  );
}

function MarkdownList(props: {
  block: Extract<MarkdownBlockModel, { kind: "list" }>;
  variant: MarkdownVariant;
}): JSX.Element {
  return (
    <View
      class={
        props.variant === "conversation"
          ? "flex flex-col gap-1.5"
          : "flex flex-col gap-2"
      }
    >
      <For each={props.block.items}>
        {(item, index) => (
          <View class="min-w-0 flex flex-row items-start gap-2">
            <Text aria-hidden="true" class="flex-none text-secondary">
              {typeof item.checked === "boolean"
                ? item.checked
                  ? "[x]"
                  : "[ ]"
                : props.block.ordered
                  ? `${props.block.start + index()}.`
                  : "•"}
            </Text>
            <View class="min-w-0 flex-1 flex flex-col gap-1.5">
              <MarkdownBlocks blocks={item.blocks} variant={props.variant} />
            </View>
          </View>
        )}
      </For>
    </View>
  );
}

function MarkdownTable(props: {
  block: Extract<MarkdownBlockModel, { kind: "table" }>;
  variant: MarkdownVariant;
}): JSX.Element {
  const rows = () => [props.block.header, ...props.block.rows];
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
              {(runs) => (
                <View class="min-w-0 flex-1 px-3 py-2 border-r border-subtle">
                  <InlineMarkdown runs={runs} variant={props.variant} />
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
  block: MarkdownBlockModel;
  variant: MarkdownVariant;
}): JSX.Element {
  const block = props.block;
  switch (block.kind) {
    case "heading":
      return <Heading block={block} variant={props.variant} />;
    case "paragraph":
      return <InlineMarkdown runs={block.runs} variant={props.variant} />;
    case "blockquote":
      return (
        <View class="min-w-0 flex flex-row items-stretch gap-3">
          <View
            aria-hidden="true"
            class="w-1 flex-none rounded-full bg-strong"
          />
          <View class="min-w-0 flex-1 flex flex-col gap-2">
            <MarkdownBlocks blocks={block.blocks} variant={props.variant} />
          </View>
        </View>
      );
    case "list":
      return <MarkdownList block={block} variant={props.variant} />;
    case "table":
      return <MarkdownTable block={block} variant={props.variant} />;
    case "code":
      return (
        <CodeBlock
          code={block.code}
          language={block.language ?? "text"}
          copyable={false}
        />
      );
    case "rule":
      return <Separator />;
    case "literal":
      return (
        <Text class="text-sm text-muted whitespace-normal">{block.text}</Text>
      );
  }
}

export interface MarkdownProps {
  source: string;
  /** Repair an incomplete Markdown tail while text is still arriving. */
  streaming?: boolean;
  /** Document typography by default; conversation keeps agent replies compact. */
  variant?: MarkdownVariant;
  class?: string;
  "aria-label"?: string;
}

/** Parses GFM in JavaScript and renders native Wabou components, without HTML or a DOM. */
export function Markdown(props: MarkdownProps): JSX.Element {
  const blocks = createMemo(() => parseMarkdown(props.source, props.streaming));
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
      <MarkdownBlocks blocks={blocks()} variant={variant()} />
    </View>
  );
}
