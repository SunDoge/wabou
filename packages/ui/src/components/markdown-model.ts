import { lexer, type Token, type Tokens } from "marked";
import remend from "remend";

export interface MarkdownInlineStyle {
  strong?: boolean;
  emphasis?: boolean;
  code?: boolean;
  deleted?: boolean;
  href?: string;
}

export interface MarkdownRun {
  text: string;
  style: MarkdownInlineStyle;
}

export interface MarkdownListItem {
  checked?: boolean;
  blocks: MarkdownBlock[];
}

export type MarkdownBlock =
  | { kind: "heading"; depth: number; runs: MarkdownRun[] }
  | { kind: "paragraph"; runs: MarkdownRun[] }
  | { kind: "blockquote"; blocks: MarkdownBlock[] }
  | {
      kind: "list";
      ordered: boolean;
      start: number;
      items: MarkdownListItem[];
    }
  | {
      kind: "table";
      align: Array<"left" | "center" | "right" | null>;
      header: MarkdownRun[][];
      rows: MarkdownRun[][][];
    }
  | { kind: "code"; code: string; language?: string }
  | { kind: "rule" }
  | { kind: "literal"; text: string };

interface ParsedTopLevelToken {
  start: number;
  end: number;
  blockCount: number;
}

interface ParsedMarkdown {
  blocks: MarkdownBlock[];
  tokens: ParsedTopLevelToken[];
}

// Link reference definitions can change links in already-parsed blocks. Keep
// this deliberately conservative: falling back to a full parse is cheap and
// preserves CommonMark semantics for the uncommon non-local construct.
const LINK_REFERENCE_DEFINITION = /^ {0,3}\[[^\]\n]+\]:/m;

function sameRuns(left: readonly MarkdownRun[], right: readonly MarkdownRun[]) {
  return (
    left.length === right.length &&
    left.every(
      (run, index) =>
        run.text === right[index]?.text &&
        sameStyle(run.style, right[index]?.style ?? {}),
    )
  );
}

function reconcileRuns(
  previous: readonly MarkdownRun[],
  next: readonly MarkdownRun[],
): MarkdownRun[] {
  const reconciled: MarkdownRun[] = [];
  let previousIndex = 0;
  let canReuse = true;
  for (const run of next) {
    let remaining = run.text;
    while (canReuse && remaining.length > 0) {
      const prior = previous[previousIndex];
      if (
        !prior ||
        !sameStyle(prior.style, run.style) ||
        !remaining.startsWith(prior.text)
      ) {
        canReuse = false;
        break;
      }
      reconciled.push(prior);
      remaining = remaining.slice(prior.text.length);
      previousIndex += 1;
    }
    if (remaining) reconciled.push({ text: remaining, style: run.style });
  }
  return previousIndex === previous.length && sameRuns(previous, reconciled)
    ? (previous as MarkdownRun[])
    : reconciled;
}

function reconcileBlock(
  previous: MarkdownBlock | undefined,
  next: MarkdownBlock,
): MarkdownBlock {
  if (!previous || previous.kind !== next.kind) return next;
  if (sameBlock(previous, next)) return previous;
  switch (next.kind) {
    case "heading": {
      if (previous.kind !== "heading") return next;
      const runs = reconcileRuns(previous.runs, next.runs);
      return runs === previous.runs && previous.depth === next.depth
        ? previous
        : { ...next, runs };
    }
    case "paragraph": {
      if (previous.kind !== "paragraph") return next;
      const runs = reconcileRuns(previous.runs, next.runs);
      return runs === previous.runs ? previous : { ...next, runs };
    }
    case "blockquote":
      return {
        ...next,
        blocks:
          previous.kind === "blockquote"
            ? reconcileMarkdownBlocks(previous.blocks, next.blocks)
            : next.blocks,
      };
    case "list":
      return {
        ...next,
        items: next.items.map((item, index) => ({
          ...item,
          blocks:
            previous.kind === "list"
              ? reconcileMarkdownBlocks(
                  previous.items[index]?.blocks ?? [],
                  item.blocks,
                )
              : item.blocks,
        })),
      };
    case "table":
      return {
        ...next,
        header: next.header.map((runs, index) =>
          previous.kind === "table"
            ? reconcileRuns(previous.header[index] ?? [], runs)
            : runs,
        ),
        rows: next.rows.map((row, rowIndex) =>
          row.map((runs, columnIndex) =>
            previous.kind === "table"
              ? reconcileRuns(
                  previous.rows[rowIndex]?.[columnIndex] ?? [],
                  runs,
                )
              : runs,
          ),
        ),
      };
    default:
      return next;
  }
}

