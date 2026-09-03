//#region src/runtime/client.ts
const wabouGlobal = globalThis;
const existingRecords = wabouGlobal.__wabou_hmr_records;
const records = existingRecords ?? /* @__PURE__ */ new Map();
let fullReloadSnapshot = null;
if (!existingRecords) wabouGlobal.__wabou_hmr_records = records;
function nextContext(record) {
	return record.next;
}
function createHotContext(ownerPath) {
	let record = records.get(ownerPath);
	if (!record) {
		record = {
			data: {},
			current: null,
			next: null,
			loading: false
		};
		records.set(ownerPath, record);
	}
	const context = {
		data: record.data,
		accepted: [],
		disposed: [],
		invalidated: false,
		accept(callback) {
			if (typeof callback === "function") this.accepted.push(callback);
		},
		dispose(callback) {
			this.disposed.push(callback);
		},
		decline() {
			this.invalidated = true;
		},
		invalidate() {
			this.invalidated = true;
		},
		on() {},
		send() {},
		prune() {}
	};
	if (record.loading) record.next = context;
	else record.current = context;
	return context;
}
function updateStyle(id, css) {
	__wabou_vite_update_style(id, css);
}
function removeStyle(id) {
	__wabou_vite_remove_style(id);
}
/**
* Apply one Vite JS HMR update. Returns:
* - `true` — Solid refresh (or another accept handler) took the update
* - `false` — no hot context / declined / invalidate / import error → host
*   should full-reload the entry rather than leave a half-applied tree
*/
wabouGlobal.__wabou_apply_hmr = async (path, acceptedPath, timestamp) => {
	const record = records.get(path);
	if (!record?.current) {
		console.warn(`[wabou-hmr] no hot context for ${path}; host will full-reload`);
		return false;
	}
	const previous = record.current;
	record.loading = true;
	record.next = null;
	try {
		const module = await import(`${acceptedPath}${acceptedPath.includes("?") ? "&" : "?"}t=${timestamp}`);
		const next = nextContext(record);
		if (!next || next.invalidated) {
			console.warn(`[wabou-hmr] update for ${path} was invalidated/declined; host will full-reload`);
			return false;
		}
		for (const dispose of previous.disposed) try {
			dispose(record.data);
		} catch (error) {
			console.error(`[wabou-hmr] dispose failed for ${path}`, error);
		}
		record.current = next;
		for (const accept of previous.accepted) try {
			accept(module);
		} catch (error) {
			console.error(`[wabou-hmr] accept handler failed for ${path}`, error);
			return false;
		}
		if (previous.invalidated) {
			console.warn(`[wabou-hmr] ${path} invalidated during accept; host will full-reload`);
			return false;
		}
		return true;
	} catch (error) {
		console.error(`[wabou-hmr] failed to import ${acceptedPath}`, error);
		return false;
	} finally {
		record.loading = false;
		record.next = null;
	}
};
/**
* Replace hot records transactionally during an entry reload. Vite can report
* a reload while a saved file still fails to transform; retaining the previous
* records lets the next valid save use HMR instead of leaving the runtime in a
* permanently degraded state.
*/
function beginFullReload() {
	if (fullReloadSnapshot) return;
	fullReloadSnapshot = new Map(records);
	records.clear();
}
function commitFullReload() {
	fullReloadSnapshot = null;
}
function rollbackFullReload() {
	if (!fullReloadSnapshot) return;
	records.clear();
	for (const [path, record] of fullReloadSnapshot) records.set(path, record);
	fullReloadSnapshot = null;
}
wabouGlobal.__wabou_hmr_begin_full_reload = beginFullReload;
wabouGlobal.__wabou_hmr_commit_full_reload = commitFullReload;
wabouGlobal.__wabou_hmr_rollback_full_reload = rollbackFullReload;
//#endregion
export { beginFullReload, commitFullReload, createHotContext, removeStyle, rollbackFullReload, updateStyle };

//# sourceMappingURL=runtime.mjs.map