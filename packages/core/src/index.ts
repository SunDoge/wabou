// @wabou/core — installs the Web-API surface the host (Rust) doesn't
// provide natively. Importing this package for side effects is enough: each
// module below self-installs onto globalThis. The host fn contract lives in
// `./host` (ambient `declare global`); runtime host fns are injected by
// crates/wabou-runtime/src/jsrt.rs before the app boots.
//
// Load order matters only where a later module relies on an earlier one's
// global (e.g. the lazy TextEncoder in the protocol layer, resolved on
// first use well after this init). Polyfills that need no host fn can go in
// any order; glue modules import the bundled renderer layer directly.

import "./host";
import "./polyfills/abort-controller";
import "./polyfills/dom-exception";
import "./polyfills/crypto";
import "./polyfills/streams";

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
import "./glue/file-drop";
import "./glue/effects";
import "./glue/clipboard";
import "./glue/app-dirs";
import "./glue/application";
import "./glue/dialog";
import "./glue/notification";
import "./glue/intl";

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
export { type Application, application } from "./glue/application";
export {
  type AsyncAction,
  AsyncActionConflictError,
  type AsyncActionResult,
  createAsyncAction,
  createKeyedAsyncAction,
  type KeyedAsyncAction,
} from "./glue/async-action";
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
  type FileDropEvent,
  type FileDropHandler,
  type FileDropPhase,
  type FileDropPosition,
  subscribeFileDrop,
  useFileDrop,
} from "./glue/file-drop";
export {
  createEventEffect,
  type EventEffectOptions,
} from "./glue/event-effect";
export type {
  HostJsonSubscriptionOptions,
  HostMessage,
  HostMessageAllHandler,
  HostMessageHandler,
} from "./glue/host-messages";
export {
  hostMessages,
  subscribe as subscribeHostMessages,
  subscribeAll as subscribeAllHostMessages,
  subscribeJson as subscribeJsonHostMessages,
} from "./glue/host-messages";
export {
  createRevisionedHostResource,
  RevisionedHostWaitError,
  type RevisionedHostPatch,
  type RevisionedHostResource,
  type RevisionedHostResourceOptions,
  type RevisionedHostWaitOptions,
  type RevisionedHostWaitErrorReason,
  type RevisionedHostValue,
} from "./glue/host-resource";
export {
  type CalendarDateFields,
  intl,
} from "./glue/intl";
export {
  bindJsonCapability,
  type JsonCapabilityClient,
  type JsonCapabilityClientOptions,
  JsonCapabilityError,
  type JsonCapabilityMethodName,
  type NativeJsonCapability,
} from "./glue/json-capability";
export {
  createLatestAsyncResource,
  type LatestAsyncResource,
  type LatestAsyncResourceOptions,
  type LatestAsyncResourceStatus,
} from "./glue/latest-async-resource";
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
  currentWindowOptions,
  type WindowHandle,
  type WindowKey,
} from "./glue/window";
export {
  createWindowMatch,
  useWindow,
  type WindowMetrics,
  type WindowSizeQuery,
  type WindowState,
} from "./glue/window-metrics";
export * from "./keyed-list";
export * from "./registry";
// The renderer and typed style surface are re-exported here so application
// code has one stable runtime entry point. Separate source workspaces remain
// an implementation detail and are bundled into this package for release.
export * from "./renderer";
export * from "./style";
export * from "./vector-path";
