---
"@wabou/core": minor
"@wabou/animation": patch
"@wabou/components": patch
"@wabou/primitives": patch
"@wabou/router": patch
"@wabou/terminal": patch
"@wabou/test": patch
"@wabou/vite": patch
---

Bundle the protocol, renderer, and typed style implementation into
`@wabou/core`. Expose stable `core` subpaths for advanced imports and stop
publishing the three implementation workspaces as standalone packages. Add a
tree-shakeable `@wabou/core/i18n` entry for compiled message libraries.
