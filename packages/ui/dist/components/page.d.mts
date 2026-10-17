import { ScrollAreaProps } from "../primitives/scroll-area.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/page.d.ts
export declare const pageViewportClass: (className?: string) => string;
export declare const pageViewportContentClass: (className?: string) => string;
export interface PageViewportProps extends Omit<ScrollAreaProps, "class" | "contentClass" | "ref"> {
  children?: JSX.Element;
  /** Classes applied to the bounded scrolling viewport. */
  class?: string;
  /** Classes applied to the intrinsic-height page content wrapper. */
  contentClass?: string;
  /** Reset the page to its origin whenever this identity changes. */
  resetKey?: unknown;
  ref?: (node: Handle) => void;
}
/**
 * A full-height application page boundary.
 *
 * This composes native scrolling with an explicitly sized content wrapper and
 * optional identity-based scroll reset. Page implementations can therefore
 * focus on their own layout instead of reconstructing flex/overflow rules.
 */
export declare function PageViewport(props: PageViewportProps): JSX.Element;
export declare const pageHeaderClass: (className?: string, stacked?: boolean) => string;
export declare const pageHeaderTitleClass: () => string;
export declare const pageHeaderDescriptionClass: (stacked?: boolean) => string;
export interface PageHeaderProps {
  title: string;
  description?: string;
  /** Optional content rendered immediately after the title. */
  titleAdornment?: JSX.Element;
  /** Page-level controls rendered at the trailing edge. */
  actions?: JSX.Element;
  /** Stack actions below the title for narrow application windows. */
  stacked?: boolean;
  class?: string;
}
/** Consistent page title, supporting text and trailing application actions. */
export declare function PageHeader(props: PageHeaderProps): JSX.Element;
//#endregion
//# sourceMappingURL=page.d.mts.map