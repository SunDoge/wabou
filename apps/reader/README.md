# Wabou Reader

A local-first reading and research workspace used to develop Wabou against a
real desktop application. The current slice opens native PDF files, renders them
through the pure-Rust `wabou-pdf` widget, and keeps navigation and page notes in
the Solid UI layer.

```bash
bun run wabou dev apps/reader
```

The built-in guide is available without a document. Use **Open PDF** to exercise
the native file dialog and PDF renderer.

Next product slices are persistent library metadata and reading positions,
document text/search, durable annotations, and EPUB layout.
