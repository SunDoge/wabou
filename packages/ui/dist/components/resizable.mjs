import { View } from "../primitives/view.mjs";
import { createMeasuredSize } from "../primitives/measure.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createContext, createMemo, createSignal, useContext } from "solid-js";
import { applyRef, createComponent as createComponent$1 } from "@wabou/core/renderer";
//#region src/components/resizable.tsx
function finitePercentage(value, name) {
	if (!Number.isFinite(value) || value < 0 || value > 100) throw new RangeError(`${name} must be a finite percentage from 0 to 100`);
	return value;
}
function validateResizableSizes(panels, sizes) {
	if (panels.length < 2) throw new RangeError("resizable panels require at least two definitions");
	const ids = /* @__PURE__ */ new Set();
	const candidates = panels.map((panel) => {
		if (!panel.id || ids.has(panel.id)) throw new Error(`resizable panel id must be unique: ${panel.id}`);
		ids.add(panel.id);
		const min = finitePercentage(panel.minSize ?? 0, `${panel.id}.minSize`);
		const max = finitePercentage(panel.maxSize ?? 100, `${panel.id}.maxSize`);
		if (min > max) throw new RangeError(`${panel.id}.minSize exceeds maxSize`);
		const candidate = finitePercentage(sizes?.[panel.id] ?? panel.defaultSize, `${panel.id}.size`);
		if (candidate < min || candidate > max) throw new RangeError(`${panel.id}.size is outside its min/max range`);
		return candidate;
	});
	const total = candidates.reduce((sum, value) => sum + value, 0);
	if (Math.abs(total - 100) > .001) throw new RangeError(`resizable panel sizes must total 100; received ${total}`);
	return Object.fromEntries(panels.map((panel, index) => [panel.id, candidates[index]]));
}
function createResizablePanelState(options) {
	const defaults = validateResizableSizes(options.panels, options.defaultValue);
	const definitions = new Map(options.panels.map((panel) => [panel.id, panel]));
	const state = createControllableState({
		value: () => {
			const value = options.value?.();
			return value === void 0 ? void 0 : validateResizableSizes(options.panels, value);
		},
		defaultValue: defaults,
		onChange: options.onValueChange
	});
	const requirePanel = (id) => {
		const panel = definitions.get(id);
		if (!panel) throw new Error(`unknown resizable panel: ${id}`);
		return panel;
	};
	const size = (id) => {
		requirePanel(id);
		const value = state.value()[id];
		if (!Number.isFinite(value)) throw new Error(`missing resizable panel size: ${id}`);
		return value;
	};
	const pairRange = (before, after) => {
		const beforePanel = requirePanel(before);
		const afterPanel = requirePanel(after);
		if (before === after) throw new Error("resizable handle requires two panels");
		const pair = size(before) + size(after);
		return {
			min: Math.max(beforePanel.minSize ?? 0, pair - (afterPanel.maxSize ?? 100)),
			max: Math.min(beforePanel.maxSize ?? 100, pair - (afterPanel.minSize ?? 0))
		};
	};
	const resizePair = (before, after, beforeSize) => {
		const range = pairRange(before, after);
		const pair = size(before) + size(after);
		const nextBefore = Math.max(range.min, Math.min(range.max, beforeSize));
		const nextAfter = pair - nextBefore;
		const current = state.value();
		if (current[before] === nextBefore && current[after] === nextAfter) return false;
		return state.set({
			...current,
			[before]: nextBefore,
			[after]: nextAfter
		});
	};
	return {
		panels: options.panels,
		sizes: state.value,
		size,
		pairRange,
		resizePair,
		resetPair(before, after) {
			const beforeDefault = defaults[before];
			const afterDefault = defaults[after];
			if (beforeDefault === void 0 || afterDefault === void 0) {
				pairRange(before, after);
				return false;
			}
			const pair = size(before) + size(after);
			return resizePair(before, after, pair * (beforeDefault / (beforeDefault + afterDefault)));
		}
	};
}
const ResizableContext = createContext();
function useResizable() {
	const context = useContext(ResizableContext);
	if (!context) throw new Error("Resizable parts must be inside ResizablePanelGroup");
	return context;
}
function ResizablePanelGroup(props) {
	let measuredWidth = 0;
	let measuredHeight = 0;
	const measured = createMeasuredSize({ onChange(size) {
		measuredWidth = size.width;
		measuredHeight = size.height;
	} });
	const state = createResizablePanelState({
		panels: props.panels,
		value: () => props.value,
		defaultValue: props.defaultValue,
		onValueChange: props.onValueChange
	});
	const direction = () => props.direction ?? "horizontal";
	return createComponent$1(ResizableContext, {
		value: {
			direction,
			state,
			axisSize: () => direction() === "horizontal" ? measuredWidth : measuredHeight
		},
		get children() {
			return createComponent$1(View, {
				ref(r$) {
					var _ref$ = measured.ref;
					typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
				},
				role: "group",
				get "aria-label"() {
					return props["aria-label"];
				},
				get "class"() {
					return mergeClasses("w-full h-full min-w-0 min-h-0 flex overflow-hidden", direction() === "horizontal" ? "flex-row" : "flex-col", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function ResizablePanel(props) {
	const context = useResizable();
	const style = createMemo(() => context.direction() === "horizontal" ? { width: `${context.state.size(props.id)}%` } : { height: `${context.state.size(props.id)}%` });
	return createComponent$1(View, {
		role: "group",
		get "aria-label"() {
			return props.id;
		},
		get "class"() {
			return mergeClasses("min-w-0 min-h-0 flex-none overflow-hidden", props.class);
		},
		get style() {
			return style();
		},
		get children() {
			return props.children;
		}
	});
}
function ResizableHandle(props) {
	const context = useResizable();
	const [dragging, setDragging] = createSignal(false);
	const [hovered, setHovered] = createSignal(false);
	let startCoordinate = 0;
	let startSize = 0;
	const coordinate = (event) => context.direction() === "horizontal" ? event.clientX : event.clientY;
	const range = () => context.state.pairRange(props.before, props.after);
	const moveTo = (value) => context.state.resizePair(props.before, props.after, value);
	const onPointerDown = (event) => {
		if (event.button !== 0) return;
		event.preventDefault();
		startCoordinate = coordinate(event);
		startSize = context.state.size(props.before);
		setDragging(true);
	};
	const onPointerMove = (event) => {
		if (!dragging() || event.buttons === 0) return;
		const axisSize = context.axisSize();
		if (axisSize <= 0) return;
		moveTo(startSize + (coordinate(event) - startCoordinate) / axisSize * 100);
	};
	const stopDragging = () => setDragging(false);
	const onKeyDown = (event) => {
		const step = Math.max(.1, props.keyboardStep ?? 2);
		const current = context.state.size(props.before);
		const next = event.key === "Home" ? range().min : event.key === "End" ? range().max : event.key === (context.direction() === "horizontal" ? "ArrowLeft" : "ArrowUp") ? current - step : event.key === (context.direction() === "horizontal" ? "ArrowRight" : "ArrowDown") ? current + step : void 0;
		if (next === void 0) return;
		event.preventDefault();
		moveTo(next);
	};
	return createComponent$1(View, {
		role: "separator",
		get "aria-label"() {
			return props["aria-label"];
		},
		get "aria-valuemin"() {
			return range().min;
		},
		get "aria-valuemax"() {
			return range().max;
		},
		get "aria-valuenow"() {
			return context.state.size(props.before);
		},
		get "aria-valuetext"() {
			return `${Math.round(context.state.size(props.before))} percent`;
		},
		focusOrder: 0,
		get "class"() {
			return mergeClasses("flex-none rounded-sm", context.direction() === "horizontal" ? "w-2 h-full" : "w-full h-2", dragging() || hovered() ? "bg-accent" : "bg-control", props.class);
		},
		onPointerEnter: () => setHovered(true),
		onPointerLeave: () => setHovered(false),
		onPointerDown,
		onPointerMove,
		onPointerUp: stopDragging,
		onPointerCancel: stopDragging,
		onDblClick: () => context.state.resetPair(props.before, props.after),
		onKeyDown
	});
}
//#endregion
export { ResizableHandle, ResizablePanel, ResizablePanelGroup, createResizablePanelState, validateResizableSizes };

//# sourceMappingURL=resizable.mjs.map