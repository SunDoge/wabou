import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/settings.d.ts
export type SettingsItemOrientation = "horizontal" | "vertical";
export interface SettingsItemProps extends Omit<ViewProps, "children" | "class"> {
  title: string;
  description?: JSX.Element;
  children?: JSX.Element;
  orientation?: SettingsItemOrientation;
  disabled?: boolean;
  class?: string;
  labelClass?: string;
  controlClass?: string;
}
/**
 * One settings row with stable explanatory and control regions.
 * The row blocks its complete subtree when disabled; controls should also
 * receive `disabled` when they need to expose that state independently.
 */
export declare function SettingsItem(props: SettingsItemProps): JSX.Element;
export interface SettingsSectionProps {
  title: string;
  description?: string;
  children?: JSX.Element;
  /** Stack the explanation above the controls at a constrained viewport. */
  stacked?: boolean;
  class?: string;
  contentClass?: string;
}
/**
 * A settings-page section with one explanatory label and one control surface.
 * The page owns the responsive breakpoint and passes `stacked`; the component
 * owns the repeated alignment, spacing, and surface contract.
 */
export declare function SettingsSection(props: SettingsSectionProps): JSX.Element;
export interface SettingsGroupProps {
  title: string;
  description?: string;
  children?: JSX.Element;
  class?: string;
}
/** A titled field group inside a SettingsSection control surface. */
export declare function SettingsGroup(props: SettingsGroupProps): JSX.Element;
//#endregion
//# sourceMappingURL=settings.d.mts.map