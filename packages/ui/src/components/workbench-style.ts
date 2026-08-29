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
    "h-12 min-w-0 flex-none px-4 flex flex-row items-center gap-3 border-b border-subtle bg-surface",
    className,
  );

export const workbenchContentClass = (className?: string) =>
  mergeClasses(
    "min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden bg-canvas",
    className,
  );

export const workbenchFooterClass = (className?: string) =>
  mergeClasses(
    "min-w-0 flex-none border-t border-subtle bg-surface",
    className,
  );
