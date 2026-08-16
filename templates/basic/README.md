# __WABOU_PROJECT_NAME__

A native desktop application built with [Wabou](https://github.com/SunDoge/wabou).

```bash
git submodule update --init
bun install
bun run dev
```

The `vendor/wabou` submodule pins the Rust host and JavaScript packages to one
compatible revision. Install the CLI from that same tag. Use
`bun run wabou --help` to list development, testing, rendering, and packaging
commands.
