import { createFocus } from "./focus.mjs";
import { createHover } from "./hover.mjs";
import { createPress } from "./press.mjs";
import { createInternalPrimitive } from "./view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { untrack } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps, useHost } from "@wabou/core/renderer";
//#region src/primitives/button.tsx
const ACCENTS = {
	neutral: "#475569",
	sky: "#0284c7",
	amber: "#d97706"
};
function InternalButton(props) {
	return createInternalPrimitive("button", props);
}
function resolveButtonFocusOrder(disabled, focusOrder) {
	return disabled ? -1 : focusOrder ?? 0;
}
/** Headless button state and event normalization. */
function createButton(options = {}) {
	const hover = createHover();
	const focus = createFocus();
	const disabled = () => typeof options.disabled === "function" ? options.disabled() : options.disabled ?? false;
	const selected = () => typeof options.selected === "function" ? options.selected() : options.selected ?? false;
	const press = createPress({
		disabled,
		onPress: (event) => options.onPress?.(event)
	});
	return {
		state: () => ({
			hovered: hover.hovered(),
			pressed: press.pressed(),
			focused: focus.focused(),
			focusVisible: focus.focusVisible(),
			selected: selected(),
			disabled: disabled()
		}),
		bindings: {
			onPointerEnter: hover.bindings.onPointerEnter,
			onPointerLeave: () => {
				hover.bindings.onPointerLeave();
				press.bindings.onPointerLeave();
			},
			onPointerDown: () => {
				focus.pointerModality();
				press.bindings.onPointerDown();
			},
			onPointerUp: press.bindings.onPointerUp,
			onPointerCancel: press.bindings.onPointerCancel,
			onFocus: focus.bindings.onFocus,
			onBlur: focus.bindings.onBlur,
			onClick: press.bindings.onClick,
			onKeyDown: (event) => {
				focus.keyboardModality();
				options.onKeyDown?.(event);
				if (event.defaultPrevented || event.repeat) return;
				if (event.key !== "Enter" && event.key !== " ") return;
				event.preventDefault();
				if (!disabled()) options.onPress?.(event);
			}
		}
	};
}
/**
* A native button with consistent hover, pressed, focus and disabled feedback.
*
* Interaction styling is deliberately implemented with reactive inline styles:
* applications do not need CSS pseudo-class support to get a responsive button.
*/
function Button(props) {
	const forwardedRef = untrack(() => props.ref);
	const renderContent = untrack(() => props.renderContent);
	const refProps = forwardedRef ? { ref: forwardedRef } : {};
	const disabled = () => props.disabled ?? false;
	const primitive = createButton({
		disabled,
		selected: () => props.selected ?? false,
		onPress: (event) => props.onClick?.(event),
		onKeyDown: (event) => props.onKeyDown?.(event)
	});
	const variant = () => props.variant ?? "solid";
	const accent = () => ACCENTS[props.tone ?? "neutral"];
	const state = primitive.state;
	const customStyle = () => typeof props.style === "function" ? props.style(state()) : props.style;
	const structuralStyle = () => ({
		display: "flex",
		"align-items": "center",
		"flex-shrink": 0,
		"white-space": "nowrap",
		cursor: disabled() ? "not-allowed" : "pointer"
	});
	const defaultStyle = () => {
		if (props.unstyled) return structuralStyle();
		return {
			...structuralStyle(),
			"justify-content": "center",
			"min-height": "32px",
			padding: "6px 12px",
			"border-radius": "6px",
			"border-width": "1px",
			"border-color": state().focusVisible ? "#7dd3fc" : "#64748b",
			"background-color": background(),
			color: "#f8fafc",
			opacity: disabled() ? .45 : 1
		};
	};
	const background = () => {
		if (variant() === "ghost" && !props.selected) {
			if (state().pressed) return "#1e293b";
			return state().hovered ? "#334155" : "transparent";
		}
		if (state().pressed) return "#1e293b";
		if (state().hovered && !props.selected) return "#334155";
		return accent();
	};
	return createComponent$1(InternalButton, mergeProps(refProps, {
		get disabled() {
			return disabled();
		},
		get "aria-disabled"() {
			return disabled();
		},
		get focusOrder() {
			return resolveButtonFocusOrder(disabled(), props.focusOrder);
		},
		get role() {
			return props.role ?? "button";
		},
		get "aria-haspopup"() {
			return props["aria-haspopup"];
		},
		get "aria-expanded"() {
			return props["aria-expanded"];
		},
		get "aria-controls"() {
			return props["aria-controls"];
		},
		get "aria-label"() {
			return props["aria-label"];
		},
		get "aria-checked"() {
			return props["aria-checked"];
		},
		get "aria-current"() {
			return props["aria-current"];
		},
		get "aria-selected"() {
			return props["aria-selected"];
		},
		get "aria-pressed"() {
			return props["aria-pressed"];
		},
		get "aria-busy"() {
			return props["aria-busy"];
		},
		get "aria-valuetext"() {
			return props["aria-valuetext"];
		},
		get "class"() {
			return mergeClasses("select-none", typeof props.class === "function" ? props.class(state()) : props.class);
		},
		get classList() {
			return memo(() => {
				return typeof props.classList === "function";
			})() ? props.classList(state()) : props.classList;
		},
		get style() {
			return {
				...defaultStyle(),
				...customStyle()
			};
		},
		onPointerEnter: (event) => {
			primitive.bindings.onPointerEnter();
			props.onPointerEnter?.(event);
		},
		onPointerLeave: (event) => {
			primitive.bindings.onPointerLeave();
			props.onPointerLeave?.(event);
		},
		onPointerDown: (event) => {
			primitive.bindings.onPointerDown();
			props.onPointerDown?.(event);
		},
		onPointerUp: (event) => {
			primitive.bindings.onPointerUp();
			props.onPointerUp?.(event);
		},
		onPointerCancel: (event) => {
			primitive.bindings.onPointerCancel();
			props.onPointerCancel?.(event);
		},
		get onPointerMove() {
			return props.onPointerMove;
		},
		get onPointerOver() {
			return props.onPointerOver;
		},
		get onPointerOut() {
			return props.onPointerOut;
		},
		onFocus: (event) => {
			primitive.bindings.onFocus();
			props.onFocus?.(event);
		},
		onBlur: (event) => {
			primitive.bindings.onBlur();
			props.onBlur?.(event);
		},
		get onFocusIn() {
			return props.onFocusIn;
		},
		get onFocusOut() {
			return props.onFocusOut;
		},
		get onClick() {
			return primitive.bindings.onClick;
		},
		get onContextMenu() {
			return props.onContextMenu;
		},
		get onDblClick() {
			return props.onDblClick;
		},
		get onKeyDown() {
			return primitive.bindings.onKeyDown;
		},
		get onKeyUp() {
			return props.onKeyUp;
		},
		get onWheel() {
			return props.onWheel;
		},
		get children() {
			return renderContent?.(state) ?? props.children;
		}
	}));
}
/**
* An explicit external-link interaction.
*
* Wabou does not assign browser behavior to an `a` tag or `href` attribute;
* the JS primitive owns activation while Rust only executes `openUrl`.
*/
function Link(props) {
	const host = useHost();
	return createComponent$1(Button, mergeProps(props, {
		role: "link",
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented) host.system.openUrl(props.url);
		}
	}));
}
//#endregion
export { Button, Link, createButton, resolveButtonFocusOrder };

//# sourceMappingURL=button.mjs.map