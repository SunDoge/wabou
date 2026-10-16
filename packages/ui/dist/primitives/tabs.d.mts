import { Accessor } from "solid-js";
//#region src/primitives/tabs.d.ts
export type TabKey = string | number;
export interface TabsOptions<T, K extends TabKey> {
  initialTabs?: readonly T[];
  key: (tab: T) => K;
  initialActiveKey?: K;
  onActiveChange?: (key: K | undefined) => void;
  orientation?: "horizontal" | "vertical";
}
export interface AddTabOptions {
  activate?: boolean;
  index?: number;
}
/** Minimal native capability needed for keyboard focus movement. */
export interface FocusTarget {
  focus(): void;
}
export interface TabsResult<T, K extends TabKey> {
  tabs: Accessor<readonly T[]>;
  activeKey: Accessor<K | undefined>;
  activeTab: Accessor<T | undefined>;
  select: (key: K) => boolean;
  selectNext: () => boolean;
  selectPrevious: () => boolean;
  selectFirst: () => boolean;
  selectLast: () => boolean;
  add: (tab: T, options?: AddTabOptions) => boolean;
  close: (key: K) => boolean;
  move: (key: K, index: number) => boolean;
  register: (key: K, node: FocusTarget) => void;
  focus: (key: K) => boolean;
  handleKeyDown: (key: K, event: TabKeyEvent) => boolean;
}
export interface TabKeyEvent {
  key: string;
  preventDefault?: () => void;
}
/**
 * Stateful tab collection with stable identity and deterministic activation.
 *
 * Closing the active tab selects its right-hand neighbour, or the previous
 * tab when the closed tab was last. Reordering never changes the active key.
 */
export declare function createTabs<T, K extends TabKey>(options: TabsOptions<T, K>): TabsResult<T, K>;
//#endregion
//# sourceMappingURL=tabs.d.mts.map