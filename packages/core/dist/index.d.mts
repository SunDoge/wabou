import { T as ResourceKey } from "./protocol-C7gC5Ufi.mjs";
import { a as subscribeFileDrop, i as FileDropPosition, n as FileDropHandler, o as useFileDrop, r as FileDropPhase, t as FileDropEvent } from "./file-drop-Ci2RppE8.mjs";
import { n as WabouIntrinsicElements, t as HostCapabilities } from "./registry-DXOPfC3L.mjs";
import { A as WabouStyle, C as WabouBaseUtility, D as WabouStaticUtility, E as WabouSpacingToken, O as WabouUtility, S as utilityConflictProperties, T as WabouDynamicUtility, _ as rgba, a as ShadowOptions, b as shadow, c as assertInlineStyleValue, d as classes, f as isTypedStyleValue, g as px, h as percent, i as Shadow, k as INLINE_STYLE_CONTRACT, l as auto, m as number, n as ClassValue, o as StyleValueKind, p as mergeClasses, r as STYLE_VALUE, s as TypedStyleValue, t as Affine2D, u as bool, v as rotate2d, w as WabouColorToken, x as translate2d, y as scale2d } from "./style-D-UEvXmH.mjs";
import { A as WabouSvgProps, At as LayoutSnapshot, Bt as PathPoint, C as WabouNativeTransition, Ct as defaultHost, D as WabouScrollEvent, Dt as LayoutNodeMetrics, E as WabouPositionedEvent, Et as FrameStats, F as WabouValueChangeEvent, Ht as VectorPathPaint, I as WabouVectorPathProps, It as PathBuilder, L as WabouWheelEvent, Lt as PathFillRule, M as WabouTextCommitEvent, N as WabouTextSelectionChangeEvent, O as WabouSemanticRole, Ot as LayoutRect, P as WabouTransitionEvent, Rt as PathLineCap, S as WabouNativeTag, St as LayoutTarget, T as WabouPointerEvent, Tt as DebugOverlayPaintStats, Ut as isVectorPath, Vt as VectorPath, X as isDirectEvent, _ as WabouImePreeditEvent, _t as BuiltinHost, a as WabouBuiltinIntrinsicElements, b as WabouKeyEvent, bt as HostProvider, c as WabouEventTarget, d as WabouFloatingPlacement, et as mount, f as WabouFloatingPosition, g as WabouImeDeleteSurroundingEvent, gt as PortalProps, h as WabouImageProps, ht as Portal, j as WabouSvgShapeProps, k as WabouSubmitEvent, kt as LayoutScrollMetrics, l as WabouExposedSemanticRole, m as WabouGlobalPointerListener, mt as VirtualListProps, n as DynamicProps, o as WabouControlProps, p as WabouGlobalPointerEventType, pt as VirtualList, r as Handle, s as WabouElementProps, t as Dynamic, tt as observeGlobalPointerEvent, u as WabouFloatingAnchor, ut as setTransform2D, v as WabouInputEvent, vt as DebugOverlayOptions, w as WabouNodeEvent, wt as useHost, x as WabouNativeElements, xt as HostProviderProps, y as WabouInputProps, yt as Host, zt as PathLineJoin } from "./index-CbP-wDGo.mjs";
import { t as createFps } from "./renderer-nqfecC76.mjs";
import { Accessor, JSX, SourceAccessor } from "solid-js";
//#region src/generated/host-abi.d.ts
declare global {
  const __wabou_capabilities: Record<string, object>;
  function __wabou_intern(value: string): number;
  function __wabou_open_url(url: string): boolean;
  function __wabou_set_stylesheet(json: string): void;
  function __wabou_set_color_theme(name: string): void;
  function __wabou_get_color_theme_palette(name: string, output: Uint32Array | undefined): number;
  function __wabou_set_color_palette(colors: Uint32Array): void;
  function __wabou_load_font(path: string): boolean;
  function __wabou_frame_stats(): string;
  function __wabou_set_debug_overlay(layout: boolean, clips: boolean, hitTarget: boolean): boolean;
  function __wabou_debug_overlay_paint_stats(): string;
  function __wabou_layout_snapshot(ids: Uint32Array, output: Float64Array | undefined): number;
  function __wabou_system_locale(): string;
  function __wabou_system_time_zone(): string;
  function __wabou_system_calendar_date(): string;
  function __wabou_flush(buf: Uint8Array): void;
  function __wabou_log(level: "debug" | "info" | "warn" | "error" | "log", message: string): void;
  function __wabou_utf8_encode(value: string): Uint8Array;
  function __wabou_utf8_decode(bytes: Uint8Array): string;
  function __wabou_fetch(url: string, initJson: string): Promise<{
    status: number;
    statusText: string;
    headers: Record<string, string>;
    body: Uint8Array;
  }>;
  function __wabou_crypto_random(output: Uint8Array): void;
  function __wabou_crypto_digest(algorithm: number, input: Uint8Array): Promise<Uint8Array>;
  function __wabou_sleep(delayMs: number): Promise<void>;
  function __wabou_resize_observe(nodeLo: number, nodeHi: number): void;
  function __wabou_resize_unobserve(nodeLo: number, nodeHi: number): void;
  const __wabou_effect_abi: number;
  function __wabou_effect_submit(capability: number, method: number, payloadJson: string): number;
  const __wabou_window_id_lo: number;
  const __wabou_window_id_hi: number;
  function __wabou_vite_update_style(id: string, css: string): void;
  function __wabou_vite_remove_style(id: string): void;
  function __wabou_tick(): boolean;
  function __wabou_has_raf(): boolean;
  function __wabou_dispatch_host_frame(data: Uint8Array | ArrayBuffer): {
    preventedEventIds?: Uint32Array;
    needsTick: boolean;
    protocolFrame?: Uint8Array;
  };
  function __wabou_apply_hmr(path: string, acceptedPath: string, timestamp: number): Promise<boolean>;
  function __wabou_hmr_clear_records(): void;
  function __wabou_effect_complete(requestId: number, capability: number, method: number, status: number, payloadJson: string): void;
}
//#endregion
//#region src/glue/app-lifecycle.d.ts
type AppLifecycleState = "resumed" | "suspended" | "memory-warning";
interface AppLifecycleEvent {
  state: AppLifecycleState;
}
/** Subscribe to operating-system lifecycle notifications. */
declare function subscribeAppLifecycle(handler: (event: AppLifecycleEvent) => void): () => void;
/** Subscribe for the lifetime of the current Solid owner. */
declare function useAppLifecycle(handler: (event: AppLifecycleEvent) => void): void;
//#endregion
//#region src/glue/host-messages.d.ts
type HostMessageHandler = (payload: unknown) => void;
type HostMessageAllHandler = (topic: string, payload: unknown) => void;
interface HostMessage {
  topic: string;
  payload: unknown;
}
interface HostJsonSubscriptionOptions<T> {
  decode?: (value: unknown) => T;
  onError?: (error: unknown, payload: unknown) => void;
}
/**
 * Subscribe to host messages on `topic`.
 * Returns an unsubscribe function.
 */
