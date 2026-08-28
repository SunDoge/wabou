# Changelog

All notable changes to Wabou are documented here. Until the public API reaches
stability, preview releases may contain breaking changes between prerelease
tags.

## Unreleased

## 0.1.0-alpha.3 - 2026-08-27

### Changed

- Public JavaScript packages now converge on `@wabou/core` and `@wabou/ui`,
  reducing the package surface applications need to understand.
- Native windows, resources, and behavior tests use typed generational handles,
  including asynchronous window creation and native file drag-and-drop events.
- The router adapter now supports typed search, asynchronous loaders, caching,
  preloading, guards, and native memory history.
- Native windows can select the compiled Vello or experimental Skia renderer
  backend, and persist their logical size for responsive desktop layouts.
- Behavior tests now resolve controls from the rendered semantic tree, scroll
  off-screen targets into view before native pointer routing, and support
  alert and status locators.
- Headless renders and behavior tests isolate application data and rendering
  state, making local and CI results deterministic across repeated runs.
- Gallery interaction scenarios now run as part of CI, including text input,
  modal, and router coverage.
- Package boundary checks now require every runtime import to be declared by
  its package manifest.
- JavaScript packages now publish compiled ESM and declaration artifacts built
  with tsdown instead of requiring consumers to compile Wabou's TSX source.
- Pushes to the development branch run the same CI and commit-convention gates
  used for the release branch.

## 0.1.0-alpha.1 - 2026-08-17

First developer preview.

### Included

- SolidJS 2 universal renderer hosted in QuickJS.
- Retained Taffy layout, Parley text, and Vello rendering.
- Typed Style IR with build-time utility validation and theme support.
- Native widgets, overlays, scrolling, input, clipboard, app directories,
  dialogs, notifications, tray integration, accessibility semantics, and a
  terminal widget.
- Vite development workflow, HMR, source-mapped QuickJS errors, DevTools,
  headless rendering, and behavior-test infrastructure.
- `wabou new` scaffolding with a single Git revision shared by Rust and
  JavaScript dependencies.
- Component gallery and example applications.

### Preview constraints

- The Rust and TypeScript APIs are unstable and may change between alpha tags.
- The supported frontend line is SolidJS `2.0.0-rc.2`.
- Linux builds require the native dependencies installed by the CI workflow.
- Distribution is currently through immutable Git tags; crates.io and npm
  publication are intentionally deferred.
