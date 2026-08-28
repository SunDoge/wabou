import { EditorSelection, EditorState, findClusterBreak, type TransactionSpec } from "@codemirror/state";
import { TreeFragment, type Tree } from "@lezer/common";
import { highlightTree, tagHighlighter, tags } from "@lezer/highlight";
import { parser as jsonParser } from "@lezer/json";

export type CodeEditorLanguage = "json";
export type CodeEditorHighlightKind = "property" | "string" | "number" | "boolean" | "null";

export interface CodeEditorHighlightRange {
  /** CodeMirror document offsets are UTF-16 code-unit offsets. */
  from: number;
  to: number;
  kind: CodeEditorHighlightKind;
}

export interface CodeEditorWidgetConfig {
  selection: { anchor: number; head: number };
  composition: { text: string; cursorStart: number | null; cursorEnd: number | null } | null;
  syntax: {
    language: CodeEditorLanguage;
    offsetEncoding: "utf16";
    documentLength: number;
    ranges: readonly CodeEditorHighlightRange[];
  } | null;
}

export interface CodeEditorKey {
  key: string;
  shift: boolean;
  primary: boolean;
  readOnly?: boolean;
}

interface Snapshot {
  value: string;
  anchor: number;
  head: number;
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
  while (prefix < shared && previous.charCodeAt(prefix) === next.charCodeAt(prefix)) prefix += 1;
  let previousEnd = previous.length;
  let nextEnd = next.length;
  while (
    previousEnd > prefix && nextEnd > prefix &&
    previous.charCodeAt(previousEnd - 1) === next.charCodeAt(nextEnd - 1)
  ) {
    previousEnd -= 1;
    nextEnd -= 1;
  }
  return { prefix, previousEnd, nextEnd };
}

/**
 * DOM-free CodeMirror document used by Wabou's config/Markdown editor.
 *
 * CodeMirror owns text, selection, transactions and undo. The Rust widget is
 * only a controlled native viewport. A future Helix frontend deliberately
 * uses helix-core instead while reusing the viewport contract.
 */
export class CodeEditorDocument {
  #state: EditorState;
  #tree: Tree | null = null;
  #language: CodeEditorLanguage | undefined;
  #composition: CodeEditorWidgetConfig["composition"] = null;
  #undo: Snapshot[] = [];
  #redo: Snapshot[] = [];

  constructor(value = "", language?: CodeEditorLanguage) {
    this.#state = EditorState.create({ doc: value });
    this.setLanguage(language);
  }

