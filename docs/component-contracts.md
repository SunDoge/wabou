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
