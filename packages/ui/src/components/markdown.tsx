import { mergeClasses } from "@wabou/core/style";
import {
  createMemo,
  For as ForValue,
  type JSX,
  Match,
  Switch as SolidSwitch,
  untrack,
} from "solid-js";
import { createKeyframeAnimation, useReducedMotion } from "../animation";
import { RichText, RichTextSpan, Text, View } from "../primitives";
import { CodeBlock } from "./code-block";
import {
  type MarkdownBlock as MarkdownBlockModel,
  MarkdownDocument,
  type MarkdownRun,
} from "./markdown-model";
import { Separator } from "./separator";

export type MarkdownVariant = "document" | "conversation" | "prompt";

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
          ? "text-base leading-relaxed text-primary"
          : props.variant === "prompt"
            ? "text-sm leading-relaxed text-primary"
            : "text-base leading-relaxed text-secondary",
        props.class,
      )}
    >
      <ForValue each={props.runs}>
        {(run) => (
          <MarkdownSpan run={run} reveal={props.animateRun?.(run) ?? false} />
        )}
      </ForValue>
    </RichText>
  );
}

function MarkdownSpan(props: { run: MarkdownRun; reveal: boolean }) {
  const reducedMotion = useReducedMotion();
  const reveal = untrack(() => props.reveal)
    ? createKeyframeAnimation([0.72, 1], {
        duration: 0.12,
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
  const className = createMemo(() => {
    if (props.variant === "prompt") {
      return props.block.depth === 1
        ? "text-base font-semibold text-primary whitespace-normal"
        : "text-sm font-semibold text-primary whitespace-normal";
    }
    if (props.variant === "conversation") {
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
    }
    switch (props.block.depth) {
      case 1:
        return "text-4xl font-bold text-primary";
      case 2:
        return "text-3xl font-semibold text-primary";
      case 3:
        return "text-2xl font-semibold text-primary";
      default:
        return "text-xl font-semibold text-primary";
    }
  });
  return (
    <InlineMarkdown
      runs={props.block.runs}
      variant={props.variant}
      class={className()}
      animateRun={props.animateRun}
    />
  );
}

function MarkdownBlocks(props: {
  blocks: MarkdownBlockModel[];
  variant: MarkdownVariant;
  animateRun?: (run: MarkdownRun) => boolean;
}): JSX.Element {
  return (
    <ForValue each={props.blocks} keyed={false}>
      {(block) => (
        <MarkdownBlock
          block={block()}
          variant={props.variant}
          animateRun={props.animateRun}
        />
      )}
    </ForValue>
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
        props.variant === "conversation" || props.variant === "prompt"
          ? "flex flex-col gap-1.5"
          : "flex flex-col gap-2"
      }
    >
      <ForValue each={props.block.items}>
        {(item, index) => (
          <View class="min-w-0 flex flex-row items-start gap-2">
            <Text
              aria-hidden="true"
              class={mergeClasses(
                "flex-none leading-relaxed text-secondary",
                props.variant === "prompt" ? "text-sm" : "text-base",
              )}
            >
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
      </ForValue>
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
      <ForValue each={rows()}>
        {(row, rowIndex) => (
          <View
            class={mergeClasses(
              "min-w-0 flex flex-row border-b border-subtle",
              rowIndex() === 0 ? "bg-control" : "bg-surface",
            )}
          >
            <ForValue each={row}>
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
            </ForValue>
          </View>
        )}
      </ForValue>
    </View>
  );
}

function MarkdownBlock(props: {
  block: MarkdownBlockModel;
  variant: MarkdownVariant;
  animateRun?: (run: MarkdownRun) => boolean;
}): JSX.Element {
  const heading = () =>
    props.block.kind === "heading" ? props.block : undefined;
  const paragraph = () =>
    props.block.kind === "paragraph" ? props.block : undefined;
  const blockquote = () =>
    props.block.kind === "blockquote" ? props.block : undefined;
  const list = () => (props.block.kind === "list" ? props.block : undefined);
  const table = () => (props.block.kind === "table" ? props.block : undefined);
  const code = () => (props.block.kind === "code" ? props.block : undefined);
  const literal = () =>
    props.block.kind === "literal" ? props.block : undefined;
  return (
    <SolidSwitch>
      <Match when={heading()}>
        {(block) => (
          <Heading
            block={block()}
            variant={props.variant}
            animateRun={props.animateRun}
          />
        )}
      </Match>
      <Match when={paragraph()}>
        {(block) => (
          <InlineMarkdown
            runs={block().runs}
            variant={props.variant}
            animateRun={props.animateRun}
          />
        )}
      </Match>
      <Match when={blockquote()}>
        {(block) => (
          <View class="min-w-0 flex flex-row items-stretch gap-3">
            <View
              aria-hidden="true"
              class="w-1 flex-none rounded-full bg-strong"
            />
            <View class="min-w-0 flex-1 flex flex-col gap-2">
              <MarkdownBlocks
                blocks={block().blocks}
                variant={props.variant}
                animateRun={props.animateRun}
              />
            </View>
          </View>
        )}
      </Match>
      <Match when={list()}>
        {(block) => (
          <MarkdownList
            block={block()}
            variant={props.variant}
            animateRun={props.animateRun}
          />
        )}
      </Match>
      <Match when={table()}>
        {(block) => (
          <MarkdownTable
            block={block()}
            variant={props.variant}
            animateRun={props.animateRun}
          />
        )}
      </Match>
      <Match when={code()}>
        {(block) => (
          <CodeBlock
            code={block().code}
            language={block().language ?? "text"}
            copyLabel="Copy code"
          />
        )}
      </Match>
      <Match when={props.block.kind === "rule"}>
        <Separator />
      </Match>
      <Match when={literal()}>
        {(block) => (
          <Text
            class={mergeClasses(
              "text-muted whitespace-normal leading-relaxed",
              props.variant === "prompt" ? "text-sm" : "text-base",
            )}
          >
            {block().text}
          </Text>
        )}
      </Match>
    </SolidSwitch>
  );
}

export interface MarkdownProps {
  source: string;
  /** Repair an incomplete Markdown tail while text is still arriving. */
  streaming?: boolean;
  /** Document typography by default; conversation and prompt stay message-sized. */
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
  const document = new MarkdownDocument();
  let initialized = false;
  const knownRuns = new WeakSet<MarkdownRun>();
  const blocks = createMemo(() => {
    const parsed = document.setSource(props.source, props.streaming);
    if (!initialized) {
      visitMarkdownRuns(parsed, (run) => {
        knownRuns.add(run);
      });
      initialized = true;
    }
    return parsed;
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
        variant() === "conversation" && "w-full",
        variant() === "document"
          ? "gap-4"
          : variant() === "prompt"
            ? "gap-2"
            : "gap-3",
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
