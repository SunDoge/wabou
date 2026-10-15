# @wabou/ui

## 0.1.0-alpha.4

### Minor Changes

- Add standard Headers and Response globals for host-backed fetch, and an
  Router Core adapter for typed search, async loaders, caching, preloading,
  guards, and native memory navigation.
- 6520e0f: Drive CodeEditor JSON syntax ranges with headless CodeMirror State and incremental Lezer parsing.
- Remove the deprecated `useFps` and `AnimationPlaybackControls` aliases before
  the developer preview, keeping effect-owning primitives and animation handles
  under their single canonical names. Keep the numeric native-effect dispatcher
  inside the framework; application integrations use generated JSON capabilities
  and host messages instead of depending on Wabou's private effect ABI.
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
- 851d731: Add an explicit styled-text behavior flag and public `RichText` and
  `RichTextSpan` primitives that shape styled fragments as one native paragraph.
- 625e23c: Discover complete utility classes in static variant maps without requiring a
  separate registration API. Add typed support for clipped multi-line text,
  static transforms, explicit line heights, and three-digit arbitrary colors.
  Keep component animation tests on Wabou's frame clock under Vitest 5 and allow
  lifecycle-owned animation signals to update from frame callbacks.
- Make `@wabou/ui` the single application-facing UI package. Bundle the
  animation, component, primitive, and router implementation workspaces into its
  published JavaScript and declarations instead of publishing those source
  boundaries independently.

### Patch Changes

- Declare the Solid web type dependency used by public component APIs, and verify
  that every runtime source import is backed by its package manifest.
- 86be071: Migrate the DataTable adapter to TanStack Table v9 with explicit row models and reactive state bindings. Export TanStackDataTableRow alongside the existing column helper type for application callbacks.
- be282e7: Upgrade the Solid 2 runtime to rc.6 and the matching Solid Vite compiler to
  the current Vite 8-compatible preview.
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

- 789e4c3: Add standard Headers and Response globals for host-backed fetch, and an
  Router Core adapter for typed search, async loaders, caching, preloading,
  guards, and native memory navigation.
- 6d01dcd: Remove the deprecated `useFps` and `AnimationPlaybackControls` aliases before
  the developer preview, keeping effect-owning primitives and animation handles
  under their single canonical names. Keep the numeric native-effect dispatcher
  inside the framework; application integrations use generated JSON capabilities
  and host messages instead of depending on Wabou's private effect ABI.
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
- 1e7d362: Make `@wabou/ui` the single application-facing UI package. Bundle the
  animation, component, primitive, and router implementation workspaces into its
  published JavaScript and declarations instead of publishing those source
  boundaries independently.

### Patch Changes

- cc687d8: Declare the Solid web type dependency used by public component APIs, and verify
  that every runtime source import is backed by its package manifest.
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