declare function subscribe(topic: string, handler: HostMessageHandler): () => void;
/** Subscribe to every topic; handler receives `(topic, payload)`. */
declare function subscribeAll(handler: HostMessageAllHandler): () => void;
/** Subscribe to a host topic carrying JSON text or UTF-8 bytes. */
declare function subscribeJson<T>(topic: string, handler: (value: T) => void, options?: HostJsonSubscriptionOptions<T>): () => void;
declare const hostMessages: {
  subscribe: typeof subscribe;
  subscribeAll: typeof subscribeAll;
  subscribeJson: typeof subscribeJson;
};
//#endregion
//#region src/glue/keyboard-modifiers.d.ts
interface KeyboardModifiers {
  readonly bits: number;
  readonly shift: boolean;
  readonly control: boolean;
  readonly alt: boolean;
  readonly meta: boolean;
  readonly primary: boolean;
}
/** Reactive, Host-authoritative physical modifier-key state. */
declare function useKeyboardModifiers(): Accessor<KeyboardModifiers>;
/** Subscribe to physical modifier changes without creating a Solid owner. */
declare function subscribeKeyboardModifiers(handler: (modifiers: KeyboardModifiers) => void): () => void;
/** Subscribe for the lifetime of the current Solid owner. */
declare function useKeyboardModifierChanges(handler: (modifiers: KeyboardModifiers) => void): void;
//#endregion
//#region src/glue/window.d.ts
type WindowKey = ResourceKey<"window">;
interface CreateWindowOptions {
  title?: string;
  width?: number;
  height?: number;
  minWidth?: number;
  minHeight?: number;
  resizable?: boolean;
  /** Show native window-manager borders and title bar. */
  decorations?: boolean;
  /** Native compositor material visible through transparent app content. */
  background?: "opaque" | "transparent" | "blurred" | "mica" | "micaAlt";
  /** Initial native stacking request. Wayland generally ignores non-normal levels. */
  windowLevel?: "alwaysOnBottom" | "normal" | "alwaysOnTop";
  /** Allow pointer input to pass through this native window where supported. */
  inputMode?: "interactive" | "passthrough";
}
/** Immutable native creation options for the JavaScript runtime's window. */
declare function currentWindowOptions(): Readonly<CreateWindowOptions>;
interface WindowHandle {
  readonly id: WindowKey;
  close(): void;
  minimize(): void;
  setMaximized(value: boolean): void;
  setTitle(title: string): void;
  /** Begin a compositor-managed move operation for a custom title bar. */
  startDragging(): void;
  /** Restore this window if it was hidden or released to the tray. */
  show(): void;
}
/** Create an independent native window running this application's bundle. */
declare function createWindow(options?: CreateWindowOptions): Promise<WindowHandle>;
/** An imperative handle for the native window that owns this JS runtime. */
declare function currentWindow(): WindowHandle;
//#endregion
//#region src/glue/window-metrics.d.ts
interface WindowMetrics {
  windowId: WindowKey;
  logicalWidth: number;
  logicalHeight: number;
  physicalWidth: number;
  physicalHeight: number;
  scaleFactor: number;
  maximized: boolean;
  focused: boolean;
  outerX: number | null;
  outerY: number | null;
  occluded: boolean;
  colorScheme: "light" | "dark" | null;
  reducedMotion: boolean;
}
interface WindowState extends WindowHandle {
  metrics: Accessor<WindowMetrics>;
  width: Accessor<number>;
  height: Accessor<number>;
  scaleFactor: Accessor<number>;
  maximized: Accessor<boolean>;
  focused: Accessor<boolean>;
  outerX: Accessor<number | null>;
  outerY: Accessor<number | null>;
  occluded: Accessor<boolean>;
  colorScheme: Accessor<"light" | "dark">;
  reducedMotion: Accessor<boolean>;
}
/** Inclusive logical-pixel constraints evaluated against the native client area. */
interface WindowSizeQuery {
  minWidth?: number;
  maxWidth?: number;
  minHeight?: number;
  maxHeight?: number;
}
/**
 * Create a reactive native-window size query without CSS media-query semantics.
 * A zero-sized pre-boot viewport never matches, avoiding a compact-layout flash.
 */
