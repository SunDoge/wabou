import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/typography.d.ts
type TypographyTextProps = Omit<TextProps, "class"> & {
  class?: string;
};
export declare const TypographyH1: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare const TypographyH2: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare const TypographyH3: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare const TypographyH4: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare const TypographyP: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare const TypographyLead: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare const TypographyLarge: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare const TypographySmall: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare const TypographyMuted: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare const TypographyInlineCode: (props: TypographyTextProps) => import("@wabou/core/jsx-runtime").JSX.Element;
export declare function TypographyBlockquote(props: TypographyTextProps): JSX.Element;
export declare function TypographyList(props: ViewProps): JSX.Element;
export declare function TypographyListItem(props: TypographyTextProps): JSX.Element;
//#endregion
export {};
//# sourceMappingURL=typography.d.mts.map