import { DropdownMenu } from "./dropdown-menu.mjs";
import { createSignal } from "solid-js";
import { createComponent as createComponent$1 } from "@wabou/core/renderer";
//#region src/components/context-menu.tsx
/** An action menu anchored to the native secondary-click coordinate. */
function ContextMenu(props) {
	const [open, setOpen] = createSignal(false);
	const [point, setPoint] = createSignal();
	return createComponent$1(DropdownMenu, {
		get "aria-label"() {
			return props["aria-label"];
		},
		get items() {
			return props.items;
		},
		get open() {
			return open();
		},
		onOpenChange: setOpen,
		get onAction() {
			return props.onAction;
		},
		anchorPoint: point,
		get contentClass() {
			return props.contentClass;
		},
		get contentShadows() {
			return props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		trigger: (menu) => props.trigger({
			ref: menu.ref,
			onContextMenu: (event) => {
				event.preventDefault();
				event.stopPropagation();
				setPoint({
					x: event.clientX,
					y: event.clientY
				});
				setOpen(true);
			},
			onKeyDown: (event) => {
				if (event.key === "ContextMenu" || event.key === "F10") {
					event.preventDefault();
					event.stopPropagation();
					setPoint(void 0);
					setOpen(true);
					return;
				}
				menu.onKeyDown(event);
			},
			"aria-haspopup": "menu",
			get "aria-expanded"() {
				return open();
			}
		})
	});
}
//#endregion
export { ContextMenu };

//# sourceMappingURL=context-menu.mjs.map