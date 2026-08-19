import { _ as nodeKey, a as HOST_FRAME, c as INTERACTION_POLICY, i as GRAPHIC_SOURCE, l as OP, n as EVENT_DATA_LEN, o as HOST_NODE_PAYLOAD, p as NodeKeyTable, s as HOST_RECORD_KIND, t as EVENT_CODE, u as TEXT_BEHAVIOR, x as createResourceKeyFamily } from "./protocol-DfLpXnPC.mjs";
import { a as bool, c as number, d as rgba, f as rotate2d, h as INLINE_STYLE_CONTRACT, i as auto, l as percent, m as translate2d, n as StyleValueKind, o as classes, p as shadow, r as assertInlineStyleValue, s as isTypedStyleValue, t as STYLE_VALUE, u as px } from "./style-B_gSda0o.mjs";
import { A as Portal, C as runSweep, D as writer, E as spread, M as defaultHost, N as useHost, O as VirtualList, S as render, T as setTransform2D, _ as mount, a as createElement, b as releaseOverlayRoot, c as dispatchEvent, d as getRequestEvent, f as insert, g as mergeProps, h as memo, i as createComponent, j as HostProvider, k as createFps, l as effect, m as isServer, n as acquireOverlayRoot, o as createTextNode, p as insertNode, r as applyRef, s as delegateEvents, t as Dynamic, u as getMountRoot, v as ref, w as setProp, x as removeNode, y as registerRoot } from "./renderer-aT76Sl0b.mjs";
import "./registry.mjs";
import AbortControllerPolyfill, { AbortSignal } from "abort-controller/dist/abort-controller";
import { createComponent as createComponent$1, createContext, createEffect, createSignal, flush, getOwner, useContext } from "solid-js";
//#region src/polyfills/abort-controller.ts
/** Install cancellation primitives when the embedding runtime lacks them. */
function installAbortControllerPolyfill() {
	if (!("AbortSignal" in globalThis)) Object.defineProperty(globalThis, "AbortSignal", {
		configurable: true,
		writable: true,
		value: AbortSignal
	});
	if (!("AbortController" in globalThis)) Object.defineProperty(globalThis, "AbortController", {
		configurable: true,
		writable: true,
		value: AbortControllerPolyfill
	});
}
installAbortControllerPolyfill();
//#endregion
//#region src/polyfills/fetch.ts
function normalizeHeaderName(name) {
	return String(name).toLowerCase();
}
var WabouHeaders = class {
	entriesByName = /* @__PURE__ */ new Map();
	constructor(init) {
		if (!init) return;
		if (Symbol.iterator in Object(init)) {
			for (const [name, value] of init) this.append(name, value);
			return;
		}
		for (const [name, value] of Object.entries(init)) this.append(name, value);
	}
	append(name, value) {
		const key = normalizeHeaderName(name);
		const current = this.entriesByName.get(key);
		this.entriesByName.set(key, current ? `${current}, ${String(value)}` : String(value));
	}
	delete(name) {
		this.entriesByName.delete(normalizeHeaderName(name));
	}
	get(name) {
		return this.entriesByName.get(normalizeHeaderName(name)) ?? null;
	}
	has(name) {
		return this.entriesByName.has(normalizeHeaderName(name));
	}
	set(name, value) {
		this.entriesByName.set(normalizeHeaderName(name), String(value));
	}
	entries() {
		return this.entriesByName.entries();
	}
	keys() {
		return this.entriesByName.keys();
	}
	values() {
		return this.entriesByName.values();
	}
	forEach(callback, thisArg) {
		for (const [key, value] of this.entriesByName) callback.call(thisArg, value, key, this);
	}
	[Symbol.iterator]() {
		return this.entries();
	}
	toRecord() {
		return Object.fromEntries(this.entriesByName);
	}
};
var WabouResponse = class WabouResponse {
	headers;
	status;
	statusText;
	url;
	bodyText;
	constructor(body = null, init = {}, url = "") {
		this.bodyText = body ?? "";
		this.status = init.status ?? 200;
		this.statusText = init.statusText ?? "";
		this.headers = new WabouHeaders(init.headers);
		this.url = url;
	}
	get ok() {
		return this.status >= 200 && this.status < 300;
	}
	text() {
		return Promise.resolve(this.bodyText);
	}
	json() {
		return Promise.resolve(JSON.parse(this.bodyText));
	}
	clone() {
		return new WabouResponse(this.bodyText, {
			status: this.status,
			statusText: this.statusText,
			headers: this.headers
		}, this.url);
	}
	static json(value, init = {}) {
		const headers = new WabouHeaders(init.headers);
		if (!headers.has("content-type")) headers.set("content-type", "application/json");
		return new WabouResponse(JSON.stringify(value), {
			...init,
			headers
		});
	}
};
/** Install the host-backed Fetch API surface. Safe to call again in tests. */
function installFetchPolyfill() {
	if (!("Headers" in globalThis)) Object.defineProperty(globalThis, "Headers", {
		configurable: true,
		writable: true,
		value: WabouHeaders
	});
	if (!("Response" in globalThis)) Object.defineProperty(globalThis, "Response", {
		configurable: true,
		writable: true,
		value: WabouResponse
	});
	if (!("__wabou_fetch" in globalThis)) return;
	globalThis.fetch = ((input, init) => {
		const url = typeof input === "string" ? input : input.url;
		const serializedInit = init ? {
			...init,
			headers: init.headers instanceof WabouHeaders ? init.headers.toRecord() : init.headers
		} : {};
		return globalThis.__wabou_fetch(url, JSON.stringify(serializedInit)).then((json) => {
			const data = JSON.parse(json);
			const ResponseConstructor = globalThis.Response;
			return new ResponseConstructor(data.body, {
				status: data.status,
				statusText: data.statusText,
				headers: data.headers
			}, url);
		});
	});
}
installFetchPolyfill();
//#endregion
//#region src/glue/animation-frame.ts
const rafQueue = /* @__PURE__ */ new Map();
let nextRafId = 1;
function requestAnimationFrameImpl(cb) {
	const id = nextRafId++;
	rafQueue.set(id, cb);
	return id;
}
function cancelAnimationFrameImpl(id) {
	rafQueue.delete(id);
}
function __wabou_tick(frameTime) {
	const entries = Array.from(rafQueue.entries());
	rafQueue.clear();
	flush(() => {
		for (const [_, cb] of entries) try {
			cb(frameTime);
		} catch (e) {
			__wabou_log("error", e.stack ? String(e.stack) : String(e));
		}
	});
	runSweep();
	const bytes = writer.flush();
	if (bytes) __wabou_flush(bytes);
	return rafQueue.size > 0;
}
function __wabou_has_raf() {
	return rafQueue.size > 0;
}
globalThis.requestAnimationFrame = requestAnimationFrameImpl;
globalThis.cancelAnimationFrame = cancelAnimationFrameImpl;
globalThis.__wabou_tick = __wabou_tick;
globalThis.__wabou_has_raf = __wabou_has_raf;
//#endregion
//#region src/glue/timers.ts
let nextTimerId = 1;
const active = /* @__PURE__ */ new Set();
const nativeSetTimeout = globalThis.setTimeout?.bind(globalThis);
function sleep(delay) {
	if (typeof __wabou_sleep === "function") return __wabou_sleep(delay);
	if (nativeSetTimeout) return new Promise((resolve) => nativeSetTimeout(resolve, delay));
	return Promise.reject(/* @__PURE__ */ new Error("Wabou timer host is unavailable"));
}
function reportTimerError(error) {
	const message = error instanceof Error && error.stack ? error.stack : String(error);
	if (typeof __wabou_log === "function") __wabou_log("error", message);
	else console.error(message);
}
function schedule(callback, delay, repeat, args) {
	const id = nextTimerId++;
	active.add(id);
	const run = async () => {
		await sleep(delay);
		if (!active.has(id)) return;
		try {
			callback(...args);
		} catch (error) {
			reportTimerError(error);
		}
		if (repeat && active.has(id)) run();
		else active.delete(id);
	};
	run();
	return id;
}
globalThis.setTimeout = (callback, delay = 0, ...args) => schedule(callback, Number(delay) || 0, false, args);
globalThis.setInterval = (callback, delay = 0, ...args) => schedule(callback, Number(delay) || 0, true, args);
function clearTimer(id) {
	active.delete(id);
}
globalThis.clearTimeout = clearTimer;
globalThis.clearInterval = clearTimer;
//#endregion
//#region src/glue/resize-observer.ts
const observers = new NodeKeyTable();
var WabouResizeObserver = class {
	callback;
	targets = /* @__PURE__ */ new Set();
	constructor(callback) {
		this.callback = callback;
	}
	observe(target) {
		const id = target.id;
		if (this.targets.has(id)) return;
		this.targets.add(id);
		let observed = observers.get(id);
		if (!observed) {
			observed = {
				target,
				callbacks: /* @__PURE__ */ new Set()
			};
			observers.set(id, observed);
			__wabou_resize_observe(id.lo, id.hi);
		}
		observed.callbacks.add(this.callback);
	}
	unobserve(target) {
		this.remove(target.id);
	}
	disconnect() {
		for (const id of this.targets) this.remove(id);
	}
	remove(id) {
		if (!this.targets.delete(id)) return;
		const observed = observers.get(id);
		observed?.callbacks.delete(this.callback);
		if (observed?.callbacks.size === 0) {
			observers.delete(id);
			__wabou_resize_unobserve(id.lo, id.hi);
		}
	}
};
function dispatchResizeObservation(solidId, width, height) {
	const observed = observers.get(solidId);
	if (!observed) return;
	const entry = {
		target: observed.target,
		contentRect: {
			width,
			height
		}
	};
	for (const callback of observed.callbacks) try {
		callback([entry]);
	} catch (error) {
		__wabou_log("error", error?.stack ? String(error.stack) : String(error));
	}
}
globalThis.ResizeObserver = WabouResizeObserver;
//#endregion
//#region src/glue/host-messages.ts
const listeners = /* @__PURE__ */ new Map();
const allListeners = /* @__PURE__ */ new Set();
/**
* Subscribe to host messages on `topic`.
* Returns an unsubscribe function.
*/
function subscribe(topic, handler) {
	let set = listeners.get(topic);
	if (!set) {
		set = /* @__PURE__ */ new Set();
		listeners.set(topic, set);
	}
	set.add(handler);
	return () => {
		set.delete(handler);
		if (set.size === 0) listeners.delete(topic);
	};
}
/** Subscribe to every topic; handler receives `(topic, payload)`. */
function subscribeAll(handler) {
	allListeners.add(handler);
	return () => {
		allListeners.delete(handler);
	};
}
function dispatchHostMessage(topic, payload) {
	const set = listeners.get(topic);
	if (set) for (const handler of set) try {
		handler(payload);
	} catch (error) {
		console.error(`[wabou-host] subscriber for "${topic}" threw`, error);
	}
	for (const handler of allListeners) try {
		handler(topic, payload);
	} catch (error) {
		console.error(`[wabou-host] subscribeAll handler threw`, error);
	}
}
const hostMessages = {
	subscribe,
	subscribeAll
};
//#endregion
//#region src/glue/host-frame.ts
const RECORD_HEADER_LEN = 8;
const FLAG_CANCELLABLE = 1;
const textDecoder = new TextDecoder();
function viewOf(input) {
	const bytes = input instanceof ArrayBuffer ? new Uint8Array(input) : new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
	return {
		bytes,
		view: new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
	};
}
/**
* Decode the complete frame before dispatching any record. Malformed frames
* are atomic: no listener, observer or application subscriber is called.
*/
function decodeAndDispatchHostFrame(input) {
	const { bytes, view } = viewOf(input);
	if (view.byteLength < HOST_FRAME.HeaderLen) throw new TypeError("short HostEventFrame header");
	if (view.getUint32(0, true) !== HOST_FRAME.Magic) throw new TypeError("invalid HostEventFrame magic");
	if (view.getUint16(4, true) !== HOST_FRAME.Version) throw new TypeError("unsupported HostEventFrame version");
	const count = view.getUint32(24, true);
	const byteLen = view.getUint32(28, true);
	if (byteLen !== view.byteLength) throw new TypeError("HostEventFrame byte length mismatch");
	let offset = HOST_FRAME.HeaderLen;
	const records = [];
	const requireBytes = (length, end) => {
		if (length < 0 || offset + length > end) throw new TypeError("truncated HostEventFrame record");
	};
	for (let index = 0; index < count; index++) {
		if (offset + RECORD_HEADER_LEN > byteLen) throw new TypeError("truncated HostEventFrame record header");
		const kind = view.getUint8(offset);
		const flags = view.getUint8(offset + 1);
		const recordLen = view.getUint32(offset + 4, true);
		if (recordLen < RECORD_HEADER_LEN || offset + recordLen > byteLen) throw new TypeError("invalid HostEventFrame record length");
		const end = offset + recordLen;
		offset += RECORD_HEADER_LEN;
		if (kind === HOST_RECORD_KIND.NodeEvent) {
			requireBytes(16, end);
			const target = nodeKey(view.getUint32(offset, true), view.getUint32(offset + 4, true));
			const eventCode = view.getUint8(offset + 8);
			const payloadKind = view.getUint8(offset + 9);
			const eventId = view.getUint32(offset + 12, true);
			offset += 16;
			if (payloadKind === HOST_NODE_PAYLOAD.None) records.push({
				kind: "node",
				flags,
				target,
				eventCode,
				eventId,
				json: ""
			});
			else if (payloadKind === HOST_NODE_PAYLOAD.Numeric) {
				requireBytes(8 * EVENT_DATA_LEN, end);
				const numeric = new Float64Array(EVENT_DATA_LEN);
				for (let slot = 0; slot < numeric.length; slot++) numeric[slot] = view.getFloat64(offset + slot * 8, true);
				offset += 8 * numeric.length;
				records.push({
					kind: "node",
					flags,
					target,
					eventCode,
					eventId,
					json: "",
					numeric
				});
			} else if (payloadKind === HOST_NODE_PAYLOAD.Json) {
				requireBytes(4, end);
				const len = view.getUint32(offset, true);
				offset += 4;
				requireBytes(len, end);
				const json = textDecoder.decode(bytes.subarray(offset, offset + len));
				offset += len;
				records.push({
					kind: "node",
					flags,
					target,
					eventCode,
					eventId,
					json
				});
			} else throw new TypeError(`unknown node payload kind ${payloadKind}`);
		} else if (kind === HOST_RECORD_KIND.Resize) {
			requireBytes(16, end);
			records.push({
				kind: "resize",
				target: nodeKey(view.getUint32(offset, true), view.getUint32(offset + 4, true)),
				width: view.getFloat32(offset + 8, true),
				height: view.getFloat32(offset + 12, true)
			});
			offset += 16;
		} else if (kind === HOST_RECORD_KIND.ApplicationMessage) {
			requireBytes(2, end);
			const topicLen = view.getUint16(offset, true);
			offset += 2;
			requireBytes(topicLen + 1, end);
			const topic = textDecoder.decode(bytes.subarray(offset, offset + topicLen));
			offset += topicLen;
			const payloadKind = view.getUint8(offset++);
			let payload;
			if (payloadKind === 0) payload = null;
			else if (payloadKind === 1) {
				requireBytes(1, end);
				payload = view.getUint8(offset++) !== 0;
			} else if (payloadKind === 2) {
				requireBytes(4, end);
				payload = view.getInt32(offset, true);
				offset += 4;
			} else if (payloadKind === 3) {
				requireBytes(8, end);
				payload = view.getFloat64(offset, true);
				offset += 8;
			} else if (payloadKind === 4) {
				requireBytes(2, end);
				const len = view.getUint16(offset, true);
				offset += 2;
				requireBytes(len, end);
				payload = textDecoder.decode(bytes.subarray(offset, offset + len));
				offset += len;
			} else if (payloadKind === 5) {
				requireBytes(4, end);
				const len = view.getUint32(offset, true);
				offset += 4;
				requireBytes(len, end);
				payload = bytes.subarray(offset, offset + len).slice();
				offset += len;
			} else throw new TypeError(`unknown application payload kind ${payloadKind}`);
			records.push({
				kind: "message",
				topic,
				payload
			});
		} else records.push({ kind: "unknown" });
		if (offset > end) throw new TypeError("HostEventFrame record overflow");
		offset = end;
	}
	if (offset !== byteLen) throw new TypeError("trailing HostEventFrame bytes");
	const prevented = [];
	let needsTick = false;
	flush(() => {
		for (const record of records) if (record.kind === "node") {
			if (dispatchEvent(record.target, record.eventCode, record.json, record.numeric) && (record.flags & FLAG_CANCELLABLE) !== 0 && record.eventId !== 0) prevented.push(record.eventId);
			needsTick = true;
		} else if (record.kind === "resize") {
			dispatchResizeObservation(record.target, record.width, record.height);
			needsTick = true;
		} else if (record.kind === "message") {
			dispatchHostMessage(record.topic, record.payload);
			needsTick = true;
		}
	});
	return {
		preventedEventIds: prevented.length > 0 ? Uint32Array.from(prevented) : void 0,
		needsTick
	};
}
function __wabou_dispatch_host_frame(frame) {
	return decodeAndDispatchHostFrame(frame);
}
globalThis.__wabou_dispatch_host_frame = __wabou_dispatch_host_frame;
//#endregion
//#region src/generated/effect-abi.ts
const effectOps = Object.freeze({
	clipboardRead: {
		capability: 1,
		method: 1
	},
	clipboardWrite: {
		capability: 1,
		method: 2
	},
	windowCreate: {
		capability: 2,
		method: 1
	},
	windowClose: {
		capability: 2,
		method: 2
	},
	windowSetMaximized: {
		capability: 2,
		method: 3
	},
	windowSetTitle: {
		capability: 2,
		method: 4
	},
	windowMinimize: {
		capability: 2,
		method: 5
	},
	windowStartDragging: {
		capability: 2,
		method: 6
	},
	contextMenuShow: {
		capability: 3,
		method: 1
	},
	appDirsResolve: {
		capability: 4,
		method: 1
	},
	dialogOpen: {
		capability: 5,
		method: 1
	},
	dialogSave: {
		capability: 5,
		method: 2
	},
	dialogPickDirectory: {
		capability: 5,
		method: 3
	},
	dialogMessage: {
		capability: 5,
		method: 4
	},
	notificationShow: {
		capability: 6,
		method: 1
	}
});
//#endregion
//#region src/glue/effects.ts
const pending = /* @__PURE__ */ new Map();
function assertAbi() {
	if (__wabou_effect_abi !== 3) throw new Error(`Wabou effect ABI mismatch: bundle=3, host=${__wabou_effect_abi}`);
}
function submit(op, payload) {
	assertAbi();
	return __wabou_effect_submit(op.capability, op.method, JSON.stringify(payload ?? null));
}
function dispatchEffect(op, payload) {
	return new Promise((resolve, reject) => {
		const id = submit(op, payload);
		pending.set(id, {
			op,
			resolve,
			reject
		});
	});
}
/** Submit a command without retaining a Promise or callback. */
function dispatchFireAndForget(op, payload) {
	submit(op, payload);
}
function complete(id, capability, method, status, payloadJson) {
	const request = pending.get(id);
	if (!request) return;
	pending.delete(id);
	if (request.op.capability !== capability || request.op.method !== method) {
		request.reject(/* @__PURE__ */ new Error(`Native effect ${id} completed with the wrong operation`));
		return;
	}
	if (status === 1) {
		const error = /* @__PURE__ */ new Error("Native effect was cancelled");
		error.name = "AbortError";
		request.reject(error);
		return;
	}
	const payload = JSON.parse(payloadJson);
	if (status === 2) {
		const error = payload;
		request.reject(new Error(error.message ?? "Native effect failed"));
		return;
	}
	request.resolve(payload);
}
globalThis.__wabou_effect_complete = complete;
//#endregion
//#region src/glue/window.ts
const windowKeys = createResourceKeyFamily("window");
function windowKeyFromJSON(value) {
	return windowKeys.fromJSON(value);
}
function handle(id) {
	return Object.freeze({
		id,
		close: () => dispatchFireAndForget(effectOps.windowClose, { windowId: id }),
		minimize: () => dispatchFireAndForget(effectOps.windowMinimize, { windowId: id }),
		setMaximized: (value) => dispatchFireAndForget(effectOps.windowSetMaximized, {
			windowId: id,
			value
		}),
		setTitle: (title) => dispatchFireAndForget(effectOps.windowSetTitle, {
			windowId: id,
			title
		}),
		startDragging: () => dispatchFireAndForget(effectOps.windowStartDragging, { windowId: id })
	});
}
/** Create an independent native window running this application's bundle. */
function createWindow(options = {}) {
	return dispatchEffect(effectOps.windowCreate, options).then((key) => handle(windowKeyFromJSON(key)));
}
/** An imperative handle for the native window that owns this JS runtime. */
function currentWindow() {
	return handle(windowKeys.fromParts(__wabou_window_id_lo, __wabou_window_id_hi));
}
//#endregion
//#region src/glue/platform-context.ts
const PlatformContext = createContext({});
/** Override native services for one Solid subtree, primarily for tests and previews. */
function PlatformProvider(props) {
	const parent = useContext(PlatformContext) ?? {};
	return createComponent$1(PlatformContext, {
		value: {
			get clipboard() {
				return props.value.clipboard ?? parent.clipboard;
			},
			get dialog() {
				return props.value.dialog ?? parent.dialog;
			},
			get notification() {
				return props.value.notification ?? parent.notification;
			},
			get window() {
				return props.value.window ?? parent.window;
			}
		},
		get children() {
			return props.children;
		}
	});
}
function usePlatformServices() {
	return getOwner() ? useContext(PlatformContext) : {};
}
//#endregion
//#region src/glue/window-metrics.ts
/**
* Create a reactive native-window size query without CSS media-query semantics.
* A zero-sized pre-boot viewport never matches, avoiding a compact-layout flash.
*/
function createWindowMatch(query, window = useWindow()) {
	const entries = [
		["minWidth", query.minWidth],
		["maxWidth", query.maxWidth],
		["minHeight", query.minHeight],
		["maxHeight", query.maxHeight]
	];
	for (const [name, value] of entries) if (value !== void 0 && (!Number.isFinite(value) || value < 0)) throw new RangeError(`${name} must be a finite non-negative number`);
	if (query.minWidth !== void 0 && query.maxWidth !== void 0 && query.minWidth > query.maxWidth) throw new RangeError("minWidth cannot exceed maxWidth");
	if (query.minHeight !== void 0 && query.maxHeight !== void 0 && query.minHeight > query.maxHeight) throw new RangeError("minHeight cannot exceed maxHeight");
	return () => {
		const width = window.width();
		const height = window.height();
		if (width <= 0 || height <= 0) return false;
		return (query.minWidth === void 0 || width >= query.minWidth) && (query.maxWidth === void 0 || width <= query.maxWidth) && (query.minHeight === void 0 || height >= query.minHeight) && (query.maxHeight === void 0 || height <= query.maxHeight);
	};
}
const initial = {
	windowId: windowKeyFromJSON({
		lo: globalThis.__wabou_window_id_lo ?? 1,
		hi: globalThis.__wabou_window_id_hi ?? 1
	}),
	logicalWidth: 0,
	logicalHeight: 0,
	physicalWidth: 0,
	physicalHeight: 0,
	scaleFactor: 1,
	maximized: false,
	focused: false
};
const [metrics, setMetrics] = createSignal(initial, { equals: false });
subscribe("wabou:window-metrics", (payload) => {
	if (typeof payload !== "string") return;
	const next = JSON.parse(payload);
	setMetrics({
		...next,
		windowId: windowKeyFromJSON(next.windowId)
	});
});
const state = {
	get id() {
		return metrics().windowId;
	},
	close: () => currentWindow().close(),
	minimize: () => currentWindow().minimize(),
	setMaximized: (value) => currentWindow().setMaximized(value),
	setTitle: (title) => currentWindow().setTitle(title),
	startDragging: () => currentWindow().startDragging(),
	metrics,
	width: () => metrics().logicalWidth,
	height: () => metrics().logicalHeight,
	scaleFactor: () => metrics().scaleFactor,
	maximized: () => metrics().maximized,
	focused: () => metrics().focused
};
/** Reactive state and controls for the native window owning this JS runtime. */
function useWindow() {
	return usePlatformServices().window ?? state;
}
//#endregion
//#region src/glue/clipboard.ts
const clipboard = Object.freeze({
	readText: () => dispatchEffect(effectOps.clipboardRead),
	writeText: (text) => dispatchEffect(effectOps.clipboardWrite, { text: String(text) }).then(() => void 0)
});
/** Stable clipboard capability for use inside Solid components. */
function useClipboard() {
	return usePlatformServices().clipboard ?? clipboard;
}
//#endregion
//#region src/glue/app-dirs.ts
let resolved;
/** Resolve all app-private roots once and reuse the same native result. */
function resolveAppDirectories() {
	return resolved ??= dispatchEffect(effectOps.appDirsResolve);
}
const appDirs = Object.freeze({
	resolve: resolveAppDirectories,
	config: () => resolveAppDirectories().then((paths) => paths.configDir),
	data: () => resolveAppDirectories().then((paths) => paths.dataDir),
	localData: () => resolveAppDirectories().then((paths) => paths.localDataDir),
	cache: () => resolveAppDirectories().then((paths) => paths.cacheDir),
	log: () => resolveAppDirectories().then((paths) => paths.logDir),
	resource: () => resolveAppDirectories().then((paths) => paths.resourceDir),
	temp: () => resolveAppDirectories().then((paths) => paths.tempDir)
});
const appConfigDir = appDirs.config;
const appDataDir = appDirs.data;
const appLocalDataDir = appDirs.localData;
const appCacheDir = appDirs.cache;
const appLogDir = appDirs.log;
const resourceDir = appDirs.resource;
const tempDir = appDirs.temp;
//#endregion
//#region src/glue/dialog.ts
function normalizeFilters(filters) {
	return (filters ?? []).map((filter) => ({
		name: String(filter.name),
		extensions: filter.extensions.map((extension) => String(extension).replace(/^\./, "")).filter(Boolean)
	}));
}
const dialog = Object.freeze({
	open(options = {}) {
		return dispatchEffect(effectOps.dialogOpen, {
			...options,
			filters: normalizeFilters(options.filters),
			multiple: options.multiple ?? false
		});
	},
	save(options = {}) {
		return dispatchEffect(effectOps.dialogSave, {
			...options,
			filters: normalizeFilters(options.filters)
		}).then((paths) => paths?.[0] ?? null);
	},
	pickDirectory(options = {}) {
		return dispatchEffect(effectOps.dialogPickDirectory, options).then((paths) => paths?.[0] ?? null);
	},
	message(options) {
		return dispatchEffect(effectOps.dialogMessage, {
			...options,
			message: String(options.message),
			level: options.level ?? "info",
			buttons: options.buttons ?? "ok"
		});
	}
});
function useDialog() {
	return usePlatformServices().dialog ?? dialog;
}
//#endregion
//#region src/glue/notification.ts
const notification = Object.freeze({ show(options) {
	return dispatchEffect(effectOps.notificationShow, {
		...options,
		title: String(options.title),
		silent: options.silent ?? false
	}).then(() => void 0);
} });
function useNotification() {
	return usePlatformServices().notification ?? notification;
}
//#endregion
//#region src/glue/intl.ts
/**
* Operating-system locale facts. Standards-compatible formatting is installed
* separately by the FormatJS-backed Intl polyfill.
*/
const intl = Object.freeze({
	locale() {
		return defaultHost.intl.locale();
	},
	timeZone() {
		return defaultHost.intl.timeZone();
	},
	today() {
		return defaultHost.intl.today();
	}
});
//#endregion
//#region src/glue/color-theme.tsx
const [current, setCurrent] = createSignal();
let currentPalette;
let activeAnimation;
function paletteFor(name) {
	if (!name) throw new Error("Wabou color theme name cannot be empty");
	const parsed = JSON.parse(globalThis.__wabou_get_color_theme_palette(name));
	if (!Array.isArray(parsed) || !parsed.every((value) => Number.isInteger(value))) throw new Error(`Wabou color theme \`${name}\` returned an invalid palette`);
	return Uint32Array.from(parsed);
}
function easingFunction(easing) {
	if (typeof easing === "function") return easing;
	switch (easing) {
		case "linear": return (t) => t;
		case "ease-in": return (t) => t * t * t;
		case "ease-in-out": return (t) => t < .5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2;
		default: return (t) => 1 - (1 - t) ** 3;
	}
}
function srgbToLinear(value) {
	return value <= .04045 ? value / 12.92 : ((value + .055) / 1.055) ** 2.4;
}
function linearToSrgb(value) {
	return value <= .0031308 ? 12.92 * value : 1.055 * value ** (1 / 2.4) - .055;
}
function srgbToOklab(red, green, blue) {
	const r = srgbToLinear(red);
	const g = srgbToLinear(green);
	const b = srgbToLinear(blue);
	const l = Math.cbrt(.4122214708 * r + .5363325363 * g + .0514459929 * b);
	const m = Math.cbrt(.2119034982 * r + .6806995451 * g + .1073969566 * b);
	const s = Math.cbrt(.0883024619 * r + .2817188376 * g + .6299787005 * b);
	return [
		.2104542553 * l + .793617785 * m - .0040720468 * s,
		1.9779984951 * l - 2.428592205 * m + .4505937099 * s,
		.0259040371 * l + .7827717662 * m - .808675766 * s
	];
}
function oklabToSrgb(lightness, a, b) {
	const l = (lightness + .3963377774 * a + .2158037573 * b) ** 3;
	const m = (lightness - .1055613458 * a - .0638541728 * b) ** 3;
	const s = (lightness - .0894841775 * a - 1.291485548 * b) ** 3;
	return [
		linearToSrgb(4.0767416621 * l - 3.3077115913 * m + .2309699292 * s),
		linearToSrgb(-1.2684380046 * l + 2.6097574011 * m - .3413193965 * s),
		linearToSrgb(-.0041960863 * l - .7034186147 * m + 1.707614701 * s)
	];
}
function channel(value) {
	return Math.round(Math.min(1, Math.max(0, value)) * 255);
}
function mixColor(from, to, progress, oklab) {
	const fromRgb = [
		(from >>> 24 & 255) / 255,
		(from >>> 16 & 255) / 255,
		(from >>> 8 & 255) / 255
	];
	const toRgb = [
		(to >>> 24 & 255) / 255,
		(to >>> 16 & 255) / 255,
		(to >>> 8 & 255) / 255
	];
	const fromValue = oklab ? srgbToOklab(...fromRgb) : fromRgb;
	const toValue = oklab ? srgbToOklab(...toRgb) : toRgb;
	const mixed = [
		fromValue[0] + (toValue[0] - fromValue[0]) * progress,
		fromValue[1] + (toValue[1] - fromValue[1]) * progress,
		fromValue[2] + (toValue[2] - fromValue[2]) * progress
	];
	const [red, green, blue] = oklab ? oklabToSrgb(...mixed) : mixed;
	const alpha = channel(((from & 255) + ((to & 255) - (from & 255)) * progress) / 255);
	return (channel(red) << 24 | channel(green) << 16 | channel(blue) << 8 | alpha) >>> 0;
}
function submitPalette(colors) {
	currentPalette = colors;
	globalThis.__wabou_set_color_palette(colors);
}
const colorTheme = {
	current,
	set(name) {
		activeAnimation?.cancel();
		const palette = paletteFor(name);
		globalThis.__wabou_set_color_theme(name);
		currentPalette = palette;
		setCurrent(name);
	},
	getPalette: paletteFor,
	setPalette(colors) {
		if (!(colors instanceof Uint32Array)) throw new TypeError("Wabou color palette must be a Uint32Array");
		submitPalette(colors.slice());
	},
	animateTo(name, options = {}) {
		const target = paletteFor(name);
		const source = currentPalette?.slice();
		if (!source || options.duration === 0) {
			this.set(name);
			return {
				finished: Promise.resolve(),
				cancel() {}
			};
		}
		if (source.length !== target.length) throw new Error("Wabou color theme palettes have inconsistent lengths");
		activeAnimation?.cancel();
		const durationMs = Math.max(0, options.duration ?? .28) * 1e3;
		const ease = easingFunction(options.easing);
		const frame = new Uint32Array(source.length);
		let raf = 0;
		let start;
		let settled = false;
		let finish;
		const controls = {
			finished: new Promise((resolve) => {
				finish = resolve;
			}),
			cancel() {
				if (settled) return;
				settled = true;
				cancelAnimationFrame(raf);
				finish();
			}
		};
		activeAnimation = controls;
		const tick = (timestamp) => {
			if (settled) return;
			start ??= timestamp;
			const linear = durationMs === 0 ? 1 : Math.min(1, (timestamp - start) / durationMs);
			const progress = Math.min(1, Math.max(0, ease(linear)));
			for (let index = 0; index < frame.length; index++) frame[index] = mixColor(source[index], target[index], progress, options.colorSpace !== "srgb");
			submitPalette(frame.slice());
			if (linear < 1) {
				raf = requestAnimationFrame(tick);
				return;
			}
			settled = true;
			currentPalette = target;
			globalThis.__wabou_set_color_theme(name);
			setCurrent(name);
			if (activeAnimation === controls) activeAnimation = void 0;
			finish();
		};
		raf = requestAnimationFrame(tick);
		return controls;
	}
};
const ColorThemeContext = createContext(colorTheme);
/** Selects one compiled color palette for the current native window. */
function ColorThemeProvider(props) {
	let initialized = false;
	createEffect(() => [props.theme, props.transition], ([theme, transition]) => {
		const animation = initialized && transition ? colorTheme.animateTo(theme, transition) : (colorTheme.set(theme), void 0);
		initialized = true;
		return animation ? () => animation.cancel() : void 0;
	});
	return createComponent$1(ColorThemeContext, {
		value: colorTheme,
		get children() {
			return props.children;
		}
	});
}
function useColorTheme() {
	return useContext(ColorThemeContext);
}
//#endregion
//#region src/glue/native-menu.ts
/** Show a platform context menu and resolve with the selected item id. */
function showNativeMenu(options) {
	return dispatchEffect(effectOps.contextMenuShow, {
		windowId: options.windowId ?? currentWindow().id,
		position: options.position,
		items: options.items
	});
}
//#endregion
export { ColorThemeProvider, Dynamic, EVENT_CODE, GRAPHIC_SOURCE, HostProvider, INLINE_STYLE_CONTRACT, INTERACTION_POLICY, OP, PlatformProvider, Portal, STYLE_VALUE, StyleValueKind, TEXT_BEHAVIOR, VirtualList, acquireOverlayRoot, appCacheDir, appConfigDir, appDataDir, appDirs, appLocalDataDir, appLogDir, applyRef, assertInlineStyleValue, auto, bool, classes, clipboard, colorTheme, createComponent, createElement, createFps, createTextNode, createWindow, createWindowMatch, currentWindow, defaultHost, delegateEvents, dialog, dispatchEvent, effect, getMountRoot, getRequestEvent, hostMessages, insert, insertNode, intl, isServer, isTypedStyleValue, memo, mergeProps, mount, notification, number, percent, px, ref, registerRoot, releaseOverlayRoot, removeNode, render, resolveAppDirectories, resourceDir, rgba, rotate2d, runSweep, setProp, setTransform2D, shadow, showNativeMenu, spread, subscribeAll as subscribeAllHostMessages, subscribe as subscribeHostMessages, tempDir, translate2d, useClipboard, useColorTheme, useDialog, useHost, useNotification, useWindow, writer };

//# sourceMappingURL=index.mjs.map