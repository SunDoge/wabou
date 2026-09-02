import { createSignal, flush } from "solid-js";
//#region src/i18n.ts
/**
* Own native-app locale state while leaving message compilation to libraries
* such as Paraglide. Locale selection is always explicit, so generated Web
* URL, cookie, and local-storage strategies are unnecessary.
*/
function createLocale(initialLocale, options = {}) {
	let current = initialLocale;
	let flushScheduled = false;
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
			write(() => next);
			if (!flushScheduled) {
				flushScheduled = true;
				Promise.resolve().then(() => {
					flushScheduled = false;
					flush();
				});
			}
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