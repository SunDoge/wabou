# Markdown

Wabou renders Markdown as native components. It does not generate HTML and it
does not require a DOM.

## Decision

Use `marked` as the replaceable GFM parsing kernel and `remend` to repair an
incomplete streaming tail. Convert parser tokens immediately into Wabou-owned
`MarkdownBlock` and `MarkdownRun` values; neither components nor applications
may depend on the parser's token types.

This split fits Wabou better than the common alternatives:

- `react-markdown` and the remark renderer are designed around React elements
  and the DOM. Waku uses them in its web frontend, but its native frontend also
  owns a separate parser and renderer.
- `micromark` is the strongest choice when exact CommonMark compliance,
  positional syntax data, or a large remark plugin ecosystem is required. A
  useful AST needs additional mdast packages, however, and its streaming input
  interface still buffers constructs before producing a completed document.
- `markdown-it` exposes a capable token stream, but converting its paired
  open/close tokens into Wabou's nested render model is more work without a
  current product benefit.
- `marked` is dependency-free, exposes nested block and inline tokens directly,
  supports GFM, and stays behind a small adapter that can be replaced later.

Do not treat parser replacement as the way to solve streaming UI. Following
Waku's native renderer, `MarkdownDocument` owns the UI-specific behavior:

1. retain a stable prefix and reparse only the final two source-level blocks;
2. fall back to a full parse for non-local link reference definitions;
3. reconcile unchanged blocks and inline runs by identity so Solid retains the
   corresponding native subtree;
4. repair incomplete emphasis, links, and fences only for display while the
   response is streaming;
5. animate only newly appended runs and respect reduced motion.

## Parser contract

The adapter must preserve headings, paragraphs, nested block quotes, ordered
and unordered lists, task state, GFM tables, fenced code, rules, links,
emphasis, code spans, and strikethrough. Raw HTML is displayed as literal text;
it is never interpreted.

For every append prefix, the incremental document must equal a fresh parse of
that same prefix. Tests cover ordinary prose, tables, code fences, and inline
images using several chunk sizes. Parser-specific extensions should first be
added to this contract and only then mapped into native components.

Reconsider `micromark` plus mdast when Wabou needs source ranges for selection,
editor round-tripping, directives, footnotes, math, or a broad remark plugin
ecosystem. That migration changes only the adapter if this boundary is kept.
