import { EditorState } from "@codemirror/state";
import { TreeFragment, type Tree } from "@lezer/common";
import { highlightTree, tagHighlighter, tags } from "@lezer/highlight";
import { parser as jsonParser } from "@lezer/json";

export type CodeEditorLanguage = "json";

export type CodeEditorHighlightKind =
  | "property"
  | "string"
  | "number"
  | "boolean"
  | "null";

export interface CodeEditorHighlightRange {
  /** CodeMirror document offsets are UTF-16 code-unit offsets. */
  from: number;
  to: number;
  kind: CodeEditorHighlightKind;
}

export interface CodeEditorWidgetConfig {
  syntax: {
    language: CodeEditorLanguage;
    offsetEncoding: "utf16";
    documentLength: number;
    ranges: readonly CodeEditorHighlightRange[];
  } | null;
}

const jsonHighlighter = tagHighlighter([
  { tag: tags.propertyName, class: "property" },
  { tag: tags.string, class: "string" },
  { tag: tags.number, class: "number" },
  { tag: tags.bool, class: "boolean" },
  { tag: tags.null, class: "null" },
]);

function changedRange(previous: string, next: string) {
  let prefix = 0;
  const shared = Math.min(previous.length, next.length);
  while (
    prefix < shared &&
    previous.charCodeAt(prefix) === next.charCodeAt(prefix)
  ) {
    prefix += 1;
  }
  let previousEnd = previous.length;
  let nextEnd = next.length;
  while (
    previousEnd > prefix &&
    nextEnd > prefix &&
    previous.charCodeAt(previousEnd - 1) === next.charCodeAt(nextEnd - 1)
  ) {
    previousEnd -= 1;
    nextEnd -= 1;
  }
  return { prefix, previousEnd, nextEnd };
}

/**
 * Headless CodeMirror document state used by the native CodeEditor viewport.
 *
 * This deliberately has no DOM/View dependency. Native input may report the
 * complete value today, while this adapter turns it back into one incremental
 * CodeMirror transaction and an incremental Lezer parse.
 */
export class CodeEditorDocument {
  #state: EditorState;
  #tree: Tree | null = null;
  #language: CodeEditorLanguage | undefined;

  constructor(value = "", language?: CodeEditorLanguage) {
    this.#state = EditorState.create({ doc: value });
    this.setLanguage(language);
  }

  get value(): string {
    return this.#state.doc.toString();
  }

  setLanguage(language?: CodeEditorLanguage): void {
    if (language === this.#language) return;
    this.#language = language;
    this.#tree = language === "json" ? jsonParser.parse(this.value) : null;
  }

  update(value: string, language = this.#language): CodeEditorWidgetConfig {
    this.setLanguage(language);
    if (value !== this.value) {
      const previous = this.value;
      const { prefix, previousEnd, nextEnd } = changedRange(previous, value);
      const transaction = this.#state.update({
        changes: {
          from: prefix,
          to: previousEnd,
          insert: value.slice(prefix, nextEnd),
        },
      });
      const changedRanges: Array<{
        fromA: number;
        toA: number;
        fromB: number;
        toB: number;
      }> = [];
      transaction.changes.iterChangedRanges((fromA, toA, fromB, toB) => {
        changedRanges.push({ fromA, toA, fromB, toB });
      });
      const fragments = this.#tree
        ? TreeFragment.applyChanges(
            TreeFragment.addTree(this.#tree),
            changedRanges,
          )
        : undefined;
      this.#state = transaction.state;
      this.#tree =
        this.#language === "json"
          ? jsonParser.parse(this.value, fragments)
          : null;
    }

    if (!this.#tree || !this.#language) return { syntax: null };
    const ranges: CodeEditorHighlightRange[] = [];
    highlightTree(this.#tree, jsonHighlighter, (from, to, kind) => {
      ranges.push({
        from,
        to,
        kind: kind as CodeEditorHighlightKind,
      });
    });
    return {
      syntax: {
        language: this.#language,
        offsetEncoding: "utf16",
        documentLength: this.#state.doc.length,
        ranges,
      },
    };
  }
}
