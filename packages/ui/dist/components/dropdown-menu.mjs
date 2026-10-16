import { Icon, Text, View } from "../primitives/view.mjs";
import { Popover } from "../primitives/popover.mjs";
import { componentsDisabledItemClass, componentsElevation, componentsSurfaceClass, useComponentsTheme } from "./theme.mjs";
import { createTypeahead } from "../primitives/interactions/typeahead.mjs";
import { moveMenuHighlight } from "./menu-state.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, createEffect, createSignal } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import check from "lucide-static/icons/check.svg?raw";
//#region src/components/dropdown-menu.tsx
/** A compact action menu with native focus, typeahead, and overlay routing. */
function DropdownMenu(props) {
	const theme = useComponentsTheme();
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const [highlighted, setHighlighted] = createSignal();
	const open = () => props.open ?? uncontrolledOpen();
	const hasLeadingSlot = () => props.items.some((item) => item.icon !== void 0 || item.checked !== void 0);
	const typeahead = createTypeahead();
	let trigger;
	let content;
	let openEdge = "first";
	let wasOpen = false;
	const setOpen = (next, edge = "first") => {
		openEdge = edge;
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next);
	};
	createEffect(() => ({
		open: open(),
		items: props.items,
		restoreFocus: props.restoreFocus ?? true
	}), ({ open: isOpen, items, restoreFocus }) => {
		if (isOpen && !wasOpen) {
			setHighlighted(moveMenuHighlight(items, void 0, openEdge));
			requestAnimationFrame(() => content?.focus());
		} else if (!isOpen && wasOpen) {
			setHighlighted(void 0);
			typeahead.reset();
			if (restoreFocus) requestAnimationFrame(() => trigger?.focus());
		}
		wasOpen = isOpen;
	});
	const select = (id) => {
		const item = props.items.find((candidate) => candidate.id === id);
		if (!item || item.disabled) return false;
		item.onSelect?.();
		props.onAction?.(item.id);
		setOpen(false);
		return true;
	};
	const move = (direction) => {
		const next = moveMenuHighlight(props.items, highlighted(), direction);
		if (next === void 0) return false;
		setHighlighted(next);
		return true;
	};
	const handleMenuKey = (event) => {
		props.onContentKeyDown?.(event);
		if (event.defaultPrevented) return;
		if (match(event.key).with("ArrowDown", () => move("next")).with("ArrowUp", () => move("previous")).with("Home", () => move("first")).with("End", () => move("last")).with("Enter", () => select(highlighted())).with(" ", () => select(highlighted())).with("Escape", () => {
			setOpen(false);
			return true;
		}).otherwise((key) => {
			const item = typeahead.search(props.items, key, highlighted());
			if (item) setHighlighted(item.id);
			return item !== void 0;
		})) event.preventDefault();
	};
	return createComponent$1(Popover, {
		contentRole: "presentation",
		popupRole: "menu",
		get open() {
			return open();
		},
		onOpenChange: (next) => setOpen(next),
		get placement() {
			return props.placement ?? "bottom-end";
		},
		get restoreFocus() {
			return props.restoreFocus;
		},
		get outsidePointerStrategy() {
			return props.outsidePointerStrategy;
		},
		get anchorPoint() {
			return props.anchorPoint;
		},
		get contentClass() {
			return mergeClasses("w-56 p-1 flex flex-col gap-0.5", componentsSurfaceClass("floating"), props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		trigger: (popover) => props.trigger({
			ref: (node) => {
				trigger = node;
				popover.ref(node);
			},
			onClick: popover.onClick,
			onKeyDown: (event) => {
				popover.onKeyDown(event);
				if (match(event.key).with("ArrowDown", () => {
					setOpen(true, "first");
					return true;
				}).with("ArrowUp", () => {
					setOpen(true, "last");
					return true;
				}).otherwise(() => false)) event.preventDefault();
			},
			"aria-haspopup": "menu",
			get "aria-expanded"() {
				return open();
			}
		}),
		get children() {
			return createComponent$1(View, {
				ref: (node) => content = node,
				role: "menu",
				get "aria-label"() {
					return props["aria-label"];
				},
				focusOrder: 0,
				class: "min-w-0 flex flex-col gap-0.5",
				onKeyDown: handleMenuKey,
				get children() {
					return createComponent$1(For, {
						get each() {
							return props.items;
						},
						keyed: false,
						children: (item) => [memo(() => {
							return memo(() => {
								return !!item().separatorBefore;
							})() ? createComponent$1(View, {
								role: "presentation",
								class: "h-px flex-none my-1 bg-control"
							}) : item().separatorBefore;
						}), createComponent$1(View, {
							role: "menuitem",
							get "aria-label"() {
								return item().label;
							},
							get "aria-disabled"() {
								return item().disabled;
							},
							get "aria-checked"() {
								return item().checked;
							},
							get "class"() {
								return mergeClasses("w-full min-h-8 flex-none px-2 py-1.5 flex flex-row items-center gap-2 rounded-md", highlighted() === item().id && !item().disabled ? "bg-control-hover" : "bg-transparent", item().destructive ? "text-danger-primary" : "text-primary", componentsDisabledItemClass(item().disabled ?? false));
							},
							onPointerMove: () => !item().disabled && setHighlighted(item().id),
							onClick: () => select(item().id),
							get children() {
								return [
									memo(() => {
										return memo(() => {
											return !!hasLeadingSlot();
										})() ? createComponent$1(View, {
											role: "presentation",
											class: "w-4 h-4 flex-none flex items-center justify-center",
											get children() {
												return memo(() => {
													return !!item().checked;
												})() ? createComponent$1(Icon, {
													source: check,
													class: "text-accent",
													size: 14
												}) : memo(() => {
													return !!item().icon;
												})() ? createComponent$1(Icon, {
													get source() {
														return item().icon ?? "";
													},
													size: 14
												}) : null;
											}
										}) : hasLeadingSlot();
									}),
									createComponent$1(View, {
										class: "min-w-0 flex-1 flex flex-col justify-center",
										get children() {
											return [createComponent$1(Text, {
												class: "min-w-0 truncate text-sm",
												get children() {
													return item().label;
												}
											}), memo(() => {
												return memo(() => {
													return !!item().description;
												})() ? createComponent$1(Text, {
													class: "min-w-0 truncate text-xs text-muted",
													get children() {
														return item().description;
													}
												}) : item().description;
											})];
										}
									}),
									memo(() => {
										return memo(() => {
											return !!item().shortcut;
										})() ? createComponent$1(Text, {
											"aria-hidden": "true",
											class: "flex-none rounded border border-subtle bg-surface px-1.5 py-0.5 text-xs text-muted",
											get children() {
												return item().shortcut;
											}
										}) : item().shortcut;
									})
								];
							}
						})]
					});
				}
			});
		}
	});
}
//#endregion
export { DropdownMenu, moveMenuHighlight };

//# sourceMappingURL=dropdown-menu.mjs.map