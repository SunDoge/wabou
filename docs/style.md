# Styling architecture

Wabou deliberately separates static presentation, dynamic values, and
interaction semantics:

- Static layout and visual choices use `class` utilities.
- Dynamic, continuous runtime values use typed `style` values.
- Interaction semantics and state machines use primitive/component props.

For example, a draggable button keeps its fixed size and colors in `class`,
its current translation in typed style, and press/disabled behavior in the
Button primitive. Wabou deliberately does not implement CSS/UnoCSS state
variants because native hover, focus and active behavior cannot promise browser
equivalence.

## Wabou utilities

`wabou-style` is the semantic source of truth. Its Winnow parsers accept a
Tailwind-compatible subset and produce typed declarations for Taffy, Parley,
Vello, and Wabou interaction state. Arbitrary syntax is typed rather than
arbitrary CSS: `p-[13px]` is valid, while `p-[var(--space)]` and unsupported
`calc()` expressions are build errors.

The Rust definitions export a generated manifest. `@wabou/unocss-preset`
adapts that manifest for UnoCSS editor tooling and lets the Vite plugin compile
matched classes directly to Style IR. It is not the semantic authority and no
production utility CSS is emitted. Rust can also parse classes directly at
runtime when a precompiled stylesheet rule is absent.

```text
Rust utility definitions
  ├─ Winnow runtime parser → typed native declarations
  ├─ generated manifest
  │   └─ @wabou/unocss-preset → completion and build-time Style IR adapter
  └─ conformance fixtures → Rust/TypeScript parity tests
```

Unknown preset-mini candidates are rejected during the Vite build rather than
silently omitted. The supported syntax is called **Wabou Utilities**; it uses
familiar Tailwind conventions but does not claim browser CSS or full UnoCSS
compatibility.

Utility declarations are applied in class-list order. Later classes override
earlier classes after shorthands have expanded to their native properties:

```tsx
<View class="p-4 px-2 w-4 w-8" />
```

This resolves to 16px vertical padding, 8px horizontal padding, and a 32px
width. Transform utilities use independent translate/scale/rotate slots, so
different slots compose while a later utility for the same slot replaces the
earlier value.

The default color palette includes the complete `50`, `100`, `200`, `300`,
`400`, `500`, `600`, `700`, `800`, `900`, and `950` scale for `rose`, `pink`,
`fuchsia`, `purple`, `violet`, `indigo`, `blue`, `sky`, `cyan`, `teal`,
`emerald`, `green`, `lime`, `yellow`, `amber`, `orange`, `red`, `gray`, `slate`,
`zinc`, `neutral`, and `stone`. The same tokens work with `text-`, `bg-`, and
`border-`; literal `[#rrggbb]` and `[#rrggbbaa]` colors remain available.

Projects should add stable brand or semantic colors to the theme instead of
constructing utility names dynamically.

The default theme can be extended at generation time with a JSON file:

```json
{
  "spacing": { "18.5": 74 },
  "colors": { "brand": 862362111 }
}
```

```sh
WABOU_THEME=./wabou-theme.json bun run gen
```

Theme files extend the defaults. They are static build inputs: runtime theme
switching belongs in explicit component state and `classList`, not in a CSS
variant. Rust integrations can use `Theme`, `parse_utility_with_theme`, and
`manifest_with_theme` directly.

## Typed dynamic styles

Import value constructors from `@wabou/style` when a value changes at runtime:

```tsx
import { number, percent, px } from "@wabou/style";

<View style={{ width: px(width()), height: percent(0.5), opacity: number(0.8) }} />;
```

These values cross the JS → Rust bridge as a compact tag and numeric payload.
They do not create values such as `"123px"` and Rust does not parse them again.
Plain string styles remain available as a compatibility path.

## Vello-native shadows

Shadows use an explicit primitive prop rather than CSS `box-shadow` syntax.
Each layer maps to one Vello blurred rounded rectangle, and layers paint in
array order:

```tsx
import { shadow } from "@wabou/style";

<View
  shadows={[
    shadow({ offsetY: 8, spread: -2, stdDev: 6, color: 0x00000040 }),
    shadow({ offsetY: 1, stdDev: 1, color: 0x00000020 }),
  ]}
/>
```

`offsetX`, `offsetY`, and signed `spread` use logical pixels. `stdDev` is the
Gaussian standard deviation passed directly to Vello; it is not CSS blur
radius. `color` is packed sRGBA in `0xRRGGBBAA` order. An optional `radius`
overrides the node-derived rounded-rectangle radius for that layer. Shadows
follow the node's affine transform and do not affect layout.

The `shadow-*` utilities are convenience presets, not a Tailwind compatibility
contract. CSS inset and gradient shadows are not exposed because Vello's
blurred-rounded-rectangle primitive accepts neither.

The same package exports generated `WabouUtility` and `classes()` types:

```ts
const className = classes("flex", "px-[13px]", "bg-slate-900");
```

Invalid individual utilities are rejected by TypeScript. The generated unions
and template-literal types come from the Rust utility manifest.

Interaction state must be explicit through Solid and Wabou primitives:

```tsx
const hover = createHover();

<View
  class="bg-slate-900"
  classList={{
    "bg-slate-900": !hover.hovered(),
    "bg-slate-700": hover.hovered(),
  }}
  {...hover.bindings}
/>;
```

`hover:`, `focus:`, `active:`, `disabled:` and authored CSS pseudo-classes are
build errors rather than approximate browser behavior.

The same rule applies to responsive/theme variants, transitions, animations,
and dynamically assembled names such as `` `bg-${color()}` ``. Select between
complete static utilities in `classList`; use typed style values for continuous
runtime values.
