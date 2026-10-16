import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ButtonProps } from "./button.mjs";
import { TextAreaProps } from "./input.mjs";
import { JSX } from "solid-js";
//#region src/components/prompt-composer.d.ts
export interface PromptComposerProps extends Omit<ViewProps, "class"> {
  class?: string;
  disabled?: boolean;
  invalid?: boolean;
  /** Background utility owned by the compound surface. */
  surfaceClass?: string;
}
export interface PromptComposerRowProps extends Omit<ViewProps, "class"> {
  class?: string;
  /** Allow controls to form additional rows when the embedding surface opts in. */
  wrap?: boolean;
}
export declare function promptComposerEditorHeightClass(value: string): string;
export interface PromptComposerEditorProps extends TextAreaProps {
  value?: string;
}
/** Native multiline editor with density and chrome owned by PromptComposer. */
export declare function PromptComposerEditor(props: PromptComposerEditorProps): JSX.Element;
/** Stable circular primary action for a PromptComposer toolbar. */
export declare function PromptComposerAction(props: ButtonProps): JSX.Element;
export declare function promptComposerClass(focused: boolean, invalid: boolean, disabled: boolean, className?: string): string;
/** Shared compound surface for prompts, attachments, controls and status. */
export declare function PromptComposer(props: PromptComposerProps): JSX.Element;
/** Compact metadata row above the authored prompt. */
export declare function PromptComposerStatus(props: ViewProps): JSX.Element;
/** Responsive action row below the authored prompt. */
export declare function PromptComposerToolbar(props: PromptComposerRowProps): JSX.Element;
/** Shrink-safe group for the composer tools preceding its primary action. */
export declare function PromptComposerTools(props: PromptComposerRowProps): JSX.Element;
//#endregion
//# sourceMappingURL=prompt-composer.d.mts.map