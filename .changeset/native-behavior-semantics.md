---
"@wabou/components": minor
"@wabou/primitives": minor
"@wabou/test": minor
"@wabou/core": patch
---

Add replayable native behavior assertions for semantic state, bounds, numeric
ranges with explicit floating-point tolerance, match counts, multiple windows,
and strict indexed locators. Tighten overlay,
selection, alert, progress, image, and disclosure semantics so shipped
components expose stable AccessKit roles and hide decorative implementation
nodes. Normalize non-finite Slider and Progress inputs before they reach native
layout or accessibility state. Preserve menu, menu-item, tree, and tree-item
roles across the native accessibility and behavior-test bridges.
Resolve `aria-controls` and `aria-activedescendant` into live AccessKit node
relationships while respecting modal isolation.
