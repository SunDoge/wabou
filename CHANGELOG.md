# Changelog

All notable changes to Wabou are documented here. Until the public API reaches
stability, preview releases may contain breaking changes between prerelease
tags.

## Unreleased

## 0.1.0-alpha.4 - 2026-10-19

The staged alpha.3 preview was never tagged. Alpha.4 contains those changes and
is the first published preview after alpha.2.

### Changed

- Consolidate the native runtime on Vello GPU 0.3 and the generated widget
  contract while keeping `wabou`, `@wabou/ui`, and `@wabou/vite` as the normal
  application-facing entry points.
- Add a pure-Rust PDF widget, retained rich-text runs, CodeMirror-backed editor
  state, and the GPUI shell visual benchmark application.
- Upgrade the frontend toolchain to Solid 2 RC 14 and Vite 8.
- Generate QuickJS names and browser/Wasm installation adapters from one host
  ABI schema, and route generated capability clients through a side-effect-free
  host-context entry so bindings cannot drift or install a second runtime.
- Discover complete utility classes in static variant maps without requiring
  manual registration. Dynamic construction from runtime fragments remains a
  build error.
- Extend the typed style contract with static transforms, `leading-6`, and
  three-digit arbitrary colors.
- Migrate the DataTable integration to TanStack Table v9 while preserving
  Solid-controlled filtering, sorting, selection, and reactive data updates.
- Update Valibot to 1.5.0, Bun type definitions to 1.4.2, and Node type
  definitions to 26.5.1.
- Replace clap with usage-rs for the `wabou` command-line interface while
  preserving existing commands, validation, and ordered render actions.
- Keep the main `wabou` executable free of the renderer engine and forward
  headless rendering to an independently built `wabou-render` helper.
- Add centered native window placement plus launcher and reader reference
  applications with high-resolution layout coverage.
- Generate application host bindings from Rust types and automatically enable
  the generator target's required Cargo features.
- Use a pinned `vendor/wabou` Git submodule as the preview integration path so
  Rust crates, JavaScript packages, and the CLI share one revision, with
  `wabou vendor` adding, initializing, or updating that checkout safely.
- Converge the public JavaScript surface on `@wabou/core` and `@wabou/ui`, ship
  compiled ESM and declarations, and enforce runtime package boundaries.
- Add typed generational handles for native windows and resources, a native
  memory-history router with typed search and async loaders, and deterministic
  semantic behavior tests for scrolling, controls, dialogs, and navigation.
- Replace the legacy capability and application-integration paths with one
  generated QuickJS host ABI and protocol, keeping Rust as the source of truth
  for host bindings.
- Encode stylesheet IR as a compact binary format and reduce the resident
  QuickJS web compatibility surface and memory overhead.
- Add an injected host clock and byte-reproducible virtual time for headless
  behavior tests, animations, native widgets, and terminal cursor timing.
- Upgrade Taffy to 0.15 and align out-of-flow layout diagnostics and component
  containing blocks with its CSS-compatible absolute positioning behavior.
- Gate Biome formatting and linting in CI and keep generated JavaScript,
  TypeScript declarations, Rust bindings, and release bundles reproducible.

### Fixed

- Allow bounded text previews to clip at Parley's shaped line boundary without
  inserting an ellipsis.
- Keep Wabou's animation frame queue authoritative under Vitest 5 and allow
  lifecycle-owned animation signals to update safely from frame callbacks.
- Component tests share Solid's client runtime when using Vitest 5, preserving
  reactive context and updates through the universal renderer.
- Keep contextual message actions within their content column after Taffy 0.15
  hoists absolutely positioned nodes to their containing block.

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
- The supported frontend line is SolidJS `2.0.0-rc.6`.
- Linux builds require the native dependencies installed by the CI workflow.
- Distribution is currently through immutable Git tags; crates.io and npm
  publication are intentionally deferred.
