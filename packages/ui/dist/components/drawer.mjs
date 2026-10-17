import { useReducedMotion } from "../animation/config.mjs";
import { animate } from "../animation.mjs";
import { Text, View } from "../primitives/view.mjs";
import { createMeasuredSize } from "../primitives/measure.mjs";
import { Modal } from "../primitives/modal.mjs";
import { componentsElevation, useComponentsTheme } from "./theme.mjs";
import { Button } from "./button.mjs";
import { rgba } from "@wabou/core";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createContext, createEffect, createSignal, omit, onCleanup, untrack, useContext } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/drawer.tsx
const DrawerContext = createContext();
function useDrawer() {
	const context = useContext(DrawerContext);
	if (!context) throw new Error("Drawer child must be used inside Drawer");
	return context;
}
const drawerGeometry = (direction) => match(direction).with("left", () => ({
	backdrop: {
		"align-items": "stretch",
		"justify-content": "flex-start"
	},
	content: "h-full w-[420px] max-w-[80%] rounded-xl border-r",
	motion: { fromX: -48 }
})).with("right", () => ({
	backdrop: {
		"align-items": "stretch",
		"justify-content": "flex-end"
	},
	content: "h-full w-[420px] max-w-[80%] rounded-xl border-l",
	motion: { fromX: 48 }
})).with("top", () => ({
	backdrop: {
		"align-items": "flex-start",
		"justify-content": "stretch"
	},
	content: "w-full max-h-[80%] rounded-xl border-b",
	motion: { fromY: -48 }
})).with("bottom", () => ({
	backdrop: {
		"align-items": "flex-end",
		"justify-content": "stretch"
	},
	content: "w-full max-h-[80%] rounded-xl border-t",
	motion: { fromY: 48 }
})).exhaustive();
function drawerDragOffset(direction, rawDelta) {
	const outwardSign = direction === "right" || direction === "bottom" ? 1 : -1;
	return outwardSign * Math.max(0, outwardSign * rawDelta);
}
function drawerShouldDismiss(offset, size, threshold) {
	const dismissDistance = size > 0 ? size * threshold : 80;
	return Math.abs(offset) >= dismissDistance;
}
/** A focus-isolated edge drawer with a captured native drag-to-dismiss gesture. */
function Drawer(props) {
	const theme = useComponentsTheme();
	const reducedMotion = useReducedMotion();
	const measured = createMeasuredSize();
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(untrack(() => props.defaultOpen ?? false));
	const [dragOffset, setDragOffset] = createSignal(0, { ownedWrite: true });
	const [dragging, setDragging] = createSignal(false, { ownedWrite: true });
	const direction = () => props.direction ?? "bottom";
	const open = () => props.open ?? uncontrolledOpen();
	const axisSize = () => direction() === "left" || direction() === "right" ? measured.width() : measured.height();
	let startCoordinate = 0;
	let snapControls;
	const setOpen = (next, reason) => {
		if (next) setDragOffset(0);
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next, reason);
	};
	const snapBack = () => {
		snapControls?.cancel();
		if (reducedMotion()) {
			setDragOffset(0);
			return;
		}
		snapControls = animate(dragOffset(), 0, {
			duration: .22,
			ease: [
				.22,
				1,
				.36,
				1
			],
			onUpdate: setDragOffset
		});
	};
	const finishDrag = (cancelled = false) => {
		if (!dragging()) return;
		setDragging(false);
		const threshold = Math.min(.9, Math.max(.05, props.dismissThreshold ?? .25));
		if (!cancelled && (props.dismissible ?? true) && drawerShouldDismiss(dragOffset(), axisSize(), threshold)) {
			setOpen(false, "drag");
			return;
		}
		snapBack();
	};
	const coordinate = (event) => direction() === "left" || direction() === "right" ? event.clientX : event.clientY;
	const context = {
		direction,
		close: () => setOpen(false, "programmatic"),
		onPointerDown(event) {
			if (event.button !== 0 || !(props.dismissible ?? true)) return;
			snapControls?.cancel();
			startCoordinate = coordinate(event);
			setDragging(true);
			event.preventDefault();
		},
		onPointerMove(event) {
			if (!dragging() || event.buttons === 0) return;
			setDragOffset(drawerDragOffset(direction(), coordinate(event) - startCoordinate));
		},
		onPointerUp: () => finishDrag(false),
		onPointerCancel: () => finishDrag(true)
	};
	const placement = () => drawerGeometry(direction());
	const transform = (base) => [
		base[0],
		base[1],
		base[2],
		base[3],
		base[4] + (direction() === "left" || direction() === "right" ? dragOffset() : 0),
		base[5] + (direction() === "top" || direction() === "bottom" ? dragOffset() : 0)
	];
	createEffect(open, (isOpen) => {
		if (isOpen) setDragOffset(0);
	});
	onCleanup(() => snapControls?.cancel());
	return createComponent(DrawerContext, {
		value: context,
		get children() {
			return createComponent$1(Modal, mergeProps(props, {
				backdropFade: false,
				contentFade: false,
				get open() {
					return open();
				},
				onOpenChange: (next, reason) => setOpen(next, reason),
				get motion() {
					return {
						duration: .22,
						...placement().motion
					};
				},
				get backdropStyle() {
					return {
						"background-color": rgba(102),
						...placement().backdrop,
						...props.backdropStyle
					};
				},
				contentRef: (node) => {
					measured.ref(node);
					props.contentRef?.(node);
				},
				get contentClass() {
					return mergeClasses("relative min-w-0 min-h-0 flex flex-col border-subtle bg-surface", placement().content, props.contentClass);
				},
				contentTransform: transform,
				get contentShadows() {
					return memo(() => {
						return props.contentShadows === void 0;
					})() ? componentsElevation(theme(), "modal") : props.contentShadows;
				},
				get children() {
					return memo(() => {
						return typeof props.children === "function";
					})() ? props.children({ close: context.close }) : props.children;
				}
			}));
		}
	});
}
function DrawerHandle(props) {
	const drawer = useDrawer();
	const forwarded = omit(props, "class", "onPointerDown", "onPointerMove", "onPointerUp", "onPointerCancel", "onClick", "onKeyDown");
	const hitArea = () => match(drawer.direction()).with("left", () => "absolute right-0 top-0 w-8 h-full").with("right", () => "absolute left-0 top-0 w-8 h-full").with("top", "bottom", () => "w-full h-8").exhaustive();
	const indicator = () => match(drawer.direction()).with("left", "right", () => "w-1.5 h-20").with("top", "bottom", () => "w-20 h-1.5").exhaustive();
	return createComponent$1(View, mergeProps(forwarded, {
		role: "button",
		get "aria-label"() {
			return props["aria-label"] ?? "Drag or click to close drawer";
		},
		get focusOrder() {
			return props.focusOrder ?? 0;
		},
		get "class"() {
			return mergeClasses("flex flex-none items-center justify-center", hitArea(), props.class);
		},
		onPointerDown: (event) => {
			drawer.onPointerDown(event);
			props.onPointerDown?.(event);
		},
		onPointerMove: (event) => {
			drawer.onPointerMove(event);
			props.onPointerMove?.(event);
		},
		onPointerUp: (event) => {
			drawer.onPointerUp();
			props.onPointerUp?.(event);
		},
		onPointerCancel: (event) => {
			drawer.onPointerCancel();
			props.onPointerCancel?.(event);
		},
		onClick: (event) => {
			drawer.close();
			props.onClick?.(event);
		},
		onKeyDown: (event) => {
			if (event.key === "Enter" || event.key === " ") {
				event.preventDefault();
				drawer.close();
			}
			props.onKeyDown?.(event);
		},
		get children() {
			return createComponent$1(View, {
				"aria-hidden": "true",
				get "class"() {
					return mergeClasses("pointer-events-none rounded-full bg-strong", indicator());
				}
			});
		}
	}));
}
function DrawerHeader(props) {
	return createComponent$1(View, mergeProps(props, { get "class"() {
		return mergeClasses("flex flex-col gap-1 p-5", props.class);
	} }));
}
function DrawerFooter(props) {
	return createComponent$1(View, mergeProps(props, { get "class"() {
		return mergeClasses("mt-auto flex flex-col gap-2 p-5", props.class);
	} }));
}
function DrawerTitle(props) {
	return createComponent$1(Text, mergeProps(props, { get "class"() {
		return mergeClasses("text-lg font-semibold text-primary", props.class);
	} }));
}
function DrawerDescription(props) {
	return createComponent$1(Text, mergeProps(props, { get "class"() {
		return mergeClasses("whitespace-normal text-sm text-muted", props.class);
	} }));
}
function DrawerClose(props) {
	const drawer = useDrawer();
	const forwarded = omit(props, "onClick");
	return createComponent$1(Button, mergeProps(forwarded, { onClick: (event) => {
		drawer.close();
		props.onClick?.(event);
	} }));
}
//#endregion
export { Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHandle, DrawerHeader, DrawerTitle, drawerDragOffset, drawerShouldDismiss };

//# sourceMappingURL=drawer.mjs.map