import { mergeClasses } from "@wabou/core/style";
import { createSignal, For as ForValue, type JSX } from "solid-js";
import { match } from "ts-pattern";
import {
  type ButtonKeyEvent,
  Button as HeadlessButton,
  Text,
  View,
} from "../primitives";
import { Kbd } from "./display";

const MOD_SHIFT = 1;
const MOD_CONTROL = 2;
const MOD_ALT = 4;
const MOD_META = 8;
const MODIFIER_KEYS = new Set(["Shift", "Control", "Alt", "Meta"]);

export interface RecordedShortcut {
  chord: string;
  parts: readonly string[];
}

export function shortcutFromKeyEvent(
  event: Pick<ButtonKeyEvent, "key" | "mods" | "primary">,
): RecordedShortcut | undefined {
  if (MODIFIER_KEYS.has(event.key)) return undefined;
  const mods = event.mods ?? 0;
  const parts: string[] = [];
  if (event.primary) parts.push("Primary");
  if (!event.primary && (mods & MOD_CONTROL) !== 0) parts.push("Control");
  if ((mods & MOD_ALT) !== 0) parts.push("Alt");
  if ((mods & MOD_SHIFT) !== 0) parts.push("Shift");
  if (!event.primary && (mods & MOD_META) !== 0) parts.push("Meta");
  const key = match(event.key)
    .with(" ", () => "Space")
    .with("ArrowUp", () => "Up")
    .with("ArrowDown", () => "Down")
    .with("ArrowLeft", () => "Left")
    .with("ArrowRight", () => "Right")
    .otherwise((value) => (value.length === 1 ? value.toUpperCase() : value));
  parts.push(key);
  return { chord: parts.join("+"), parts };
}

export interface ShortcutRecorderProps {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  label?: string;
  placeholder?: string;
  disabled?: boolean;
  class?: string;
}

export function ShortcutRecorder(props: ShortcutRecorderProps): JSX.Element {
  const [local, setLocal] = createSignal(props.defaultValue ?? "");
  const [recording, setRecording] = createSignal(false);
  const value = () => props.value ?? local();
  const setValue = (next: string) => {
    if (props.value === undefined) setLocal(next);
    props.onValueChange?.(next);
  };
  const handleKeyDown = (event: ButtonKeyEvent) => {
    if (!recording()) return;
    event.preventDefault();
    if (event.key === "Escape") {
      setRecording(false);
      return;
    }
    if (event.key === "Backspace" || event.key === "Delete") {
      setValue("");
      setRecording(false);
      return;
    }
    const shortcut = shortcutFromKeyEvent(event);
    if (!shortcut) return;
    setValue(shortcut.chord);
    setRecording(false);
  };
  const parts = () => value().split("+").filter(Boolean);
  return (
    <View class={mergeClasses("min-w-0 flex flex-col gap-2", props.class)}>
      {props.label && (
        <Text class="text-sm font-medium text-primary">{props.label}</Text>
      )}
      <HeadlessButton
        unstyled
        role="button"
        aria-label={props.label ?? "Keyboard shortcut"}
        aria-pressed={recording()}
        disabled={props.disabled}
        class={(state) =>
          mergeClasses(
            "h-10 min-w-48 px-3 flex flex-row items-center gap-1 rounded-lg border bg-input",
            recording() || state.focusVisible
              ? "border-focus"
              : "border-strong",
          )
        }
        onClick={() => setRecording(true)}
        onKeyDown={handleKeyDown}
      >
        {recording() ? (
          <Text class="text-sm text-muted">Press a shortcut…</Text>
        ) : parts().length > 0 ? (
          <ForValue each={parts()}>{(part) => <Kbd>{part}</Kbd>}</ForValue>
        ) : (
          <Text class="text-sm text-muted">
            {props.placeholder ?? "Record shortcut"}
          </Text>
        )}
      </HeadlessButton>
    </View>
  );
}
