# @wabou/test

## 0.1.0-alpha.4

### Minor Changes

- Add replayable native behavior assertions for semantic state, bounds, numeric
  ranges with explicit floating-point tolerance, match counts, multiple windows,
  and strict indexed locators. Tighten overlay,
  selection, alert, progress, image, and disclosure semantics so shipped
  components expose stable AccessKit roles and hide decorative implementation
  nodes. Normalize non-finite Slider and Progress inputs before they reach native
  layout or accessibility state. Preserve menu, menu-item, tree, and tree-item
  roles across the native accessibility and behavior-test bridges.
  Resolve `aria-controls` and `aria-activedescendant` into live AccessKit node
  relationships while respecting modal isolation.
- Use typed `{ lo, hi }` generational window keys throughout behavior-test APIs,
  native capability calls, recorded traces, and replay instead of unsafe numeric
  window identifiers.

### Patch Changes

- Bundle the protocol, renderer, and typed style implementation into
  `@wabou/core`. Expose stable `core` subpaths for advanced imports and stop
  publishing the three implementation workspaces as standalone packages. Add a
  tree-shakeable `@wabou/core/i18n` entry for compiled message libraries.
- be282e7: Upgrade the Solid 2 runtime to rc.6 and the matching Solid Vite compiler to
  the current Vite 8-compatible preview.
- 625e23c: Discover complete utility classes in static variant maps without requiring a
  separate registration API. Add typed support for clipped multi-line text,
  static transforms, explicit line heights, and three-digit arbitrary colors.
  Keep component animation tests on Wabou's frame clock under Vitest 5 and allow
  lifecycle-owned animation signals to update from frame callbacks.
- Updated dependencies
- Updated dependencies
- Updated dependencies
- Updated dependencies
- Updated dependencies
- Updated dependencies
- Updated dependencies
- Updated dependencies
- Updated dependencies [851d731]
- Updated dependencies
- Updated dependencies
- Updated dependencies [be282e7]
- Updated dependencies [625e23c]
- Updated dependencies
- Updated dependencies
  - @wabou/core@0.1.0-alpha.4

## 0.1.0-alpha.3

### Minor Changes

- bb91058: Add replayable native behavior assertions for semantic state, bounds, numeric
  ranges with explicit floating-point tolerance, match counts, multiple windows,
  and strict indexed locators. Tighten overlay,
  selection, alert, progress, image, and disclosure semantics so shipped
  components expose stable AccessKit roles and hide decorative implementation
  nodes. Normalize non-finite Slider and Progress inputs before they reach native
  layout or accessibility state. Preserve menu, menu-item, tree, and tree-item
  roles across the native accessibility and behavior-test bridges.
  Resolve `aria-controls` and `aria-activedescendant` into live AccessKit node
  relationships while respecting modal isolation.
- e21aee3: Use typed `{ lo, hi }` generational window keys throughout behavior-test APIs,
  native capability calls, recorded traces, and replay instead of unsafe numeric
  window identifiers.

### Patch Changes

- 48ce10c: Bundle the protocol, renderer, and typed style implementation into
  `@wabou/core`. Expose stable `core` subpaths for advanced imports and stop
  publishing the three implementation workspaces as standalone packages. Add a
  tree-shakeable `@wabou/core/i18n` entry for compiled message libraries.
- Updated dependencies [48ce10c]
- Updated dependencies [789e4c3]
- Updated dependencies [fc05862]
- Updated dependencies [0d4dcde]
- Updated dependencies [6d01dcd]
- Updated dependencies [bb91058]
- Updated dependencies [89ad8ab]
- Updated dependencies [1ee236b]
- Updated dependencies [26c2a18]
- Updated dependencies [c4b7235]
- Updated dependencies [e21aee3]
- Updated dependencies [3b8a18a]
  - @wabou/core@0.1.0-alpha.3

## 0.1.0-alpha.1

### Patch Changes

- Updated dependencies [7d22458]
  - @wabou/solid-renderer@0.1.0-alpha.1
