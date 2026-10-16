import { Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { Kbd } from "./display.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, createSignal } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/shortcut-recorder.tsx
const MOD_SHIFT = 1;
const MOD_CONTROL = 2;
const MOD_ALT = 4;
const MOD_META = 8;
const MODIFIER_KEYS = /* @__PURE__ */ new Set([
	"Shift",
	"Control",
	"Alt",
	"Meta"
]);
function shortcutFromKeyEvent(event) {
	if (MODIFIER_KEYS.has(event.key)) return void 0;
	const mods = event.mods ?? 0;
	const parts = [];
	if (event.primary) parts.push("Primary");
	if (!event.primary && (mods & MOD_CONTROL) !== 0) parts.push("Control");
	if ((mods & MOD_ALT) !== 0) parts.push("Alt");
	if ((mods & MOD_SHIFT) !== 0) parts.push("Shift");
	if (!event.primary && (mods & MOD_META) !== 0) parts.push("Meta");
	const key = match(event.key).with(" ", () => "Space").with("ArrowUp", () => "Up").with("ArrowDown", () => "Down").with("ArrowLeft", () => "Left").with("ArrowRight", () => "Right").otherwise((value) => value.length === 1 ? value.toUpperCase() : value);
	parts.push(key);
	return {
		chord: parts.join("+"),
		parts
	};
}
function ShortcutRecorder(props) {
	const [local, setLocal] = createSignal(props.defaultValue ?? "");
	const [recording, setRecording] = createSignal(false);
	const value = () => props.value ?? local();
	const setValue = (next) => {
		if (props.value === void 0) setLocal(next);
		props.onValueChange?.(next);
	};
	const handleKeyDown = (event) => {
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
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("min-w-0 flex flex-col gap-2", props.class);
		},
		get children() {
			return [memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent$1(Text, {
					class: "text-sm font-medium text-primary",
					get children() {
						return props.label;
					}
				}) : props.label;
			}), createComponent$1(Button, {
				unstyled: true,
				role: "button",
				get "aria-label"() {
					return props.label ?? "Keyboard shortcut";
				},
				get "aria-pressed"() {
					return recording();
				},
				get disabled() {
					return props.disabled;
				},
				class: (state) => mergeClasses("h-10 min-w-48 px-3 flex flex-row items-center gap-1 rounded-lg border bg-input", recording() || state.focusVisible ? "border-focus" : "border-strong"),
				onClick: () => setRecording(true),
				onKeyDown: handleKeyDown,
				get children() {
					return memo(() => {
						return !!recording();
					})() ? createComponent$1(Text, {
						class: "text-sm text-muted",
						children: "Press a shortcut…"
					}) : memo(() => {
						return parts().length > 0;
					})() ? createComponent$1(For, {
						get each() {
							return parts();
						},
						children: (part) => createComponent$1(Kbd, { children: part })
					}) : createComponent$1(Text, {
						class: "text-sm text-muted",
						get children() {
							return props.placeholder ?? "Record shortcut";
						}
					});
				}
			})];
		}
	});
}
//#endregion
export { ShortcutRecorder, shortcutFromKeyEvent };

//# sourceMappingURL=shortcut-recorder.mjs.map