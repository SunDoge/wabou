import { View } from "../primitives/view.mjs";
import { Button } from "./button.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { createRovingFocus } from "../primitives/interactions/roving-focus.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createContext, createUniqueId, omit, onCleanup, useContext } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/toolbar.tsx
const ToolbarContext = createContext();
/** A compact command surface with one native tab stop and arrow navigation. */
function Toolbar(props) {
	const orientation = () => props.orientation ?? "horizontal";
	const roving = createRovingFocus({
		orientation,
		loop: props.loop
	});
	const context = {
		orientation,
		register: (id, target, disabled) => roving.register({
			id,
			target,
			disabled
		}),
		activate: (id) => {
			roving.activate(id);
		},
		isTabStop: roving.isTabStop,
		move: roving.move
	};
	return createComponent(ToolbarContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				get role() {
					return props.role ?? "toolbar";
				},
				get "aria-label"() {
					return props["aria-label"];
				},
				get "aria-orientation"() {
					return orientation();
				},
				get "class"() {
					return mergeClasses("flex-none flex items-center gap-1 rounded-lg border border-subtle bg-control p-1", match(orientation()).with("horizontal", () => "flex-row").with("vertical", () => "flex-col").exhaustive(), props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function ToolbarButton(props) {
	const toolbar = useContext(ToolbarContext);
	if (!toolbar) throw new Error("ToolbarButton must be used inside Toolbar");
	const id = createUniqueId();
	const forwarded = omit(props, "ref", "onFocus", "onKeyDown");
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button, mergeProps(forwarded, {
		get variant() {
			return props.variant ?? "ghost";
		},
		get size() {
			return props.size ?? "sm";
		},
		get focusOrder() {
			return toolbar.isTabStop(id) ? 0 : -1;
		},
		ref: (node) => {
			unregister?.();
			unregister = toolbar.register(id, node, () => props.disabled ?? false);
			props.ref?.(node);
		},
		onFocus: (event) => {
			toolbar.activate(id);
			props.onFocus?.(event);
		},
		onKeyDown: (event) => {
			props.onKeyDown?.(event);
			if (event.defaultPrevented) return;
			if (toolbar.move(id, event.key)) event.preventDefault();
		}
	}));
}
function ToolbarToggle(props) {
	const state = createControllableState({
		value: () => props.pressed,
		defaultValue: props.defaultPressed ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onPressedChange
	});
	const forwarded = omit(props, "pressed", "defaultPressed", "onPressedChange");
	return createComponent$1(ToolbarButton, mergeProps(forwarded, {
		get "aria-pressed"() {
			return state.value();
		},
		get variant() {
			return state.value() ? "secondary" : "ghost";
		},
		onClick: () => state.set(!state.value())
	}));
}
function ToolbarGroup(props) {
	const toolbar = useContext(ToolbarContext);
	if (!toolbar) throw new Error("ToolbarGroup must be used inside Toolbar");
	return createComponent$1(View, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"];
		},
		get "class"() {
			return mergeClasses("flex items-center gap-0.5", toolbar.orientation() === "horizontal" ? "flex-row" : "flex-col", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function ToolbarSeparator(props) {
	const toolbar = useContext(ToolbarContext);
	if (!toolbar) throw new Error("ToolbarSeparator must be used inside Toolbar");
	return createComponent$1(View, {
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("flex-none bg-subtle", toolbar.orientation() === "horizontal" ? "w-px h-5" : "h-px w-5", props.class);
		}
	});
}
//#endregion
export { Toolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator, ToolbarToggle };

//# sourceMappingURL=toolbar.mjs.map