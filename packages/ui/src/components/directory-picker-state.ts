import type { PickDirectoryOptions } from "@wabou/core";

export function directoryPickerOptions(
  value: string,
  options: PickDirectoryOptions | undefined,
): PickDirectoryOptions {
  return {
    ...options,
    directory: options?.directory ?? (value.trim() || undefined),
  };
}
