import { mergeClasses } from "@wabou/core/style";
import { createMemo, For, type JSX, untrack } from "solid-js";
import { createKeyframeAnimation, useReducedMotion } from "../animation";
import { RichText, RichTextSpan, Text, View } from "../primitives";
import { CodeBlock } from "./code-block";
import {
  type MarkdownBlock as MarkdownBlockModel,
  type MarkdownRun,
  parseMarkdown,
  reconcileMarkdownBlocks,
} from "./markdown-model";
import { Separator } from "./separator";

export type MarkdownVariant = "document" | "conversation";

function runClass(run: MarkdownRun): string | undefined {
  return mergeClasses(
    run.style.strong && "font-semibold text-primary",
    run.style.emphasis && "italic text-primary",
    run.style.code && "font-mono text-sm font-normal text-primary",
    run.style.deleted && "text-muted",
    run.style.href && "font-medium text-accent",
  );
}

function InlineMarkdown(props: {
  runs: MarkdownRun[];
  variant: MarkdownVariant;
  class?: string;
  animateRun?: (run: MarkdownRun) => boolean;
}): JSX.Element {
  return (
    <RichText
      class={mergeClasses(
        "min-w-0 whitespace-normal",
        props.variant === "conversation"
          ? "text-sm leading-relaxed text-primary"
          : "text-base leading-relaxed text-secondary",
        props.class,
      )}
    >
      <For each={props.runs}>
        {(run) => (
          <MarkdownSpan run={run} reveal={props.animateRun?.(run) ?? false} />
        )}
      </For>
    </RichText>
  );
}

function MarkdownSpan(props: { run: MarkdownRun; reveal: boolean }) {
  const reducedMotion = useReducedMotion();
  const reveal = untrack(() => props.reveal)
    ? createKeyframeAnimation([0.28, 1], {
        duration: 0.18,
        ease: "easeOut",
        reducedMotion,
        reducedValue: 1,
      }).value
    : () => 1;
  return (
    <RichTextSpan
      class={runClass(props.run)}
      data-motion={props.reveal ? "markdown-stream-reveal" : undefined}
      style={{ opacity: reveal() }}
    >
      {props.run.text}
    </RichTextSpan>
  );
}

function Heading(props: {
  block: Extract<MarkdownBlockModel, { kind: "heading" }>;
  variant: MarkdownVariant;
  animateRun?: (run: MarkdownRun) => boolean;
}): JSX.Element {
  if (props.variant === "conversation") {
    const className = () => {
      switch (props.block.depth) {
        case 1:
          return "text-xl font-semibold tracking-tight text-primary whitespace-normal";
        case 2:
          return "text-lg font-semibold tracking-tight text-primary whitespace-normal";
        case 3:
          return "text-base font-semibold tracking-tight text-primary whitespace-normal";
        default:
          return "text-sm font-semibold text-primary whitespace-normal";
      }
    };
    return (
      <InlineMarkdown
        runs={props.block.runs}
        variant={props.variant}
        class={className()}
        animateRun={props.animateRun}
      />
    );
  }
  switch (props.block.depth) {
    case 1:
      return (
        <InlineMarkdown
          runs={props.block.runs}
          variant={props.variant}
          class="text-4xl font-bold text-primary"
          animateRun={props.animateRun}
        />
      );
    case 2:
      return (
        <InlineMarkdown
          runs={props.block.runs}
          variant={props.variant}
          class="text-3xl font-semibold text-primary"
          animateRun={props.animateRun}
        />
      );
    case 3:
      return (
        <InlineMarkdown
          runs={props.block.runs}
          variant={props.variant}
          class="text-2xl font-semibold text-primary"
          animateRun={props.animateRun}
        />
      );
    default:
      return (
        <InlineMarkdown
          runs={props.block.runs}
          variant={props.variant}
          class="text-xl font-semibold text-primary"
          animateRun={props.animateRun}
        />
      );
  }
}

function MarkdownBlocks(props: {
  blocks: MarkdownBlockModel[];
  variant: MarkdownVariant;
  animateRun?: (run: MarkdownRun) => boolean;
}): JSX.Element {
  return (
    <For each={props.blocks}>
      {(block) => (
        <MarkdownBlock
          block={block}
          variant={props.variant}
          animateRun={props.animateRun}
        />
      )}
    </For>
  );
}