declare function createWindowMatch(query: WindowSizeQuery, window?: WindowState): Accessor<boolean>;
/** Reactive state and controls for the native window owning this JS runtime. */
declare function useWindow(): WindowState;
//#endregion
//#region src/glue/gesture.d.ts
type GesturePhase = "started" | "changed" | "ended" | "cancelled";
type GestureEvent = {
  type: "pinch";
  delta: number;
  phase: GesturePhase;
} | {
  type: "pan";
  deltaX: number;
  deltaY: number;
  phase: GesturePhase;
} | {
  type: "rotation";
  delta: number;
  phase: GesturePhase;
} | {
  type: "double-tap";
} | {
  type: "pressure";
  pressure: number;
  stage: number;
};
type GestureHandler = (event: GestureEvent) => void;
/** Subscribe to native trackpad and touchscreen gestures for the current window. */
declare function subscribeGesture(handler: GestureHandler): () => void;
/** Subscribe for the lifetime of the current Solid owner. */
declare function useGesture(handler: GestureHandler): void;
//#endregion
//#region src/glue/clipboard.d.ts
interface Clipboard {
  readText(): Promise<string | null>;
  writeText(text: string): Promise<void>;
}
declare const clipboard: Clipboard;
/** Stable clipboard capability for use inside Solid components. */
declare function useClipboard(): Clipboard;
//#endregion
//#region src/glue/app-dirs.d.ts
/** Native, absolute roots owned by the current application. */
interface AppDirectories {
  readonly configDir: string;
  readonly dataDir: string;
  readonly localDataDir: string;
  readonly cacheDir: string;
  readonly logDir: string;
  readonly resourceDir: string;
  readonly tempDir: string;
}
declare function resolve$1(): Promise<AppDirectories>;
/** Resolve app-private native roots, caching the host result for this runtime. */
declare const appDirs: Readonly<{
  resolve: typeof resolve$1;
  config: () => Promise<string>;
  data: () => Promise<string>;
  localData: () => Promise<string>;
  cache: () => Promise<string>;
  log: () => Promise<string>;
  resource: () => Promise<string>;
  temp: () => Promise<string>;
}>;
//#endregion
//#region src/glue/application.d.ts
interface Application {
  /** Terminate the full native application, including tray-resident windows. */
  exit(): void;
  /** Gracefully stop the application and launch the same executable again. */
  relaunch(): void;
}
declare const application: Application;
//#endregion
//#region src/glue/dialog.d.ts
interface DialogFilter {
  readonly name: string;
  readonly extensions: readonly string[];
}
interface OpenDialogOptions {
  readonly title?: string;
  readonly directory?: string;
  readonly filters?: readonly DialogFilter[];
  readonly multiple?: boolean;
}
interface SaveDialogOptions {
  readonly title?: string;
  readonly directory?: string;
  readonly defaultName?: string;
  readonly filters?: readonly DialogFilter[];
}
interface PickDirectoryOptions {
  readonly title?: string;
  readonly directory?: string;
}
type MessageDialogLevel = "info" | "warning" | "error";
type MessageDialogButtons = "ok" | "okCancel" | "yesNo" | "yesNoCancel";
type MessageDialogResult = "ok" | "cancel" | "yes" | "no" | "custom";
interface MessageDialogOptions {
  readonly title?: string;
  readonly message: string;
  readonly level?: MessageDialogLevel;
  readonly buttons?: MessageDialogButtons;
}
interface Dialog {
  open(options?: OpenDialogOptions): Promise<readonly string[] | null>;
  save(options?: SaveDialogOptions): Promise<string | null>;
  pickDirectory(options?: PickDirectoryOptions): Promise<string | null>;
  message(options: MessageDialogOptions): Promise<MessageDialogResult>;
}
declare const dialog: Dialog;
declare function useDialog(): Dialog;
//#endregion
//#region src/glue/notification.d.ts
interface NotificationOptions {
  readonly title: string;
  readonly body?: string;
  readonly icon?: string;
  readonly silent?: boolean;
}
interface Notification {
  show(options: NotificationOptions): Promise<void>;
}
declare const notification: Notification;
declare function useNotification(): Notification;
//#endregion
//#region src/glue/intl.d.ts
interface CalendarDateFields {
  year: number;
  month: number;
  day: number;
}
/**
 * Operating-system locale facts. Standards-compatible formatting is installed
 * separately by the FormatJS-backed Intl polyfill.
 */
