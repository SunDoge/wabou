import { Popover } from "../primitives/popover.mjs";
import { componentsElevation, componentsSurfaceClass, useComponentsTheme } from "./theme.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { createDelayedOpenController } from "./delayed-open.mjs";
import { mergeClasses } from "@wabou/core/style";
import { onCleanup } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
//#region src/components/hover-card.tsx
/** A preview surface that tolerates pointer travel between trigger and card. */
function HoverCard(props) {
	const theme = useComponentsTheme();
	const state = createControllableState({
		value: () => props.open,
		defaultValue: props.defaultOpen ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onOpenChange
	});
	const open = () => !(props.disabled ?? false) && state.value();
	const delay = createDelayedOpenController({
		openDelay: () => Math.max(0, props.openDelay ?? 400),
		closeDelay: () => Math.max(0, props.closeDelay ?? 200),
		setOpen: (next) => state.set(next)
	});
	onCleanup(delay.dispose);
	return createComponent$1(Popover, {
		get open() {
			return open();
		},
		onOpenChange: (next) => {
			if (!next) delay.closeNow();
		},
		get "aria-label"() {
			return props["aria-label"];
		},
		get placement() {
			return props.placement ?? "bottom-start";
		},
		get offset() {
			return props.offset ?? 8;
		},
		closeOnEscape: true,
		restoreFocus: false,
		get contentClass() {
			return mergeClasses("min-w-56 max-w-sm min-h-0 p-4 flex flex-col gap-3", componentsSurfaceClass("floating"), props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		get onContentPointerEnter() {
			return delay.openNow;
		},
		get onContentPointerLeave() {
			return delay.scheduleClose;
		},
		get onContentFocusIn() {
			return delay.openNow;
		},
		get onContentFocusOut() {
			return delay.scheduleClose;
		},
		trigger: (popover) => {
			const trigger = {
				ref: popover.ref,
				onPointerEnter: delay.scheduleOpen,
				onPointerLeave: delay.scheduleClose,
				onFocus: delay.openNow,
				onBlur: delay.scheduleClose,
				onKeyDown: (event) => {
					if (event.key === "Escape") delay.closeNow();
				},
				"aria-haspopup": "dialog",
				get "aria-expanded"() {
					return open();
				}
			};
			return props.trigger(trigger);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
export { HoverCard, createDelayedOpenController };

//# sourceMappingURL=hover-card.mjs.map