import { EVENT_CODE, HostProvider, dispatchEvent, mount, writer } from "@wabou/core/renderer";
import { INTERACTION_POLICY } from "@wabou/core/protocol";
import { dispatchResizeObservation } from "@wabou/core/testing";
import { createComponent, flush } from "solid-js";
import { onTestFinished, vi } from "vitest";
//#region src/component.ts
function missingHostMethod(path) {
	throw new Error(`test host method ${path} is not configured`);
}
/** Create a typed, deterministic Host with automatic call recording. */
function createTestHost(capabilities, builtins = {}) {
	const calls = [];
	const base = Object.assign({
		system: {
			openUrl: (url) => missingHostMethod(`system.openUrl(${url})`),
			...builtins.system
		},
		fonts: {
			load: (path) => missingHostMethod(`fonts.load(${path})`),
			...builtins.fonts
		},
		diagnostics: {
			frameStats: () => null,
			setOverlay: () => false,
			overlayPaintStats: () => null,
			...builtins.diagnostics
		},
		intl: {
			locale: () => "en-US",
			timeZone: () => "UTC",
			today: () => ({
				year: 1970,
				month: 1,
				day: 1
			}),
			...builtins.intl
		},
		layout: {
			snapshot: () => missingHostMethod("layout.snapshot"),
			measure: () => missingHostMethod("layout.measure"),
			clippingRect: () => missingHostMethod("layout.clippingRect"),
			viewport: () => missingHostMethod("layout.viewport"),
			...builtins.layout
		}
	}, capabilities ?? {});
	const cache = /* @__PURE__ */ new WeakMap();
	const wrap = (value, path) => {
		const cached = cache.get(value);
		if (cached) return cached;
		const methods = /* @__PURE__ */ new Map();
		const proxy = new Proxy(value, { get(target, property, receiver) {
			const child = Reflect.get(target, property, receiver);
			if (typeof property !== "string") return child;
			const childPath = path ? `${path}.${property}` : property;
			if (typeof child === "function") {
				const existing = methods.get(property);
				if (existing) return existing;
				const method = (...args) => {
					calls.push({
						path: childPath,
						args
					});
					return Reflect.apply(child, target, args);
				};
				methods.set(property, method);
				return method;
			}
			if (child && typeof child === "object") return wrap(child, childPath);
			return child;
		} });
		cache.set(value, proxy);
		return proxy;
	};
	return {
		host: wrap(base, ""),
		calls,
		callsTo: (path) => calls.filter((call) => call.path === path),
		clearCalls: () => calls.splice(0)
	};
}
let activeHarness = false;
let activeScreen = null;
/** Dispose the active component tree. Vitest users get this automatically. */
function cleanupComponents() {
	activeScreen?.dispose();
}
const key = (id) => `${id.lo}:${id.hi}`;
const implicitRole = (tag) => {
	if (tag === "button") return "button";
	if (tag === "input") return "textbox";
	return null;
};
function installHostStub(name, stub = () => {}) {
	const target = globalThis;
	const hadOwn = Object.hasOwn(target, name);
	const previous = target[name];
	if (typeof previous !== "function") target[name] = stub;
	return () => {
		if (hadOwn) target[name] = previous;
		else delete target[name];
	};
}
/**
* Mount a component into Wabou's real Solid renderer while recording its
* authored host tree. This is intentionally a fast component-contract test:
* native layout, hit testing, and final semantic projection remain the job of
* `wabou test` behavior scenarios.
*/
function renderComponent(render, options = {}) {
	if (options.clock !== void 0 && options.clock !== "real" && options.clock !== "fake") throw new RangeError(`unsupported component clock ${JSON.stringify(options.clock)}`);
	if (activeHarness) throw new Error("renderComponent supports one active component screen at a time");
	activeHarness = true;
	const restoreHostStubs = [
		installHostStub("__wabou_resize_observe"),
		installHostStub("__wabou_resize_unobserve"),
		installHostStub("__wabou_flush"),
		installHostStub("__wabou_log"),
		installHostStub("__wabou_layout_snapshot", (ids, output) => {
			const values = [
				1,
				0,
				0,
				0,
				0,
				1024,
				768,
				ids.length / 2
			];
			for (let index = 0; index < ids.length / 2; index++) values.push(ids[index * 2], ids[index * 2 + 1], 0, 0, 0, 0, 0, 0, 1024, 768, 0, 0, 0, 0);
			if (output && output.length >= values.length) output.set(values);
			return values.length;
		})
	];
	const nodes = /* @__PURE__ */ new Map();
	const roots = [];
	const originals = {
		createElement: writer.createElement,
		createText: writer.createText,
		appendChild: writer.appendChild,
		insertBefore: writer.insertBefore,
		removeChild: writer.removeChild,
		setText: writer.setText,
		setAttribute: writer.setAttribute,
		removeAttribute: writer.removeAttribute,
		setClassName: writer.setClassName,
		setTransform2D: writer.setTransform2D,
		setInteractionPolicy: writer.setInteractionPolicy,
		dropNode: writer.dropNode,
		focusNode: writer.focusNode
	};
	const create = (id, tag, text = "") => {
		nodes.set(key(id), {
			id,
			tag,
			parent: null,
			children: [],
			attributes: /* @__PURE__ */ new Map(),
			focusOrder: null,
			interactionBlocked: false,
			focusContained: false,
			className: "",
			transform: null,
			text
		});
	};
	const detach = (node) => {
		const siblings = node.parent?.children ?? roots;
		const index = siblings.indexOf(node);
		if (index >= 0) siblings.splice(index, 1);
		node.parent = null;
	};
	const attach = (parentId, childId, refId) => {
		const child = nodes.get(key(childId));
		if (!child) return;
		detach(child);
		const parent = nodes.get(key(parentId)) ?? null;
		const siblings = parent?.children ?? roots;
		const refIndex = refId ? siblings.findIndex((candidate) => key(candidate.id) === key(refId)) : -1;
		siblings.splice(refIndex < 0 ? siblings.length : refIndex, 0, child);
		child.parent = parent;
	};
	writer.createElement = (id, tag) => {
		create(id, tag);
		originals.createElement.call(writer, id, tag);
	};
	writer.createText = (id, text) => {
		create(id, "#text", text);
		originals.createText.call(writer, id, text);
	};
	writer.appendChild = (parent, child) => {
		attach(parent, child);
		originals.appendChild.call(writer, parent, child);
	};
	writer.insertBefore = (parent, child, ref) => {
		attach(parent, child, ref);
		originals.insertBefore.call(writer, parent, child, ref);
	};
	writer.removeChild = (parent, child) => {
		const node = nodes.get(key(child));
		if (node) detach(node);
		originals.removeChild.call(writer, parent, child);
	};
	writer.setText = (id, text) => {
		const node = nodes.get(key(id));
		if (node) node.text = text;
		originals.setText.call(writer, id, text);
	};
	writer.setAttribute = (id, name, value) => {
		nodes.get(key(id))?.attributes.set(name, value);
		originals.setAttribute.call(writer, id, name, value);
	};
	writer.removeAttribute = (id, name) => {
		nodes.get(key(id))?.attributes.delete(name);
		originals.removeAttribute.call(writer, id, name);
	};
	writer.setClassName = (id, value) => {
		const node = nodes.get(key(id));
		if (node) node.className = value;
		originals.setClassName.call(writer, id, value);
	};
	writer.setTransform2D = (id, value) => {
		const node = nodes.get(key(id));
		if (node) node.transform = [...value];
		originals.setTransform2D.call(writer, id, value);
	};
	writer.setInteractionPolicy = (id, flags, focusOrder) => {
		const node = nodes.get(key(id));
		if (node) {
			node.focusOrder = (flags & INTERACTION_POLICY.Focusable) !== 0 ? focusOrder : null;
			node.interactionBlocked = (flags & INTERACTION_POLICY.BlockSubtree) !== 0;
			node.focusContained = (flags & INTERACTION_POLICY.ContainFocus) !== 0;
		}
		originals.setInteractionPolicy.call(writer, id, flags, focusOrder);
	};
	writer.dropNode = (id) => {
		const node = nodes.get(key(id));
		if (node) detach(node);
		nodes.delete(key(id));
		originals.dropNode.call(writer, id);
	};
	let disposeMount = null;
	let flushDepth = 0;
	let fakeFrameTime = 0;
	let restorePerformanceNow;
	const flushUpdates = () => {
		if (flushDepth > 0) return;
		flushDepth += 1;
		try {
			flush();
		} finally {
			flushDepth -= 1;
		}
		writer.flush();
	};
	const restore = () => {
		Object.assign(writer, originals);
		restoreHostStubs.forEach((restoreStub) => {
			restoreStub();
		});
		restorePerformanceNow?.();
		if (options.clock === "fake") vi.useRealTimers();
		activeHarness = false;
	};
	try {
		if (options.clock === "fake") {
			vi.useFakeTimers();
			fakeFrameTime = performance.now();
			const performanceNow = vi.spyOn(performance, "now").mockImplementation(() => fakeFrameTime);
			restorePerformanceNow = () => performanceNow.mockRestore();
		}
		disposeMount = mount(() => options.host ? createComponent(HostProvider, {
			value: options.host,
			get children() {
				return render();
			}
		}) : render());
		flushUpdates();
	} catch (error) {
		restore();
		throw error;
	}
	const textOf = (node) => node.tag === "#text" ? node.text : node.children.map(textOf).join("");
	const roleOf = (node) => node.attributes.get("role") ?? implicitRole(node.tag);
	const nameOf = (node) => node.attributes.get("aria-label") ?? textOf(node).trim();
	const booleanState = (node, name) => {
		const value = node.attributes.get(name);
		if (value === void 0) return null;
		if (value === "true") return true;
		if (value === "false") return false;
		throw new Error(`${name} must be true or false, received ${JSON.stringify(value)}`);
	};
	const toggleState = (node, name) => {
		if (node.attributes.get(name) === "mixed") return "mixed";
		return booleanState(node, name);
	};
	const disabledState = (node) => node.attributes.has("disabled") || node.attributes.get("aria-disabled") === "true";
	const numericState = (node, name) => {
		const value = node.attributes.get(name);
		if (value === void 0) return null;
		const number = Number(value);
		if (!Number.isFinite(number)) throw new Error(`${name} must be a finite number, received ${JSON.stringify(value)}`);
		return number;
	};
	const currentState = (node) => {
		const value = node.attributes.get("aria-current");
		if (value === void 0) return null;
		if (value === "true") return true;
		if (value === "false") return false;
		return value;
	};
	const orientationState = (node) => {
		const value = node.attributes.get("aria-orientation");
		if (value === void 0) return null;
		if (value === "horizontal" || value === "vertical") return value;
		throw new Error(`aria-orientation must be horizontal or vertical, received ${JSON.stringify(value)}`);
	};
	const all = () => {
		const result = [];
		const visit = (node) => {
			result.push(node);
			node.children.forEach(visit);
		};
		roots.forEach(visit);
		return result;
	};
	const descendantsOf = (root) => {
		const result = [];
		const visit = (node) => {
			node.children.forEach((child) => {
				result.push(child);
				visit(child);
			});
		};
		visit(root);
		return result;
	};
	const scopeNodes = (root) => {
		if (root === null) return all();
		if (!all().includes(root)) throw new Error(`cannot query within detached component ${roleOf(root) ?? root.tag} "${nameOf(root)}"`);
		return descendantsOf(root);
	};
	const describeRole = (role, options) => {
		return `role=${role}${Object.entries(options).filter(([name, value]) => name !== "index" && value !== void 0).map(([name, value]) => ` ${name}=${JSON.stringify(value)}`).join("")}`;
	};
	const matchesState = (node, options) => (options.disabled === void 0 || disabledState(node) === options.disabled) && (options.checked === void 0 || toggleState(node, "aria-checked") === options.checked) && (options.selected === void 0 || booleanState(node, "aria-selected") === options.selected) && (options.expanded === void 0 || booleanState(node, "aria-expanded") === options.expanded) && (options.pressed === void 0 || toggleState(node, "aria-pressed") === options.pressed) && (options.current === void 0 || currentState(node) === options.current) && (options.orientation === void 0 || orientationState(node) === options.orientation) && (options.focused === void 0 || focusedNode === node === options.focused);
	const matchingRole = (root, role, options) => scopeNodes(root).filter((node) => roleOf(node) === role && (options.name === void 0 || nameOf(node) === options.name) && matchesState(node, options));
	const scopeSuffix = (root) => root === null ? "" : ` within ${roleOf(root) ?? root.tag} "${nameOf(root)}"`;
	const resolveOne = (root, role, options, required) => {
		if (options.index !== void 0 && (!Number.isSafeInteger(options.index) || options.index < 0)) throw new RangeError("component locator index must be non-negative");
		const matches = matchingRole(root, role, options);
		const description = `${describeRole(role, options)}${scopeSuffix(root)}`;
		if (options.index === void 0 && matches.length > 1) throw new Error(`found ${matches.length} matches for ${description}; pass an index or use getAllByRole`);
		const match = matches[options.index ?? 0];
		if (!match) {
			if (required) throw new Error(`no component found for ${description}`);
			return null;
		}
		return locator(match);
	};
	const resolveAll = (root, role, options, required) => {
		const matches = matchingRole(root, role, options);
		if (required && matches.length === 0) throw new Error(`no components found for ${describeRole(role, options)}${scopeSuffix(root)}`);
		return matches.map(locator);
	};
	const queries = (root) => ({
		getByRole: (role, options = {}) => {
			const result = resolveOne(root, role, options, true);
			if (!result) throw new Error("required component query returned no result");
			return result;
		},
		queryByRole: (role, options = {}) => resolveOne(root, role, options, false),
		getAllByRole: (role, options = {}) => resolveAll(root, role, options, true),
		queryAllByRole: (role, options = {}) => resolveAll(root, role, options, false)
	});
	const commitEvent = (node, eventCode, payload = "") => {
		dispatchEvent(node.id, eventCode, payload);
		flushUpdates();
	};
	let focusedNode = null;
	const blurFocusedNode = () => {
		if (!focusedNode) return;
		const previous = focusedNode;
		focusedNode = null;
		commitEvent(previous, EVENT_CODE.blur);
		commitEvent(previous, EVENT_CODE.focusout);
	};
	const focusAuthoredNode = (node) => {
		if (focusedNode === node) return;
		blurFocusedNode();
		focusedNode = node;
		commitEvent(node, EVENT_CODE.focus);
		commitEvent(node, EVENT_CODE.focusin);
	};
	writer.focusNode = (id) => {
		originals.focusNode.call(writer, id);
		const node = nodes.get(key(id));
		if (node) focusAuthoredNode(node);
	};
	const ensureAttached = (node, action) => {
		if (all().includes(node)) return;
		throw new Error(`cannot ${action} detached component ${roleOf(node) ?? node.tag} "${nameOf(node)}"`);
	};
	const ensureEnabled = (node, action) => {
		ensureAttached(node, action);
		if (disabledState(node)) throw new Error(`cannot ${action} disabled component ${roleOf(node) ?? node.tag} "${nameOf(node)}"`);
	};
	const pointerPayload = (position, buttons, button = 0) => {
		const clientX = position.clientX ?? position.offsetX ?? 0;
		const clientY = position.clientY ?? position.offsetY ?? 0;
		const offsetX = position.offsetX ?? clientX;
		const offsetY = position.offsetY ?? clientY;
		if (!Number.isFinite(clientX) || !Number.isFinite(clientY) || !Number.isFinite(offsetX) || !Number.isFinite(offsetY)) throw new RangeError("component pointer coordinates must be finite");
		return JSON.stringify({
			clientX,
			clientY,
			offsetX,
			offsetY,
			button,
			buttons,
			mods: 0
		});
	};
	function locator(node) {
		return {
			...queries(node),
			get tag() {
				return node.tag;
			},
			get role() {
				return roleOf(node) ?? "";
			},
			get name() {
				return nameOf(node);
			},
			get text() {
				return textOf(node);
			},
			get className() {
				return node.className;
			},
			get children() {
				return node.children.map(locator);
			},
			get disabled() {
				return disabledState(node);
			},
			get checked() {
				return toggleState(node, "aria-checked");
			},
			get selected() {
				return booleanState(node, "aria-selected");
			},
			get expanded() {
				return booleanState(node, "aria-expanded");
			},
			get pressed() {
				return toggleState(node, "aria-pressed");
			},
			get current() {
				return currentState(node);
			},
			get orientation() {
				return orientationState(node);
			},
			get value() {
				return node.attributes.get("value") ?? null;
			},
			get numericValue() {
				return numericState(node, "aria-valuenow");
			},
			get minNumericValue() {
				return numericState(node, "aria-valuemin");
			},
			get maxNumericValue() {
				return numericState(node, "aria-valuemax");
			},
			get valueText() {
				return node.attributes.get("aria-valuetext") ?? null;
			},
			get transform() {
				return node.transform;
			},
			get focused() {
				return focusedNode === node;
			},
			get focusOrder() {
				return node.focusOrder;
			},
			get interactionBlocked() {
				return node.interactionBlocked;
			},
			get focusContained() {
				return node.focusContained;
			},
			attribute: (name) => node.attributes.get(name) ?? null,
			pointerDown: (position = {}) => {
				ensureEnabled(node, "press");
				commitEvent(node, EVENT_CODE.pointerdown, pointerPayload(position, 1));
			},
			pointerMove: (position = {}) => {
				ensureEnabled(node, "drag");
				commitEvent(node, EVENT_CODE.pointermove, pointerPayload(position, 1));
			},
			pointerUp: (position = {}) => {
				ensureEnabled(node, "release");
				commitEvent(node, EVENT_CODE.pointerup, pointerPayload(position, 0));
			},
			click: () => {
				ensureEnabled(node, "click");
				commitEvent(node, EVENT_CODE.pointerdown, pointerPayload({}, 1));
				commitEvent(node, EVENT_CODE.pointerup, pointerPayload({}, 0));
				commitEvent(node, EVENT_CODE.click);
			},
			contextMenu: (position = {}) => {
				ensureEnabled(node, "open context menu for");
				commitEvent(node, EVENT_CODE.contextmenu, pointerPayload(position, 0, 2));
			},
			press: (pressedKey) => {
				ensureEnabled(node, "press");
				if (pressedKey.length === 0) throw new Error("key must not be empty");
				const payload = JSON.stringify({
					key: pressedKey,
					repeat: false
				});
				commitEvent(node, EVENT_CODE.keydown, payload);
				commitEvent(node, EVENT_CODE.keyup, payload);
			},
			input: (value) => {
				ensureEnabled(node, "input");
				commitEvent(node, EVENT_CODE.input, JSON.stringify({ value }));
			},
			focus: () => {
				ensureEnabled(node, "focus");
				focusAuthoredNode(node);
			},
			blur: () => {
				ensureAttached(node, "blur");
				if (focusedNode === node) blurFocusedNode();
			},
			hover: () => {
				ensureEnabled(node, "hover");
				commitEvent(node, EVENT_CODE.pointerenter);
			},
			unhover: () => {
				ensureAttached(node, "unhover");
				commitEvent(node, EVENT_CODE.pointerleave);
			},
			resize: ({ width, height }) => {
				ensureAttached(node, "resize");
				if (!Number.isFinite(width) || width < 0 || !Number.isFinite(height) || height < 0) throw new RangeError("component size must be finite and non-negative");
				dispatchResizeObservation(node.id, width, height);
				flushUpdates();
			}
		};
	}
	let disposed = false;
	const screen = {
		...queries(null),
		flush() {
			flushUpdates();
		},
		async advanceTime(milliseconds) {
			if (options.clock !== "fake") throw new Error("advanceTime requires renderComponent(..., { clock: \"fake\" })");
			if (!Number.isFinite(milliseconds) || milliseconds < 0) throw new RangeError("component clock duration must be finite and non-negative");
			const tick = globalThis.__wabou_tick;
			const frameInterval = 16;
			let remaining = milliseconds;
			if (remaining === 0) {
				vi.advanceTimersByTime(0);
				if (typeof tick === "function") tick(fakeFrameTime);
				flushUpdates();
				await Promise.resolve();
				return;
			}
			while (remaining > 0) {
				const elapsed = Math.min(frameInterval, remaining);
				vi.advanceTimersByTime(elapsed);
				fakeFrameTime += elapsed;
				if (typeof tick === "function") tick(fakeFrameTime);
				flushUpdates();
				await Promise.resolve();
				remaining -= elapsed;
			}
		},
		async waitFor(assertion, waitOptions = {}) {
			const timeout = waitOptions.timeout ?? 1e3;
			const interval = waitOptions.interval ?? 10;
			if (!Number.isFinite(timeout) || timeout < 0) throw new RangeError("component wait timeout must be finite and non-negative");
			if (!Number.isFinite(interval) || interval <= 0) throw new RangeError("component wait interval must be finite and positive");
			let lastError;
			for (let elapsed = 0; elapsed <= timeout; elapsed += interval) {
				await Promise.resolve();
				flushUpdates();
				try {
					return await assertion();
				} catch (error) {
					lastError = error;
				}
				if (elapsed + interval <= timeout) {
					if (options.clock === "fake") await Promise.resolve();
					else await new Promise((resolve) => setTimeout(resolve, interval));
				}
			}
			const detail = lastError instanceof Error ? `: ${lastError.message}` : "";
			throw new Error(`component wait timed out after ${timeout}ms${detail}`, { cause: lastError });
		},
		dispose() {
			if (disposed) return;
			disposed = true;
			focusedNode = null;
			try {
				disposeMount?.();
			} finally {
				restore();
				if (activeScreen === screen) activeScreen = null;
			}
		}
	};
	activeScreen = screen;
	onTestFinished(() => screen.dispose());
	return screen;
}
//#endregion
export { cleanupComponents, createTestHost, renderComponent };

//# sourceMappingURL=component.mjs.map