function sameBlock(left: MarkdownBlock, right: MarkdownBlock): boolean {
  if (left.kind !== right.kind) return false;
  switch (left.kind) {
    case "heading":
      return (
        right.kind === "heading" &&
        left.depth === right.depth &&
        sameRuns(left.runs, right.runs)
      );
    case "paragraph":
      return right.kind === "paragraph" && sameRuns(left.runs, right.runs);
    case "blockquote":
      return (
        right.kind === "blockquote" && sameBlocks(left.blocks, right.blocks)
      );
    case "list":
      return (
        right.kind === "list" &&
        left.ordered === right.ordered &&
        left.start === right.start &&
        left.items.length === right.items.length &&
        left.items.every(
          (item, index) =>
            item.checked === right.items[index]?.checked &&
            sameBlocks(item.blocks, right.items[index]?.blocks ?? []),
        )
      );
    case "table":
      return (
        right.kind === "table" &&
        left.align.length === right.align.length &&
        left.align.every((align, index) => align === right.align[index]) &&
        sameRunRows(left.header, right.header) &&
        left.rows.length === right.rows.length &&
        left.rows.every((row, index) =>
          sameRunRows(row, right.rows[index] ?? []),
        )
      );
    case "code":
      return (
        right.kind === "code" &&
        left.code === right.code &&
        left.language === right.language
      );
    case "rule":
      return right.kind === "rule";
    case "literal":
      return right.kind === "literal" && left.text === right.text;
  }
}

function sameRunRows(
  left: readonly (readonly MarkdownRun[])[],
  right: readonly (readonly MarkdownRun[])[],
) {
  return (
    left.length === right.length &&
    left.every((runs, index) => sameRuns(runs, right[index] ?? []))
  );
}

function sameBlocks(
  left: readonly MarkdownBlock[],
  right: readonly MarkdownBlock[],
) {
  return (
    left.length === right.length &&
    left.every((block, index) =>
      sameBlock(block, right[index] as MarkdownBlock),
    )
  );
}

/**
 * Preserve unchanged block identities across streaming parses. Solid can then
 * retain the already-rendered native subtree while only replacing the block
 * whose Markdown source is still growing.
 */
export function reconcileMarkdownBlocks(
  previous: readonly MarkdownBlock[],
  next: readonly MarkdownBlock[],
): MarkdownBlock[] {
  let changed = previous.length !== next.length;
  const reconciled = next.map((block, index) => {
    const result = reconcileBlock(previous[index], block);
    if (result !== previous[index]) changed = true;
    return result;
  });
  return changed ? reconciled : (previous as MarkdownBlock[]);
}

function sameStyle(left: MarkdownInlineStyle, right: MarkdownInlineStyle) {
  return (
    left.strong === right.strong &&
    left.emphasis === right.emphasis &&
    left.code === right.code &&
    left.deleted === right.deleted &&
    left.href === right.href
  );
}

function appendRun(
  runs: MarkdownRun[],
  text: string,
  style: MarkdownInlineStyle,
) {
  if (!text) return;
  const previous = runs.at(-1);
  if (previous && sameStyle(previous.style, style)) {
    previous.text += text;
  } else {
    runs.push({ text, style });
  }
}

function inlineRuns(
  tokens: readonly Token[],
  inherited: MarkdownInlineStyle = {},
): MarkdownRun[] {
  const runs: MarkdownRun[] = [];
  for (const token of tokens) {
    const nested = (style: MarkdownInlineStyle) => {
      if ("tokens" in token && Array.isArray(token.tokens)) {
        for (const run of inlineRuns(token.tokens, style)) {
          appendRun(runs, run.text, run.style);
        }
      } else if ("text" in token && typeof token.text === "string") {
        appendRun(runs, token.text, style);
      }
    };

    switch (token.type) {
      case "strong":
        nested({ ...inherited, strong: true });
        break;
      case "em":
        nested({ ...inherited, emphasis: true });
        break;
      case "codespan":
        appendRun(runs, token.text, { ...inherited, code: true });
        break;
      case "del":
        nested({ ...inherited, deleted: true });
        break;
      case "link":
        nested({ ...inherited, href: token.href });
        break;
      case "image":
        appendRun(runs, token.text || token.href, inherited);
        break;
      case "br":
        appendRun(runs, "\n", inherited);
        break;
      case "checkbox":
        break;
      default:
        nested(inherited);
        break;
    }
  }
  return runs;
}

