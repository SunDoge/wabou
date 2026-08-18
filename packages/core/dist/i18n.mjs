import { createSignal, flush } from "solid-js";
//#region src/i18n.ts
/**
* Own native-app locale state while leaving message compilation to libraries
* such as Paraglide. Locale selection is always explicit, so generated Web
* URL, cookie, and local-storage strategies are unnecessary.
*/
function createLocale(initialLocale, options = {}) {
	let current = initialLocale;
	const [tracked, write] = createSignal(() => current);
	const locale = () => {
		tracked();
		return current;
	};
	return {
		locale,
		set(next) {
			if (next === current) return;
			current = next;
			flush(() => write(() => next));
			options.onChange?.(next);
		},
		message(message, inputs) {
			return message(inputs, { locale: locale() });
		}
	};
}
//#endregion
export { createLocale };

//# sourceMappingURL=i18n.mjs.map