//#region src/components/directory-picker-state.ts
function directoryPickerOptions(value, options) {
	return {
		...options,
		directory: options?.directory ?? (value.trim() || void 0)
	};
}
//#endregion
export { directoryPickerOptions };

//# sourceMappingURL=directory-picker-state.mjs.map