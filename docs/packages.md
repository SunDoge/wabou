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
- `@wabou/primitives` — unstyled components plus renderer-independent behavior
  available from `@wabou/primitives/interactions`.
- `@wabou/router` — native application routing.
- `@wabou/animation` — Motion-backed animation helpers.
- `@wabou/terminal` — native terminal component.
- `@wabou/test` — TypeScript behavior tests, normally a dev dependency.

The source workspaces `@wabou/protocol`, `@wabou/solid-renderer` and
`@wabou/style` are private implementation details. Their release artifacts are
bundled into `@wabou/core` and exposed, when an advanced import is useful, as
`@wabou/core/protocol`, `@wabou/core/renderer` and `@wabou/core/style`.
Applications normally use the root `@wabou/core` export and set
`jsxImportSource` to `@wabou/core`.

`@wabou/core/i18n` is a separate tree-shakeable entry for reactive locale
state and compiler-neutral message functions. The Gallery uses it with
Paraglide: translation catalogs are compiled during the Vite build, while
Wabou passes the selected locale explicitly instead of emulating browser URL,
cookie, or local-storage strategies.
`@wabou/vite` exposes its bundled HMR client as `@wabou/vite/runtime`; there is
no separate runtime package.

All `@wabou/*` packages use one fixed version. Add a Changeset for a public
change, then run `bun run version-packages` and `bun run release-packages`.
The release command first compiles package source with tsdown into ESM and
bundled declarations, then publishes with Bun so `workspace:*` dependencies
are rewritten to the shared concrete version in the npm tarball. Published
packages expose `dist` artifacts; consumers do not need to compile Wabou's
TypeScript or TSX source. These artifacts are tracked in Git as well as npm so
applications pinned to an immutable Wabou Git revision consume exactly the
same package surface as registry installations.
`bun run packages:check` verifies aligned versions, publication metadata and
that public artifacts and application manifests do not depend on private
workspace packages.

The component stack remains layered inside `@wabou/primitives`: its
`interactions` subpath provides headless behavior, the root connects behavior
to the native host, and `@wabou/components` supplies themed,
shadcn-inspired recipes.

On the Rust side, applications use the `wabou` facade from a pinned Git tag.
The facade is deliberately not on crates.io for the first preview, allowing
internal crates to be merged or renamed without reserving permanent public
crate names. `wabou-runtime`, `wabou-shell`, `wabou-widgets`, and the remaining
workspace crates are implementation details; the preview tag and the facade
are the supported Rust boundary.
