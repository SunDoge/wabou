import { m as NodeKeyTable } from "./protocol-CraQTlUq.mjs";
//#region src/glue/resize-observer.ts
const registryKey = Symbol.for("@wabou/core.resize-observers");
const realm = globalThis;
const observers = (() => {
	const existing = realm[registryKey];
	if (existing) return existing;
	const created = new NodeKeyTable();
	realm[registryKey] = created;
	return created;
})();
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
const utf8 = new TextDecoder();
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
/** Subscribe to a host topic carrying JSON text or UTF-8 bytes. */
function subscribeJson(topic, handler, options = {}) {
	return subscribe(topic, (payload) => {
		try {
			const source = typeof payload === "string" ? payload : payload instanceof Uint8Array ? utf8.decode(payload) : void 0;
			if (source === void 0) throw new TypeError(`host message "${topic}" does not contain JSON text`);
			const parsed = JSON.parse(source);
			handler(options.decode ? options.decode(parsed) : parsed);
		} catch (error) {
			if (options.onError) options.onError(error, payload);
			else console.error(`[wabou-host] invalid JSON message for "${topic}"`, error);
		}
	});
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
	subscribeAll,
	subscribeJson
};
//#endregion
export { subscribeJson as a, subscribeAll as i, hostMessages as n, dispatchResizeObservation as o, subscribe as r, dispatchHostMessage as t };

//# sourceMappingURL=host-messages-CiIPMkh6.mjs.map