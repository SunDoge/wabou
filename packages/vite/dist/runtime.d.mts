//#region src/runtime/client.d.ts
type AcceptCallback = (module: unknown) => void;
type DisposeCallback = (data: Record<string, unknown>) => void;
interface HotContext {
  data: Record<string, unknown>;
  accepted: AcceptCallback[];
  disposed: DisposeCallback[];
  invalidated: boolean;
  accept(callback?: AcceptCallback): void;
  dispose(callback: DisposeCallback): void;
  decline(): void;
  invalidate(): void;
  on(): void;
  send(): void;
  prune(): void;
}
declare function createHotContext(ownerPath: string): HotContext;
declare function updateStyle(id: string, css: string): void;
declare function removeStyle(id: string): void;
/**
 * Replace hot records transactionally during an entry reload. Vite can report
 * a reload while a saved file still fails to transform; retaining the previous
 * records lets the next valid save use HMR instead of leaving the runtime in a
 * permanently degraded state.
 */
declare function beginFullReload(): void;
declare function commitFullReload(): void;
declare function rollbackFullReload(): void;
//#endregion
export { beginFullReload, commitFullReload, createHotContext, removeStyle, rollbackFullReload, updateStyle };
//# sourceMappingURL=runtime.d.mts.map