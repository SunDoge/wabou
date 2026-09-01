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
import "./polyfills/encoding-streams";

// URL and URLSearchParams are installed by the host's core-prelude (platform.ts,
// bundled with whatwg-url/@ungap via gen-core-prelude) BEFORE the app boots.
// We don't re-import them here: in vite dev the live `import { URL } from
// "whatwg-url"` fails to resolve CJS named exports via raw /@fs/ serving.
import "./polyfills/fetch";

import "./glue/animation-frame";
import "./glue/app-lifecycle";
import "./glue/timers";
import "./glue/resize-observer";
import "./glue/host-messages";
import "./glue/host-frame";
import "./glue/keyboard-modifiers";
import "./glue/window-metrics";
import "./glue/file-drop";
import "./glue/gesture";
import "./glue/effects";
import "./glue/clipboard";
import "./glue/app-dirs";
import "./glue/application";
import "./glue/dialog";
import "./glue/notification";
import "./glue/intl";

export {
  type AppDirectories,
  appDirs,
} from "./glue/app-dirs";
export {
  type AppLifecycleEvent,
  type AppLifecycleState,
  subscribeAppLifecycle,
  useAppLifecycle,
} from "./glue/app-lifecycle";
export { type Application, application } from "./glue/application";
export {
  type AsyncAction,
  AsyncActionConflictError,
  type AsyncActionResult,
  createAsyncAction,
  createKeyedAsyncAction,
  type KeyedAsyncAction,
} from "./glue/async-action";
export {
  type AsyncQuery,
  type AsyncQueryOptions,
  createAsyncQuery,
} from "./glue/async-query";
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
  type EntityKey,
  ForEntity,
  type ForEntityProps,
  validateEntityKeys,
} from "./glue/entity-list";
export {
  createEventEffect,
  type EventEffectOptions,
} from "./glue/event-effect";
export {
  type FileDropEvent,
  type FileDropHandler,
  type FileDropPhase,
  type FileDropPosition,
  subscribeFileDrop,
  useFileDrop,
} from "./glue/file-drop";
export {
  type GestureEvent,
  type GestureHandler,
  type GesturePhase,
  subscribeGesture,
  useGesture,
} from "./glue/gesture";
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
  type RevisionedHostPatch,
  type RevisionedHostResource,
  type RevisionedHostResourceOptions,
  type RevisionedHostValue,
  RevisionedHostWaitError,
  type RevisionedHostWaitErrorReason,
  type RevisionedHostWaitOptions,
} from "./glue/host-resource";
export {
  type CalendarDateFields,
  intl,
} from "./glue/intl";
export {
  bindJsonCapability,
  type JsonCapabilityClient,
  type JsonCapabilityMethodName,
} from "./glue/json-capability";
export {
  type KeyboardModifiers,
  subscribeKeyboardModifiers,
  useKeyboardModifierChanges,
  useKeyboardModifiers,
} from "./glue/keyboard-modifiers";
export {
  createKvSignal,
  type Kv,
  KvAtomicOperation,
  type KvCheck,
  type KvCommitResult,
  type KvEntry,
  type KvKey,
  type KvKeyPart,
  type KvListOptions,
  type KvSetOptions,
  type KvSignal,
  type KvValue,
  type KvVersionstamp,
  openKv,
} from "./glue/kv";
export {
  createLatestAsyncResource,
  type LatestAsyncResource,
  type LatestAsyncResourceOptions,
  type LatestAsyncResourceStatus,
} from "./glue/latest-async-resource";
export {
  bindCapability,
  type CapabilityClientOptions,
  CapabilityError,
  type NativeCapability,
} from "./glue/native-capability";
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
// Keep application-facing renderer types at the stable root. Reconciler hooks,
// protocol writers, dispatch functions, and generated opcodes remain available
// only from `@wabou/core/renderer` to JSX tooling and host infrastructure.
export {
  type BuiltinHost,
  type DebugOverlayOptions,
  type DebugOverlayPaintStats,
  defaultHost,
  Dynamic,
  type DynamicProps,
  type FrameStats,
  type Handle,
  type Host,
  HostProvider,
  type HostProviderProps,
  isDirectEvent,
  type LayoutNodeMetrics,
  type LayoutRect,
  type LayoutScrollMetrics,
  type LayoutSnapshot,
  type LayoutTarget,
  mount,
  observeGlobalPointerEvent,
  Portal,
  type PortalProps,
  setTransform2D,
  useHost,
  type WabouBuiltinIntrinsicElements,
  type WabouControlProps,
  type WabouElementProps,
  type WabouEventTarget,
  type WabouExposedSemanticRole,
  type WabouGlobalPointerEventType,
  type WabouGlobalPointerListener,
  type WabouImageProps,
  type WabouImeDeleteSurroundingEvent,
  type WabouImePreeditEvent,
  type WabouInputEvent,
  type WabouInputProps,
  type WabouKeyEvent,
  type WabouNativeElements,
  type WabouNativeTag,
  type WabouNativeTransition,
  type WabouNodeEvent,
  type WabouPointerEvent,
  type WabouPositionedEvent,
  type WabouScrollEvent,
  type WabouSemanticRole,
  type WabouSubmitEvent,
  type WabouSvgProps,
  type WabouSvgShapeProps,
  type WabouTextCommitEvent,
  type WabouTextSelectionChangeEvent,
  type WabouTransitionEvent,
  type WabouValueChangeEvent,
  type WabouVectorPathProps,
  type WabouWheelEvent,
} from "./renderer";
export { createFps, VirtualList, type VirtualListProps } from "./renderer";
export * from "./style";
export * from "./vector-path";
