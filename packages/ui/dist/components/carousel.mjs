import { useReducedMotion } from "../animation/config.mjs";
import { animate } from "../animation.mjs";
import { Icon, View, translate2d as translate2d$1 } from "../primitives/view.mjs";
import { createMeasuredSize } from "../primitives/measure.mjs";
import { Button } from "./button.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createContext, createEffect, createSignal, createUniqueId, omit, onCleanup, useContext } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import arrowDown from "lucide-static/icons/arrow-down.svg?raw";
import arrowLeft from "lucide-static/icons/arrow-left.svg?raw";
import arrowRight from "lucide-static/icons/arrow-right.svg?raw";
import arrowUp from "lucide-static/icons/arrow-up.svg?raw";
//#region src/components/carousel.tsx
const CarouselContext = createContext();
function useCarousel() {
	const context = useContext(CarouselContext);
	if (!context) throw new Error("Carousel child must be used inside Carousel");
	return context;
}
function normalizeCarouselIndex(index, count, loop) {
	if (count <= 0) return 0;
	if (loop) return (Math.trunc(index) % count + count) % count;
	return Math.min(count - 1, Math.max(0, Math.trunc(index)));
}
/** A native snapping carousel with captured pointer dragging and keyboard navigation. */
function Carousel(props) {
	const items = [];
	const [revision, setRevision] = createSignal(0, { ownedWrite: true });
	const state = createControllableState({
		value: () => props.index,
		defaultValue: normalizeCarouselIndex(props.defaultIndex ?? 0, 1, false),
		onChange: props.onIndexChange
	});
	const count = () => {
		revision();
		return items.length;
	};
	const selectedIndex = () => normalizeCarouselIndex(state.value(), count(), props.loop ?? false);
	const scrollTo = (index) => {
		if (count() === 0) return;
		state.set(normalizeCarouselIndex(index, count(), props.loop ?? false));
	};
	const context = {
		orientation: () => props.orientation ?? "horizontal",
		selectedIndex,
		itemCount: count,
		canScrollPrevious: () => count() > 1 && ((props.loop ?? false) || selectedIndex() > 0),
		canScrollNext: () => count() > 1 && ((props.loop ?? false) || selectedIndex() < count() - 1),
		scrollPrevious: () => scrollTo(selectedIndex() - 1),
		scrollNext: () => scrollTo(selectedIndex() + 1),
		scrollTo,
		register(id) {
			items.push(id);
			setRevision((value) => value + 1);
			return () => {
				const index = items.indexOf(id);
				if (index >= 0) items.splice(index, 1);
				setRevision((value) => value + 1);
			};
		}
	};
	props.setApi?.(context);
	return createComponent(CarouselContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				role: "group",
				get "aria-label"() {
					return props["aria-label"];
				},
				get "class"() {
					return mergeClasses("relative min-w-0 min-h-0", props.class);
				},
				onKeyDown: (event) => {
					if (match({
						orientation: context.orientation(),
						key: event.key
					}).with({
						orientation: "horizontal",
						key: "ArrowLeft"
					}, () => {
						context.scrollPrevious();
						return true;
					}).with({
						orientation: "horizontal",
						key: "ArrowRight"
					}, () => {
						context.scrollNext();
						return true;
					}).with({
						orientation: "vertical",
						key: "ArrowUp"
					}, () => {
						context.scrollPrevious();
						return true;
					}).with({
						orientation: "vertical",
						key: "ArrowDown"
					}, () => {
						context.scrollNext();
						return true;
					}).otherwise(() => false)) event.preventDefault();
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function CarouselContent(props) {
	const carousel = useCarousel();
	const measured = createMeasuredSize();
	const reducedMotion = useReducedMotion();
	const [offset, setOffset] = createSignal(0, { ownedWrite: true });
	const [dragDelta, setDragDelta] = createSignal(0, { ownedWrite: true });
	const [dragging, setDragging] = createSignal(false, { ownedWrite: true });
	let startCoordinate = 0;
	let controls;
	const axisSize = () => carousel.orientation() === "horizontal" ? measured.width() : measured.height();
	const targetOffset = () => -carousel.selectedIndex() * axisSize();
	const coordinate = (event) => carousel.orientation() === "horizontal" ? event.clientX : event.clientY;
	const settle = () => {
		controls?.cancel();
		const target = targetOffset();
		if (reducedMotion() || Math.abs(target - offset()) < .5) {
			setOffset(target);
			return;
		}
		controls = animate(offset(), target, {
			duration: .28,
			ease: [
				.22,
				1,
				.36,
				1
			],
			onUpdate: setOffset
		});
	};
	createEffect(() => [targetOffset(), measured.measured()], () => {
		if (!dragging()) settle();
	});
	onCleanup(() => controls?.cancel());
	const stopDragging = (cancelled = false) => {
		if (!dragging()) return;
		const delta = dragDelta();
		const threshold = Math.max(12, axisSize() * (props.dragThreshold ?? .15));
		setOffset(offset() + delta);
		setDragDelta(0);
		setDragging(false);
		if (!cancelled && Math.abs(delta) >= threshold) {
			if (delta < 0) carousel.scrollNext();
			else carousel.scrollPrevious();
		}
		requestAnimationFrame(settle);
	};
	const forwarded = omit(props, "trackClass", "dragThreshold", "children", "class", "ref", "onPointerDown", "onPointerMove", "onPointerUp", "onPointerCancel");
	return createComponent$1(View, mergeProps(forwarded, {
		ref: (node) => {
			measured.ref(node);
			props.ref?.(node);
		},
		get "class"() {
			return mergeClasses("min-w-0 min-h-0 overflow-hidden", props.class);
		},
		onPointerDown: (event) => {
			if (event.button !== 0 || axisSize() <= 0) return;
			controls?.cancel();
			startCoordinate = coordinate(event);
			setDragDelta(0);
			setDragging(true);
			event.preventDefault();
			props.onPointerDown?.(event);
		},
		onPointerMove: (event) => {
			if (dragging() && event.buttons !== 0) setDragDelta(coordinate(event) - startCoordinate);
			props.onPointerMove?.(event);
		},
		onPointerUp: (event) => {
			stopDragging();
			props.onPointerUp?.(event);
		},
		onPointerCancel: (event) => {
			stopDragging(true);
			props.onPointerCancel?.(event);
		},
		get children() {
			return createComponent$1(View, {
				get "class"() {
					return mergeClasses("flex w-full h-full flex-none", carousel.orientation() === "horizontal" ? "flex-row" : "flex-col", props.trackClass);
				},
				get transform() {
					return memo(() => {
						return carousel.orientation() === "horizontal";
					})() ? translate2d$1(offset() + dragDelta(), 0) : translate2d$1(0, offset() + dragDelta());
				},
				get children() {
					return props.children;
				}
			});
		}
	}));
}
function CarouselItem(props) {
	const carousel = useCarousel();
	const id = createUniqueId();
	const unregister = carousel.register(id);
	onCleanup(unregister);
	return createComponent$1(View, mergeProps(props, {
		role: "group",
		get "aria-label"() {
			return props["aria-label"];
		},
		get "class"() {
			return mergeClasses("min-w-0 min-h-0 flex-none", carousel.orientation() === "horizontal" ? "w-full" : "h-full", props.class);
		}
	}));
}
function CarouselNavigationButton(props) {
	const carousel = useCarousel();
	const previous = props.direction === "previous";
	const forwarded = omit(props, "direction", "children", "onClick");
	return createComponent$1(Button, mergeProps(forwarded, {
		get "aria-label"() {
			return props["aria-label"] ?? (previous ? "Previous slide" : "Next slide");
		},
		get variant() {
			return props.variant ?? "outline";
		},
		get size() {
			return props.size ?? "icon";
		},
		get disabled() {
			return props.disabled ?? (previous ? !carousel.canScrollPrevious() : !carousel.canScrollNext());
		},
		onClick: (event) => {
			if (previous) carousel.scrollPrevious();
			else carousel.scrollNext();
			props.onClick?.(event);
		},
		get children() {
			return props.children ?? createComponent$1(Icon, {
				"aria-hidden": "true",
				get source() {
					return match({
						previous,
						orientation: carousel.orientation()
					}).with({
						orientation: "vertical",
						previous: true
					}, () => arrowUp).with({
						orientation: "vertical",
						previous: false
					}, () => arrowDown).with({ previous: true }, () => arrowLeft).otherwise(() => arrowRight);
				},
				size: 16
			});
		}
	}));
}
function CarouselPrevious(props) {
	return createComponent$1(CarouselNavigationButton, mergeProps(props, { direction: "previous" }));
}
function CarouselNext(props) {
	return createComponent$1(CarouselNavigationButton, mergeProps(props, { direction: "next" }));
}
//#endregion
export { Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious, normalizeCarouselIndex };

//# sourceMappingURL=carousel.mjs.map