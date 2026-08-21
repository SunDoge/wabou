import type { Handle } from "@wabou/core/renderer";
import search from "lucide-static/icons/search.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { type JSX, omit, Show } from "solid-js";
import { Icon, View } from "../primitives";
import { InputGroup, InputGroupButton, InputGroupInput } from "./forms";
import type { InputProps } from "./input";
import { createControllableState } from "./state";

export interface SearchFieldProps
  extends Omit<InputProps, "class" | "onInput" | "ref" | "value"> {
  /** `undefined` selects uncontrolled mode. */
  value?: string;
  defaultValue?: string;
  onValueChange?(value: string): void;
  /** Called when Enter is pressed with the current query. */
  onSearch?(value: string): void;
  onClear?(): void;
  clearLabel?: string;
  class?: string;
  /** Background utility for the complete search field. Defaults to `bg-input`. */
  surfaceClass?: string;
  inputClass?: string;
  inputRef?: (input: Handle) => void;
}

/** A native search input with consistent clear, Escape, and submit behavior. */
export function SearchField(props: SearchFieldProps): JSX.Element {
  const forwarded = omit(
    props,
    "value",
    "defaultValue",
    "onValueChange",
    "onSearch",
    "onClear",
    "clearLabel",
    "class",
    "surfaceClass",
    "inputClass",
    "inputRef",
  );
  const state = createControllableState({
    value: () => props.value,
    defaultValue: props.defaultValue ?? "",
    onChange: props.onValueChange,
  });
  let input: Handle | undefined;
  const clear = () => {
    if (!state.value() || props.disabled || props.readOnly) return false;
    if (!state.set("")) return false;
    props.onClear?.();
    input?.focus();
    return true;
  };

  return (
    <InputGroup class={props.class} surfaceClass={props.surfaceClass}>
      <View
        aria-hidden="true"
        class="flex-none pl-2.5 flex items-center text-muted"
      >
        <Icon source={search} size={15} />
      </View>
      <InputGroupInput
        {...forwarded}
        ref={(node) => {
          input = node;
          props.inputRef?.(node);
        }}
        value={state.value()}
        class={props.inputClass}
        onInput={(event) => state.set(event.currentTarget.value)}
        onKeyDown={(event) => {
          props.onKeyDown?.(event);
          if (event.key === "Escape" && clear()) event.preventDefault();
          if (event.key === "Enter") props.onSearch?.(state.value());
        }}
      />
      <Show when={state.value() && !props.disabled && !props.readOnly}>
        <InputGroupButton
          size="icon"
          aria-label={props.clearLabel ?? "Clear search"}
          onClick={clear}
        >
          <Icon source={x} aria-hidden="true" size={15} />
        </InputGroupButton>
      </Show>
    </InputGroup>
  );
}
