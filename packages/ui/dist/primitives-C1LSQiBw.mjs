import { t as __exportAll } from "./rolldown-runtime-D7D4PA-g.mjs";
import { PathBuilder } from "@wabou/core";
import { number, px, rotate2d, rotate2d as rotate2d$1, scale2d, translate2d } from "@wabou/core/style";
import { animateValue } from "motion-dom";
import { For, Show, createComponent, createContext, createEffect, createMemo, createSignal, omit, onCleanup, untrack, useContext } from "solid-js";
import { Portal, TEXT_BEHAVIOR, applyRef, createComponent as createComponent$1, createElement, insert, memo, mergeProps, ref, spread, useHost as useHost$1 } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import { arrow, autoPlacement, computePosition, flip, offset, shift, size } from "@floating-ui/core";
import { formatNodeKey } from "@wabou/core/protocol";
//#region src/animation/index.ts
var Controls = class {
	backend;
	constructor(backend) {
		this.backend = backend;
	}
	get time() {
		return this.backend.time;
	}
	set time(value) {
		this.backend.time = value;
	}
	get speed() {
		return this.backend.speed;
	}
	set speed(value) {
		this.backend.speed = value;
	}
	get duration() {
		return this.backend.duration;
	}
	get state() {
		const state = this.backend.state;
		if (state === "running" || state === "paused" || state === "finished") return state;
		return "idle";
	}
	get finished() {
		return this.backend.finished.then(() => void 0);
	}
	play() {
		this.backend.play();
	}
	pause() {
		this.backend.pause();
	}
	stop() {
		this.backend.stop();
	}
	cancel() {
		this.backend.cancel();
	}
	complete() {
		this.backend.complete();
	}
	then(onfulfilled, onrejected) {
		return this.finished.then(onfulfilled, onrejected);
	}
};
function normalizeOptions(options) {
	const normalized = { ...options };
	for (const key of [
		"delay",
		"duration",
		"repeatDelay"
	]) {
		const seconds = options[key];
		if (typeof seconds === "number") normalized[key] = seconds * 1e3;
	}
	if (typeof options.visualDuration === "number") normalized.visualDuration = options.visualDuration * 1e3;
	return normalized;
}
/** Animate between two scalar values. */
function animate(from, to, options = {}) {
	return animateKeyframes([from, to], options);
}
/** Animate through two or more scalar keyframes. */
function animateKeyframes(keyframes, options = {}) {
	return new Controls(animateValue({
		...normalizeOptions(options),
		keyframes: [...keyframes]
	}));
}
const read = (value, fallback) => typeof value === "function" ? value() : value ?? fallback;
/**
* Lifecycle-owned scalar transition that retargets from its current value.
*
* Unlike a one-shot animation, changing `target` while a run is active does
* not restart from the previous keyframe. This makes it suitable for rapidly
* toggled disclosure, hover and selection state.
*/
function createTransition(target, options = {}) {
	const [value, setValue] = createSignal(untrack(target));
	const [state, setState] = createSignal("idle");
	let controls;
	let generation = 0;
	const stop = () => {
		generation++;
		controls?.stop();
		controls = void 0;
		setState("idle");
	};
	const jump = (next) => {
		stop();
		setValue(next);
		options.onUpdate?.(next);
		options.onComplete?.(next);
	};
	createEffect(() => [
		target(),
		read(options.reducedMotion, false),
		untrack(value)
	], ([next, reduced, current]) => {
		if (Object.is(next, current)) return;
		if (reduced || options.type === false || options.duration === 0) {
			jump(next);
			return;
		}
		const run = ++generation;
		controls?.stop();
		setState("running");
		const { reducedMotion: _reducedMotion, onUpdate, onComplete, ...animationOptions } = options;
		controls = animate(current, next, {
			...animationOptions,
			onUpdate(current) {
				if (run !== generation) return;
				setValue(current);
				onUpdate?.(current);
			},
			onComplete() {
				if (run !== generation) return;
				controls = void 0;
				setValue(next);
				setState("finished");
				onComplete?.(next);
			}
		});
	});
	onCleanup(stop);
	return {
		value,
		state,
		jump,
		stop
	};
}
/**
* Lifecycle-owned repeating scalar animation for Solid components.
*
* The controls stop automatically with the current Solid owner.
*/
function createLoop(options = {}) {
	const from = options.from ?? 0;
	const to = options.to ?? 1;
	const { from: _from, to: _to, onUpdate, ...animationOptions } = options;
	const [value, setValue] = createSignal(from);
	const controls = animate(from, to, {
		duration: 1,
		ease: "linear",
		repeat: Infinity,
		...animationOptions,
		onUpdate(next) {
			setValue(next);
			onUpdate?.(next);
		}
	});
	onCleanup(() => controls.stop());
	return {
		value,
		controls
	};
}
/** Repeating center-pivoted rotation backed by Motion value animation. */
function createRotation(options = {}) {
	const loop = createLoop({
		...options,
		from: options.from ?? 0,
		to: options.to ?? Math.PI * 2
	});
	return {
		...loop,
		angle: loop.value,
		transform: () => rotate2d(loop.value())
	};
}
/** Repeating from→to→from value animation with automatic cleanup. */
function createPulse(options = {}) {
	const from = options.from ?? .5;
	const to = options.to ?? 1;
	const { from: _from, to: _to, onUpdate, ...animationOptions } = options;
	const [value, setValue] = createSignal(from);
	const controls = animateKeyframes([
		from,
		to,
		from
	], {
		duration: 1,
		ease: "easeInOut",
		repeat: Infinity,
		...animationOptions,
		onUpdate(next) {
			setValue(next);
			onUpdate?.(next);
		}
	});
	onCleanup(() => controls.stop());
	return {
		value,
		controls
	};
}
//#endregion
//#region src/primitives/animation-frame.ts
/**
* Drive explicit paint state from the native host's animation clock.
* Return `false` to stop scheduling frames before the owner is disposed.
*/
function createAnimationFrame(callback) {
	let frame = 0;
	let active = true;
	const tick = (timestamp) => {
		if (!active) return;
		if (callback(timestamp) === false) {
			active = false;
			return;
		}
		frame = requestAnimationFrame(tick);
	};
	frame = requestAnimationFrame(tick);
	const stop = () => {
		if (!active) return;
		active = false;
		cancelAnimationFrame(frame);
	};
	onCleanup(stop);
	return stop;
}
//#endregion
//#region src/primitives/focus.ts
let keyboardModality = false;
/** Reactive focus state and event bindings for a single target. */
function createFocus() {
	const [focused, setFocused] = createSignal(false);
	const [focusVisible, setFocusVisible] = createSignal(false);
	return {
		focused,
		focusVisible,
		pointerModality: () => {
			keyboardModality = false;
			if (focused()) setFocusVisible(false);
		},
		keyboardModality: () => {
			keyboardModality = true;
			if (focused()) setFocusVisible(true);
		},
		bindings: {
			onFocus: (event) => {
				setFocused(true);
				setFocusVisible(event?.payload?.focusVisible ?? keyboardModality);
			},
			onBlur: () => {
				setFocused(false);
				setFocusVisible(false);
			}
		}
	};
}
/** Reactive equivalent of `:focus-within`, using bubbling focus events. */
function createFocusWithin() {
	const [focusWithin, setFocusWithin] = createSignal(false);
	return {
		focusWithin,
		bindings: {
			onFocusIn: () => setFocusWithin(true),
			onFocusOut: () => setFocusWithin(false)
		}
	};
}
//#endregion
//#region src/primitives/hover.ts
/** Reactive hover state and event bindings for a single target. */
function createHover() {
	const [hovered, setHovered] = createSignal(false);
	return {
		hovered,
		bindings: {
			onPointerEnter: () => setHovered(true),
			onPointerLeave: () => setHovered(false)
		}
	};
}
//#endregion
//#region src/primitives/press.ts
/** Reactive pointer-press state with disabled-aware activation. */
function createPress(options = {}) {
	const [pressed, setPressed] = createSignal(false);
	const disabled = () => typeof options.disabled === "function" ? options.disabled() : !!options.disabled;
	const release = () => setPressed(false);
	return {
		pressed,
		bindings: {
			onPointerDown: () => {
				if (!disabled()) setPressed(true);
			},
			onPointerUp: release,
			onPointerCancel: release,
			onPointerLeave: release,
			onClick: (event) => {
				if (!disabled()) options.onPress?.(event);
			}
		}
	};
}
/** CSS `:active`-like state without an activation callback. */
function createActive(disabled) {
	const { pressed, bindings } = createPress({ disabled });
	return {
		active: pressed,
		bindings
	};
}
//#endregion
//#region src/primitives/button.tsx
const ACCENTS = {
	neutral: "#475569",
	sky: "#0284c7",
	amber: "#d97706"
};
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
		"user-select": props.selectable ? "text" : "none",
		cursor: disabled() ? "not-allowed" : "pointer",
		"outline-width": state().focusVisible ? "2px" : "0px",
		"outline-offset": "2px",
		"outline-color": "#38bdf8",
		"outline-style": "solid"
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
	var _el$ = createElement("button");
	var _ref$ = props.ref;
	typeof _ref$ === "function" || Array.isArray(_ref$) ? ref(() => {
		return _ref$;
	}, _el$) : props.ref = _el$;
	spread(_el$, mergeProps({
		get disabled() {
			return disabled();
		},
		get ["aria-disabled"]() {
			return disabled();
		},
		get focusOrder() {
			return resolveButtonFocusOrder(disabled(), props.focusOrder);
		},
		get role() {
			return props.role ?? "button";
		},
		get ["aria-haspopup"]() {
			return props["aria-haspopup"];
		},
		get ["aria-expanded"]() {
			return props["aria-expanded"];
		},
		get ["aria-controls"]() {
			return props["aria-controls"];
		},
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-checked"]() {
			return props["aria-checked"];
		},
		get ["aria-current"]() {
			return props["aria-current"];
		},
		get ["aria-selected"]() {
			return props["aria-selected"];
		},
		get ["aria-pressed"]() {
			return props["aria-pressed"];
		},
		get ["class"]() {
			return memo(() => {
				return typeof props.class === "function";
			})() ? props.class(state()) : props.class;
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
		}
	}, () => {
		return primitive.bindings;
	}), true);
	insert(_el$, () => {
		return props.children;
	});
	return _el$;
}
/**
* An explicit external-link interaction.
*
* Wabou does not assign browser behavior to an `a` tag or `href` attribute;
* the JS primitive owns activation while Rust only executes `openUrl`.
*/
function Link(props) {
	const host = useHost$1();
	return createComponent$1(Button, mergeProps(props, {
		role: "link",
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented) host.system.openUrl(props.url);
		}
	}));
}
//#endregion
//#region src/primitives/measure.ts
function validateSizeQuery(query) {
	const entries = [
		["minWidth", query.minWidth],
		["maxWidth", query.maxWidth],
		["minHeight", query.minHeight],
		["maxHeight", query.maxHeight]
	];
	for (const [name, value] of entries) if (value !== void 0 && (!Number.isFinite(value) || value < 0)) throw new RangeError(`${name} must be a finite non-negative number`);
	if (query.minWidth !== void 0 && query.maxWidth !== void 0 && query.minWidth > query.maxWidth) throw new RangeError("minWidth cannot exceed maxWidth");
	if (query.minHeight !== void 0 && query.maxHeight !== void 0 && query.minHeight > query.maxHeight) throw new RangeError("minHeight cannot exceed maxHeight");
}
/** Observe the completed native content-box size of a host node. */
function createMeasuredSize(options = {}) {
	const [width, setWidth] = createSignal(0);
	const [height, setHeight] = createSignal(0);
	const [measured, setMeasured] = createSignal(false);
	let observer;
	const ref = (node) => {
		observer?.disconnect();
		observer = new ResizeObserver(([entry]) => {
			if (!entry) return;
			const size = {
				width: entry.contentRect.width,
				height: entry.contentRect.height
			};
			setWidth(size.width);
			setHeight(size.height);
			setMeasured(true);
			options.onChange?.(size);
		});
		observer.observe(node);
	};
	onCleanup(() => observer?.disconnect());
	return {
		ref,
		width,
		height,
		measured
	};
}
/**
* Match constraints against a component's completed native content-box size.
* The result remains false until the first measurement, avoiding a compact
* layout flash during boot.
*/
function createContainerMatch(query, options = {}) {
	validateSizeQuery(query);
	const size = createMeasuredSize(options);
	const matches = () => size.measured() && (query.minWidth === void 0 || size.width() >= query.minWidth) && (query.maxWidth === void 0 || size.width() <= query.maxWidth) && (query.minHeight === void 0 || size.height() >= query.minHeight) && (query.maxHeight === void 0 || size.height() <= query.maxHeight);
	return {
		...size,
		matches
	};
}
//#endregion
//#region src/primitives/presence.ts
/** Explicit mount lifecycle for content whose exit must finish before removal. */
function createPresence(open) {
	const [phase, setPhase] = createSignal(open() ? "present" : "unmounted");
	createEffect(open, (isOpen) => {
		if (isOpen) {
			if (phase() === "unmounted" || phase() === "exiting") setPhase("entering");
		} else if (phase() === "present" || phase() === "entering") setPhase("exiting");
	});
	return {
		phase,
		mounted: () => phase() !== "unmounted",
		finishEnter() {
			if (open() && phase() === "entering") setPhase("present");
		},
		finishExit() {
			if (!open() && phase() === "exiting") setPhase("unmounted");
		}
	};
}
//#endregion
//#region src/primitives/view.ts
const ICON_SIZE_UNITLESS_RE = /^-?\d*\.?\d+$/;
function normalizeIconSize(size) {
	if (size == null) return "1em";
	if (typeof size === "number") return size;
	const value = size.trim();
	if (!value) return "1em";
	const parsed = Number.parseFloat(value);
	if (Number.isFinite(parsed) && ICON_SIZE_UNITLESS_RE.test(value)) return parsed;
	return value;
}
function applyIconFill(source, fill) {
	return source.replace(/fill=(["'])none\1/, `fill="${fill}"`);
}
function primitive(tag, props) {
	const node = createElement(tag);
	spread(node, props, false);
	return node;
}
function editorPrimitive(tag, props) {
	return primitive(tag, mergeProps(props, {
		get role() {
			return props.role ?? "textbox";
		},
		get focusOrder() {
			return props.disabled ? -1 : props.focusOrder ?? 0;
		},
		get "aria-disabled"() {
			return props.disabled ?? false;
		}
	}));
}
function semanticPrimitive(tag, role, props) {
	const node = createElement(tag);
	spread(node, { role }, false);
	spread(node, props, false);
	return node;
}
/** A layout container. Text content should be placed in a {@link Text}. */
function View(props) {
	return primitive("view", props);
}
function resolvedTextBehavior(maxLines) {
	if (maxLines != null && (!Number.isInteger(maxLines) || maxLines < 1)) throw new RangeError("Text maxLines must be a positive integer");
	return {
		flags: TEXT_BEHAVIOR.AggregateDirectText | (maxLines == null || maxLines === 1 ? TEXT_BEHAVIOR.SingleLine : 0),
		maxLines: maxLines ?? 0
	};
}
/**
* A single measured text run.
*
* Static and reactive child text nodes are concatenated by the native host and
* participate in the parent layout as one item.
*/
function Text(props) {
	resolvedTextBehavior(untrack(() => props.maxLines));
	const node = createElement("text");
	spread(node, omit(props, "maxLines"), false);
	spread(node, {
		role: props.role ?? "label",
		get textBehavior() {
			return resolvedTextBehavior(props.maxLines);
		}
	}, false);
	return node;
}
/** A static SVG asset rendered through the native usvg/Vello pipeline. */
function Svg(props) {
	return semanticPrimitive("svg", "img", props);
}
/** A native Vello vector path in local logical-pixel coordinates. */
function Path(props) {
	return primitive("vector-path", props);
}
/** A theme-colored SVG icon with stable native sizing and semantics. */
function Icon(props) {
	const rest = omit(props, "source", "size", "fill", "label", "class");
	const node = createElement("svg");
	spread(node, rest, false);
	spread(node, {
		get class() {
			return props.class ? `self-center shrink-0 ${props.class}` : "self-center shrink-0";
		},
		get style() {
			const iconSize = normalizeIconSize(props.size);
			return {
				display: "inline-flex",
				"align-items": "center",
				"justify-content": "center",
				"align-self": "center",
				width: iconSize,
				height: iconSize,
				"flex-shrink": 0,
				"line-height": "1",
				...props.style ?? {}
			};
		},
		get width() {
			const iconSize = normalizeIconSize(props.size);
			return typeof iconSize === "number" ? String(iconSize) : void 0;
		},
		get source() {
			return props.fill && props.fill !== "none" ? applyIconFill(props.source, props.fill) : props.source;
		},
		get height() {
			const iconSize = normalizeIconSize(props.size);
			return typeof iconSize === "number" ? String(iconSize) : void 0;
		},
		get role() {
			return props.label ? "img" : void 0;
		},
		get "aria-label"() {
			return props.label;
		},
		get "aria-hidden"() {
			return props.label ? void 0 : "true";
		}
	}, false);
	return node;
}
/** A replaced image node rendered by the native host. */
function Image(props) {
	return semanticPrimitive("img", "img", props);
}
/** An explicit network-backed image with bounded decoding and host caching. */
function NetworkImage(props) {
	const rest = omit(props, "url", "format", "cache");
	const node = createElement("img");
	spread(node, { role: "img" }, false);
	spread(node, rest, false);
	spread(node, { get source() {
		return {
			kind: "network",
			url: props.url,
			format: props.format,
			cache: props.cache
		};
	} }, false);
	return node;
}
/** A native single-line text editor with selection and scrolling. */
function TextInput(props) {
	return editorPrimitive("input", props);
}
/** A native multiline text editor with wrapping, selection, and scrolling. */
function TextArea(props) {
	return editorPrimitive("textarea", props);
}
/** Native password editor whose value remains in a Rust SecretStore. */
function PasswordInput(props) {
	return editorPrimitive("password-input", props);
}
/** Experimental native editor for config and script-sized documents. */
function CodeEditor(props) {
	return editorPrimitive("code-editor", props);
}
//#endregion
//#region src/primitives/collapsible-presence.tsx
/**
* Measured disclosure content with explicit presence and subtree isolation.
* Height participates in layout while a subtree opacity layer masks glyphs
* crossing the moving clip edge.
*/
function CollapsiblePresence(props) {
	const open = () => props.open;
	const initiallyOpen = open();
	const presence = createPresence(open);
	let initialMeasurement = true;
	let heightTransition;
	const opacityTransition = createTransition(() => open() ? 1 : 0, {
		duration: props.duration ?? .2,
		ease: props.ease ?? "easeOut",
		reducedMotion: () => props.reducedMotion ?? false
	});
	const measured = createMeasuredSize({ onChange(size) {
		if (initialMeasurement && initiallyOpen && !props.animateInitial) heightTransition?.jump(size.height);
		initialMeasurement = false;
	} });
	const transitionOptions = () => ({
		duration: props.duration ?? .2,
		ease: props.ease ?? "easeOut",
		reducedMotion: () => props.reducedMotion ?? false
	});
	heightTransition = createTransition(() => open() && measured.measured() ? measured.height() : 0, {
		...transitionOptions(),
		onComplete(value) {
			if (value === 0 && !open()) presence.finishExit();
			else if (open()) presence.finishEnter();
		}
	});
	createEffect(() => [
		open(),
		measured.measured(),
		measured.height()
	], ([isOpen, isMeasured, height]) => {
		if (isOpen && isMeasured && height === 0) presence.finishEnter();
	});
	const style = () => ({
		...props.style,
		height: px(heightTransition?.value() ?? 0),
		opacity: number(opacityTransition.value())
	});
	return createComponent$1(View, {
		get ["class"]() {
			return props.class;
		},
		classList: { "overflow-hidden": true },
		get style() {
			return style();
		},
		get interactionBlocked() {
			return !open();
		},
		get ["aria-hidden"]() {
			return open() ? void 0 : "true";
		},
		get children() {
			return createComponent$1(Show, {
				get when() {
					return presence.mounted();
				},
				get children() {
					return createComponent$1(View, {
						ref(r$) {
							var _ref$ = measured.ref;
							typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
						},
						get children() {
							return createComponent$1(View, {
								get ["class"]() {
									return props.contentClass;
								},
								get style() {
									return props.contentStyle;
								},
								get children() {
									return props.children;
								}
							});
						}
					});
				}
			});
		}
	});
}
//#endregion
//#region src/primitives/interactions/form-draft.ts
/**
* A small immutable draft for form fields with explicit reset and commit
* semantics. Transient request/error state belongs outside this model.
*/
function createFormDraft(initial, options = {}) {
	const [baselineBox, setBaselineBox] = createSignal({ value: { ...initial } });
	const [box, setBox] = createSignal({ value: { ...initial } });
	const value = () => box().value;
	const equals = options.equals ?? shallowEqual;
	const dirty = createMemo(() => !equals(value(), baselineBox().value));
	const errors = createMemo(() => options.validate?.(value()) ?? {});
	const valid = createMemo(() => Reflect.ownKeys(errors()).length === 0);
	const replace = (next) => setBox({ value: next });
	const resetTo = (next) => {
		setBaselineBox({ value: { ...next } });
		replace({ ...next });
	};
	const setField = (key, updater) => {
		setBox((currentBox) => {
			const current = currentBox.value;
			const previous = current[key];
			const next = typeof updater === "function" ? updater(previous) : updater;
			return Object.is(previous, next) ? currentBox : { value: {
				...current,
				[key]: next
			} };
		});
	};
	return {
		value,
		dirty,
		errors,
		valid,
		fieldError: (key) => errors()[key],
		field: (key) => value()[key],
		control: (key) => [() => value()[key], (next) => setField(key, next)],
		set: setField,
		patch: (patch) => {
			setBox((currentBox) => {
				const current = currentBox.value;
				for (const key of Reflect.ownKeys(patch)) if (!Object.is(current[key], patch[key])) return { value: {
					...current,
					...patch
				} };
				return currentBox;
			});
		},
		reset: () => replace({ ...baselineBox().value }),
		resetTo,
		commit: () => {
			setBox((currentBox) => {
				setBaselineBox({ value: { ...currentBox.value } });
				return currentBox;
			});
		}
	};
}
function shallowEqual(left, right) {
	const keys = Reflect.ownKeys(left);
	if (keys.length !== Reflect.ownKeys(right).length) return false;
	return keys.every((key) => Object.is(left[key], right[key]));
}
//#endregion
//#region src/primitives/interactions/selection.ts
function toggleSelection(current, item, mode, allowEmpty = false) {
	return match(mode).with("single", () => current === item && allowEmpty ? void 0 : item).with("multiple", () => {
		const values = Array.isArray(current) ? current : [];
		return values.includes(item) ? values.filter((value) => value !== item) : [...values, item];
	}).exhaustive();
}
function isSelected(selection, item) {
	return Array.isArray(selection) ? selection.includes(item) : selection === item;
}
/**
* Selection state owned by stable keys while values remain host-owned.
* Selected items always resolve to the latest objects from `items`; keys that
* disappear from the source are removed instead of becoming ghost selections.
*/
function createKeyedSelection(options) {
	const initialAvailable = new Set(untrack(options.items).map(options.key));
	const initial = normalizeKeys(options.initialKeys ?? [], initialAvailable, options.mode);
	const [keys, setKeys] = createSignal(initial);
	createEffect(() => new Set(options.items().map(options.key)), (available) => {
		setKeys((current) => {
			const next = normalizeKeys(current, available, options.mode);
			return setsEqual(current, next) ? current : next;
		});
	});
	const selectedItems = createMemo(() => {
		const selected = keys();
		return options.items().filter((item) => selected.has(options.key(item)));
	});
	const set = (nextKeys) => {
		const next = normalizeKeys(nextKeys, new Set(options.items().map(options.key)), options.mode);
		setKeys((current) => setsEqual(current, next) ? current : next);
	};
	const select = (key) => {
		if (options.mode === "single") {
			set([key]);
			return;
		}
		setKeys((current) => {
			if (current.has(key)) return current;
			if (!new Set(options.items().map(options.key)).has(key)) return current;
			return /* @__PURE__ */ new Set([...current, key]);
		});
	};
	const deselect = (key) => {
		setKeys((current) => {
			if (!current.has(key)) return current;
			const next = new Set(current);
			next.delete(key);
			return next;
		});
	};
	return {
		keys,
		items: selectedItems,
		item: () => selectedItems()[0],
		isSelected: (key) => keys().has(key),
		select,
		deselect,
		toggle: (key) => keys().has(key) ? deselect(key) : select(key),
		set,
		clear: () => setKeys((current) => current.size === 0 ? current : /* @__PURE__ */ new Set())
	};
}
function normalizeKeys(keys, available, mode) {
	const next = /* @__PURE__ */ new Set();
	for (const key of keys) {
		if (!available.has(key)) continue;
		next.add(key);
		if (mode === "single") break;
	}
	return next;
}
function setsEqual(left, right) {
	if (left.size !== right.size) return false;
	for (const key of left) if (!right.has(key)) return false;
	return true;
}
//#endregion
//#region src/primitives/class-names.ts
function join(...values) {
	return values.filter(Boolean).join(" ");
}
//#endregion
//#region src/primitives/layout.tsx
/** Horizontal flex container. No wrapper node is added beyond the host View. */
function Row(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join("flex flex-row", props.class);
	} }));
}
/** Vertical flex container. No wrapper node is added beyond the host View. */
function Column(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join("flex flex-col", props.class);
	} }));
}
/** Flex container that centers children on both axes. */
function Center(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join("flex items-center justify-center", props.class);
	} }));
}
//#endregion
//#region src/primitives/overlay-layer.ts
const openLayers = [];
let nextOrder = 1;
const OverlayPlaneContext = createContext("floating");
/** Make nested portals inherit the current native stacking plane. */
function OverlayPlaneProvider(props) {
	return createComponent(OverlayPlaneContext, {
		get value() {
			return props.plane;
		},
		get children() {
			return props.children;
		}
	});
}
function useOverlayPlane() {
	return useContext(OverlayPlaneContext);
}
function planeRank(plane) {
	return plane === "modal" ? 1 : 0;
}
function removeLayer(token) {
	const index = openLayers.findIndex((layer) => layer.token === token);
	if (index >= 0) openLayers.splice(index, 1);
}
function pushLayer(token, plane) {
	removeLayer(token);
	const order = nextOrder++;
	openLayers.push({
		token,
		plane,
		order
	});
	return order;
}
function topmostLayer() {
	return openLayers.reduce((topmost, candidate) => {
		if (!topmost) return candidate;
		const rank = planeRank(candidate.plane) - planeRank(topmost.plane);
		return rank > 0 || rank === 0 && candidate.order > topmost.order ? candidate : topmost;
	}, void 0);
}
function createOverlayLayer(options) {
	const token = Symbol("wabou-overlay-layer");
	const plane = () => options.plane?.() ?? "floating";
	let wasOpen = untrack(options.open);
	let activePlane = untrack(plane);
	const [zIndex, setZIndex] = createSignal(wasOpen ? pushLayer(token, activePlane) : 0);
	const restoreFocus = () => {
		if (options.restoreFocus?.() ?? true) options.returnFocus?.()?.focus();
	};
	createEffect(options.open, (open) => {
		const currentPlane = plane();
		if (open && !wasOpen) setZIndex(pushLayer(token, currentPlane));
		else if (!open && wasOpen) {
			removeLayer(token);
			restoreFocus();
		}
		wasOpen = open;
		activePlane = currentPlane;
	});
	createEffect(plane, (currentPlane) => {
		if (wasOpen && activePlane !== currentPlane) setZIndex(pushLayer(token, currentPlane));
		activePlane = currentPlane;
	});
	onCleanup(() => {
		removeLayer(token);
		if (wasOpen) restoreFocus();
		wasOpen = false;
	});
	const isTopmost = () => topmostLayer()?.token === token;
	return {
		plane,
		zIndex,
		isTopmost,
		onEscape(event) {
			if (!options.open() || !isTopmost() || event.key !== "Escape" || options.closeOnEscape?.() === false) return;
			event.preventDefault();
			event.stopPropagation();
			options.onDismiss("escape");
		},
		onOutside(event) {
			if (!options.open() || !isTopmost() || options.closeOnOutside?.() === false) return;
			event.preventDefault();
			event.stopPropagation();
			options.onDismiss("outside");
		}
	};
}
//#endregion
//#region src/primitives/modal.tsx
/**
* A native modal plane with host-enforced focus, hit-test, and accessibility
* isolation. Visual styling remains explicit so applications can own it.
*/
function Modal(props) {
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(untrack(() => props.defaultOpen ?? false));
	const open = () => props.open ?? uncontrolledOpen();
	let trigger;
	let focusFrame = 0;
	let wasOpenForInitialFocus = false;
	const setOpen = (next, reason) => {
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next, reason);
	};
	const close = (reason = "programmatic") => setOpen(false, reason);
	const controls = { close: () => close("programmatic") };
	const layer = createOverlayLayer({
		open,
		plane: () => "modal",
		onDismiss: (reason) => close(reason === "outside" ? "backdrop" : "escape"),
		closeOnEscape: () => props.closeOnEscape ?? true,
		closeOnOutside: () => props.closeOnBackdrop ?? true,
		restoreFocus: () => props.restoreFocus ?? true,
		returnFocus: () => trigger
	});
	const handleEscape = (event) => layer.onEscape(event);
	createEffect(open, (isOpen) => {
		if (isOpen && !wasOpenForInitialFocus && props.initialFocus) {
			cancelAnimationFrame(focusFrame);
			focusFrame = requestAnimationFrame(() => {
				focusFrame = 0;
				props.initialFocus?.()?.focus();
			});
		} else if (!isOpen) {
			if (focusFrame) cancelAnimationFrame(focusFrame);
			focusFrame = 0;
		}
		wasOpenForInitialFocus = isOpen;
	});
	onCleanup(() => {
		if (focusFrame) cancelAnimationFrame(focusFrame);
	});
	const triggerProps = {
		ref: (node) => {
			trigger = node;
		},
		onClick: (event) => {
			event.stopPropagation();
			setOpen(true, "trigger");
		},
		onKeyDown: handleEscape,
		"aria-haspopup": "dialog",
		get "aria-expanded"() {
			return open();
		}
	};
	const overlay = createComponent(Show, {
		get when() {
			return open();
		},
		get children() {
			return createComponent(Portal, {
				plane: "modal",
				role: "presentation",
				"aria-modal": "true",
				focusContained: true,
				get class() {
					return props.backdropClass;
				},
				get style() {
					return {
						position: "absolute",
						left: 0,
						top: 0,
						width: "100%",
						height: "100%",
						display: "flex",
						"align-items": "center",
						"justify-content": "center",
						...props.backdropStyle,
						"z-index": layer.zIndex()
					};
				},
				onClick: layer.onOutside,
				onKeyDown: handleEscape,
				get children() {
					return createComponent(View, {
						get ref() {
							return props.contentRef;
						},
						role: "dialog",
						"aria-modal": "true",
						get "aria-label"() {
							return props["aria-label"];
						},
						get class() {
							return props.contentClass;
						},
						get style() {
							return props.contentStyle;
						},
						get shadows() {
							return props.contentShadows;
						},
						onClick: (event) => event.stopPropagation(),
						get children() {
							return createComponent(OverlayPlaneProvider, {
								plane: "modal",
								get children() {
									return typeof props.children === "function" ? props.children(controls) : props.children;
								}
							});
						}
					});
				}
			});
		}
	});
	return [untrack(() => props.trigger?.(triggerProps)), overlay];
}
//#endregion
//#region src/primitives/motion.tsx
function bindPlayback(controls, props) {
	createEffect(() => props.speed ?? 1, (speed) => {
		controls.speed = speed;
	});
	createEffect(() => props.paused, (paused) => {
		if (paused) controls.pause();
		else controls.play();
	});
}
/** A single native View whose contents rotate around its border-box center. */
function Spin(props) {
	const motion = props;
	const view = omit(props, "duration", "speed", "paused");
	const rotation = createRotation({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1
	});
	bindPlayback(rotation.controls, motion);
	return createComponent$1(View, mergeProps(view, { get transform() {
		return rotation.transform();
	} }));
}
/** A single native View with a repeating opacity pulse. */
function Pulse(props) {
	const motion = props;
	const view = omit(props, "duration", "speed", "paused", "from", "to", "style");
	const pulse = createPulse({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1,
		from: motion.from,
		to: motion.to
	});
	bindPlayback(pulse.controls, motion);
	const style = () => ({
		...motion.style ?? {},
		opacity: pulse.value()
	});
	return createComponent$1(View, mergeProps(view, { get style() {
		return style();
	} }));
}
/** A center-originating ring that expands while fading out, then repeats. */
function Ripple(props) {
	const motion = props;
	const view = omit(props, "duration", "speed", "paused", "fromScale", "style", "transform");
	const ripple = createLoop({
		autoplay: !motion.paused,
		duration: motion.duration ?? 1.4,
		from: 0,
		to: 1
	});
	bindPlayback(ripple.controls, motion);
	const progress = () => ripple.value();
	return createComponent$1(View, mergeProps(view, {
		get transform() {
			return scale2d((motion.fromScale ?? .35) + progress() * (1 - (motion.fromScale ?? .35)));
		},
		get style() {
			return {
				...motion.style ?? {},
				opacity: 1 - progress()
			};
		}
	}));
}
//#endregion
//#region src/primitives/notification.ts
const finiteNonNegative = (value, fallback) => Number.isFinite(value) ? Math.max(0, value) : fallback;
/** Create an owner-scoped notification queue with explicit JavaScript timers. */
function createNotifications(options = {}) {
	const [items, setItems] = createSignal([]);
	const records = /* @__PURE__ */ new Map();
	const defaultDuration = finiteNonNegative(options.defaultDuration ?? 5e3, 5e3);
	const configuredLimit = options.limit ?? 5;
	const limit = Number.isFinite(configuredLimit) ? Math.max(1, Math.floor(configuredLimit)) : 5;
	let nextId = 1;
	const dismiss = (id, reason = "programmatic") => {
		const record = records.get(id);
		if (!record) return false;
		if (record.timer !== void 0) clearTimeout(record.timer);
		records.delete(id);
		setItems((current) => current.filter((item) => item.id !== id));
		record.item.onDismiss?.(reason);
		return true;
	};
	const schedule = (record) => {
		if (!record.autoDismiss) return;
		if (record.remaining <= 0) {
			dismiss(record.item.id, "timeout");
			return;
		}
		record.startedAt = Date.now();
		record.timer = setTimeout(() => dismiss(record.item.id, "timeout"), record.remaining);
	};
	const notifications = {
		items,
		show(input) {
			while (records.size >= limit) {
				const oldest = items()[0];
				if (!oldest || !dismiss(oldest.id, "overflow")) break;
			}
			const duration = finiteNonNegative(input.duration ?? defaultDuration, defaultDuration);
			const item = {
				...input,
				id: nextId++,
				duration,
				priority: input.priority ?? "polite"
			};
			const record = {
				item,
				autoDismiss: duration > 0,
				pauseCount: 0,
				remaining: duration,
				startedAt: 0
			};
			records.set(item.id, record);
			setItems((current) => [...current, item]);
			schedule(record);
			return item.id;
		},
		dismiss,
		pause(id) {
			const record = records.get(id);
			if (!record) return;
			record.pauseCount++;
			if (record.pauseCount > 1 || record.timer === void 0) return;
			clearTimeout(record.timer);
			record.timer = void 0;
			record.remaining = Math.max(0, record.remaining - (Date.now() - record.startedAt));
		},
		resume(id) {
			const record = records.get(id);
			if (!record || record.pauseCount === 0) return;
			record.pauseCount--;
			if (record.pauseCount > 0 || record.timer !== void 0) return;
			schedule(record);
		},
		clear() {
			for (const id of [...records.keys()]) dismiss(id, "programmatic");
		}
	};
	onCleanup(() => {
		for (const record of records.values()) if (record.timer !== void 0) clearTimeout(record.timer);
		records.clear();
	});
	return notifications;
}
const alignment = (placement) => ({
	"align-items": placement.endsWith("start") ? "flex-start" : placement.endsWith("end") ? "flex-end" : "center",
	"justify-content": placement.startsWith("bottom") ? "flex-end" : "flex-start"
});
/** Render a non-blocking stack on the native floating overlay plane. */
function NotificationRegion(props) {
	const items = createComponent(For, {
		get each() {
			return props.notifications.items();
		},
		children: (item) => createComponent(View, {
			role: item.priority === "assertive" ? "alert" : "status",
			"aria-label": item["aria-label"],
			get class() {
				return `pointer-events-auto ${props.itemClass ?? ""}`;
			},
			get style() {
				return props.itemStyle;
			},
			onPointerEnter: () => props.notifications.pause(item.id),
			onPointerLeave: () => props.notifications.resume(item.id),
			onFocusIn: () => props.notifications.pause(item.id),
			onFocusOut: () => props.notifications.resume(item.id),
			get children() {
				return item.content({ dismiss: () => props.notifications.dismiss(item.id, "dismiss") });
			}
		})
	});
	return createComponent(Portal, {
		plane: "floating",
		role: "presentation",
		get class() {
			return `pointer-events-none ${props.class ?? ""}`;
		},
		get style() {
			const placement = props.placement ?? "top-end";
			return {
				position: "absolute",
				left: 0,
				top: 0,
				width: "100%",
				height: "100%",
				display: "flex",
				"flex-direction": "column",
				gap: 8,
				padding: 16,
				...alignment(placement),
				...props.style
			};
		},
		children: items
	});
}
//#endregion
//#region src/primitives/positioner.ts
/**
* Position two Wabou layout targets with Floating UI's renderer-independent
* geometry engine. Measurement remains host-owned and is supplied explicitly;
* no DOM-compatible Handle methods are required.
*/
function computeFloatingPosition(reference, floating, options) {
	const isRTL = options.platform.isRTL;
	return computePosition(reference, floating, {
		platform: {
			async getElementRects({ reference, floating }) {
				const [referenceRect, floatingRect] = await Promise.all([options.platform.getRect(reference), options.platform.getRect(floating)]);
				return {
					reference: referenceRect,
					floating: {
						x: 0,
						y: 0,
						width: floatingRect.width,
						height: floatingRect.height
					}
				};
			},
			getDimensions: (target) => options.platform.getRect(target),
			getClippingRect: ({ element }) => options.platform.getClippingRect(element),
			isElement: () => true,
			isRTL: isRTL ? (target) => isRTL(target) : void 0
		},
		placement: options.placement,
		strategy: options.strategy,
		middleware: options.middleware
	});
}
var LayoutTargetUnavailableError = class extends Error {
	name = "LayoutTargetUnavailableError";
};
/** Position two native handles from a single coherent Host layout snapshot. */
function computeHostFloatingPosition(reference, floating, host, options = {}) {
	const snapshot = host.layout.snapshot([reference, floating]);
	const nodes = new Map(snapshot.nodes.map((node) => [formatNodeKey(node.id), node]));
	const id = (target) => "id" in target ? target.id : target;
	const rect = (target) => {
		const targetId = id(target);
		const node = nodes.get(formatNodeKey(targetId));
		if (!node) throw new LayoutTargetUnavailableError(`Layout target ${targetId.lo}v${targetId.hi} is not present in completed revision ${snapshot.revision}`);
		return node;
	};
	return computeFloatingPosition(reference, floating, {
		...options,
		platform: {
			getRect: (target) => rect(target).rect,
			getClippingRect: (target) => rect(target).clip
		}
	});
}
//#endregion
//#region src/primitives/popover.tsx
/** A root-layer floating panel positioned from native layout snapshots. */
function Popover(props) {
	const host = useHost$1();
	const inheritedPlane = useOverlayPlane();
	const plane = () => props.plane ?? inheritedPlane;
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const [position, setPosition] = createSignal({
		x: 0,
		y: 0
	});
	const [positioned, setPositioned] = createSignal(false);
	const open = () => props.open ?? uncontrolledOpen();
	let anchor;
	let content;
	let frame = 0;
	let positionRequest = 0;
	let observer;
	const setOpen = (next, reason) => {
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next, reason);
	};
	const layer = createOverlayLayer({
		open,
		plane,
		onDismiss: (reason) => setOpen(false, reason),
		closeOnEscape: () => props.closeOnEscape ?? true,
		returnFocus: () => anchor,
		restoreFocus: () => props.restoreFocus ?? true
	});
	const updatePosition = async () => {
		if (!open() || !anchor || !content) return;
		const request = ++positionRequest;
		try {
			const result = await computeHostFloatingPosition(anchor, content, host, {
				placement: props.placement ?? "bottom-start",
				middleware: [
					offset(props.offset ?? 6),
					flip(),
					shift({ padding: 8 })
				]
			});
			if (!open() || request !== positionRequest) return;
			setPosition({
				x: result.x,
				y: result.y
			});
			setPositioned(true);
		} catch (error) {
			if (error instanceof LayoutTargetUnavailableError && open() && request === positionRequest) {
				schedulePosition();
				return;
			}
			throw error;
		}
	};
	const schedulePosition = () => {
		cancelAnimationFrame(frame);
		frame = requestAnimationFrame(() => void updatePosition());
	};
	const observe = (node) => {
		observer?.observe(node);
		schedulePosition();
	};
	createEffect(open, (isOpen) => {
		if (!isOpen) {
			positionRequest++;
			setPositioned(false);
			observer?.disconnect();
			observer = void 0;
			return;
		}
		observer = new ResizeObserver(schedulePosition);
		if (anchor) observer.observe(anchor);
		frame = requestAnimationFrame(() => {
			frame = requestAnimationFrame(() => void updatePosition());
		});
	});
	onCleanup(() => {
		cancelAnimationFrame(frame);
		observer?.disconnect();
	});
	const handleEscape = (event) => layer.onEscape(event);
	return [memo(() => {
		return props.trigger({
			ref: (node) => {
				anchor = node;
				if (open()) observe(node);
			},
			onClick: (event) => {
				event.stopPropagation();
				setOpen(!open(), "trigger");
			},
			onKeyDown: handleEscape,
			"aria-haspopup": props.contentRole === "presentation" ? props.popupRole : "dialog",
			"aria-expanded": open()
		});
	}), createComponent$1(Show, {
		get when() {
			return open();
		},
		get children() {
			return createComponent$1(Portal, {
				get plane() {
					return plane();
				},
				ref: (node) => {
					observe(node);
				},
				role: "presentation",
				get style() {
					return {
						position: "absolute",
						left: 0,
						top: 0,
						width: "100%",
						height: "100%",
						"z-index": layer.zIndex()
					};
				},
				get onClick() {
					return layer.onOutside;
				},
				onKeyDown: handleEscape,
				onWheel: schedulePosition,
				get children() {
					return createComponent$1(View, {
						ref: (node) => {
							content = node;
							observe(node);
						},
						get role() {
							return props.contentRole ?? "dialog";
						},
						get ["aria-label"]() {
							return props["aria-label"];
						},
						get ["class"]() {
							return props.contentClass;
						},
						get shadows() {
							return props.contentShadows;
						},
						get style() {
							return {
								position: "absolute",
								left: positioned() ? `${position().x}px` : "-100000px",
								top: positioned() ? `${position().y}px` : "-100000px",
								...props.contentStyle
							};
						},
						onClick: (event) => event.stopPropagation(),
						onKeyDown: handleEscape,
						get children() {
							return props.children;
						}
					});
				}
			});
		}
	})];
}
//#endregion
//#region src/primitives/scroll-area.tsx
const scrollAreaViewportClass = (className) => join("min-w-0 min-h-0 overflow-x-hidden overflow-y-auto", className);
/**
* Vertical native scroll viewport with explicit sizing.
*
* The inner wrapper deliberately cannot shrink. This makes its intrinsic
* height become the viewport's scroll extent instead of allowing a flex
* parent to compress overflowing sections until no scroll range remains.
* The viewport also locks its cross axis. Otherwise focus reveal can move a
* nominally vertical viewport sideways when a descendant is slightly wider,
* making split-pane edges appear clipped. It deliberately does not grow:
* implicit `flex-1` makes a
* nested scroll area expand with an ancestor's intrinsic content instead of
* establishing its own scroll range.
*/
function ScrollArea(props) {
	return createComponent$1(View, {
		ref(r$) {
			var _ref$ = props.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : props.ref = r$;
		},
		get ["class"]() {
			return scrollAreaViewportClass(props.class);
		},
		get style() {
			return props.style;
		},
		get scrollbar() {
			return props.scrollbar;
		},
		get onScroll() {
			return props.onScroll;
		},
		get children() {
			return createComponent$1(View, {
				get ["class"]() {
					return join("flex-none flex flex-col min-h-full", props.contentClass);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/primitives/scroll-reset.ts
/** Reset one explicitly selected native viewport after its key changes. */
function createScrollReset(options) {
	const reset = () => {
		options.target()?.scrollTo({
			left: options.left ?? 0,
			top: options.top ?? 0
		});
	};
	createEffect(options.key, reset, { defer: true });
	return reset;
}
//#endregion
//#region src/primitives/shortcuts.ts
const MODIFIER_BITS = {
	shift: 1,
	control: 2,
	alt: 4,
	meta: 8
};
const ALL_MODIFIERS = 15;
/**
* Compile declarative application shortcuts into one keydown binding.
*
* Chords use names such as `Primary+T`, `Control+Tab`, and
* `Control+Shift+Tab`. `Primary` resolves to Command on macOS and Control on
* other platforms while still requiring an exact modifier match.
*/
function createShortcuts(shortcuts) {
	const compiled = Object.entries(shortcuts).map(([chord, value]) => compileShortcut(chord, value));
	assertNoAmbiguousShortcuts(compiled);
	const invokeKeyDown = (event) => {
		const key = normalizeKey(event.key);
		const modifiers = event.mods & ALL_MODIFIERS;
		const shortcut = compiled.find((candidate) => candidate.key === key && (candidate.primary ? event.primary && candidate.modifierMasks.includes(modifiers & ~(MODIFIER_BITS.control | MODIFIER_BITS.meta)) : candidate.modifierMasks.includes(modifiers)));
		if (shortcut === void 0 || event.repeat && !shortcut.allowRepeat) return { handled: false };
		if (shortcut.preventDefault !== false) event.preventDefault();
		return {
			handled: true,
			result: shortcut.handler(event)
		};
	};
	return {
		handleKeyDown: (event) => invokeKeyDown(event).handled,
		bindings: { onKeyDown: (event) => invokeKeyDown(event).result }
	};
}
function compileShortcut(chord, value) {
	const parts = chord.split("+").map((part) => part.trim()).filter(Boolean);
	if (parts.length === 0) throw new Error("Shortcut chord cannot be empty");
	const key = normalizeKey(parts.pop());
	if (key.length === 0) throw new Error(`Shortcut chord has no key: ${chord}`);
	let mask = 0;
	let primary = false;
	for (const rawModifier of parts) {
		const modifier = rawModifier.toLowerCase();
		const bit = modifier === "shift" ? MODIFIER_BITS.shift : modifier === "control" || modifier === "ctrl" ? MODIFIER_BITS.control : modifier === "alt" || modifier === "option" ? MODIFIER_BITS.alt : modifier === "meta" || modifier === "cmd" || modifier === "command" ? MODIFIER_BITS.meta : void 0;
		if (modifier === "primary" || modifier === "mod") {
			if (primary) throw new Error(`Duplicate modifier in shortcut: ${chord}`);
			primary = true;
		} else if (bit === void 0) throw new Error(`Unknown shortcut modifier '${rawModifier}' in ${chord}`);
		else if ((mask & bit) !== 0) throw new Error(`Duplicate modifier in shortcut: ${chord}`);
		else mask |= bit;
	}
	if (primary && (mask & (MODIFIER_BITS.control | MODIFIER_BITS.meta)) !== 0) throw new Error(`Primary cannot be combined with Control or Meta: ${chord}`);
	return {
		...typeof value === "function" ? { handler: value } : value,
		chord,
		key,
		modifierMasks: [mask],
		primary
	};
}
function assertNoAmbiguousShortcuts(shortcuts) {
	const owners = /* @__PURE__ */ new Map();
	for (const shortcut of shortcuts) {
		const masks = shortcut.primary ? shortcut.modifierMasks.flatMap((mask) => [mask | MODIFIER_BITS.control, mask | MODIFIER_BITS.meta]) : shortcut.modifierMasks;
		for (const mask of masks) {
			const signature = `${mask}:${shortcut.key}`;
			const previous = owners.get(signature);
			if (previous !== void 0) throw new Error(`Ambiguous shortcuts '${previous}' and '${shortcut.chord}'`);
			owners.set(signature, shortcut.chord);
		}
	}
}
function normalizeKey(key) {
	return key.trim().toLowerCase();
}
//#endregion
//#region src/primitives/tabs.ts
/**
* Stateful tab collection with stable identity and deterministic activation.
*
* Closing the active tab selects its right-hand neighbour, or the previous
* tab when the closed tab was last. Reordering never changes the active key.
*/
function createTabs(options) {
	const initialTabs = [...options.initialTabs ?? []];
	assertUniqueKeys(initialTabs, options.key);
	const initialActiveKey = options.initialActiveKey !== void 0 && initialTabs.some((tab) => options.key(tab) === options.initialActiveKey) ? options.initialActiveKey : initialTabs[0] === void 0 ? void 0 : options.key(initialTabs[0]);
	const [tabs, setTabs] = createSignal(initialTabs);
	const [activeKey, setActiveKey] = createSignal(() => initialActiveKey);
	const focusTargets = /* @__PURE__ */ new Map();
	const commitActiveKey = (key) => {
		if (activeKey() === key) return false;
		setActiveKey(() => key);
		options.onActiveChange?.(key);
		return true;
	};
	const select = (key) => {
		if (!tabs().some((tab) => options.key(tab) === key)) return false;
		return commitActiveKey(key);
	};
	const selectAt = (index) => {
		const values = tabs();
		if (values.length === 0) return false;
		const normalized = (index % values.length + values.length) % values.length;
		return select(options.key(values[normalized]));
	};
	const activeIndex = () => {
		const current = activeKey();
		return current === void 0 ? -1 : tabs().findIndex((tab) => options.key(tab) === current);
	};
	const focus = (key) => {
		const target = focusTargets.get(key);
		if (target === void 0) return false;
		target.focus();
		return true;
	};
	const activateAt = (index, moveFocus) => {
		const values = tabs();
		if (values.length === 0) return false;
		const normalized = (index % values.length + values.length) % values.length;
		const key = options.key(values[normalized]);
		select(key);
		if (moveFocus) focus(key);
		return true;
	};
	return {
		tabs,
		activeKey,
		activeTab: createMemo(() => {
			const current = activeKey();
			return current === void 0 ? void 0 : tabs().find((tab) => options.key(tab) === current);
		}),
		select,
		selectNext: () => selectAt(activeIndex() + 1),
		selectPrevious: () => selectAt(activeIndex() <= 0 ? tabs().length - 1 : activeIndex() - 1),
		selectFirst: () => selectAt(0),
		selectLast: () => selectAt(tabs().length - 1),
		add: (tab, addOptions = {}) => {
			const key = options.key(tab);
			const current = tabs();
			if (current.some((candidate) => options.key(candidate) === key)) return false;
			const index = Math.max(0, Math.min(addOptions.index ?? current.length, current.length));
			const next = [...current];
			next.splice(index, 0, tab);
			setTabs(next);
			if (activeKey() === void 0 || addOptions.activate !== false) commitActiveKey(key);
			return true;
		},
		close: (key) => {
			const current = tabs();
			const index = current.findIndex((tab) => options.key(tab) === key);
			if (index < 0) return false;
			const next = current.filter((_, candidateIndex) => candidateIndex !== index);
			setTabs(next);
			focusTargets.delete(key);
			if (activeKey() === key) {
				const neighbour = next[Math.min(index, next.length - 1)];
				commitActiveKey(neighbour === void 0 ? void 0 : options.key(neighbour));
			}
			return true;
		},
		move: (key, requestedIndex) => {
			const current = tabs();
			const index = current.findIndex((tab) => options.key(tab) === key);
			if (index < 0) return false;
			const target = Math.max(0, Math.min(requestedIndex, current.length - 1));
			if (target === index) return false;
			const next = [...current];
			const [tab] = next.splice(index, 1);
			next.splice(target, 0, tab);
			setTabs(next);
			return true;
		},
		register: (key, node) => {
			if (tabs().some((tab) => options.key(tab) === key)) focusTargets.set(key, node);
		},
		focus,
		handleKeyDown: (key, event) => {
			const index = tabs().findIndex((tab) => options.key(tab) === key);
			if (index < 0) return false;
			const horizontal = options.orientation !== "vertical";
			const target = event.key === "Home" ? 0 : event.key === "End" ? tabs().length - 1 : horizontal && event.key === "ArrowRight" || !horizontal && event.key === "ArrowDown" ? index + 1 : horizontal && event.key === "ArrowLeft" || !horizontal && event.key === "ArrowUp" ? index - 1 : void 0;
			if (target === void 0) return false;
			event.preventDefault?.();
			return activateAt(target, true);
		}
	};
}
function assertUniqueKeys(tabs, key) {
	const keys = /* @__PURE__ */ new Set();
	for (const tab of tabs) {
		const value = key(tab);
		if (keys.has(value)) throw new Error(`Duplicate tab key: ${String(value)}`);
		keys.add(value);
	}
}
//#endregion
//#region src/primitives/index.ts
var primitives_exports = /* @__PURE__ */ __exportAll({
	Button: () => Button,
	Center: () => Center,
	CodeEditor: () => CodeEditor,
	CollapsiblePresence: () => CollapsiblePresence,
	Column: () => Column,
	Icon: () => Icon,
	Image: () => Image,
	Link: () => Link,
	Modal: () => Modal,
	NetworkImage: () => NetworkImage,
	NotificationRegion: () => NotificationRegion,
	OverlayPlaneProvider: () => OverlayPlaneProvider,
	PasswordInput: () => PasswordInput,
	Path: () => Path,
	PathBuilder: () => PathBuilder,
	Popover: () => Popover,
	Pulse: () => Pulse,
	Ripple: () => Ripple,
	Row: () => Row,
	ScrollArea: () => ScrollArea,
	Spin: () => Spin,
	Svg: () => Svg,
	Text: () => Text,
	TextArea: () => TextArea,
	TextInput: () => TextInput,
	View: () => View,
	arrow: () => arrow,
	autoPlacement: () => autoPlacement,
	computeFloatingPosition: () => computeFloatingPosition,
	computeHostFloatingPosition: () => computeHostFloatingPosition,
	createActive: () => createActive,
	createAnimationFrame: () => createAnimationFrame,
	createButton: () => createButton,
	createContainerMatch: () => createContainerMatch,
	createFocus: () => createFocus,
	createFocusWithin: () => createFocusWithin,
	createFormDraft: () => createFormDraft,
	createHover: () => createHover,
	createKeyedSelection: () => createKeyedSelection,
	createMeasuredSize: () => createMeasuredSize,
	createNotifications: () => createNotifications,
	createOverlayLayer: () => createOverlayLayer,
	createPresence: () => createPresence,
	createPress: () => createPress,
	createScrollReset: () => createScrollReset,
	createShortcuts: () => createShortcuts,
	createTabs: () => createTabs,
	flip: () => flip,
	offset: () => offset,
	rotate2d: () => rotate2d$1,
	shift: () => shift,
	size: () => size,
	translate2d: () => translate2d,
	useOverlayPlane: () => useOverlayPlane
});
//#endregion
export { createPress as $, createFormDraft as A, Text as B, useOverlayPlane as C, createKeyedSelection as D, Row as E, NetworkImage as F, translate2d as G, TextInput as H, PasswordInput as I, createMeasuredSize as J, createPresence as K, Path as L, CodeEditor as M, Icon as N, isSelected as O, Image as P, createActive as Q, PathBuilder as R, createOverlayLayer as S, Column as T, View as U, TextArea as V, rotate2d$1 as W, Link as X, Button as Y, createButton as Z, Pulse as _, ScrollArea as a, animateKeyframes as at, Modal as b, autoPlacement as c, createRotation as ct, flip as d, createHover as et, offset as f, createNotifications as g, NotificationRegion as h, createScrollReset as i, animate as it, CollapsiblePresence as j, toggleSelection as k, computeFloatingPosition as l, createTransition as lt, size as m, createTabs as n, createFocusWithin as nt, Popover as o, createLoop as ot, shift as p, createContainerMatch as q, createShortcuts as r, createAnimationFrame as rt, arrow as s, createPulse as st, primitives_exports as t, createFocus as tt, computeHostFloatingPosition as u, Ripple as v, Center as w, OverlayPlaneProvider as x, Spin as y, Svg as z };

//# sourceMappingURL=primitives-C1LSQiBw.mjs.map