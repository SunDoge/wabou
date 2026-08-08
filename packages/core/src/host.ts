// The generated declarations are sourced from ../host-abi.json. Two halves:
//
//  1. Host-provided (JS calls Rust): private `__wabou_*` functions Rust injects
//     before the app boots. Applications use `useHost()` instead.
//     Registered in wabou-quick/src/jsrt.rs (`register_core_host_fns` +
//     `register_fetch`/`register_sleep`/the `vite` feature gates).
//
//  2. Guest-provided (Rust calls JS): callbacks the JS side installs so Rust
//     can drive rendering and deliver events. Installed by the glue modules in
//     `./glue/*` (and `@wabou/vite-runtime` for HMR); Rust looks them up via
//     `JsRuntime::{tick, has_raf, dispatch_host_frame, apply_hmr_update}`.
//
// Naming: every `__`-prefixed global uses one convention — `__wabou_*` = the
// internal host↔guest bridge (neither app-facing nor web-standard). A
// drift-guard tests lock the registered set against the generated inventory.

import "./generated/host-abi";
