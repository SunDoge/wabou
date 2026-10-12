# __WABOU_PROJECT_NAME__

Use an installed Wabou CLI matching the pinned `vendor/wabou` revision.

```sh
bun install
bun run doctor
bun run check
bun run dev
```

Edit UI in `ui/index.tsx` (Vite HMR) and Rust in `native/src/lib.rs` (plugin replacement).
Change `STEP` after clicking either counter: each retains its independent state.
`bun run test` exercises real Rust capability calls through the headless runtime.
`bun run run` builds and starts the application with the CLI runtime.
A failed compilation keeps the previous implementation running.
The CLI contains the resident runtime; development only builds the small native plugin.

```sh
wabou bundle . --out dist/app
wabou bundle . --static --out dist/static-app
```

Dynamic packaging includes one shared library. Static packaging builds a resident
application executable with the same plugin registry linked in; no plugin library
is needed at runtime. Both retain the frontend resources beside the executable.
The native crate must offer `rlib` for static packaging.

`wabou.config.ts` selects the capability methods and native widget tags.
`export_plugin!` registers them together. Keep schema versions unchanged for
implementation-only edits; incompatible schema edits reject replacement.

`bun run check` checks TypeScript and every configured Rust project, including
capabilities and widgets exported from the same plugin. Shared implementations
with identical build settings are checked once. `bun run doctor` diagnoses setup
without requiring a resident-runtime application crate at the project root.
