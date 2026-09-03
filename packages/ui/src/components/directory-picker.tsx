import {
  createAsyncAction,
  type PickDirectoryOptions,
  useDialog,
} from "@wabou/core";
import folder from "lucide-static/icons/folder.svg?raw";
import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { Icon } from "../primitives";
import { directoryPickerOptions } from "./directory-picker-state";
import {
  InputGroup,
  InputGroupButton,
  InputGroupInput,
} from "./forms";
import type { InputProps } from "./input";

export interface DirectoryPickerProps
  extends Omit<InputProps, "class" | "onInput" | "value"> {
  value: string;
  onValueChange: (value: string) => void;
  /** Options forwarded to the native picker. `directory` defaults to `value`. */
  dialogOptions?: PickDirectoryOptions;
  browseLabel?: string;
  pendingLabel?: string;
  browseAriaLabel?: string;
  class?: string;
  inputClass?: string;
  buttonClass?: string;
  onBrowseError?: (error: unknown) => void;
  /** Called only after the native picker commits a directory selection. */
  onBrowseSelect?: (value: string) => void;
}

/** A controlled path input paired with the operating system directory picker. */
export function DirectoryPicker(props: DirectoryPickerProps): JSX.Element {
  const nativeDialog = useDialog();
  const local = props;
  const selection = createAsyncAction(() =>
    nativeDialog.pickDirectory(
      directoryPickerOptions(local.value, local.dialogOptions),
    ),
  );
  const inputProps = omit(
    props,
    "value",
    "onValueChange",
    "dialogOptions",
    "browseLabel",
    "pendingLabel",
    "browseAriaLabel",
    "class",
    "inputClass",
    "buttonClass",
    "onBrowseError",
    "onBrowseSelect",
  );

  async function browse(): Promise<void> {
    if (inputProps.disabled) return;
    const result = await selection.run();
    if (!result.ok) {
      if (local.onBrowseError) local.onBrowseError(result.error);
      else throw result.error;
      return;
    }
    if (result.value !== null) {
      local.onValueChange(result.value);
      local.onBrowseSelect?.(result.value);
    }
  }

  return (
    <InputGroup
      disabled={Boolean(inputProps.disabled) || selection.pending()}
      class={local.class}
    >
      <InputGroupInput
        {...inputProps}
        class={local.inputClass}
        value={local.value}
        onInput={(event) => local.onValueChange(event.currentTarget.value)}
      />
      <InputGroupButton
        class={mergeClasses("flex-none", local.buttonClass)}
        disabled={Boolean(inputProps.disabled) || selection.pending()}
        aria-label={
          local.browseAriaLabel ?? local.browseLabel ?? "Browse directory"
        }
        onClick={() => void browse()}
      >
        <Icon source={folder} size={14} />
        {selection.pending()
          ? (local.pendingLabel ?? "Opening…")
          : (local.browseLabel ?? "Browse…")}
      </InputGroupButton>
    </InputGroup>
  );
}
