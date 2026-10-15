# @wabou/vite

## 0.1.0-alpha.4

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
- c47c253: Keep Solid dependencies in Vitest's module graph so component tests share the client reactive runtime when using Vitest 5.
- 7cffa29: Upgrade the application build pipeline to Vite 8 and its Rolldown/Oxc toolchain while preserving Wabou's single-IIFE QuickJS bundle and paired Solid/Style IR hot updates.
- Resolve generated FormatJS imports from `@wabou/vite` and declare the
  polyfills as package-owned dependencies, allowing isolated and vendored Wabou
  applications to build without repeating internal Intl dependencies.
- Require Vite 6 so Wabou's build packages match the supported range of the
  Solid 2 Vite plugin and install without invalid peer-dependency warnings.
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
