import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/listbox.d.ts
export interface ListboxOption {
  value: string;
  label: string;
  /** Accessible identity when the visible label is not unique or sufficiently descriptive. */
  accessibilityLabel?: string;
  description?: string;
  disabled?: boolean;
}
export interface ListboxProps {
  options: readonly ListboxOption[];
  value?: string;
  defaultValue?: string;
  "aria-label": string;
  emptyText?: string;
  class?: string;
  listClass?: string;
  itemClass?: string;
  maxVisibleItems?: number;
  /** Fixed row height used for both layout and viewport calculation. */
  itemHeight?: number;
  /** Explicit scroll viewport height for inspector and split-pane layouts. */
  viewportHeight?: number;
  /** Fill the available flex height instead of deriving a fixed viewport. */
  fill?: boolean;
  /** Receives the focusable listbox handle for dialog and popover composition. */
  ref?: (node: Handle) => void;
  renderLeading?: (option: ListboxOption) => JSX.Element;
  renderTrailing?: (option: ListboxOption) => JSX.Element;
  onValueChange?: (value: string) => void;
  /** Invoked after pointer or keyboard activation of an enabled option. */
  onAction?: (value: string) => void;
  onDismiss?: () => void;
}
/**
 * Focusable inline single-selection list.
 *
 * Unlike Select, Listbox owns no popup or trigger. It is suitable for dialogs,
 * inspectors and other surfaces where the choices are already visible.
 */
export declare function Listbox(props: ListboxProps): JSX.Element;
//#endregion
//# sourceMappingURL=listbox.d.mts.map