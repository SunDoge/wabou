# CodeMirror headless experiment

This experiment checks whether CodeMirror's state and Lezer packages can run in
Wabou's embedded QuickJS runtime without `EditorView`, DOM, or browser input
APIs.

```bash
bun install --cwd experiments/codemirror-headless
bun run --cwd experiments/codemirror-headless test
```

The result demonstrates immutable document edits, selection, reversible
transactions, incremental JSON parsing, and syntax highlight ranges. Wabou
would still own history policy, pointer/IME/clipboard routing, visual-line
geometry, scrolling, and painting.

## Result

`@codemirror/state`, `@lezer/common`, `@lezer/json`, and
`@lezer/highlight` run successfully in QuickJS. The minified experiment bundle
is approximately 105 KB and 1,000 state transactions plus parsing/highlighting
complete in roughly 15–20 ms on the development machine.

`@codemirror/commands` is not headless: importing it pulls in
`@codemirror/view`, whose module initialization accesses DOM style APIs and
fails in QuickJS. Wabou therefore cannot reuse CodeMirror's complete command
and history layer without either maintaining a DOM-free extraction or owning
those policies itself.
