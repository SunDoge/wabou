import { dispatchEffect, effectOps } from "./effects";
import { usePlatformServices } from "./platform-context";

export interface DialogFilter {
  readonly name: string;
  readonly extensions: readonly string[];
}

export interface OpenDialogOptions {
  readonly title?: string;
  readonly directory?: string;
  readonly filters?: readonly DialogFilter[];
  readonly multiple?: boolean;
}

export interface SaveDialogOptions {
  readonly title?: string;
  readonly directory?: string;
  readonly defaultName?: string;
  readonly filters?: readonly DialogFilter[];
}

export interface PickDirectoryOptions {
  readonly title?: string;
  readonly directory?: string;
}

export type MessageDialogLevel = "info" | "warning" | "error";
export type MessageDialogButtons = "ok" | "okCancel" | "yesNo" | "yesNoCancel";
export type MessageDialogResult = "ok" | "cancel" | "yes" | "no" | "custom";

export interface MessageDialogOptions {
  readonly title?: string;
  readonly message: string;
  readonly level?: MessageDialogLevel;
  readonly buttons?: MessageDialogButtons;
}

export interface Dialog {
  open(options?: OpenDialogOptions): Promise<readonly string[] | null>;
  save(options?: SaveDialogOptions): Promise<string | null>;
  pickDirectory(options?: PickDirectoryOptions): Promise<string | null>;
  message(options: MessageDialogOptions): Promise<MessageDialogResult>;
}

function normalizeFilters(
  filters: readonly DialogFilter[] | undefined,
): DialogFilter[] {
  return (filters ?? []).map((filter) => ({
    name: String(filter.name),
    extensions: filter.extensions
      .map((extension) => String(extension).replace(/^\./, ""))
      .filter(Boolean),
  }));
}

export const dialog: Dialog = Object.freeze({
  open(options: OpenDialogOptions = {}) {
    return dispatchEffect<string[] | null>(effectOps.dialogOpen, {
      ...options,
      filters: normalizeFilters(options.filters),
      multiple: options.multiple ?? false,
    });
  },
  save(options: SaveDialogOptions = {}) {
    return dispatchEffect<string[] | null>(effectOps.dialogSave, {
      ...options,
      filters: normalizeFilters(options.filters),
    }).then((paths) => paths?.[0] ?? null);
  },
  pickDirectory(options: PickDirectoryOptions = {}) {
    return dispatchEffect<string[] | null>(
      effectOps.dialogPickDirectory,
      options,
    ).then((paths) => paths?.[0] ?? null);
  },
  message(options: MessageDialogOptions) {
    return dispatchEffect<MessageDialogResult>(effectOps.dialogMessage, {
      ...options,
      message: String(options.message),
      level: options.level ?? "info",
      buttons: options.buttons ?? "ok",
    });
  },
});

export function useDialog(): Dialog {
  return usePlatformServices().dialog ?? dialog;
}
