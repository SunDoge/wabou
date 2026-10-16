import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { PageViewportProps } from "./page.mjs";
import { JSX } from "solid-js";
//#region src/components/onboarding.d.ts
export interface OnboardingProps extends Omit<PageViewportProps, "class" | "contentClass"> {
  class?: string;
  contentClass?: string;
}
/**
 * Full-height first-run boundary with native scrolling and a readable measure.
 * Content stays centered when it fits and remains reachable when it grows.
 */
export declare function Onboarding(props: OnboardingProps): JSX.Element;
export declare function OnboardingHeader(props: ViewProps): JSX.Element;
export declare function OnboardingHeading(props: ViewProps): JSX.Element;
export declare function OnboardingTitle(props: TextProps): JSX.Element;
export declare function OnboardingDescription(props: TextProps): JSX.Element;
export declare function OnboardingFooter(props: ViewProps): JSX.Element;
//#endregion
//# sourceMappingURL=onboarding.d.mts.map