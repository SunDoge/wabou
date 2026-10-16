import { MenuStateItem } from "./menu-state.mjs";
//#region src/components/command-state.d.ts
export interface CommandStateItem extends MenuStateItem {
  keywords?: readonly string[];
}
export declare function filterCommandItems<T extends CommandStateItem>(items: readonly T[], query: string): T[];
export declare function reconcileCommandHighlight<T extends CommandStateItem>(items: readonly T[], highlighted: string | undefined): string | undefined;
//#endregion
//# sourceMappingURL=command-state.d.mts.map