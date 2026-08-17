import { Accessor, JSX } from "solid-js";
export * from "@wabou/solid-renderer";
export * from "@wabou/style";
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
  function __wabou_flush(buf: Uint8Array): void;
  function __wabou_log(level: "debug" | "info" | "warn" | "error" | "log", message: string): void;
  function __wabou_utf8_encode(value: string): Uint8Array;
  function __wabou_utf8_decode(bytes: Uint8Array): string;
  function __wabou_fetch(url: string, initJson: string): Promise<string>;
  function __wabou_sleep(delayMs: number): Promise<void>;
  function __wabou_resize_observe(solidId: number): void;
  function __wabou_resize_unobserve(solidId: number): void;
  const __wabou_effect_abi: number;
  function __wabou_effect_submit(capability: number, method: number, payloadJson: string): number;
  const __wabou_window_id: number;
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
  readonly id: number;
  close(): void;
  minimize(): void;
  setMaximized(value: boolean): void;
  setTitle(title: string): void;
  /** Begin a compositor-managed move operation for a custom title bar. */
  startDragging(): void;
}
/** Create an independent native window running this application's bundle. */
declare function createWindow(options?: CreateWindowOptions): WindowHandle;
/** An imperative handle for the native window that owns this JS runtime. */
declare function currentWindow(): WindowHandle;
//#endregion
//#region src/glue/window-metrics.d.ts
interface WindowMetrics {
  windowId: number;
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
/** Reactive state and controls for the native window owning this JS runtime. */
declare function useWindow(): WindowState;
//#endregion
//#region src/glue/effects.d.ts
declare const EFFECT_ABI_VERSION = 1;
interface EffectOp {
  readonly capability: number;
  readonly method: number;
}
declare const effectOps: Readonly<{
  clipboardRead: {
    capability: number;
    method: number;
  };
  clipboardWrite: {
    capability: number;
    method: number;
  };
  windowCreate: {
    capability: number;
    method: number;
  };
  windowClose: {
    capability: number;
    method: number;
  };
  windowSetMaximized: {
    capability: number;
    method: number;
  };
  windowSetTitle: {
    capability: number;
    method: number;
  };
  windowMinimize: {
    capability: number;
    method: number;
  };
  windowStartDragging: {
    capability: number;
    method: number;
  };
  contextMenuShow: {
    capability: number;
    method: number;
  };
  appDirsResolve: {
    capability: number;
    method: number;
  };
  dialogOpen: {
    capability: number;
    method: number;
  };
  dialogSave: {
    capability: number;
    method: number;
  };
  dialogPickDirectory: {
    capability: number;
    method: number;
  };
  dialogMessage: {
    capability: number;
    method: number;
  };
  notificationShow: {
    capability: number;
    method: number;
  };
}>;
declare function dispatchEffect<T>(op: EffectOp, payload?: unknown): Promise<T>;
/** Submit a command whose resource handle is its effect id. */
declare function dispatchResourceEffect(op: EffectOp, payload?: unknown): number;
/** Submit a command without retaining a Promise or callback. */
declare function dispatchFireAndForget(op: EffectOp, payload?: unknown): void;
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
//#region src/glue/color-theme.d.ts
type ColorThemeEasing = "linear" | "ease-in" | "ease-out" | "ease-in-out" | ((progress: number) => number);
interface ColorThemeAnimationOptions {
  /** Animation duration in seconds, matching @wabou/animation. */
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
  readonly windowId?: number;
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
export { type AppDirectories, type Clipboard, type ColorPalette, type ColorThemeAnimation, type ColorThemeAnimationOptions, type ColorThemeController, type ColorThemeEasing, ColorThemeProvider, type CreateWindowOptions, type Dialog, type DialogFilter, EFFECT_ABI_VERSION, type EffectOp, type HostMessage, type HostMessageAllHandler, type HostMessageHandler, type MessageDialogButtons, type MessageDialogLevel, type MessageDialogOptions, type MessageDialogResult, type NativeMenuItem, type NativeMenuOptions, type NativeMenuPosition, type Notification, type NotificationOptions, type OpenDialogOptions, type PickDirectoryOptions, PlatformProvider, type PlatformProviderProps, type PlatformServices, type SaveDialogOptions, type WindowHandle, type WindowMetrics, type WindowState, appCacheDir, appConfigDir, appDataDir, appDirs, appLocalDataDir, appLogDir, clipboard, colorTheme, createWindow, currentWindow, dialog, dispatchEffect, dispatchFireAndForget, dispatchResourceEffect, effectOps, hostMessages, notification, resolveAppDirectories, resourceDir, showNativeMenu, subscribeAll as subscribeAllHostMessages, subscribe as subscribeHostMessages, tempDir, useClipboard, useColorTheme, useDialog, useNotification, useWindow };
//# sourceMappingURL=index.d.mts.map