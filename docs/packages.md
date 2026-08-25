# JavaScript packages

Most applications install one UI entry point and the Vite integration:

```bash
bun add @wabou/ui solid-js
bun add -d @wabou/vite vite
```

`@wabou/ui` is the application-facing facade. It exports the renderer and host
APIs, styled components, common scene primitives, animation, and routing.
Applications should not need to understand Wabou's internal package graph and
should set `jsxImportSource` to `@wabou/ui`.

The 0.1 developer preview targets the exact `solid-js@2.0.0-rc.2` line. Keep
Solid and Wabou packages pinned together until Solid 2 and the universal
renderer publish stable releases; minor RC changes may alter renderer behavior.

The following directories remain useful internal source boundaries inside
`packages/ui/src`, but are not package or publication boundaries:

- `components` — styled application components.
- `primitives` — unstyled components plus renderer-independent interactions.
- `router` — native application routing.
- `animation` — Motion-backed animation helpers.

Their public capabilities are available from `@wabou/ui` and
`@wabou/ui/primitives`. This keeps implementation ownership and tests modular
without making source topology part of the installation contract.

`@wabou/terminal` remains an optional public package for applications that use
the native terminal widget. `@wabou/test` is the public behavior-test package,
normally installed as a dev dependency.

The protocol, Solid renderer, and style value model are ordinary source modules
inside `@wabou/core`, not separate packages. Their advanced entry points are
`@wabou/core/protocol`, `@wabou/core/renderer`, and `@wabou/core/style`.
Applications normally use `@wabou/ui`; `@wabou/core` remains the lower-level
runtime boundary.

`@wabou/ui/i18n` is a separate tree-shakeable entry for reactive locale
state and compiler-neutral message functions. The Gallery uses it with
Paraglide: translation catalogs are compiled during the Vite build, while
Wabou passes the selected locale explicitly instead of emulating browser URL,
cookie, or local-storage strategies. The compiled messages run in QuickJS
without Node or DOM dependencies, unused messages tree-shake, and locale
changes cross Solid's explicit `flush` boundary.

`@wabou/ui/router` uses TanStack Router Core as its matching, state, loader,
caching, and navigation engine over memory history. Wabou owns the Solid 2
store adapter and native presentation concerns such as links, blocker UI, and
scroll restoration; it does not install TanStack's DOM-facing components.
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
`bun run packages:check` verifies aligned versions, publication metadata, and
that public artifacts and applications do not revive retired implementation
package names.

The component source remains layered: primitive interactions provide headless
behavior, primitives connect it to the native host, and components supply
themed, shadcn-inspired recipes. Those are development boundaries, while
`@wabou/ui` is the supported application boundary.

On the Rust side, applications use the `wabou` facade from a pinned Git tag.
The facade is deliberately not on crates.io for the first preview, allowing
internal crates to be merged or renamed without reserving permanent public
crate names. `wabou-runtime`, `wabou-shell`, `wabou-widgets`, and the remaining
workspace crates are implementation details; the preview tag and the facade
are the supported Rust boundary. Optional widget and code-generation crates may
be added alongside `wabou`, but must not replace it as the application runtime.
