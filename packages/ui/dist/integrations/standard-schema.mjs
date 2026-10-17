import { FORM_ERROR } from "../primitives/interactions/form-draft.mjs";
//#region src/integrations/standard-schema.ts
function isPromiseLike(value) {
	return typeof value.then === "function";
}
function issueKey(value, issue) {
	const segment = issue.path?.[0];
	const candidate = typeof segment === "object" && segment !== null && "key" in segment ? segment.key : segment;
	return (typeof candidate === "string" || typeof candidate === "number" || typeof candidate === "symbol") && Reflect.has(value, candidate) ? candidate : FORM_ERROR;
}
/**
* Adapt any synchronous Standard Schema V1 implementation (including Valibot,
* Zod, and ArkType) to `createFormDraft` without coupling Wabou to its API.
*/
function createStandardSchemaValidator(schema) {
	return (value) => {
		const result = schema["~standard"].validate(value);
		if (isPromiseLike(result)) throw new TypeError("createFormDraft requires synchronous validation; validate asynchronous schemas before submission");
		if (!result.issues) return {};
		const errors = {};
		for (const issue of result.issues) {
			const key = issueKey(value, issue);
			errors[key] ??= issue.message;
		}
		return errors;
	};
}
//#endregion
export { createStandardSchemaValidator };

//# sourceMappingURL=standard-schema.mjs.map