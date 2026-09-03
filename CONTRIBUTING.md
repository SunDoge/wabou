# Contributing to Wabou

Thank you for helping build Wabou.

## Development

Install the repository toolchain with [mise](https://mise.jdx.dev/):

```bash
mise install
```

Run the relevant focused tests while iterating. Before a release, run the
complete repository verification contract:

```bash
bun run verify
```

`bun run verify:js` and `bun run verify:rust` are available when a change only
needs one side of the boundary; `bun run verify:behavior` discovers every app
with authored `tests/**/*.behavior.ts` suites. Pull-request and development-push
CI runs the bounded unit/component merge gate; tag and manually dispatched runs
also execute the expensive native behavior and layout suites. Standalone
`.scenario.ts` files that require special environment setup remain explicit
commands. The full local command checks formatting, types, generated bindings,
package tarballs, unit tests, Clippy, every Rust target, native application
behavior, and every authored host-backed capture.
Capture PNGs are retained under `target/wabou-captures` for inspection. The
verification command reports stale generated output instead of rewriting the
worktree; run `bun run gen` explicitly when that diagnostic is expected.

To run the GitHub Actions workflow locally, use the repository's `act`
wrapper. It uses Docker's host network and forwards the host proxy at
`localhost:7890` into each job container:

```bash
mise exec -- bun run ci:local
```

Set `WABOU_ACT_PROXY` to override the proxy URL.
The wrapper reuses the locally cached runner image by default because Docker
daemon image pulls do not inherit job-container proxy variables; pass
`--pull=true` when the daemon itself has working registry access.

## Commits and pull requests

Wabou follows [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```text
<type>(<optional-scope>): <description>
```

Common types are `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, `chore`, and `revert`.

Prefer one of these scopes when it fits:

```text
shell runtime render layout style protocol cli devtools terminal
components primitives gallery hackernews docs ci deps
```

Examples:

```text
feat(devtools): add native layout overlay
fix(render): clip widget fragments at hidpi
refactor(style): remove authored css compilation
test(layout): cover rounded overflow clipping
```

Use `!` for a breaking change and explain it in the body or a `BREAKING CHANGE:` footer:

```text
feat(protocol)!: replace string style values with typed records
```

Cocogitto can create valid commits:

```bash
mise exec -- cog commit fix "clip widget fragments at hidpi" render
```

Optionally install the shared local validation hook:

```bash
mise exec -- cog install-hook commit-msg
```

Contributors do not need to rewrite every work-in-progress commit. Pull requests are squash-merged, so the pull-request title must follow the same convention and becomes the final commit message.

Keep the summary imperative, specific, and focused on one logical change. Explain motivation and non-obvious tradeoffs in the body rather than packing them into the summary.
