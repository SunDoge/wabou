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
import "./glue/effects";
import "./glue/clipboard";
import "./glue/app-dirs";
import "./glue/dialog";
import "./glue/notification";

// The renderer and typed style surface are re-exported here so application
// code has one stable runtime entry point. Their package names remain an
// implementation detail used by the Vite transform and Wabou's own packages.
export * from "@wabou/solid-renderer";
export * from "@wabou/style";
export {
  type AppDirectories,
  appCacheDir,
  appConfigDir,
  appDataDir,
  appDirs,
  appLocalDataDir,
  appLogDir,
  resolveAppDirectories,
  resourceDir,
  tempDir,
} from "./glue/app-dirs";
export { type Clipboard, clipboard, useClipboard } from "./glue/clipboard";
export {
  type ColorPalette,
  type ColorThemeAnimation,
  type ColorThemeAnimationOptions,
  type ColorThemeController,
  type ColorThemeEasing,
  ColorThemeProvider,
  colorTheme,
  useColorTheme,
} from "./glue/color-theme";
export {
  type Dialog,
  type DialogFilter,
  dialog,
  type MessageDialogButtons,
  type MessageDialogLevel,
  type MessageDialogOptions,
  type MessageDialogResult,
  type OpenDialogOptions,
  type PickDirectoryOptions,
  type SaveDialogOptions,
  useDialog,
} from "./glue/dialog";
export {
  dispatchEffect,
  dispatchFireAndForget,
  dispatchResourceEffect,
  EFFECT_ABI_VERSION,
  type EffectOp,
  effectOps,
} from "./glue/effects";
export type {
  HostMessage,
  HostMessageAllHandler,
  HostMessageHandler,
} from "./glue/host-messages";
export {
  hostMessages,
  subscribe as subscribeHostMessages,
  subscribeAll as subscribeAllHostMessages,
} from "./glue/host-messages";
export {
  type NativeMenuItem,
  type NativeMenuOptions,
  type NativeMenuPosition,
  showNativeMenu,
} from "./glue/native-menu";
export {
  type Notification,
  type NotificationOptions,
  notification,
  useNotification,
} from "./glue/notification";
export {
  PlatformProvider,
  type PlatformProviderProps,
  type PlatformServices,
} from "./glue/platform-context";
export {
  type CreateWindowOptions,
  createWindow,
  currentWindow,
  type WindowHandle,
} from "./glue/window";
export {
  useWindow,
  type WindowMetrics,
  type WindowState,
} from "./glue/window-metrics";
