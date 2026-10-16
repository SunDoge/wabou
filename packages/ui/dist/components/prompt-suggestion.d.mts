import { ButtonProps } from "./button.mjs";
import { ResponsiveGridProps } from "./layout.mjs";
import { JSX } from "solid-js";
//#region src/components/prompt-suggestion.d.ts
export interface PromptSuggestionsProps extends Omit<ResponsiveGridProps, "gap" | "initialColumns" | "maxColumns" | "minColumnWidth" | "balanceLastRow"> {
  itemCount?: number;
  minColumnWidth?: number;
  maxColumns?: 1 | 2 | 3;
  gap?: number;
}
/** Container-responsive starter actions for empty conversations and assistants. */
export declare function PromptSuggestions(props: PromptSuggestionsProps): JSX.Element;
export interface PromptSuggestionProps extends Omit<ButtonProps, "children" | "class"> {
  title: string;
  description?: string;
  icon?: string;
  class?: string;
}
/** One explicit prompt choice; the application remains responsible for its payload. */
export declare function PromptSuggestion(props: PromptSuggestionProps): JSX.Element;
//#endregion
//# sourceMappingURL=prompt-suggestion.d.mts.map