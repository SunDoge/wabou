import { T as ResourceKey, d as INTERACTION_POLICY, f as OP, h as Writer, m as TEXT_BEHAVIOR, s as GRAPHIC_SOURCE, t as EVENT_CODE } from "./protocol-DYWDcy_c.mjs";
import { n as WabouIntrinsicElements, t as HostCapabilities } from "./registry-DXOPfC3L.mjs";
import { C as WabouSpacingToken, D as WabouStyle, E as INLINE_STYLE_CONTRACT, S as WabouDynamicUtility, T as WabouUtility, _ as scale2d, a as StyleValueKind, b as WabouBaseUtility, c as auto, d as isTypedStyleValue, f as number, g as rotate2d, h as rgba, i as ShadowOptions, l as bool, m as px, n as STYLE_VALUE, o as TypedStyleValue, p as percent, r as Shadow, s as assertInlineStyleValue, t as Affine2D, u as classes, v as shadow, w as WabouStaticUtility, x as WabouColorToken, y as translate2d } from "./style-B1vJn3ZY.mjs";
import { $ as spread, A as createElement, At as isVectorPath, B as isServer, C as WabouSvgProps, Ct as PathBuilder, D as acquireOverlayRoot, Dt as PathPoint, E as WabouWheelEvent, Et as PathLineJoin, F as getMountRoot, G as ref, H as mergeProps, I as getRequestEvent, J as removeNode, K as registerRoot, L as insert, M as delegateEvents, N as dispatchEvent, O as applyRef, Ot as VectorPath, P as effect, Q as setTransform2D, R as insertNode, S as WabouSemanticRole, T as WabouVectorPathProps, Tt as PathLineCap, U as mount, V as memo, W as observeGlobalPointerEvent, X as runSweep, Y as render, Z as setProp, _ as WabouNativeTag, _t as LayoutSnapshot, a as WabouBuiltinIntrinsicElements, at as BuiltinHost, b as WabouPositionedEvent, c as WabouEventTarget, ct as HostProvider, d as WabouGlobalPointerListener, dt as defaultHost, et as writer, f as WabouImageProps, ft as useHost, g as WabouNativeElements, gt as LayoutScrollMetrics, h as WabouKeyEvent, ht as LayoutRect, i as NativeScrollbarStyle, it as PortalProps, j as createTextNode, k as createComponent, kt as VectorPathPaint, l as WabouExposedSemanticRole, lt as HostProviderProps, m as WabouInputProps, mt as LayoutNodeMetrics, n as DynamicProps, nt as VirtualListProps, o as WabouControlProps, ot as DebugOverlayOptions, p as WabouInputEvent, pt as FrameStats, q as releaseOverlayRoot, r as Handle, rt as Portal, s as WabouElementProps, st as Host, t as Dynamic, tt as VirtualList, u as WabouGlobalPointerEventType, ut as LayoutTarget, v as WabouNodeEvent, w as WabouSvgShapeProps, wt as PathFillRule, x as WabouScrollEvent, y as WabouPointerEvent, yt as JSX, z as isDirectEvent } from "./index-DlTMZ_Ut.mjs";
import { t as createFps } from "./renderer-BRejhoNg.mjs";
import { Accessor, JSX as JSX$1 } from "solid-js";
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
  };
  function __wabou_apply_hmr(path: string, acceptedPath: string, timestamp: number): Promise<boolean>;
  function __wabou_hmr_clear_records(): void;
  function __wabou_effect_complete(requestId: number, capability: number, method: number, status: number, payloadJson: string): void;
}
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
  /** Preserve rendered alpha when the platform compositor supports it. */
  transparent?: boolean;
}
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
  colorScheme: "light" | "dark" | null;
}
interface WindowState extends WindowHandle {
  metrics: Accessor<WindowMetrics>;
  width: Accessor<number>;
  height: Accessor<number>;
  scaleFactor: Accessor<number>;
  maximized: Accessor<boolean>;
  focused: Accessor<boolean>;
  colorScheme: Accessor<"light" | "dark">;
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
//#region src/glue/file-drop.d.ts
type FileDropPhase = "entered" | "moved" | "left" | "dropped";
interface FileDropPosition {
  x: number;
  y: number;
}
/** One native path event reported by the window system. */
interface FileDropEvent {
  phase: FileDropPhase;
  /** Native filesystem paths supplied on enter and drop events. */
  paths: string[];
  /** Logical window coordinates, or `null` when unavailable on the platform. */
  position: FileDropPosition | null;
}
type FileDropHandler = (event: FileDropEvent) => void;
/** Subscribe to native file drag-and-drop events for the current window. */
declare function subscribeFileDrop(handler: FileDropHandler): () => void;
/**
 * Subscribe for the lifetime of the current Solid owner.
 * Use `subscribeFileDrop` when no Solid owner is active.
 */
declare function useFileDrop(handler: FileDropHandler): void;
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
/** Resolve all app-private roots once and reuse the same native result. */
declare function resolveAppDirectories(): Promise<AppDirectories>;
declare const appDirs: Readonly<{
  resolve: typeof resolveAppDirectories;
  config: () => Promise<string>;
  data: () => Promise<string>;
  localData: () => Promise<string>;
  cache: () => Promise<string>;
  log: () => Promise<string>;
  resource: () => Promise<string>;
  temp: () => Promise<string>;
}>;
declare const appConfigDir: () => Promise<string>;
declare const appDataDir: () => Promise<string>;
declare const appLocalDataDir: () => Promise<string>;
declare const appCacheDir: () => Promise<string>;
declare const appLogDir: () => Promise<string>;
declare const resourceDir: () => Promise<string>;
declare const tempDir: () => Promise<string>;
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
  children: JSX$1.Element;
}): JSX$1.Element;
declare function useColorTheme(): ColorThemeController;
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
//#region src/glue/json-capability.d.ts
interface NativeJsonCapability {
  readonly __wabouCapabilityVersion: number;
}
type JsonCapabilityMethodName<Capability> = Extract<{ [Key in keyof Capability]: Capability[Key] extends ((...args: never[]) => unknown) ? Key : never; }[keyof Capability], string>;
interface JsonCapabilityClientOptions {
  name: string;
  version: number;
}
declare class JsonCapabilityError extends Error {
  readonly code?: string;
  constructor(message: string, code?: string);
}
type JsonCapabilityClient = <Response>(method: string, request?: unknown) => Promise<Response>;
/** Bind Wabou's versioned JSON capability transport to a typed app wrapper. */
declare function bindJsonCapability<Capability extends NativeJsonCapability>(capability: Capability | undefined, options: JsonCapabilityClientOptions): JsonCapabilityClient;
//#endregion
//#region src/glue/latest-async-resource.d.ts
interface LatestAsyncResourceOptions<K, T> {
  source: Accessor<K | undefined>;
  load: (key: K, context: {
    signal: AbortSignal;
  }) => Promise<T>;
  initialValue?: T;
  retainPrevious?: boolean;
  autoLoad?: boolean;
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
  children?: JSX$1.Element;
}
/** Override native services for one Solid subtree, primarily for tests and previews. */
declare function PlatformProvider(props: PlatformProviderProps): JSX$1.Element;
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
export { Affine2D, type AppDirectories, type Application, type AsyncAction, AsyncActionConflictError, type AsyncActionResult, type BuiltinHost, type CalendarDateFields, type Clipboard, type ColorPalette, type ColorThemeAnimation, type ColorThemeAnimationOptions, type ColorThemeController, type ColorThemeEasing, ColorThemeProvider, type CreateWindowOptions, type DebugOverlayOptions, type Dialog, type DialogFilter, Dynamic, DynamicProps, EVENT_CODE, type EventEffectOptions, type FileDropEvent, type FileDropHandler, type FileDropPhase, type FileDropPosition, type FrameStats, GRAPHIC_SOURCE, Handle, type Host, HostCapabilities, type HostJsonSubscriptionOptions, type HostMessage, type HostMessageAllHandler, type HostMessageHandler, HostProvider, type HostProviderProps, INLINE_STYLE_CONTRACT, INTERACTION_POLICY, type JSX, type JsonCapabilityClient, type JsonCapabilityClientOptions, JsonCapabilityError, type JsonCapabilityMethodName, type KeyedAsyncAction, KeyedListPatch, type LatestAsyncResource, type LatestAsyncResourceOptions, type LatestAsyncResourceStatus, type LayoutNodeMetrics, type LayoutRect, type LayoutScrollMetrics, type LayoutSnapshot, type LayoutTarget, type MessageDialogButtons, type MessageDialogLevel, type MessageDialogOptions, type MessageDialogResult, type NativeJsonCapability, type NativeMenuItem, type NativeMenuOptions, type NativeMenuPosition, NativeScrollbarStyle, type Notification, type NotificationOptions, OP, type OpenDialogOptions, PathBuilder, PathFillRule, PathLineCap, PathLineJoin, PathPoint, type PickDirectoryOptions, PlatformProvider, type PlatformProviderProps, type PlatformServices, Portal, type PortalProps, type RevisionedHostPatch, type RevisionedHostResource, type RevisionedHostResourceOptions, type RevisionedHostValue, RevisionedHostWaitError, type RevisionedHostWaitErrorReason, type RevisionedHostWaitOptions, STYLE_VALUE, type SaveDialogOptions, Shadow, ShadowOptions, StyleValueKind, TEXT_BEHAVIOR, TypedStyleValue, VectorPath, VectorPathPaint, VirtualList, type VirtualListProps, WabouBaseUtility, WabouBuiltinIntrinsicElements, WabouColorToken, WabouControlProps, WabouDynamicUtility, WabouElementProps, WabouEventTarget, WabouExposedSemanticRole, WabouGlobalPointerEventType, WabouGlobalPointerListener, WabouImageProps, WabouInputEvent, WabouInputProps, WabouIntrinsicElements, WabouKeyEvent, WabouNativeElements, WabouNativeTag, WabouNodeEvent, WabouPointerEvent, WabouPositionedEvent, WabouScrollEvent, WabouSemanticRole, WabouSpacingToken, WabouStaticUtility, type WabouStyle, WabouSvgProps, WabouSvgShapeProps, WabouUtility, WabouVectorPathProps, WabouWheelEvent, type WindowHandle, type WindowKey, type WindowMetrics, type WindowSizeQuery, type WindowState, type Writer, acquireOverlayRoot, appCacheDir, appConfigDir, appDataDir, appDirs, appLocalDataDir, appLogDir, application, applyRef, assertInlineStyleValue, auto, bindJsonCapability, bool, classes, clipboard, colorTheme, createAsyncAction, createComponent, createElement, createEventEffect, createFps, createKeyedAsyncAction, createLatestAsyncResource, createRevisionedHostResource, createTextNode, createWindow, createWindowMatch, currentWindow, defaultHost, delegateEvents, dialog, dispatchEvent, effect, getMountRoot, getRequestEvent, hostMessages, insert, insertNode, intl, isDirectEvent, isServer, isTypedStyleValue, isVectorPath, memo, mergeProps, mount, notification, number, observeGlobalPointerEvent, percent, px, reconcileKeyedList, ref, registerRoot, releaseOverlayRoot, removeNode, render, resolveAppDirectories, resourceDir, rgba, rotate2d, runSweep, scale2d, setProp, setTransform2D, shadow, showNativeMenu, spread, subscribeAll as subscribeAllHostMessages, subscribeFileDrop, subscribe as subscribeHostMessages, subscribeJson as subscribeJsonHostMessages, tempDir, translate2d, useClipboard, useColorTheme, useDialog, useFileDrop, useHost, useNotification, useWindow, writer };
//# sourceMappingURL=index.d.mts.map