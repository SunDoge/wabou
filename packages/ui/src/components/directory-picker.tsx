import { type PickDirectoryOptions, useDialog } from "@wabou/core";
import folder from "lucide-static/icons/folder.svg?raw";
import { createSignal, type JSX, omit } from "solid-js";
import { Icon, View } from "../primitives";
import { Button } from "./button";
import { join } from "./class-names";
import { directoryPickerOptions } from "./directory-picker-state";
import { Input, type InputProps } from "./input";

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
}

/** A controlled path input paired with the operating system directory picker. */
export function DirectoryPicker(props: DirectoryPickerProps): JSX.Element {
  const nativeDialog = useDialog();
  const [pending, setPending] = createSignal(false);
  const local = props;
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
  );

  async function browse(): Promise<void> {
    if (pending() || inputProps.disabled) return;
    setPending(true);
    try {
      const selected = await nativeDialog.pickDirectory(
        directoryPickerOptions(local.value, local.dialogOptions),
      );
      if (selected !== null) local.onValueChange(selected);
    } catch (error) {
      if (local.onBrowseError) local.onBrowseError(error);
      else throw error;
    } finally {
      setPending(false);
    }
  }

  return (
    <View class={join("w-full min-w-0 flex items-center gap-2", local.class)}>
      <Input
        {...inputProps}
        class={join("min-w-0 flex-1", local.inputClass)}
        value={local.value}
        onInput={(event) => local.onValueChange(event.currentTarget.value)}
      />
      <Button
        class={join("flex-none", local.buttonClass)}
        variant="outline"
        disabled={Boolean(inputProps.disabled) || pending()}
        aria-label={
          local.browseAriaLabel ?? local.browseLabel ?? "Browse directory"
        }
        onClick={() => void browse()}
      >
        <Icon source={folder} size={14} />
        {pending()
          ? (local.pendingLabel ?? "Opening…")
          : (local.browseLabel ?? "Browse…")}
      </Button>
    </View>
  );
}
