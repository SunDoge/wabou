// Single source of truth for the full Rust↔QuickJS FFI boundary. Two halves:
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
// drift-guard test in wabou-quick locks the registered set against this file.

export {};

declare global {
  const __wabou_capabilities: Record<string, object>;
  // --- Native ABI (not app-facing) --------------------------------------
  // --- Internal bridge: host-provided (JS calls Rust) -------------------
  function __wabou_intern(value: string): number;
  function __wabou_open_url(url: string): boolean;
  function __wabou_set_stylesheet(json: string): void;
  function __wabou_load_font(path: string): boolean;
  function __wabou_frame_stats(): string;
  function __wabou_layout_snapshot(ids: Uint32Array): string;
  /** Flush one binary protocol frame (seq u32 + count u32 + ops) to Rust. */
  function __wabou_flush(buf: Uint8Array): void;
  function __wabou_log(
    level: "debug" | "info" | "warn" | "error" | "log",
    message: string,
  ): void;
  function __wabou_utf8_encode(s: string): Uint8Array;
  function __wabou_utf8_decode(bytes: Uint8Array): string;
  /** Async HTTP. Resolves to a JSON string
   *  `{ status, statusText, headers, body }`. */
  function __wabou_fetch(url: string, initJson: string): Promise<string>;
  function __wabou_sleep(delayMs: number): Promise<void>;
  function __wabou_clipboard_write(text: string): number;
  function __wabou_clipboard_read(): number;
  function __wabou_resize_observe(solidId: number): void;
  function __wabou_resize_unobserve(solidId: number): void;
  const __wabou_window_id: number;
  function __wabou_window_create(optionsJson: string): number;
  function __wabou_window_close(windowId: number): void;
  function __wabou_window_set_maximized(windowId: number, value: boolean): void;
  function __wabou_window_set_title(windowId: number, title: string): void;
  /** Vite HMR style hooks (feature `vite` only); no-ops outside dev. */
  function __wabou_vite_update_style(id: string, css: string): void;
  function __wabou_vite_remove_style(id: string): void;

  // --- Internal bridge: guest-provided (Rust calls JS) ------------------
  // Installed by `./glue/*`; Rust drains these per frame / per event.
  /** Drain the rAF queue, run the renderer sweep, flush the frame. Returns
   *  whether more rAF callbacks remain queued. */
  function __wabou_tick(): boolean;
  function __wabou_has_raf(): boolean;
  /** Atomic, versioned stream for unsolicited Host facts. */
  function __wabou_dispatch_host_frame(data: Uint8Array | ArrayBuffer): {
    preventedEventIds?: Uint32Array;
    needsTick: boolean;
  };
  /** Vite HMR reload hook (feature `vite` only). Returns whether the JS
   *  side accepted the update (`false` → host full-reloads the entry). */
  function __wabou_apply_hmr(
    path: string,
    acceptedPath: string,
    timestamp: number,
  ): Promise<boolean>;
  /** Clear HMR hot records before an in-process full reload (vite only). */
  function __wabou_hmr_clear_records(): void;
  function __wabou_clipboard_complete(
    requestId: number,
    text: string | null,
    success: boolean,
  ): void;
}
