import { ButtonState } from "../primitives/button.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/tabs.d.ts
export interface TabsProps {
  value?: string;
  defaultValue?: string;
  orientation?: "horizontal" | "vertical";
  class?: string;
  children?: JSX.Element;
  onValueChange?: (value: string) => void;
}
export declare function Tabs(props: TabsProps): JSX.Element;
export declare function TabsList(props: {
  variant?: "default" | "line";
  /** Keep tab semantics and roving focus while leaving layout and paint to the caller. */
  unstyled?: boolean;
  "aria-label"?: string;
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
export interface TabsTriggerProps {
  value: string;
  disabled?: boolean;
  /** Keep tab behavior and semantics without applying the component skin. */
  unstyled?: boolean;
  "aria-label"?: string;
  class?: string | ((state: ButtonState) => string);
  children?: JSX.Element;
}
export interface TabsItemState {
  selected: boolean;
}
export interface TabsItemProps {
  value: string;
  disabled?: boolean;
  closeLabel?: string;
  onClose?: () => void;
  class?: string | ((state: TabsItemState) => string);
  triggerClass?: string | ((state: ButtonState) => string);
  children?: JSX.Element;
}
/**
 * Bounded, optionally closeable tab chrome. The tab trigger and close action
 * remain sibling hit targets so closing a tab never selects it first.
 */
export declare function TabsItem(props: TabsItemProps): JSX.Element;
export declare function TabsTrigger(props: TabsTriggerProps): JSX.Element;
export declare function TabsContent(props: {
  value: string;
  /** Keep stateful/native content mounted while hiding inactive panels. */
  keepMounted?: boolean;
  class?: string;
  children?: JSX.Element;
}): JSX.Element;
//#endregion
//# sourceMappingURL=tabs.d.mts.map