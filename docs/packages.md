# JavaScript packages

Most applications install the runtime entry point and the Vite integration:

```bash
bun add @wabou/core solid-js
bun add -d @wabou/vite vite
```

`@wabou/core` exports the renderer, native host APIs and typed inline-style
helpers. Applications should not import renderer, protocol or style
implementation packages directly.

Install the following public packages only when the application uses them:

- `@wabou/components` — styled application components.
- `@wabou/primitives` — unstyled, composable interaction primitives.
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
The release command publishes with Bun so `workspace:*` dependencies are
rewritten to the shared concrete version in the npm tarball.
`bun run packages:check` verifies aligned versions, publication metadata and
that application manifests do not directly depend on internal packages.