function blocks(tokens: readonly Token[]): MarkdownBlock[] {
  const result: MarkdownBlock[] = [];
  for (const token of tokens) {
    switch (token.type) {
      case "space":
        break;
      case "heading": {
        const heading = token as Tokens.Heading;
        result.push({
          kind: "heading",
          depth: heading.depth,
          runs: inlineRuns(heading.tokens),
        });
        break;
      }
      case "paragraph":
      case "text": {
        const text = token as Tokens.Paragraph | Tokens.Text;
        result.push({
          kind: "paragraph",
          runs: inlineRuns(text.tokens ?? [text]),
        });
        break;
      }
      case "blockquote":
        result.push({
          kind: "blockquote",
          blocks: blocks((token as Tokens.Blockquote).tokens),
        });
        break;
      case "list": {
        const list = token as Tokens.List;
        result.push({
          kind: "list",
          ordered: list.ordered,
          start: typeof list.start === "number" ? list.start : 1,
          items: list.items.map((item: Tokens.ListItem) => ({
            checked:
              typeof item.checked === "boolean" ? item.checked : undefined,
            blocks: blocks(item.tokens),
          })),
        });
        break;
      }
      case "table": {
        const table = token as Tokens.Table;
        result.push({
          kind: "table",
          align: table.align,
          header: table.header.map((cell: Tokens.TableCell) =>
            inlineRuns(cell.tokens),
          ),
          rows: table.rows.map((row: Tokens.TableCell[]) =>
            row.map((cell) => inlineRuns(cell.tokens)),
          ),
        });
        break;
      }
      case "code":
        result.push({
          kind: "code",
          code: token.text,
          language: token.lang || undefined,
        });
        break;
      case "hr":
        result.push({ kind: "rule" });
        break;
      case "html":
        result.push({ kind: "literal", text: token.text });
        break;
      default:
        if ("tokens" in token && Array.isArray(token.tokens)) {
          result.push(...blocks(token.tokens));
        }
        break;
    }
  }
  return result;
}

function parseWithOffsets(source: string): ParsedMarkdown {
  const parsedBlocks: MarkdownBlock[] = [];
  const parsedTokens: ParsedTopLevelToken[] = [];
  let offset = 0;
  for (const token of lexer(source, { gfm: true })) {
    const start = offset;
    offset += token.raw.length;
    const tokenBlocks = blocks([token]);
    parsedBlocks.push(...tokenBlocks);
    parsedTokens.push({ start, end: offset, blockCount: tokenBlocks.length });
  }
  return { blocks: parsedBlocks, tokens: parsedTokens };
}

/**
 * Stateful Markdown parse adapter for append-heavy streams.
 *
 * Like Waku's MarkdownView, it treats the last two top-level source tokens as
 * volatile. Appends retain everything before that boundary and only lex the
 * tail. Parser-specific tokens never escape this module.
 */
export class MarkdownDocument {
  #source = "";
  #streaming = false;
  #blocks: MarkdownBlock[] = [];
  #tokens: ParsedTopLevelToken[] = [];
  #lastParse = { incremental: false, parsedBytes: 0, sourceBytes: 0 };

  get blocks(): readonly MarkdownBlock[] {
    return this.#blocks;
  }

  /** Diagnostics for benchmarks and streaming regression tests. */
  get lastParse(): Readonly<{
    incremental: boolean;
    parsedBytes: number;
    sourceBytes: number;
  }> {
    return this.#lastParse;
  }

  setSource(source: string, streaming = false): MarkdownBlock[] {
    if (source === this.#source && streaming === this.#streaming) {
      return this.#blocks;
    }

    const displaySource = streaming ? remend(source) : source;
    const canAppend =
      streaming &&
      this.#streaming &&
      source.startsWith(this.#source) &&
      !LINK_REFERENCE_DEFINITION.test(source) &&
      this.#tokens.length > 2;

    let next: ParsedMarkdown;
    let parsedBytes = displaySource.length;
    let incremental = false;
    if (canAppend) {
      const stableTokenCount = this.#tokens.length - 2;
      const stableTokens = this.#tokens.slice(0, stableTokenCount);
      const boundary = stableTokens.at(-1)?.end ?? 0;
      // A streaming repair only appends synthetic closers. The stable boundary
      // must still point into the canonical source before it can be reused.
      if (boundary <= source.length && boundary <= displaySource.length) {
        const stableBlockCount = stableTokens.reduce(
          (count, token) => count + token.blockCount,
          0,
        );
        const tail = parseWithOffsets(displaySource.slice(boundary));
        parsedBytes = displaySource.length - boundary;
        incremental = true;
        next = {
          blocks: [...this.#blocks.slice(0, stableBlockCount), ...tail.blocks],
          tokens: [
            ...stableTokens,
            ...tail.tokens.map((token) => ({
              ...token,
              start: token.start + boundary,
              end: token.end + boundary,
            })),
          ],
        };
      } else {
        next = parseWithOffsets(displaySource);
      }
    } else {
      next = parseWithOffsets(displaySource);
    }

    this.#blocks = reconcileMarkdownBlocks(this.#blocks, next.blocks);
    this.#tokens = next.tokens;
    this.#source = source;
    this.#streaming = streaming;
    this.#lastParse = {
      incremental,
      parsedBytes,
      sourceBytes: displaySource.length,
    };
    return this.#blocks;
  }
}

/**
 * Parse GFM into Wabou-owned render data. Marked is deliberately kept behind
 * this adapter so native components never depend on a parser-specific AST.
 */
export function parseMarkdown(
  source: string,
  streaming = false,
): MarkdownBlock[] {
  return new MarkdownDocument().setSource(source, streaming);
}
