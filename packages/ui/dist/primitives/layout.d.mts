import { ViewProps } from "./view.mjs";
import { JSX } from "solid-js";
//#region src/primitives/layout.d.ts
export interface LayoutProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
}
/** Horizontal flex container. No wrapper node is added beyond the host View. */
export declare function Row(props: LayoutProps): JSX.Element;
/** Vertical flex container. No wrapper node is added beyond the host View. */
export declare function Column(props: LayoutProps): JSX.Element;
/** Flex container that centers children on both axes. */
export declare function Center(props: LayoutProps): JSX.Element;
//#endregion
//# sourceMappingURL=layout.d.mts.map