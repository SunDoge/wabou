# Releasing a Git preview

Wabou previews are distributed from immutable Git tags. crates.io and npm are
not release targets yet, although their package checks remain useful for
detecting incomplete manifests and workspace dependency drift.

## Prepare

1. Set `workspace.package.version` and every `@wabou/*` package to the exact
   prerelease version.
2. Add the matching heading to the root and package changelogs.
3. Pin build-critical tools in `mise.toml` and install with `mise install`.
4. Run the same gates as CI:

   ```bash
   mise exec -- bun install --frozen-lockfile
   cargo fmt --all -- --check
   cargo check --workspace --all-targets
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-targets
   mise exec -- bun run check
   mise exec -- bun run packages:build
   mise exec -- bun run packages:check
   mise exec -- bun run scripts/publish-packages.ts --dry-run
   mise exec -- bun --conditions=browser test packages
   mise exec -- bun run test:router
   mise exec -- bun run gen
   git diff --exit-code
   ```

5. Build both example frontends and inspect Gallery renders at 1× and 2×.
6. Run `wabou new` against the candidate Git revision, then verify `bun
   install`, `bun run check`, `bun run build`, and `cargo check` in the generated
   standalone project.

## Tag

Push the release commit and wait for both GitHub workflows to succeed. Then
create an annotated tag without moving or replacing an existing tag:

```bash
git tag -a v0.1.0-alpha.1 -m "Wabou v0.1.0-alpha.1"
git push origin v0.1.0-alpha.1
```

The tag push runs CI again. Publish the matching root changelog section as the
GitHub prerelease notes after that run succeeds.
