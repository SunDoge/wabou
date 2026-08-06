# @wabou/unocss-preset

UnoCSS tooling adapter generated from the authoritative Rust `wabou-style`
utility definitions. It provides dynamic rules and autocomplete for Wabou
Utilities. State variants are intentionally unsupported; use Solid `classList`
or typed style values driven by Wabou primitives. Production semantics are
implemented by Rust; do not add
handwritten rules here without adding the corresponding Rust parser and
conformance case.

The preset is stateless. It only emits declarations that map deterministically
to Taffy layout or Vello paint data. Static `transform` utilities are supported;
CSS transitions and animations are rejected. Drive changing geometry explicitly
with the primitive `transform` prop or `setTransform2D`, using an `Affine2D`
matrix such as `translate2d(x, y)`. Runtime state composes after the static CSS
transform and never replaces declarations from the cascade.

```ts
import { defineConfig } from "unocss";
import { presetWabou } from "@wabou/unocss-preset";

export default defineConfig({ presets: [presetWabou()] });
```
