import { EVENT_CODE, HostProvider, dispatchEvent, mount, writer } from "@wabou/core/renderer";
import { dispatchResizeObservation } from "@wabou/core/testing";
import { createComponent, flush } from "solid-js";
import { onTestFinished } from "vitest";
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
function installHostStub(name) {
	const target = globalThis;
	const hadOwn = Object.hasOwn(target, name);
	const previous = target[name];
	if (typeof previous !== "function") target[name] = () => {};
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
	if (activeHarness) throw new Error("renderComponent supports one active component screen at a time");
	activeHarness = true;
	const restoreHostStubs = [installHostStub("__wabou_resize_observe"), installHostStub("__wabou_resize_unobserve")];
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
			className: "",
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
	writer.dropNode = (id) => {
		const node = nodes.get(key(id));
		if (node) detach(node);
		nodes.delete(key(id));
		originals.dropNode.call(writer, id);
	};
	let disposeMount = null;
	let flushDepth = 0;
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
		activeHarness = false;
	};
	try {
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
	const all = () => {
		const result = [];
		const visit = (node) => {
			result.push(node);
			node.children.forEach(visit);
		};
		roots.forEach(visit);
		return result;
	};
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
	const ensureEnabled = (node, action) => {
		if (node.attributes.has("disabled") || node.attributes.get("aria-disabled") === "true") throw new Error(`cannot ${action} disabled component ${roleOf(node) ?? node.tag} "${nameOf(node)}"`);
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
	const locator = (node) => ({
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
		get focused() {
			return focusedNode === node;
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
			if (focusedNode === node) blurFocusedNode();
		},
		hover: () => {
			ensureEnabled(node, "hover");
			commitEvent(node, EVENT_CODE.pointerenter);
		},
		unhover: () => commitEvent(node, EVENT_CODE.pointerleave),
		resize: ({ width, height }) => {
			if (!Number.isFinite(width) || width < 0 || !Number.isFinite(height) || height < 0) throw new RangeError("component size must be finite and non-negative");
			dispatchResizeObservation(node.id, width, height);
			flushUpdates();
		}
	});
	const select = (matches, description, index, required = true) => {
		if (index === void 0 && matches.length > 1) throw new Error(`found ${matches.length} matches for ${description}; pass an index`);
		const match = matches[index ?? 0];
		if (!match && required) throw new Error(`no component found for ${description}`);
		return match ? locator(match) : null;
	};
	let disposed = false;
	const screen = {
		getByRole(role, options = {}) {
			return select(all().filter((node) => roleOf(node) === role && (options.name === void 0 || nameOf(node) === options.name)), `role=${role}${options.name === void 0 ? "" : ` name=${JSON.stringify(options.name)}`}`, options.index);
		},
		queryByRole(role, options = {}) {
			return select(all().filter((node) => roleOf(node) === role && (options.name === void 0 || nameOf(node) === options.name)), `role=${role}${options.name === void 0 ? "" : ` name=${JSON.stringify(options.name)}`}`, options.index, false);
		},
		flush() {
			flushUpdates();
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