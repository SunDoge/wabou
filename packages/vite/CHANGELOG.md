# @wabou/vite

## 0.1.0-alpha.3

### Patch Changes

- 48ce10c: Bundle the protocol, renderer, and typed style implementation into
  `@wabou/core`. Expose stable `core` subpaths for advanced imports and stop
  publishing the three implementation workspaces as standalone packages. Add a
  tree-shakeable `@wabou/core/i18n` entry for compiled message libraries.
- 7e4ea9d: Resolve generated FormatJS imports from `@wabou/vite` and declare the
  polyfills as package-owned dependencies, allowing isolated and vendored Wabou
  applications to build without repeating internal Intl dependencies.
- e74e9ca: Require Vite 6 so Wabou's build packages match the supported range of the
  Solid 2 Vite plugin and install without invalid peer-dependency warnings.
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

### Minor Changes

- adf4271: Add serializable class ignore patterns so third-party metadata classes can be
  excluded consistently during compilation and native runtime style resolution.

### Patch Changes

- Updated dependencies [7d22458]
- Updated dependencies [a733c86]
- Updated dependencies [2ef790a]
- Updated dependencies [adf4271]
- Updated dependencies [14a6081]
  - @wabou/solid-renderer@0.1.0-alpha.1
