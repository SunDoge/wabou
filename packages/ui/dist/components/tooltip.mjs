import { Text, View } from "../primitives/view.mjs";
import { Popover } from "../primitives/popover.mjs";
import { Kbd } from "./display.mjs";
import { componentsElevation, useComponentsTheme } from "./theme.mjs";
import { createDelayedOpenController } from "./delayed-open.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createSignal, onCleanup } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
//#region src/components/tooltip.tsx
/** Visual content shared by managed and explicitly composed tooltips. */
function TooltipContent(props) {
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("min-w-0 flex flex-row items-center gap-3", props.class);
		},
		get children() {
			return [createComponent$1(Text, {
				get id() {
					return props.id;
				},
				role: "tooltip",
				class: "min-w-0 flex-1 whitespace-normal text-xs text-primary",
				get children() {
					return props.children;
				}
			}), memo(() => {
				return memo(() => {
					return props.shortcut === void 0;
				})() ? null : createComponent$1(Kbd, {
					get "aria-label"() {
						return `${props.shortcut} shortcut`;
					},
					get children() {
						return props.shortcut;
					}
				});
			})];
		}
	});
}
let tooltipId = 0;
/** A delayed, non-interactive label for pointer and keyboard focus targets. */
function Tooltip(props) {
	const theme = useComponentsTheme();
	const id = `wabou-tooltip-${++tooltipId}`;
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const open = () => !props.disabled && (props.open ?? uncontrolledOpen());
	const setOpen = (next) => {
		if (props.disabled) next = false;
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next);
	};
	const delay = createDelayedOpenController({
		openDelay: () => props.openDelay ?? 500,
		closeDelay: () => props.closeDelay ?? 80,
		setOpen
	});
	onCleanup(delay.dispose);
	return createComponent$1(Popover, {
		get open() {
			return open();
		},
		onOpenChange: (next) => !next && delay.closeNow(),
		get placement() {
			return props.placement ?? "top";
		},
		get offset() {
			return props.offset ?? 8;
		},
		contentRole: "presentation",
		popupRole: "tooltip",
		outsidePointerStrategy: "passthrough",
		contentInteractionBlocked: true,
		closeOnEscape: true,
		restoreFocus: false,
		get contentClass() {
			return mergeClasses("max-w-xs rounded-lg border border-subtle bg-surface px-2 py-1.5", props.contentClass);
		},
		get contentShadows() {
			return componentsElevation(theme(), "floating");
		},
		get motion() {
			return props.motion;
		},
		trigger: (popover) => props.trigger({
			ref: popover.ref,
			onPointerEnter: delay.scheduleOpen,
			onPointerLeave: delay.scheduleClose,
			onFocus: delay.openNow,
			onBlur: delay.closeNow,
			onKeyDown: (event) => {
				if (event.key === "Escape") delay.closeNow();
			}
		}),
		get children() {
			return createComponent$1(TooltipContent, {
				id,
				get shortcut() {
					return props.shortcut;
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
export { Tooltip, TooltipContent, createDelayedOpenController as createTooltipDelayController };

//# sourceMappingURL=tooltip.mjs.map