import { dispatchEffect, effectOps } from "./effects";
import { usePlatformServices } from "./platform-context";

export interface Clipboard {
  readText(): Promise<string | null>;
  writeText(text: string): Promise<void>;
}

export const clipboard: Clipboard = Object.freeze({
  readText: () => dispatchEffect<string | null>(effectOps.clipboardRead),
  writeText: (text: string) =>
    dispatchEffect<null>(effectOps.clipboardWrite, { text: String(text) }).then(
      () => undefined,
    ),
});

/** Stable clipboard capability for use inside Solid components. */
export function useClipboard(): Clipboard {
  return usePlatformServices().clipboard ?? clipboard;
}
