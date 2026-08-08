type PendingClipboardRequest =
  | {
      kind: "read";
      resolve: (text: string | null) => void;
      reject: (error: Error) => void;
    }
  | {
      kind: "write";
      resolve: () => void;
      reject: (error: Error) => void;
    };

export interface Clipboard {
  readText(): Promise<string | null>;
  writeText(text: string): Promise<void>;
}

const pending = new Map<number, PendingClipboardRequest>();

function complete(
  requestId: number,
  text: string | null,
  success: boolean,
): void {
  const request = pending.get(requestId);
  if (!request) return;
  pending.delete(requestId);

  if (!success) {
    request.reject(new Error("Native clipboard operation failed"));
  } else if (request.kind === "read") {
    request.resolve(text);
  } else {
    request.resolve();
  }
}

(
  globalThis as typeof globalThis & {
    __wabou_clipboard_complete: typeof complete;
  }
).__wabou_clipboard_complete = complete;

export const clipboard: Clipboard = Object.freeze({
  readText(): Promise<string | null> {
    return new Promise((resolve, reject) => {
      const requestId = __wabou_clipboard_read();
      pending.set(requestId, { kind: "read", resolve, reject });
    });
  },
  writeText(text: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const requestId = __wabou_clipboard_write(String(text));
      pending.set(requestId, { kind: "write", resolve, reject });
    });
  },
});

/** Stable clipboard capability for use inside Solid components. */
export function useClipboard(): Clipboard {
  return usePlatformServices().clipboard ?? clipboard;
}
import { usePlatformServices } from "./platform-context";
