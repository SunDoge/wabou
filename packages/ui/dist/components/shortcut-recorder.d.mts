import { ButtonKeyEvent } from "../primitives/button.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/shortcut-recorder.d.ts
export interface RecordedShortcut {
  chord: string;
  parts: readonly string[];
}
export declare function shortcutFromKeyEvent(event: Pick<ButtonKeyEvent, "key" | "mods" | "primary">): RecordedShortcut | undefined;
export interface ShortcutRecorderProps {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  label?: string;
  placeholder?: string;
  disabled?: boolean;
  class?: string;
}
export declare function ShortcutRecorder(props: ShortcutRecorderProps): JSX.Element;
//#endregion
//# sourceMappingURL=shortcut-recorder.d.mts.map