import { m as NodeKeyTable } from "./protocol-BkE2Fvea.mjs";
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
export { dispatchResizeObservation as t };

//# sourceMappingURL=resize-observer-dWY6jptf.mjs.map