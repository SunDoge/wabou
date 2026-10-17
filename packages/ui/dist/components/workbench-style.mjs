import { mergeClasses } from "@wabou/core/style";
//#region src/components/workbench-style.ts
const workbenchClass = (className) => mergeClasses("w-full h-full min-w-0 min-h-0 flex flex-row overflow-hidden bg-canvas text-primary", className);
const workbenchSidebarClass = (className) => mergeClasses("w-64 flex-none border-r border-subtle", className);
const workbenchMainClass = (className) => mergeClasses("min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden bg-canvas", className);
const workbenchHeaderClass = (className) => mergeClasses("w-full h-12 min-w-0 flex-none px-4 flex flex-row items-center gap-3 border-b border-subtle bg-surface", className);
const workbenchContentClass = (className) => mergeClasses("min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden bg-canvas", className);
/** Shared readable column for workbench transcripts, editors and composers. */
const workbenchContentColumnClass = (className) => mergeClasses("w-full max-w-4xl mx-auto min-w-0", className);
const workbenchFooterClass = (className) => mergeClasses("min-w-0 flex-none border-t border-subtle bg-surface", className);
/** Bounded auxiliary pane beside primary workbench content. */
const workbenchInspectorClass = (className) => mergeClasses("w-96 max-w-full flex-none min-w-0 min-h-0 border-l border-subtle bg-surface flex flex-col overflow-hidden shadow-sm", className);
/** Fixed inspector chrome aligned with the inspector body. */
const workbenchInspectorHeaderClass = (className) => mergeClasses("w-full h-14 min-w-0 flex-none px-4 flex flex-row items-center justify-between gap-3 border-b border-subtle", className);
/** Shrink-safe body for inspector content and nested scroll areas. */
const workbenchInspectorContentClass = (className) => mergeClasses("min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden", className);
//#endregion
export { workbenchClass, workbenchContentClass, workbenchContentColumnClass, workbenchFooterClass, workbenchHeaderClass, workbenchInspectorClass, workbenchInspectorContentClass, workbenchInspectorHeaderClass, workbenchMainClass, workbenchSidebarClass };

//# sourceMappingURL=workbench-style.mjs.map