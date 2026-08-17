# JavaScript packages

Most applications install the runtime entry point and the Vite integration:

```bash
bun add @wabou/core solid-js
bun add -d @wabou/vite vite
```

`@wabou/core` exports the renderer, native host APIs and typed inline-style
helpers. Applications should not import renderer, protocol or style
implementation packages directly.

The 0.1 developer preview targets the exact `solid-js@2.0.0-rc.0` line. Keep
Solid and Wabou packages pinned together until Solid 2 and the universal
renderer publish stable releases; minor RC changes may alter renderer behavior.

Install the following public packages only when the application uses them:

- `@wabou/components` — styled application components.
- `@wabou/primitives` — unstyled, composable interaction primitives.
- `@wabou/interactions` — renderer-independent behavior, collections,
  selection, keyboard navigation and Solid state adapters.
- `@wabou/router` — native application routing.
- `@wabou/animation` — Motion-backed animation helpers.
- `@wabou/terminal` — native terminal component.
- `@wabou/test` — TypeScript behavior tests, normally a dev dependency.

The packages `@wabou/protocol`, `@wabou/solid-renderer`, `@wabou/style`,
`@wabou/style-compiler` and `@wabou/unocss-preset` are published because the
public packages depend on them. They are implementation details, carry
`wabou.stability: "internal"` metadata, and are not stable import targets.
`@wabou/vite` exposes its bundled HMR client as `@wabou/vite/runtime`; there is
no separate runtime package.

All `@wabou/*` packages use one fixed version. Add a Changeset for a public
change, then run `bun run version-packages` and `bun run release-packages`.
The release command first compiles package source with tsdown into ESM and
bundled declarations, then publishes with Bun so `workspace:*` dependencies
are rewritten to the shared concrete version in the npm tarball. Published
packages expose `dist` artifacts; consumers do not need to compile Wabou's
TypeScript or TSX source.
`bun run packages:check` verifies aligned versions, publication metadata and
that application manifests do not directly depend on internal packages.

The component stack is intentionally layered: `@wabou/interactions` provides
behavior, `@wabou/primitives` connects it to the native host, and
`@wabou/components` supplies themed, shadcn-inspired recipes.

On the Rust side, applications use the `wabou` facade from a pinned Git tag.
The facade is deliberately not on crates.io for the first preview, allowing
internal crates to be merged or renamed without reserving permanent public
crate names. `wabou-runtime`, `wabou-shell`, `wabou-widgets`, and the remaining
workspace crates are implementation details; the preview tag and the facade
are the supported Rust boundary.
