# Changelog

All notable changes to Wabou are documented here. Until the public API reaches
stability, preview releases may contain breaking changes between prerelease
tags.

## Unreleased

### Changed

- Behavior tests now resolve controls from the rendered semantic tree, scroll
  off-screen targets into view before native pointer routing, and support
  alert and status locators.
- Headless renders and behavior tests isolate application data and rendering
  state, making local and CI results deterministic across repeated runs.
- Gallery interaction scenarios now run as part of CI, including text input,
  modal, and router coverage.
- Package boundary checks now require every runtime import to be declared by
  its package manifest.
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
- The supported frontend line is SolidJS `2.0.0-rc.0`.
- Linux builds require the native dependencies installed by the CI workflow.
- Distribution is currently through immutable Git tags; crates.io and npm
  publication are intentionally deferred.
