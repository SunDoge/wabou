//#region src/config.ts
/** Select an implementation for a generated contract without repeating its methods. */
function rustCapability({ contract, manifest, watch = [], features = [], targetDirectory }) {
	return {
		...contract,
		implementation: {
			kind: "project",
			manifest,
			watch,
			features,
			...targetDirectory ? { targetDirectory } : {}
		}
	};
}
/** Register a resident state slot and a watched Rust widget implementation. */
function rustWidget({ tag, version, initialState, manifest, watch = [], features = [], targetDirectory }) {
	return {
		tag,
		version,
		initialState,
		implementation: {
			kind: "project",
			manifest,
			watch,
			features,
			...targetDirectory ? { targetDirectory } : {}
		}
	};
}
/** Bind exports from export_plugin! without one watcher/library per consumer. */
function rustPlugin({ name, manifest, watch = [], features = [], targetDirectory, capabilities = [], widgets = [] }) {
	return {
		name,
		capabilities,
		widgets,
		implementation: {
			kind: "project",
			manifest,
			watch,
			features,
			...targetDirectory ? { targetDirectory } : {}
		}
	};
}
/** Type-check ordinary data or an asynchronous environment-dependent factory. */
function defineConfig(config) {
	return config;
}
//#endregion
export { defineConfig, rustCapability, rustPlugin, rustWidget };

//# sourceMappingURL=config.mjs.map