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

/**
 * Parse GFM into Wabou-owned render data. Marked is deliberately kept behind
 * this adapter so native components never depend on a parser-specific AST.
 */
export function parseMarkdown(
  source: string,
  streaming = false,
): MarkdownBlock[] {
  const repaired = streaming ? remend(source) : source;
  return blocks(lexer(repaired, { gfm: true }));
}
