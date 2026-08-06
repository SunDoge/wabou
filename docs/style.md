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

## Typed dynamic styles

Import value constructors from `@wabou/style` when a value changes at runtime:

```tsx
import { number, percent, px } from "@wabou/style";

<View style={{ width: px(width()), height: percent(0.5), opacity: number(0.8) }} />;
```

These values cross the JS → Rust bridge as a compact tag and numeric payload.
They do not create values such as `"123px"` and Rust does not parse them again.
Plain string styles remain available as a compatibility path.

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