function MarkdownList(props: {
  block: Extract<MarkdownBlockModel, { kind: "list" }>;
  variant: MarkdownVariant;
  animateRun?: (run: MarkdownRun) => boolean;
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
              <MarkdownBlocks
                blocks={item.blocks}
                variant={props.variant}
                animateRun={props.animateRun}
              />
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
  animateRun?: (run: MarkdownRun) => boolean;
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
              {(runs, columnIndex) => (
                <View
                  class={mergeClasses(
                    "min-w-0 flex-1 px-3 py-2",
                    columnIndex() + 1 < row.length && "border-r border-subtle",
                  )}
                >
                  <InlineMarkdown
                    runs={runs}
                    variant={props.variant}
                    class={mergeClasses(
                      props.block.align[columnIndex()] === "center" &&
                        "text-center",
                      props.block.align[columnIndex()] === "right" &&
                        "text-right",
                    )}
                    animateRun={props.animateRun}
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
  block: MarkdownBlockModel;
  variant: MarkdownVariant;
  animateRun?: (run: MarkdownRun) => boolean;
}): JSX.Element {
  const block = props.block;
  switch (block.kind) {
    case "heading":
      return (
        <Heading
          block={block}
          variant={props.variant}
          animateRun={props.animateRun}
        />
      );
    case "paragraph":
      return (
        <InlineMarkdown
          runs={block.runs}
          variant={props.variant}
          animateRun={props.animateRun}
        />
      );
    case "blockquote":
      return (
        <View class="min-w-0 flex flex-row items-stretch gap-3">
          <View
            aria-hidden="true"
            class="w-1 flex-none rounded-full bg-strong"
          />
          <View class="min-w-0 flex-1 flex flex-col gap-2">
            <MarkdownBlocks
              blocks={block.blocks}
              variant={props.variant}
              animateRun={props.animateRun}
            />
          </View>
        </View>
      );
    case "list":
      return (
        <MarkdownList
          block={block}
          variant={props.variant}
          animateRun={props.animateRun}
        />
      );
    case "table":
      return (
        <MarkdownTable
          block={block}
          variant={props.variant}
          animateRun={props.animateRun}
        />
      );
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

function visitMarkdownRuns(
  blocks: readonly MarkdownBlockModel[],
  visit: (run: MarkdownRun) => void,
) {
  for (const block of blocks) {
    switch (block.kind) {
      case "heading":
      case "paragraph":
        block.runs.forEach(visit);
        break;
      case "blockquote":
        visitMarkdownRuns(block.blocks, visit);
        break;
      case "list":
        block.items.forEach((item) => {
          visitMarkdownRuns(item.blocks, visit);
        });
        break;
      case "table":
        block.header.forEach((runs) => {
          runs.forEach(visit);
        });
        block.rows.forEach((row) => {
          row.forEach((runs) => {
            runs.forEach(visit);
          });
        });
        break;
      default:
        break;
    }
  }
}

/** Parses GFM in JavaScript and renders native Wabou components, without HTML or a DOM. */
export function Markdown(props: MarkdownProps): JSX.Element {
  let previous: MarkdownBlockModel[] = [];
  let initialized = false;
  const knownRuns = new WeakSet<MarkdownRun>();
  const blocks = createMemo(() => {
    previous = reconcileMarkdownBlocks(
      previous,
      parseMarkdown(props.source, props.streaming),
    );
    if (!initialized) {
      visitMarkdownRuns(previous, (run) => {
        knownRuns.add(run);
      });
      initialized = true;
    }
    return previous;
  });
  const variant = () => props.variant ?? "document";
  const animateRun = (run: MarkdownRun) => {
    const reveal =
      untrack(() => props.streaming === true) && !knownRuns.has(run);
    knownRuns.add(run);
    return reveal;
  };
  return (
    <View
      role="region"
      aria-label={props["aria-label"] ?? "Markdown"}
      class={mergeClasses(
        "min-w-0 flex flex-col",
        variant() === "conversation" ? "gap-3" : "gap-4",
        props.class,
      )}
    >
      <MarkdownBlocks
        blocks={blocks()}
        variant={variant()}
        animateRun={animateRun}
      />
    </View>
  );
}
