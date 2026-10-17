# Releasing a preview

Developer previews are distributed from immutable Git tags. Applications pin
that tag as `vendor/wabou`, so the Rust crates and private TypeScript workspace
packages always come from the same revision. npm and crates.io publication are
deferred until the package boundaries stabilize.

## Prepare

1. Prepare an explicit, unused prerelease version. This synchronizes Rust and
   vendored TypeScript package versions, updates the root changelog and
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

## Tag

Push the release commit and wait for both GitHub workflows to succeed. Then
create an annotated tag without moving or replacing an existing tag:

```bash
git tag -a v0.1.0-alpha.3 -m "Wabou v0.1.0-alpha.3"
git push origin v0.1.0-alpha.3
```

The tag push runs CI again. Publish the matching root changelog section as the
GitHub prerelease notes after that run succeeds.
