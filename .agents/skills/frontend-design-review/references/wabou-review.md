# Wabou frontend review profile

Use this profile for Wabou itself and applications built inside this repository.
Wabou is a Solid renderer for native retained UI, not a browser. Treat web-oriented
guidance in the parent skill as design intent rather than an implementation mandate.

## Sources of truth

- `packages/ui`: reusable component behavior, visual tokens, and composition APIs.
- `packages/core` and Style IR: supported authored properties and host semantics.
- Gallery component examples: public component variants and visual inventory.
- Product references supplied by the user: target hierarchy, density, and interaction
  quality. Do not reproduce their branding or browser-only mechanics blindly.
- `.agents/skills/wabou-debug/SKILL.md`: required evidence ladder.

Figma and Storybook are optional references, not approval gates. Never invent a
Figma comparison when none exists.

## Design-system implementation model

Classify visual decisions before editing JSX:

1. **Global tokens** own reusable color roles, typography, spacing, radius,
   elevation, and readable widths.
2. **Component contracts** own control height, internal padding, icon/text
   alignment, state variants, and repeated surface geometry.
3. **Screen composition** owns page hierarchy and relationships between shared
   components, but should not redefine their internal metrics.

Prefer a shared component variant over repeating a nearby class recipe. Prefer a
container relationship over nudging a child with a local offset. When a literal is
truly screen-specific, keep it next to a named composition helper and test the
relationship it encodes. After fixing one member of a component family, search for
the same role across the repository before declaring the issue local.

## Required review sequence

1. For a new screen or substantial visual rewrite, complete
   `wabou-product-ui-brief.md`. For a narrow fix, state the owning contract and
   intended observable result instead.
2. Inventory existing Wabou components and tokens before creating local primitives.
3. Review hierarchy, alignment, spacing rhythm, typography, color roles, elevation,
   interaction states, loading/empty/error states, and resize behavior.
4. Classify findings as blocking, major, or minor. Fix structural and behavioral
   problems before decorative polish.
5. Prove the result at the cheapest authoritative layer:
   - component Vitest for semantics and state;
   - layout fixtures for geometry, overflow, clipping, collision, and resize;
   - native behavior tests for focus, pointer, IME, windows, tray, or widgets;
   - focused pixel/platform captures only for paint and platform differences.
6. For a shared component change, add both a component contract and a layout fixture
   when it owns geometry.

## Pressure-test matrix

Select the smallest set that can disprove the proposed design. Do not run every row
mechanically, but never approve a broad page or component-system change from only
its ideal state.

| Risk | Wabou evidence |
| --- | --- |
| Long or translated copy | Component test plus layout fixture using realistic long text |
| Narrow or resized window | The same fixture at the declared minimum and normal viewport |
| Empty, loading, failure, retry | Component states with accessible actions and preserved input |
| Dynamic lists or routing | Stable-key behavior test; assert selection and scroll continuity |
| Menus, overlays, tooltips | Layout/hit-test fixture plus pointer and keyboard behavior |
| Native widgets or IME | Native behavior scenario; restart after Rust changes |
| Light/dark theme | Focused render for both when colors, borders, shadows, or text change |
| HiDPI/platform paint | Platform capture at the affected scale; do not infer from Linux 1x |

When a fixture reveals a reusable defect, repair the owning token, primitive, or
component contract first. A screen-local offset is acceptable only when the visual
relationship is genuinely unique to that screen.

## Wabou quality gates

- A view has one clear primary action; secondary actions use progressive disclosure.
- Layout remains usable at the declared minimum window size and at the normal capture
  viewport. Content must not silently overlap, clip, or become unreachable.
- Repeated UI uses shared components or named composition patterns, not copied class
  strings with drifting metrics.
- Fixed chrome, search/actions, scrollable content, and footers remain separate
  structural regions. Scrolling a long list must not move the controls needed to
  search, create, or leave that list.
- Text, icons, controls, and headers align through component contracts; local optical
  offsets require a documented reason.
- Every interactive component exposes hover, pressed, focus, disabled, and relevant
  loading/error states through explicit semantics.
- Dynamic lists use stable product identities. Route or selection changes must not
  remount unrelated native widgets or reset scroll position.
- Motion explains state or spatial continuity. It must not delay menus, hide content,
  or run indefinitely without an idle-safe contract.
- Light theme is a first-class target. Dark theme, high DPI, and platform-specific
  rendering are verified when the affected layer requires them.
- No UI change is approved only because it compiles or because one screenshot looks
  acceptable.

## Review output

Keep the parent skill's three pillars, but attach evidence to each actionable finding:

| Finding | Severity | Source of truth | Required proof |
| --- | --- | --- | --- |
| Concrete problem | blocking/major/minor | component/token/reference | test or capture |

Conclude with `Pass`, `Needs work`, or `Blocked`, and list the next highest-leverage
change. Avoid broad aesthetic claims without an observable mechanism.

The three-layer implementation discipline in this profile is informed by the
MIT-licensed [frontend-ui-standards skill](https://github.com/MaxHan7/frontend-ui-standards-skill).