  get value(): string { return this.#state.doc.toString(); }
  get selection(): { anchor: number; head: number } {
    const { anchor, head } = this.#state.selection.main;
    return { anchor, head };
  }

  setLanguage(language?: CodeEditorLanguage): void {
    if (language === this.#language) return;
    this.#language = language;
    this.#tree = language === "json" ? jsonParser.parse(this.value) : null;
  }

  sync(value: string, language = this.#language): void {
    this.setLanguage(language);
    if (value === this.value) return;
    const { prefix, previousEnd, nextEnd } = changedRange(this.value, value);
    this.#apply({ changes: { from: prefix, to: previousEnd, insert: value.slice(prefix, nextEnd) } }, false);
    this.#undo = [];
    this.#redo = [];
  }

  setSelection(anchor: number, head: number): boolean {
    const length = this.#state.doc.length;
    anchor = Math.max(0, Math.min(anchor, length));
    head = Math.max(0, Math.min(head, length));
    const current = this.#state.selection.main;
    if (current.anchor === anchor && current.head === head) return false;
    this.#state = this.#state.update({ selection: EditorSelection.single(anchor, head) }).state;
    return true;
  }

  setComposition(text: string, cursorStart: number | null, cursorEnd: number | null): boolean {
    const next = text ? { text, cursorStart, cursorEnd } : null;
    if (JSON.stringify(next) === JSON.stringify(this.#composition)) return false;
    this.#composition = next;
    return true;
  }

  commitText(text: string): boolean {
    this.#composition = null;
    const { from, to } = this.#state.selection.main;
    return this.#apply({ changes: { from, to, insert: text }, selection: { anchor: from + text.length } });
  }

  deleteSurrounding(beforeBytes: number, afterBytes: number): boolean {
    const head = this.#state.selection.main.head;
    const before = this.#offsetByUtf8Bytes(head, -beforeBytes);
    const after = this.#offsetByUtf8Bytes(head, afterBytes);
    this.#composition = null;
    if (before === after) return false;
    return this.#apply({ changes: { from: before, to: after }, selection: { anchor: before } });
  }

  handleKey(event: CodeEditorKey): { handled: boolean; changed: boolean } {
    const key = event.key;
    if (event.primary && ["c", "v"].includes(key.toLowerCase())) return { handled: false, changed: false };
    if (event.primary && key.toLowerCase() === "a") {
      this.setSelection(0, this.#state.doc.length);
      return { handled: true, changed: false };
    }
    if (event.primary && key.toLowerCase() === "z") {
      if (event.readOnly) return { handled: true, changed: false };
      return {
        handled: true,
        changed: event.shift ? this.#restore(this.#redo, this.#undo) : this.#restore(this.#undo, this.#redo),
      };
    }

    const range = this.#state.selection.main;
    const collapseOrExtend = (head: number) => {
      this.setSelection(event.shift ? range.anchor : head, head);
      return { handled: true, changed: false };
    };
    if (key === "ArrowLeft" || key === "ArrowRight") {
      if (!event.shift && !range.empty) return collapseOrExtend(key === "ArrowLeft" ? range.from : range.to);
      return collapseOrExtend(findClusterBreak(this.value, range.head, key === "ArrowRight"));
    }
    if (key === "ArrowUp" || key === "ArrowDown") {
      const line = this.#state.doc.lineAt(range.head);
      const column = range.head - line.from;
      const number = Math.max(1, Math.min(this.#state.doc.lines, line.number + (key === "ArrowUp" ? -1 : 1)));
      const target = this.#state.doc.line(number);
      return collapseOrExtend(Math.min(target.from + column, target.to));
    }
    if (key === "Home" || key === "End") {
      const line = this.#state.doc.lineAt(range.head);
      return collapseOrExtend(key === "Home" ? line.from : line.to);
    }
    if (key === "Backspace" || key === "Delete") {
      if (event.readOnly) return { handled: true, changed: false };
      let { from, to } = range;
      if (range.empty) {
        const moved = findClusterBreak(this.value, range.head, key === "Delete");
        from = Math.min(range.head, moved);
        to = Math.max(range.head, moved);
      }
      return { handled: true, changed: from !== to && this.#apply({ changes: { from, to }, selection: { anchor: from } }) };
    }
    if (key === "Enter") {
      if (event.readOnly) return { handled: true, changed: false };
      const indentation = this.#state.doc.lineAt(range.head).text.match(/^\s*/)?.[0] ?? "";
      return { handled: true, changed: this.commitText(`\n${indentation}`) };
    }
    if (key === "Tab") {
      return {
        handled: true,
        changed: !event.readOnly && this.#indent(event.shift),
      };
    }
    return { handled: false, changed: false };
  }

  config(language = this.#language): CodeEditorWidgetConfig {
    this.setLanguage(language);
    const base = { selection: this.selection, composition: this.#composition };
    if (!this.#tree || !this.#language) return { ...base, syntax: null };
    const ranges: CodeEditorHighlightRange[] = [];
    highlightTree(this.#tree, jsonHighlighter, (from, to, kind) => {
      ranges.push({ from, to, kind: kind as CodeEditorHighlightKind });
    });
    return {
      ...base,
      syntax: {
        language: this.#language,
        offsetEncoding: "utf16",
        documentLength: this.#state.doc.length,
        ranges,
      },
    };
  }

  update(value: string, language = this.#language): CodeEditorWidgetConfig {
    this.sync(value, language);
    return this.config(language);
  }

  #snapshot(): Snapshot { return { value: this.value, ...this.selection }; }

  #apply(spec: TransactionSpec, recordHistory = true): boolean {
    const before = this.#snapshot();
    const transaction = this.#state.update(spec);
    if (!transaction.docChanged && transaction.state.selection.eq(this.#state.selection)) return false;
    if (recordHistory && transaction.docChanged) {
      this.#undo.push(before);
      this.#redo = [];
    }
    const changedRanges: Array<{ fromA: number; toA: number; fromB: number; toB: number }> = [];
    transaction.changes.iterChangedRanges((fromA, toA, fromB, toB) => changedRanges.push({ fromA, toA, fromB, toB }));
    const fragments = this.#tree && transaction.docChanged
      ? TreeFragment.applyChanges(TreeFragment.addTree(this.#tree), changedRanges)
      : undefined;
    this.#state = transaction.state;
    if (this.#language === "json" && transaction.docChanged) this.#tree = jsonParser.parse(this.value, fragments);
    return transaction.docChanged;
  }

  #restore(source: Snapshot[], destination: Snapshot[]): boolean {
    const snapshot = source.pop();
    if (!snapshot) return false;
    destination.push(this.#snapshot());
    const previous = this.value;
    this.#state = EditorState.create({ doc: snapshot.value, selection: EditorSelection.single(snapshot.anchor, snapshot.head) });
    this.#tree = this.#language === "json" ? jsonParser.parse(this.value) : null;
    return previous !== snapshot.value;
  }

  #indent(outdent: boolean): boolean {
    const range = this.#state.selection.main;
    if (range.empty && !outdent) return this.commitText("  ");
    const startLine = this.#state.doc.lineAt(range.from);
    const endLine = this.#state.doc.lineAt(range.to);
    const changes: Array<{ from: number; to?: number; insert?: string }> = [];
    for (let number = startLine.number; number <= endLine.number; number += 1) {
      const line = this.#state.doc.line(number);
      if (outdent) {
        const count = line.text.startsWith("\t") ? 1 : line.text.match(/^ {1,2}/)?.[0].length ?? 0;
        if (count) changes.push({ from: line.from, to: line.from + count });
      } else changes.push({ from: line.from, insert: "  " });
    }
    return changes.length > 0 && this.#apply({ changes });
  }

  #offsetByUtf8Bytes(start: number, delta: number): number {
    const forward = delta >= 0;
    let offset = start;
    let remaining = Math.abs(delta);
    while (remaining > 0 && (forward ? offset < this.value.length : offset > 0)) {
      const moved = findClusterBreak(this.value, offset, forward);
      if (moved === offset) break;
      const codePoint = this.value.slice(Math.min(offset, moved), Math.max(offset, moved)).codePointAt(0) ?? 0;
      const bytes = codePoint <= 0x7f ? 1 : codePoint <= 0x7ff ? 2 : codePoint <= 0xffff ? 3 : 4;
      if (bytes > remaining) break;
      remaining -= bytes;
      offset = moved;
    }
    return offset;
  }
}
