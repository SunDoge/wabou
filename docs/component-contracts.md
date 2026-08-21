# Component composition contracts

Wabou components must declare which node owns each visual and interaction
responsibility. Composition must not depend on duplicate utility classes being
resolved by CSS source order: Wabou consumes typed style declarations and does
not emulate the browser cascade.

## Ownership rule

Within one component anatomy, each responsibility has one owner:

| Responsibility | Owner | Descendant rule |
| --- | --- | --- |
| surface, border, radius, shadow | outer visual control | content nodes use no chrome |
| focus ring | node representing the focused control | descendants do not paint a second ring |
| clipping | node whose radius or viewport defines visibility | custom widgets clip their local fragment too |
| scrolling | one viewport node | content reports size and never adjusts the same offset |
| overlay plane | portal container | overlay content must not remain in the content plane |
| semantic role and state | interactive control | decorative nodes remain hidden from semantics |
| native editing or painting | native content node | its surrounding component owns optional chrome |

An application may deliberately compose multiple controls, such as a search
field with a separately focusable clear button. That composition still has one
surface owner, while its expected focus-owner count is explicitly two.

Official components publish mechanically testable ownership through the
`data-wabou-owns` attribute, for example `surface native-editor`. This metadata
is visible in DevTools and tests; the runtime does not infer behavior from it.
Tests must not guess ownership from `bg-*` utilities.

## Standard anatomies

### Standalone input

```text
Input                         surface + border + focus + native editor
```

`Input.surfaceClass` selects the authored background utility. It defaults to
`bg-input`; selecting another surface must replace that declaration rather than
append a conflicting `bg-*` utility.

### Compound input

```text
InputGroup                    surface + border + radius + focus-within
├─ leading icon/text          content only
├─ Input(chrome="none")       native editor only
└─ optional action button     action chrome only
```

`InputGroup.surfaceClass` owns the complete compound background. Inner editors
must not emit `bg-input`, `bg-transparent`, a border, radius, or shadow merely
to override standalone defaults.

### Overlay

```text
trigger                       content plane + focus owner
Portal container              floating or modal plane
└─ overlay surface            surface + focus containment when modal
```

Opening order is not a substitute for an overlay plane. Floating content must
be under the shared floating root; modal content must be under the modal root.

### Scrollable rounded surface

```text
surface                       radius + border + outer clip
└─ scroll viewport            scroll offset + viewport clip
   └─ content                 intrinsic extent
```

Do not put independent scroll ownership on both the surface and viewport.
Native widgets paint inside the clip passed through their widget contract; a
parent scene clip alone is not sufficient evidence for every backend or HiDPI
path.

## Component test assertions

`@wabou/test/component` provides structural assertions for these contracts:

```tsx
import {
  assertFocusOwnerCount,
  assertInOverlayPlane,
  assertSingleSurfaceOwner,
  renderComponent,
} from "@wabou/test/component";

const screen = renderComponent(() => <SearchField aria-label="Search" />);
const input = screen.getByRole("textbox", { name: "Search" });
assertSingleSurfaceOwner(input.parent!);
assertFocusOwnerCount(input.parent!, 1);

// After opening a popover:
assertInOverlayPlane(screen.getByRole("dialog"), "floating");
```

These assertions inspect the authored protocol tree. They catch composition
mistakes early, but they do not claim that layout or pixels are correct.

## Layout contract tests

`wabou layout` evaluates the application through QuickJS, Style IR, real text
measurement, and Taffy, then stops before Vello scene construction or GPU
initialization:

```bash
wabou layout apps/gallery --out /tmp/gallery-layout.json \
  --width 800 --height 600
```

Node or Bun tests can drive it and inspect the structured result entirely in
TypeScript:

```ts
import {
  assertNoLayoutDiagnostics,
  formatLayoutTree,
  getLayoutNode,
  siblingCollisionDiagnostics,
  visibleOverflowDiagnostics,
} from "@wabou/test/layout";
import { renderAppLayout } from "@wabou/test/layout/node";

const layout = await renderAppLayout({
  app: "apps/gallery",
  out: "/tmp/gallery-layout.json",
  width: 800,
  height: 600,
  skipBuild: true,
});
const toolbar = getLayoutNode(layout, {
  role: "toolbar",
  name: "Formatting",
});
assertNoLayoutDiagnostics(
  visibleOverflowDiagnostics(layout, { within: toolbar }),
);
assertNoLayoutDiagnostics(
  siblingCollisionDiagnostics(layout, { within: toolbar }),
);
expect(formatLayoutTree(layout)).toMatchSnapshot();
```

