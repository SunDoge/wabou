import { t as intl } from "./intl-BsQ-mO8K.mjs";
import { createSignal, flush } from "solid-js";
//#region src/i18n.ts
/** Paraglide registry adapter backed by Wabou's native formatters. */
function plural(locale, input, options) {
	return intl.pluralCategory(Number(input), {
		locale,
		...options
	});
}
/** Paraglide registry adapter backed by Wabou's native formatters. */
function number(locale, input, options) {
	return intl.formatNumber(Number(input), {
		locale,
		...options
	});
}
function icuPound(locale, input, options) {
	const value = typeof input === "number" ? input : Number(input);
	if (!Number.isFinite(value)) return String(input);
	return number(locale, value - Number(options?.offset ?? 0));
}
/** Paraglide registry adapter backed by Wabou's native formatters. */
function datetime(locale, input, options) {
	return intl.formatDate(input, {
		locale,
		...options
	});
}
/** Paraglide registry adapter backed by Wabou's native formatters. */
function relativetime(locale, input, options) {
	const { unit, ...format } = options;
	return intl.formatRelativeTime(Number(input), unit, {
		locale,
		...format
	});
}
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
export { createLocale, datetime, icuPound, intl, number, plural, relativetime };

//# sourceMappingURL=i18n.mjs.map