import { T as ResourceKey, d as INTERACTION_POLICY, f as OP, h as Writer, m as TEXT_BEHAVIOR, s as GRAPHIC_SOURCE, t as EVENT_CODE } from "./protocol-Cb5z35fp.mjs";
import { n as WabouIntrinsicElements, t as HostCapabilities } from "./registry-DXOPfC3L.mjs";
import { $ as Portal, A as delegateEvents, B as mount, C as WabouVectorPathProps, D as createComponent, E as applyRef, F as insert, G as render, H as registerRoot, I as insertNode, J as setTransform2D, K as runSweep, L as isServer, M as effect, N as getMountRoot, O as createElement, P as getRequestEvent, Q as VirtualListProps, R as memo, S as WabouSvgShapeProps, St as isVectorPath, T as acquireOverlayRoot, U as releaseOverlayRoot, V as ref, W as removeNode, X as writer, Y as spread, Z as VirtualList, _ as WabouPointerEvent, _t as PathFillRule, a as WabouBuiltinIntrinsicElements, at as defaultHost, b as WabouSemanticRole, bt as VectorPath, c as WabouEventTarget, ct as LayoutNodeMetrics, d as WabouInputEvent, et as PortalProps, f as WabouInputProps, ft as JSX, g as WabouNodeEvent, gt as PathBuilder, h as WabouNativeTag, i as NativeScrollbarStyle, it as LayoutTarget, j as dispatchEvent, k as createTextNode, l as WabouExposedSemanticRole, lt as LayoutRect, m as WabouNativeElements, n as DynamicProps, nt as HostProvider, o as WabouControlProps, ot as useHost, p as WabouKeyEvent, q as setProp, r as Handle, rt as HostProviderProps, s as WabouElementProps, st as FrameStats, t as Dynamic, tt as Host, u as WabouImageProps, ut as LayoutSnapshot, v as WabouPositionedEvent, vt as PathLineCap, w as WabouWheelEvent, x as WabouSvgProps, xt as VectorPathPaint, y as WabouScrollEvent, yt as PathLineJoin, z as mergeProps } from "./index-CClWq0PS.mjs";
import { C as WabouSpacingToken, D as WabouStyle, E as INLINE_STYLE_CONTRACT, S as WabouDynamicUtility, T as WabouUtility, _ as scale2d, a as StyleValueKind, b as WabouBaseUtility, c as auto, d as isTypedStyleValue, f as number, g as rotate2d, h as rgba, i as ShadowOptions, l as bool, m as px, n as STYLE_VALUE, o as TypedStyleValue, p as percent, r as Shadow, s as assertInlineStyleValue, t as Affine2D, u as classes, v as shadow, w as WabouStaticUtility, x as WabouColorToken, y as translate2d } from "./style-COVvh6aZ.mjs";
import { t as createFps } from "./renderer-D61Y7y7C.mjs";
import { Accessor, JSX as JSX$1 } from "solid-js";
//#region src/generated/host-abi.d.ts
declare global {
  const __wabou_capabilities: Record<string, object>;
  function __wabou_intern(value: string): number;
  function __wabou_open_url(url: string): boolean;
  function __wabou_set_stylesheet(json: string): void;
  function __wabou_set_color_theme(name: string): void;
  function __wabou_get_color_theme_palette(name: string): string;
  function __wabou_set_color_palette(colors: Uint32Array): void;
  function __wabou_load_font(path: string): boolean;
  function __wabou_frame_stats(): string;
  function __wabou_layout_snapshot(ids: Uint32Array): string;
  function __wabou_system_locale(): string;
  function __wabou_system_time_zone(): string;
  function __wabou_system_calendar_date(): string;
  function __wabou_flush(buf: Uint8Array): void;
  function __wabou_log(level: "debug" | "info" | "warn" | "error" | "log", message: string): void;
  function __wabou_utf8_encode(value: string): Uint8Array;
  function __wabou_utf8_decode(bytes: Uint8Array): string;
  function __wabou_fetch(url: string, initJson: string): Promise<string>;
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
/**
 * Subscribe to host messages on `topic`.
 * Returns an unsubscribe function.
 */
declare function subscribe(topic: string, handler: HostMessageHandler): () => void;
/** Subscribe to every topic; handler receives `(topic, payload)`. */
declare function subscribeAll(handler: HostMessageAllHandler): () => void;
declare const hostMessages: {
  subscribe: typeof subscribe;
  subscribeAll: typeof subscribeAll;
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
}
interface WindowState extends WindowHandle {
  metrics: Accessor<WindowMetrics>;
  width: Accessor<number>;
  height: Accessor<number>;
  scaleFactor: Accessor<number>;
  maximized: Accessor<boolean>;
  focused: Accessor<boolean>;
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
export { Affine2D, type AppDirectories, type Application, type CalendarDateFields, type Clipboard, type ColorPalette, type ColorThemeAnimation, type ColorThemeAnimationOptions, type ColorThemeController, type ColorThemeEasing, ColorThemeProvider, type CreateWindowOptions, type Dialog, type DialogFilter, Dynamic, DynamicProps, EVENT_CODE, type FileDropEvent, type FileDropHandler, type FileDropPhase, type FileDropPosition, type FrameStats, GRAPHIC_SOURCE, Handle, type Host, HostCapabilities, type HostMessage, type HostMessageAllHandler, type HostMessageHandler, HostProvider, type HostProviderProps, INLINE_STYLE_CONTRACT, INTERACTION_POLICY, type JSX, type LayoutNodeMetrics, type LayoutRect, type LayoutSnapshot, type LayoutTarget, type MessageDialogButtons, type MessageDialogLevel, type MessageDialogOptions, type MessageDialogResult, type NativeMenuItem, type NativeMenuOptions, type NativeMenuPosition, NativeScrollbarStyle, type Notification, type NotificationOptions, OP, type OpenDialogOptions, PathBuilder, PathFillRule, PathLineCap, PathLineJoin, type PickDirectoryOptions, PlatformProvider, type PlatformProviderProps, type PlatformServices, Portal, type PortalProps, STYLE_VALUE, type SaveDialogOptions, Shadow, ShadowOptions, StyleValueKind, TEXT_BEHAVIOR, TypedStyleValue, VectorPath, VectorPathPaint, VirtualList, type VirtualListProps, WabouBaseUtility, WabouBuiltinIntrinsicElements, WabouColorToken, WabouControlProps, WabouDynamicUtility, WabouElementProps, WabouEventTarget, WabouExposedSemanticRole, WabouImageProps, WabouInputEvent, WabouInputProps, WabouIntrinsicElements, WabouKeyEvent, WabouNativeElements, WabouNativeTag, WabouNodeEvent, WabouPointerEvent, WabouPositionedEvent, WabouScrollEvent, WabouSemanticRole, WabouSpacingToken, WabouStaticUtility, type WabouStyle, WabouSvgProps, WabouSvgShapeProps, WabouUtility, WabouVectorPathProps, WabouWheelEvent, type WindowHandle, type WindowKey, type WindowMetrics, type WindowSizeQuery, type WindowState, type Writer, acquireOverlayRoot, appCacheDir, appConfigDir, appDataDir, appDirs, appLocalDataDir, appLogDir, application, applyRef, assertInlineStyleValue, auto, bool, classes, clipboard, colorTheme, createComponent, createElement, createFps, createTextNode, createWindow, createWindowMatch, currentWindow, defaultHost, delegateEvents, dialog, dispatchEvent, effect, getMountRoot, getRequestEvent, hostMessages, insert, insertNode, intl, isServer, isTypedStyleValue, isVectorPath, memo, mergeProps, mount, notification, number, percent, px, ref, registerRoot, releaseOverlayRoot, removeNode, render, resolveAppDirectories, resourceDir, rgba, rotate2d, runSweep, scale2d, setProp, setTransform2D, shadow, showNativeMenu, spread, subscribeAll as subscribeAllHostMessages, subscribeFileDrop, subscribe as subscribeHostMessages, tempDir, translate2d, useClipboard, useColorTheme, useDialog, useFileDrop, useHost, useNotification, useWindow, writer };
//# sourceMappingURL=index.d.mts.map