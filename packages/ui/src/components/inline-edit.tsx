import pencil from "lucide-static/icons/pencil.svg?raw";
import { createSignal, type JSX, untrack } from "solid-js";
import { Icon, Text, View } from "../primitives";
import { Button } from "./button";
import { mergeClasses } from "@wabou/core/style";
import { Input } from "./input";

export interface InlineEditProps {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  onCommit?: (value: string) => void;
  onCancel?: () => void;
  disabled?: boolean;
  placeholder?: string;
  /** Accessible name for the editor and its trigger. */
  "aria-label": string;
  class?: string;
  displayClass?: string;
  inputClass?: string;
}

/** Compact rename interaction with explicit Enter, Escape, and blur behavior. */
export function InlineEdit(props: InlineEditProps): JSX.Element {
  const [internal, setInternal] = createSignal(props.defaultValue ?? "");
  const value = () => props.value ?? internal();
  const [editing, setEditing] = createSignal(false);
  const [draft, setDraft] = createSignal(untrack(value));

  const begin = () => {
    if (props.disabled) return;
    setDraft(untrack(value));
    setEditing(true);
  };
  const commit = () => {
    if (!untrack(editing)) return;
    const next = untrack(draft);
    const previous = untrack(value);
    setEditing(false);
    if (props.value === undefined) setInternal(next);
    if (next !== previous) props.onValueChange?.(next);
    props.onCommit?.(next);
  };
  const cancel = () => {
    if (!untrack(editing)) return;
    setDraft(untrack(value));
    setEditing(false);
    props.onCancel?.();
  };

  return (
    <View class={mergeClasses("min-w-0", props.class)}>
      {editing() ? (
        <Input
          ref={(node) => {
            node.focus();
          }}
          aria-label={props["aria-label"]}
          class={mergeClasses("min-w-0", props.inputClass)}
          value={draft()}
          placeholder={props.placeholder}
          onInput={(event) => setDraft(event.currentTarget.value)}
          onBlur={commit}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              commit();
            } else if (event.key === "Escape") {
              event.preventDefault();
              cancel();
            }
          }}
        />
      ) : (
        <Button
          variant="ghost"
          disabled={props.disabled}
          aria-label={`Edit ${props["aria-label"]}`}
          class={mergeClasses(
            "h-8 max-w-full min-w-0 px-2 justify-start gap-2",
            props.displayClass,
          )}
          onClick={begin}
        >
          <Text class="min-w-0 flex-1 text-sm text-primary" maxLines={1}>
            {value() || props.placeholder || "Untitled"}
          </Text>
          <Icon
            aria-hidden="true"
            class="flex-none text-muted"
            source={pencil}
            size={14}
          />
        </Button>
      )}
    </View>
  );
}
