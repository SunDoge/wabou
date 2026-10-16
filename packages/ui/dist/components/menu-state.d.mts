//#region src/components/menu-state.d.ts
export interface MenuStateItem {
  id: string;
  label: string;
  disabled?: boolean;
}
export type MenuMove = "first" | "last" | "next" | "previous";
/** Resolve one keyboard move without coupling menu state to rendering. */
export declare function moveMenuHighlight(items: readonly MenuStateItem[], current: string | undefined, move: MenuMove): string | undefined;
//#endregion
//# sourceMappingURL=menu-state.d.mts.map