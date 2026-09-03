import { mergeClasses } from "@wabou/core/style";

export const workbenchClass = (className?: string) =>
  mergeClasses(
    "w-full h-full min-w-0 min-h-0 flex flex-row overflow-hidden bg-canvas text-primary",
    className,
  );

export const workbenchSidebarClass = (className?: string) =>
  mergeClasses("w-64 flex-none border-r border-subtle", className);

export const workbenchMainClass = (className?: string) =>
  mergeClasses(
    "min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden bg-canvas",
    className,
  );

export const workbenchHeaderClass = (className?: string) =>
  mergeClasses(
    "w-full h-12 min-w-0 flex-none px-4 flex flex-row items-center gap-3 border-b border-subtle bg-surface",
    className,
  );

export const workbenchContentClass = (className?: string) =>
  mergeClasses(
    "min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden bg-canvas",
    className,
  );

/** Shared readable column for workbench transcripts, editors and composers. */
export const workbenchContentColumnClass = (className?: string) =>
  mergeClasses("w-full max-w-4xl mx-auto min-w-0", className);

export const workbenchFooterClass = (className?: string) =>
  mergeClasses(
    "min-w-0 flex-none border-t border-subtle bg-surface",
    className,
  );

/** Bounded auxiliary pane beside primary workbench content. */
export const workbenchInspectorClass = (className?: string) =>
  mergeClasses(
    "w-96 max-w-full flex-none min-w-0 min-h-0 border-l border-subtle bg-surface flex flex-col overflow-hidden shadow-sm",
    className,
  );

/** Fixed inspector chrome aligned with the inspector body. */
export const workbenchInspectorHeaderClass = (className?: string) =>
  mergeClasses(
    "w-full h-14 min-w-0 flex-none px-4 flex flex-row items-center justify-between gap-3 border-b border-subtle",
    className,
  );

/** Shrink-safe body for inspector content and nested scroll areas. */
export const workbenchInspectorContentClass = (className?: string) =>
  mergeClasses(
    "min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden",
    className,
  );