declare const intl: Readonly<{
  locale(): string;
  timeZone(): string;
  today(): CalendarDateFields;
}>;
//#endregion
//#region src/glue/async-action.d.ts
type AsyncActionResult<T> = {
  ok: true;
  value: T;
} | {
  ok: false;
  error: unknown;
};
/** A concurrent call tried to replace the arguments of an in-flight action. */
declare class AsyncActionConflictError extends Error {
  constructor();
}
interface AsyncAction<Args extends unknown[], T> {
  pending: Accessor<boolean>;
  /** Arguments owned by the current single flight, or undefined while idle. */
  pendingArgs: Accessor<Args | undefined>;
  error: Accessor<unknown | undefined>;
  run(...args: Args): Promise<AsyncActionResult<T>>;
  reset(): void;
}
interface KeyedAsyncAction<Key, Args extends unknown[], T> {
  pendingKeys: Accessor<ReadonlySet<Key>>;
  pending(key: Key): boolean;
  error(key: Key): unknown | undefined;
  run(...args: Args): Promise<AsyncActionResult<T>>;
  reset(key: Key): void;
  resetAll(): void;
}
/**
 * Run an imperative async operation as a single flight with explicit state.
 * Repeated calls with the same argument identities join the pending operation.
 * A call with different arguments returns [`AsyncActionConflictError`] rather
 * than silently discarding those arguments. Use `createKeyedAsyncAction` when
 * independently keyed operations should run concurrently.
 */
declare function createAsyncAction<Args extends unknown[], T>(action: (...args: Args) => PromiseLike<T> | T): AsyncAction<Args, T>;
/**
 * Run one async single-flight per stable key. Calls for the same key join the
 * existing operation, while unrelated keys remain independently concurrent.
 */
declare function createKeyedAsyncAction<Key, Args extends unknown[], T>(keyOf: (...args: Args) => Key, action: (...args: Args) => PromiseLike<T> | T): KeyedAsyncAction<Key, Args, T>;
//#endregion
//#region src/glue/async-query.d.ts
interface AsyncQueryOptions<K, T> {
  source: Accessor<K | undefined>;
  load: (key: K, context: {
    signal: AbortSignal;
  }) => Promise<T>;
  initialValue?: T;
}
interface AsyncQuery<T> {
  /** Read the current result, suspending through Solid's nearest Loading boundary. */
  value: SourceAccessor<T | undefined>;
  /** Read the last settled result while a replacement is loading. */
  latest: Accessor<T | undefined>;
  /** Re-run the query for its current key and await the settled result. */
  refresh(): Promise<T | undefined>;
}
/**
 * Create a latest-wins query using Solid 2's native async graph.
 *
 * Promise ownership, stale-result suppression, pending propagation, and error
 * propagation belong to Solid. Wabou only adds AbortSignal lifecycle and an
 * explicit refresh operation.
 */
