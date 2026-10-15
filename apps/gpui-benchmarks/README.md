# GPUI shell visual benchmarks

This is one Wabou application containing static visual reproductions of the
seven applications in `/home/me/code/javascript/gpui-shell-benchs`. The index
routes to each shell without restarting the native runtime:

- `/skills` — Skills
- `/folio` — Folio Desk
- `/tendra` — Tendra
- `/ocean` — OceanView
- `/signed` — Signed
- `/omanta` — Omanta
- `/disktree` — DiskTree

The fixtures are deterministic and focus on rendering, layout, component
density, and desktop application composition. Their product features are not
implemented.

Run it from the workspace root:

```sh
mise exec -- bun run wabou dev apps/gpui-benchmarks
```

Run its routed behavior and layout checks:

```sh
mise exec -- bun run wabou test apps/gpui-benchmarks
bun run apps/gpui-benchmarks/tests/layout.ts
```

The authored captures use 2× device scale and viewports derived from the
reference screenshots. This keeps text and vector comparisons independent of
thumbnail scaling.

## Framework findings

- The router can host all seven full-screen shells in one native process. Each
  shell returns to the shared index through a semantic button.
- Native 2× text and SVG output is sharp enough that the remaining differences
  are mainly font metrics, weight, and application-specific density.
- Static utility maps are discovered automatically. Runtime construction such
  as `` `bg-${color}` `` remains an error so the complete Style IR is known at
  build time.
- Static rotation utilities, `leading-6`, and three-digit arbitrary colors now
  share the same typed compiler and native-renderer contract.
- `Text` supports both ellipsis and hard clipping at a `maxLines` boundary.
- Compact desktop interfaces still require explicit component sizes and local
  geometry. This keeps layout visible at the call site instead of changing it
  through an implicit global density mode.
- Inline SVG participates in the accessibility tree and therefore needs an
  explicit accessible name when it conveys content.

See `THIRD_PARTY_NOTICES.md` for reference attribution.
