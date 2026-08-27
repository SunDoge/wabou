# @wabou/ui

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