declare function createAsyncQuery<K, T>(options: AsyncQueryOptions<K, T>): AsyncQuery<T>;
//#endregion
//#region src/glue/color-theme.d.ts
type ColorThemeEasing = "linear" | "ease-in" | "ease-out" | "ease-in-out" | ((progress: number) => number);
interface ColorThemeAnimationOptions {
  /** Animation duration in seconds, matching Wabou animation helpers. */
  duration?: number;
  easing?: ColorThemeEasing;
  colorSpace?: "oklab" | "srgb";
}
interface ColorThemeAnimation {
  readonly finished: Promise<void>;
  cancel(): void;
}
type ColorPalette = Uint32Array;
interface ColorThemeController {
  current(): string | undefined;
  set(name: string): void;
  getPalette(name: string): ColorPalette;
  setPalette(colors: ColorPalette): void;
  animateTo(name: string, options?: ColorThemeAnimationOptions): ColorThemeAnimation;
}
declare const colorTheme: ColorThemeController;
/** Selects one compiled color palette for the current native window. */
declare function ColorThemeProvider(props: {
  theme: string;
  transition?: ColorThemeAnimationOptions | false;
  children: JSX.Element;
}): JSX.Element;
declare function useColorTheme(): ColorThemeController;
//#endregion
//#region src/glue/entity-list.d.ts
type EntityKey = string | number;
interface ForEntityProps<T, K extends EntityKey> {
  each: readonly T[] | undefined | null | false;
  by: (item: T) => K;
  fallback?: JSX.Element;
  children: (item: T, index: Accessor<number>) => JSX.Element;
}
declare function validateEntityKeys<T, K extends EntityKey>(values: readonly T[], by: (item: T) => K): readonly T[];
/**
 * Render stateful entities by a stable application key.
 *
 * The entity object itself is part of the identity contract: mutate its
 * internal signals/stores instead of replacing it with a new snapshot carrying
 * the same key. This keeps native widgets and other owned resources mounted.
 */
declare function ForEntity<T, K extends EntityKey>(props: ForEntityProps<T, K>): JSX.Element;
//#endregion
//#region src/glue/event-effect.d.ts
interface EventEffectOptions<T> {
  /** A retained event feed. Items may be newest-first or oldest-first. */
  source: Accessor<readonly T[]>;
  /** A strictly increasing sequence assigned when the event is produced. */
  sequence: (event: T) => number;
  onEvent: (event: T) => unknown;
  /** Receives synchronous throws and asynchronous rejections from `onEvent`. */
  onError?: (error: unknown, event: T) => void;
  /** Consume retained history on mount. Defaults to false. */
  consumeInitial?: boolean;
}
/**
 * Consume every new event from a retained feed exactly once and in sequence
 * order. This avoids losing events when several feed updates are batched into
 * one reactive notification.
 */
declare function createEventEffect<T>(options: EventEffectOptions<T>): void;
//#endregion
//#region src/glue/host-resource.d.ts
interface RevisionedHostValue {
  revision: number;
}
interface RevisionedHostPatch {
  baseRevision: number;
}
interface RevisionedHostResourceOptions<T extends RevisionedHostValue, P extends RevisionedHostPatch> {
  initial: T;
  load: () => Promise<T>;
  snapshotTopic: string;
  patchTopic?: string;
  applyPatch?: (current: T, patch: P) => T | undefined;
  decodeSnapshot?: HostJsonSubscriptionOptions<T>["decode"];
  decodePatch?: HostJsonSubscriptionOptions<P>["decode"];
  onValue?: (value: T, source: "load" | "snapshot" | "patch") => void;
  onError?: (error: unknown, source: "load" | "snapshot" | "patch") => void;
  autoLoad?: boolean;
}
interface RevisionedHostResource<T extends RevisionedHostValue> {
  value: Accessor<T>;
  loading: Accessor<boolean>;
  error: Accessor<unknown | undefined>;
  /**
   * Load an authoritative snapshot from the host.
   *
   * Transport/load failures are stored in `error()` and also reject this
   * promise so command paths cannot accidentally continue with stale state.
   * `undefined` only means the result lost a revision race or the resource was
   * already disposed.
   */
  refresh(): Promise<T | undefined>;
  waitFor(predicate: (value: T) => boolean, options?: RevisionedHostWaitOptions): Promise<T>;
  dispose(): void;
}
interface RevisionedHostWaitOptions {
  timeout?: number;
  signal?: AbortSignal;
  /**
   * If no pushed value matches before `timeout`, perform one coalesced full
   * refresh and test the refreshed value before reporting the timeout.
   *
   * This is useful after host commands: pushes keep the common path cheap,
   * while the refresh closes over dropped/coalesced notification races.
   */
  refreshOnTimeout?: boolean;
}
type RevisionedHostWaitErrorReason = "timeout" | "aborted" | "disposed";
declare class RevisionedHostWaitError extends Error {
  readonly reason: RevisionedHostWaitErrorReason;
  constructor(reason: RevisionedHostWaitErrorReason, message: string);
}
/**
 * Keep a Solid value synchronized with a host-owned revisioned snapshot.
 *
 * A revision identifies the exact snapshot contents. After the first host
 * value, producers must increase it whenever those contents can change;
 * another payload with the same revision is treated as a duplicate.
 *
 * The initial RPC closes the subscription race by ignoring results older than
 * an already received host push. A patch whose base revision no longer
 * matches automatically falls back to one coalesced full refresh.
 */
