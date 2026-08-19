# __WABOU_PROJECT_NAME__

A native desktop application built with [Wabou](https://github.com/SunDoge/wabou).

```bash
git submodule update --init
bun install
bun run dev
bun run check
bun run build
```

The `vendor/wabou` submodule pins the Rust host and JavaScript packages to one
compatible revision. Install the CLI from that same tag. Use
`bun run wabou --help` to list development, testing, rendering, and packaging
commands.

`bun run check` verifies TypeScript, Rust, generated host bindings when present,
and every discovered `tests/**/*.behavior.ts` scenario. Use
`bun run wabou check --skip-behavior` for a faster compile-only pass.