Collision checks are opt-in because intentional overlap is valid. Scope them
to a component whose contract forbids overlap. Layout snapshots use logical
pixels; font rasterization, shadows, native-widget painting, and HiDPI still
belong to focused pixel or platform tests.

For component suites, compile a fixture registry once instead of starting a
process for every component:

```tsx
// ui/layout-fixtures.tsx
import {
  defineComponentFixtures,
  defineLayoutFixtures,
} from "@wabou/test/layout/fixtures";

defineLayoutFixtures({
  ...defineComponentFixtures(
    {
      "button/default": () => <Button>Save</Button>,
      "card/compact": () => <Card class="w-80">...</Card>,
    },
    {
      width: 800,
      height: 600,
      wrap: (content) => <View class="w-full p-6">{content}</View>,
    },
  ),
});
```

Select that entry from `defineWabouConfig` for a Vite mode, then run all cases
through one release CLI and one QuickJS runtime:

```ts
import { renderLayoutFixtures } from "@wabou/test/layout/node";

const report = await renderLayoutFixtures({
  app: "apps/gallery",
  mode: "layout-test",
  command: ["target/release/wabou"],
  cases: [
    { id: "button/default", width: 400, height: 300 },
    { id: "card/compact", width: 800, height: 600, scaleFactor: 2 },
  ],
});
```

Use `cases: "all"` to query the compiled registry and run every fixture,
including its colocated viewport and settling metadata. Style parser
diagnostics fail by default. `checks` applies geometry contracts to every
discovered fixture; `overrides` records intentional exceptions without
duplicating the registry:

```ts
await renderLayoutFixtures({
  app: "apps/gallery",
  mode: "layout-test",
  command: ["target/release/wabou"],
  cases: "all",
  checks: ["visible-overflow", "sibling-collision"],
  overrides: {
    "carousel/default": { checks: ["sibling-collision"] },
  },
});
```

Explicit cases may additionally declare an `assert(snapshot)` callback for
reactive state after Solid effects have settled. A fixture-level `waitMs`
covers timers, promises, or finite animation without slowing every fixture.
The returned `totalDurationMs` and per-case `durationMs` make regressions in
the edit-test loop visible instead of relying on anecdotes.

Each case disposes the preceding Solid owner and resets native retained state
before mounting. QuickJS, framework modules, Style IR, and font infrastructure
remain warm; component signals, effects, listeners, focus, scroll state,
widgets, and resource bindings do not cross the case boundary.

The Gallery regression command is the reference integration:

```bash
# First run, after Rust changes, or before committing:
bun run test:layout

# Reuse the release CLI and compiled fixture bundle while editing TSX/styles:
bun run test:layout:quick

# Run only the affected fixture(s):
bun run test:layout:quick widgets/Button widgets/Card
```

It compiles the fixture entry once and runs all existing component pages with
the release CLI. On the current suite, the native evaluation of 40 fixtures is
about 0.2 seconds after the bundle exists; the fixed QuickJS startup is paid
once rather than once per component. `test:layout:quick` deliberately skips
Vite and Rust builds, so use the full command after changing dependencies,
generated package output, Rust, or the fixture registry itself.

## Required evidence

Use the cheapest layer that can prove the property:

1. Vitest component tests: state, events, roles and anatomy ownership.
2. protocol/style tests: candidate resolution and computed declarations.
3. TS layout contracts: QuickJS + Style IR + Taffy geometry, overflow and
   collisions without a scene or GPU.
4. native layout fixtures: lower-level geometry, clipping and scroll ranges.
5. headless captures: pixels, text containment and real native widgets.
6. platform captures: backend-specific and HiDPI claims.

Every reusable styled component should have a component test. Components that
own clipping, native painting, overlays, or typography also need a focused
native capture. A successful build is never visual evidence.
