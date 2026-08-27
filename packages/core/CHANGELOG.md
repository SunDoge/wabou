# @wabou/core

## 0.1.0-alpha.3

### Minor Changes

- 48ce10c: Bundle the protocol, renderer, and typed style implementation into
  `@wabou/core`. Expose stable `core` subpaths for advanced imports and stop
  publishing the three implementation workspaces as standalone packages. Add a
  tree-shakeable `@wabou/core/i18n` entry for compiled message libraries.
- 0d4dcde: Make native window creation asynchronous and expose window identities as typed
  two-u32 generational keys so closed handles cannot target reused window slots.
- 6d01dcd: Remove the deprecated `useFps` and `AnimationPlaybackControls` aliases before
  the developer preview, keeping effect-owning primitives and animation handles
  under their single canonical names. Keep the numeric native-effect dispatcher
  inside the framework; application integrations use generated JSON capabilities
  and host messages instead of depending on Wabou's private effect ABI.
- 89ad8ab: Add typed window-level native file drag-and-drop events with logical coordinates and Solid lifecycle cleanup.
- 1ee236b: Add native window-size persistence and reactive logical-size queries for responsive desktop layouts.
- 26c2a18: Allow native windows to select a compiled AnyRender backend explicitly with the `vello` or `skia` renderer option.
- e21aee3: Represent native-menu and effect-tape window identities as lossless generational `{ lo, hi }` keys instead of JavaScript numbers.
- 3b8a18a: Add family-branded generational resource-key helpers and reuse their validated
  two-u32 representation for retained NodeKey storage.

### Patch Changes

- 789e4c3: Add standard Headers and Response globals for host-backed fetch, and an
  Router Core adapter for typed search, async loaders, caching, preloading,
  guards, and native memory navigation.
- fc05862: Compile published JavaScript packages to ESM and declaration files with tsdown instead of requiring consumers to load TypeScript source.
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
- c4b7235: Use Solid 2's built-in `solid-js/refresh` runtime for HMR verification and stop
  installing the obsolete standalone `solid-refresh` package in applications.

## 0.1.0-alpha.1

### Patch Changes

- Make the QuickJS timer glue use the environment's original timer while
  running package tests outside the native host.

- Updated dependencies [7d22458]
- Updated dependencies [a733c86]
- Updated dependencies [2ef790a]
- Updated dependencies [ca4b28b]
  - @wabou/solid-renderer@0.1.0-alpha.1
  - @wabou/style@0.1.0-alpha.1
  - @wabou/protocol@0.1.0-alpha.1
