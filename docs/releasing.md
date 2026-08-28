# Releasing a preview

Rust previews are distributed from immutable Git tags. JavaScript packages can
also be published to npm under the `alpha` dist-tag; crates.io remains deferred
until the Rust crate boundaries stabilize.

## Prepare

1. Prepare an explicit, unused prerelease version. This consumes pending
   Changesets, synchronizes Rust and JavaScript versions, updates changelogs and
   lockfiles, and refuses to reuse a local or remote Git tag:

   ```bash
   bun run release:prepare 0.1.0-alpha.3
   ```

2. Review the generated release metadata. The preparation command never
   commits, tags, publishes, or pushes.
3. Pin build-critical tools in `mise.toml` and install with `mise install`.
4. Run the same gates as CI:

   ```bash
   bun install --frozen-lockfile
   bun run verify
   git diff --exit-code
   ```

   `verify` is read-only: stale generated files fail with the command needed to
   refresh them. Run `bun run gen`, review the result, and repeat `verify`.

5. Build both example frontends and inspect Gallery renders at 1× and 2×.
6. Run `wabou new` against the candidate Git revision, then verify `bun
   install`, `bun run check`, `bun run build`, and `cargo check` in the generated
   standalone project.

## Publish JavaScript packages

The repository remains in Changesets prerelease mode between alpha releases.
Use `release:prepare` rather than calling `version-packages` directly so an
explicit version is shared with the Rust workspace and checked against Git
tags.

1. Confirm `bun pm whoami` reports an npm account with publish access to the
   `@wabou` scope.
2. Run `bun run release:prepare <version>`, review its output, then commit the
   release metadata.
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
git tag -a v0.1.0-alpha.3 -m "Wabou v0.1.0-alpha.3"
git push origin v0.1.0-alpha.3
```

The tag push runs CI again. Publish the matching root changelog section as the
GitHub prerelease notes after that run succeeds.
