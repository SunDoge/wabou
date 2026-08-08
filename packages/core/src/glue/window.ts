export interface CreateWindowOptions {
  title?: string;
  width?: number;
  height?: number;
  minWidth?: number;
  minHeight?: number;
  resizable?: boolean;
  /** Preserve rendered alpha when the platform compositor supports it. */
  transparent?: boolean;
}

export interface WindowHandle {
  readonly id: number;
  close(): void;
  setMaximized(value: boolean): void;
  setTitle(title: string): void;
}

function handle(id: number): WindowHandle {
  const host = globalThis as typeof globalThis & {
    __wabou_window_close(id: number): void;
    __wabou_window_set_maximized(id: number, value: boolean): void;
    __wabou_window_set_title(id: number, title: string): void;
  };
  return Object.freeze({
    id,
    close: () => host.__wabou_window_close(id),
    setMaximized: (value: boolean) =>
      host.__wabou_window_set_maximized(id, value),
    setTitle: (title: string) => host.__wabou_window_set_title(id, title),
  });
}

/** Create an independent native window running this application's bundle. */
export function createWindow(options: CreateWindowOptions = {}): WindowHandle {
  const host = globalThis as typeof globalThis & {
    __wabou_window_create(optionsJson: string): number;
  };
  return handle(host.__wabou_window_create(JSON.stringify(options)));
}

/** An imperative handle for the native window that owns this JS runtime. */
export function currentWindow(): WindowHandle {
  return handle(
    (globalThis as typeof globalThis & { __wabou_window_id: number })
      .__wabou_window_id,
  );
}