declare function createRevisionedHostResource<T extends RevisionedHostValue, P extends RevisionedHostPatch = RevisionedHostPatch>(options: RevisionedHostResourceOptions<T, P>): RevisionedHostResource<T>;
//#endregion
//#region src/glue/native-capability.d.ts
interface NativeCapability {
  readonly __wabouCapabilityVersion: number;
}
interface CapabilityClientOptions {
  name: string;
  version: number;
}
declare class CapabilityError extends Error {
  readonly code: string;
  constructor(message: string, code?: string);
}
/** Validate and expose one versioned native capability namespace. */
declare function bindCapability<Capability extends NativeCapability>(capability: Capability | undefined, options: CapabilityClientOptions): Capability;
//#endregion
//#region src/glue/json-capability.d.ts
type JsonCapabilityMethodName<Capability> = Extract<{ [Key in keyof Capability]: Capability[Key] extends ((...args: never[]) => unknown) ? Key : never; }[keyof Capability], string>;
type JsonCapabilityClient = <Response>(method: string, request?: unknown) => Promise<Response>;
/** Bind Wabou's versioned JSON capability transport to a typed app wrapper. */
declare function bindJsonCapability<Capability extends NativeCapability>(capability: Capability | undefined, options: CapabilityClientOptions): JsonCapabilityClient;
//#endregion
//#region src/glue/kv.d.ts
/** JSON-compatible values stored by the built-in KV service. */
type KvValue = null | boolean | number | string | readonly KvValue[] | {
  readonly [key: string]: KvValue;
};
/** One component of a hierarchical key. */
type KvKeyPart = string | number | boolean | Uint8Array;
/** Hierarchical key whose array boundaries define namespaces. */
type KvKey = readonly KvKeyPart[];
/** Decimal revision assigned by SQLite to an atomic commit. */
type KvVersionstamp = string;
interface KvEntry<T extends KvValue = KvValue> {
  readonly key: KvKey;
  readonly value: T;
  readonly versionstamp: KvVersionstamp;
  readonly expiresAt?: number;
}
interface KvSetOptions {
  /** Remove the entry after this many milliseconds. */
  expireIn?: number;
}
interface KvListOptions {
  prefix?: KvKey;
  limit?: number;
  reverse?: boolean;
}
interface KvCheck {
  key: KvKey;
  /** `null` requires the key to be absent. */
  versionstamp: KvVersionstamp | null;
}
interface KvCommitResult {
  readonly committed: boolean;
  readonly versionstamp?: KvVersionstamp;
}
type WireKeyPart = {
  type: "string";
  value: string;
} | {
  type: "i64";
  value: string;
} | {
  type: "bytes";
  value: number[];
} | {
  type: "bool";
  value: boolean;
};
interface WireEntry {
  key: WireKeyPart[];
  value: KvValue;
  versionstamp: string;
  expiresAt: number | null;
}
interface NativeKvCapability extends NativeCapability {
  get(request: {
    key: WireKeyPart[];
  }): Promise<WireEntry | null>;
  set(request: {
    key: WireKeyPart[];
    value: KvValue;
    expireIn?: number;
  }): Promise<{
    versionstamp: string;
  }>;
  delete(request: {
    key: WireKeyPart[];
  }): Promise<{
    versionstamp: string;
  }>;
  list(request: {
    prefix: WireKeyPart[];
    limit: number;
    reverse: boolean;
  }): Promise<WireEntry[]>;
  atomic(request: {
    checks: {
      key: WireKeyPart[];
      versionstamp: string | null;
    }[];
    mutations: ({
      type: "set";
      key: WireKeyPart[];
      value: KvValue;
      expireIn?: number;
    } | {
      type: "mergePatch";
      key: WireKeyPart[];
      patch: KvValue;
    } | {
      type: "delete";
      key: WireKeyPart[];
    })[];
  }): Promise<{
    committed: boolean;
    versionstamp: string | null;
  }>;
}
/** Fluent optimistic transaction committed as one SQLite transaction. */
declare class KvAtomicOperation {
  #private;
  constructor(prefix: KvKey, native: NativeKvCapability);
  check(check: KvCheck | Pick<KvEntry, "key" | "versionstamp">): this;
  set(key: KvKey, value: KvValue, options?: KvSetOptions): this;
  /** Apply an RFC 7396 JSON Merge Patch inside this transaction. */
  mergePatch(key: KvKey, patch: KvValue): this;
  delete(key: KvKey): this;
  commit(): Promise<KvCommitResult>;
}
/** Application-scoped view of the built-in SQLite KV service. */
interface Kv {
  get<T extends KvValue = KvValue>(key: KvKey): Promise<KvEntry<T> | null>;
  set(key: KvKey, value: KvValue, options?: KvSetOptions): Promise<KvVersionstamp>;
  /**
   * Apply an RFC 7396 JSON Merge Patch without transferring the current
   * document through JavaScript. Object fields set to `null` are removed.
   */
  mergePatch(key: KvKey, patch: KvValue): Promise<KvVersionstamp>;
  delete(key: KvKey): Promise<KvVersionstamp>;
  list<T extends KvValue = KvValue>(options?: KvListOptions): AsyncIterable<KvEntry<T>>;
  atomic(): KvAtomicOperation;
}
interface KvSignal<T extends KvValue> {
  /** Current local value; available immediately. */
  readonly value: Accessor<T>;
  /** Whether the initial durable read has settled. */
  readonly ready: Accessor<boolean>;
  /** Most recent load or write failure. */
  readonly error: Accessor<unknown>;
  /** Update locally and schedule persistence. */
  set(next: T | ((previous: T) => T)): void;
  /** Reload without overwriting a newer local edit. */
  reload(): Promise<void>;
  /** Immediately persist the latest pending value. */
  flush(): Promise<void>;
}
/**
 * Bind one explicit KV key to Solid state.
 *
 * The key is deliberately required: source location, signal creation order,
 * and variable names are not stable persistence identities across HMR or
 * refactors.
 */
