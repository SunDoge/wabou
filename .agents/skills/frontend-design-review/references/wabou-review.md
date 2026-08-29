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

## Required review sequence

1. State the screen's primary job, primary action, density, visual tone, and the one
   product-specific idea that should distinguish it.
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

## Wabou quality gates

- A view has one clear primary action; secondary actions use progressive disclosure.
- Layout remains usable at the declared minimum window size and at the normal capture
  viewport. Content must not silently overlap, clip, or become unreachable.
- Repeated UI uses shared components or named composition patterns, not copied class
  strings with drifting metrics.
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
