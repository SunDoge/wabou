//#region ../solid-renderer/src/jsx.ts
function jsx() {
	throw new Error("Wabou JSX must be compiled by the Solid transform");
}
const jsxs = jsx;
const jsxDEV = jsx;
const Fragment = (props) => props.children;
//#endregion
export { Fragment, jsx, jsxDEV, jsxs };

//# sourceMappingURL=jsx-runtime.mjs.map