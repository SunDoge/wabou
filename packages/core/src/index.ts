// @wabou/core — installs the Web-API surface the host (Rust) doesn't
// provide natively. Importing this package for side effects is enough: each
// module below self-installs onto globalThis. The host fn contract lives in
// `./host` (ambient `declare global`); runtime host fns are injected by
// crates/wabou-quick/src/jsrt.rs before the app boots.
//
// Load order matters only where a later module relies on an earlier one's
// global (e.g. the lazy TextEncoder in @wabou/protocol, resolved on
// first use well after this init). Polyfills that need no host fn can go in
// any order; glue modules import from @wabou/solid-renderer directly.

import "./host";

// URL and URLSearchParams are installed by the host's core-prelude (platform.ts,
// bundled with whatwg-url/@ungap via gen-core-prelude) BEFORE the app boots.
// We don't re-import them here: in vite dev the live `import { URL } from
// "whatwg-url"` fails to resolve CJS named exports via raw /@fs/ serving.
import "./polyfills/fetch";

import "./glue/animation-frame";
import "./glue/timers";
import "./glue/resize-observer";
import "./glue/host-messages";
import "./glue/host-frame";
import "./glue/window-metrics";
import "./glue/clipboard";

export {
  type WindowMetrics,
  type WindowState,
  useWindow,
} from "./glue/window-metrics";

export {
  createWindow,
  currentWindow,
  type CreateWindowOptions,
  type WindowHandle,
} from "./glue/window";

export { clipboard, type Clipboard, useClipboard } from "./glue/clipboard";
export {
  type PlatformProviderProps,
  PlatformProvider,
  type PlatformServices,
} from "./glue/platform-context";

export {
  hostMessages,
  subscribe as subscribeHostMessages,
  subscribeAll as subscribeAllHostMessages,
} from "./glue/host-messages";
export type {
  HostMessage,
  HostMessageAllHandler,
  HostMessageHandler,
} from "./glue/host-messages";
