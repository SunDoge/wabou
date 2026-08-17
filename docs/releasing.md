# Releasing a preview

Rust previews are distributed from immutable Git tags. JavaScript packages can
also be published to npm under the `alpha` dist-tag; crates.io remains deferred
until the Rust crate boundaries stabilize.

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

## Publish JavaScript packages

The repository remains in Changesets prerelease mode between alpha releases.
`bun run version-packages` therefore advances the fixed package group from
`0.1.0-alpha.1` to `0.1.0-alpha.2`, rather than accidentally creating the
stable `0.1.0` release.

1. Confirm `bun pm whoami` reports an npm account with publish access to the
   `@wabou` scope.
2. Run `bun run version-packages`, update the matching Rust workspace version
   and root changelog heading, then commit the release metadata.
3. Re-run every preparation gate against that commit.
4. Publish in dependency order with the prerelease dist-tag:

   ```bash
   bun run release-packages --tag alpha
   ```

The release command rebuilds all packages with tsdown before publishing. Do
not publish an alpha under npm's default `latest` tag.

## Tag

Push the release commit and wait for both GitHub workflows to succeed. Then
create an annotated tag without moving or replacing an existing tag:

```bash
git tag -a v0.1.0-alpha.2 -m "Wabou v0.1.0-alpha.2"
git push origin v0.1.0-alpha.2
```

The tag push runs CI again. Publish the matching root changelog section as the
GitHub prerelease notes after that run succeeds.
