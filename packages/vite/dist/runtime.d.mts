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
//#endregion
export { createHotContext, removeStyle, updateStyle };
//# sourceMappingURL=runtime.d.mts.map