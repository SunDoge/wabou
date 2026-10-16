import { createControllableState } from "../primitives/interactions/state.mjs";
import { DropdownMenu } from "./dropdown-menu.mjs";
import { Toolbar, ToolbarButton } from "./toolbar.mjs";
import { createComponent, createContext, onCleanup, useContext } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/menubar.tsx
const MenubarContext = createContext();
/** Persistent application menus with one tab stop and sibling menu switching. */
function Menubar(props) {
	const entries = [];
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? null,
		onChange: props.onValueChange
	});
	const enabled = () => entries.filter((entry) => !entry.disabled());
	const context = {
		openValue: state.value,
		setOpenValue: state.set,
		register(entry) {
			entries.push(entry);
			return () => {
				const index = entries.indexOf(entry);
				if (index >= 0) entries.splice(index, 1);
			};
		},
		moveOpen(value, direction) {
			const candidates = enabled();
			if (candidates.length === 0) return false;
			const index = candidates.findIndex((entry) => entry.value === value);
			const target = direction === "next" ? candidates[index + 1] ?? (props.loop === false ? void 0 : candidates[0]) : candidates[index - 1] ?? (props.loop === false ? void 0 : candidates.at(-1));
			if (!target) return false;
			state.set(target.value);
			target.target.focus();
			return true;
		}
	};
	return createComponent(MenubarContext, {
		value: context,
		get children() {
			return createComponent$1(Toolbar, {
				role: "menubar",
				get "aria-label"() {
					return props["aria-label"];
				},
				get loop() {
					return props.loop;
				},
				get "class"() {
					return props.class;
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function MenubarMenu(props) {
	const menubar = useContext(MenubarContext);
	if (!menubar) throw new Error("MenubarMenu must be used inside Menubar");
	let unregister;
	let closeOnPointerActivation;
	let switchedByHover = false;
	onCleanup(() => unregister?.());
	const handleMenuKey = (event) => {
		const direction = match(event.key).with("ArrowRight", () => "next").with("ArrowLeft", () => "previous").otherwise(() => void 0);
		if (!direction || !menubar.moveOpen(props.value, direction)) return;
		event.preventDefault();
	};
	return createComponent$1(DropdownMenu, {
		get "aria-label"() {
			return `${props.label} menu`;
		},
		get items() {
			return props.items;
		},
		get open() {
			return menubar.openValue() === props.value;
		},
		onOpenChange: (open) => menubar.setOpenValue(open ? props.value : null),
		get onAction() {
			return props.onAction;
		},
		get motion() {
			return props.motion;
		},
		placement: "bottom-start",
		outsidePointerStrategy: "passthrough",
		get restoreFocus() {
			return menubar.openValue() === null || menubar.openValue() === props.value;
		},
		onContentKeyDown: handleMenuKey,
		trigger: (trigger) => createComponent$1(ToolbarButton, mergeProps(trigger, {
			role: "menuitem",
			get "aria-label"() {
				return props.label;
			},
			get disabled() {
				return props.disabled;
			},
			ref: (node) => {
				unregister?.();
				unregister = menubar.register({
					value: props.value,
					target: node,
					disabled: () => props.disabled ?? false
				});
				trigger.ref(node);
			},
			onFocus: (event) => {
				if (event.payload.focusVisible && menubar.openValue() !== null) menubar.setOpenValue(props.value);
			},
			onPointerEnter: () => {
				if (menubar.openValue() !== null && menubar.openValue() !== props.value) {
					switchedByHover = true;
					menubar.setOpenValue(props.value);
				}
			},
			onPointerDown: () => {
				closeOnPointerActivation = menubar.openValue() === props.value && !switchedByHover;
				switchedByHover = false;
			},
			onPointerCancel: () => {
				closeOnPointerActivation = void 0;
			},
			onClick: (event) => {
				event.stopPropagation();
				const close = closeOnPointerActivation ?? menubar.openValue() === props.value;
				closeOnPointerActivation = void 0;
				menubar.setOpenValue(close ? null : props.value);
			},
			get children() {
				return props.children ?? props.label;
			}
		}))
	});
}
//#endregion
export { Menubar, MenubarMenu };

//# sourceMappingURL=menubar.mjs.map