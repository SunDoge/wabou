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
/** Type-check ordinary data or an asynchronous environment-dependent factory. */
function defineConfig(config) {
	return config;
}
//#endregion
export { defineConfig, rustCapability };

//# sourceMappingURL=config.mjs.map