declare function createKvSignal<T extends KvValue>(options: {
  kv: Kv;
  key: KvKey;
  initial: T;
  saveDelayMs?: number;
}): KvSignal<T>;
/**
 * Open a namespaced view of the host's SQLite store.
 *
 * The host must opt in with `HostBuilder::kv()` and configure stable app
 * directories. Prefixes are prepended by whole key parts, never string joined.
 */
declare function openKv(prefix?: KvKey): Kv;
//#endregion
//#region src/glue/latest-async-resource.d.ts
interface LatestAsyncResourceOptions<K, T> {
  source: Accessor<K | undefined>;
  load: (key: K, context: {
    signal: AbortSignal;
  }) => T | PromiseLike<T>;
  initialValue?: T;
  retainPrevious?: boolean;
  autoLoad?: boolean;
  /** Runs synchronously before a latest load or local mutation is published. */
  onCommit?: (value: T) => void;
}
type LatestAsyncResourceStatus = "idle" | "pending" | "ready" | "error";
interface LatestAsyncResource<T> {
  value: Accessor<T | undefined>;
  loading: Accessor<boolean>;
  error: Accessor<unknown | undefined>;
  status: Accessor<LatestAsyncResourceStatus>;
  /**
   * Start a latest-wins load for the current source.
   *
   * Load failures are represented by `status() === "error"` and `error()`;
   * they do not reject this promise. `undefined` means the load failed, was
   * superseded/aborted, or there is no active source.
   */
  refresh(): Promise<T | undefined>;
  mutate(value: T): void;
  dispose(): void;
}
/**
 * Load the latest reactive key while exposing ordinary, non-suspending state.
 * Older requests are aborted when possible and can never overwrite newer data.
 */
declare function createLatestAsyncResource<K, T>(options: LatestAsyncResourceOptions<K, T>): LatestAsyncResource<T>;
//#endregion
//#region src/glue/native-menu.d.ts
type NativeMenuItem = {
  kind: "item";
  id: string;
  label: string;
  enabled?: boolean;
  checked?: boolean;
} | {
  kind: "separator";
} | {
  kind: "submenu";
  label: string;
  items: NativeMenuItem[];
};
interface NativeMenuPosition {
  readonly x: number;
  readonly y: number;
}
interface NativeMenuOptions {
  readonly windowId?: WindowKey;
  readonly position?: NativeMenuPosition;
  readonly items: readonly NativeMenuItem[];
}
/** Show a platform context menu and resolve with the selected item id. */
declare function showNativeMenu(options: NativeMenuOptions): Promise<string>;
//#endregion
//#region src/glue/platform-context.d.ts
interface PlatformServices {
  clipboard: Clipboard;
  dialog: Dialog;
  notification: Notification;
  window: WindowState;
}
interface PlatformProviderProps {
  value: Partial<PlatformServices>;
  children?: JSX.Element;
}
/** Override native services for one Solid subtree, primarily for tests and previews. */
declare function PlatformProvider(props: PlatformProviderProps): JSX.Element;
//#endregion
//#region src/keyed-list.d.ts
interface KeyedListPatch<T, Key> {
  readonly upserted: readonly T[];
  readonly removed: readonly Key[];
  readonly order: readonly Key[];
}
/**
 * Reconcile a host-owned keyed list while validating its complete order.
 * Returns `undefined` for duplicate, missing, or unaccounted-for keys so the
 * caller can request a full snapshot instead of accepting divergent state.
 */
declare function reconcileKeyedList<T, Key>(current: readonly T[], patch: KeyedListPatch<T, Key>, keyOf: (value: T) => Key): T[] | undefined;
//#endregion
export { Affine2D, type AppDirectories, type AppLifecycleEvent, type AppLifecycleState, type Application, type AsyncAction, AsyncActionConflictError, type AsyncActionResult, type AsyncQuery, type AsyncQueryOptions, type BuiltinHost, type CalendarDateFields, type CapabilityClientOptions, CapabilityError, ClassValue, type Clipboard, type ColorPalette, type ColorThemeAnimation, type ColorThemeAnimationOptions, type ColorThemeController, type ColorThemeEasing, ColorThemeProvider, type CreateWindowOptions, type DebugOverlayOptions, type DebugOverlayPaintStats, type Dialog, type DialogFilter, Dynamic, type DynamicProps, type EntityKey, type EventEffectOptions, type FileDropEvent, type FileDropHandler, type FileDropPhase, type FileDropPosition, ForEntity, type ForEntityProps, type FrameStats, type GestureEvent, type GestureHandler, type GesturePhase, type Handle, type Host, HostCapabilities, type HostJsonSubscriptionOptions, type HostMessage, type HostMessageAllHandler, type HostMessageHandler, HostProvider, type HostProviderProps, INLINE_STYLE_CONTRACT, type JsonCapabilityClient, type JsonCapabilityMethodName, type KeyboardModifiers, type KeyedAsyncAction, KeyedListPatch, type Kv, KvAtomicOperation, type KvCheck, type KvCommitResult, type KvEntry, type KvKey, type KvKeyPart, type KvListOptions, type KvSetOptions, type KvSignal, type KvValue, type KvVersionstamp, type LatestAsyncResource, type LatestAsyncResourceOptions, type LatestAsyncResourceStatus, type LayoutNodeMetrics, type LayoutRect, type LayoutScrollMetrics, type LayoutSnapshot, type LayoutTarget, type MessageDialogButtons, type MessageDialogLevel, type MessageDialogOptions, type MessageDialogResult, type NativeCapability, type NativeMenuItem, type NativeMenuOptions, type NativeMenuPosition, type Notification, type NotificationOptions, type OpenDialogOptions, PathBuilder, PathFillRule, PathLineCap, PathLineJoin, PathPoint, type PickDirectoryOptions, PlatformProvider, type PlatformProviderProps, type PlatformServices, Portal, type PortalProps, type RevisionedHostPatch, type RevisionedHostResource, type RevisionedHostResourceOptions, type RevisionedHostValue, RevisionedHostWaitError, type RevisionedHostWaitErrorReason, type RevisionedHostWaitOptions, STYLE_VALUE, type SaveDialogOptions, Shadow, ShadowOptions, StyleValueKind, TypedStyleValue, VectorPath, VectorPathPaint, VirtualList, type VirtualListProps, WabouBaseUtility, type WabouBuiltinIntrinsicElements, WabouColorToken, type WabouControlProps, WabouDynamicUtility, type WabouElementProps, type WabouEventTarget, type WabouExposedSemanticRole, type WabouFloatingAnchor, type WabouFloatingPlacement, type WabouFloatingPosition, type WabouGlobalPointerEventType, type WabouGlobalPointerListener, type WabouImageProps, type WabouImeDeleteSurroundingEvent, type WabouImePreeditEvent, type WabouInputEvent, type WabouInputProps, WabouIntrinsicElements, type WabouKeyEvent, type WabouNativeElements, type WabouNativeTag, type WabouNativeTransition, type WabouNodeEvent, type WabouPointerEvent, type WabouPositionedEvent, type WabouScrollEvent, type WabouSemanticRole, WabouSpacingToken, WabouStaticUtility, type WabouStyle, type WabouSubmitEvent, type WabouSvgProps, type WabouSvgShapeProps, type WabouTextCommitEvent, type WabouTextSelectionChangeEvent, type WabouTransitionEvent, WabouUtility, type WabouValueChangeEvent, type WabouVectorPathProps, type WabouWheelEvent, type WindowHandle, type WindowKey, type WindowMetrics, type WindowSizeQuery, type WindowState, appDirs, application, assertInlineStyleValue, auto, bindCapability, bindJsonCapability, bool, classes, clipboard, colorTheme, createAsyncAction, createAsyncQuery, createEventEffect, createFps, createKeyedAsyncAction, createKvSignal, createLatestAsyncResource, createRevisionedHostResource, createWindow, createWindowMatch, currentWindow, currentWindowOptions, defaultHost, dialog, hostMessages, intl, isDirectEvent, isTypedStyleValue, isVectorPath, mergeClasses, mount, notification, number, observeGlobalPointerEvent, openKv, percent, px, reconcileKeyedList, rgba, rotate2d, scale2d, setTransform2D, shadow, showNativeMenu, subscribeAll as subscribeAllHostMessages, subscribeAppLifecycle, subscribeFileDrop, subscribeGesture, subscribe as subscribeHostMessages, subscribeJson as subscribeJsonHostMessages, subscribeKeyboardModifiers, translate2d, useAppLifecycle, useClipboard, useColorTheme, useDialog, useFileDrop, useGesture, useHost, useKeyboardModifierChanges, useKeyboardModifiers, useNotification, useWindow, utilityConflictProperties, validateEntityKeys };
//# sourceMappingURL=index.d.mts.map