import { n as defaultHost } from "./host-B5CKGzCs.mjs";
import "./renderer-Bu7emt2l.mjs";
//#region src/glue/intl.ts
function encode(options) {
	return JSON.stringify(options ?? {});
}
function epochMillis(input) {
	if (typeof input === "number") return input;
	if (input instanceof Date) return input.getTime();
	return new Date(input).getTime();
}
/** Small native localization surface. It deliberately does not emulate ECMA-402. */
const intl = /* @__PURE__ */ Object.freeze({
	locale() {
		return defaultHost.intl.locale();
	},
	timeZone() {
		return defaultHost.intl.timeZone();
	},
	today() {
		return defaultHost.intl.today();
	},
	firstWeekday(locale = intl.locale()) {
		return defaultHost.intl.firstWeekday(locale);
	},
	formatNumber(value, options) {
		const { locale = intl.locale(), ...format } = options ?? {};
		return defaultHost.intl.formatNumber(locale, value, encode(format));
	},
	parseNumber(input, options) {
		const { locale = intl.locale(), ...format } = options ?? {};
		return defaultHost.intl.parseNumber(locale, input, encode(format));
	},
	pluralCategory(value, options) {
		const { locale = intl.locale(), ...format } = options ?? {};
		return defaultHost.intl.pluralCategory(locale, value, encode(format));
	},
	formatDate(input, options) {
		const { locale = intl.locale(), ...format } = options ?? {};
		return defaultHost.intl.formatDate(locale, epochMillis(input), encode(format));
	},
	formatRelativeTime(value, unit, options) {
		const { locale = intl.locale(), ...format } = options ?? {};
		return defaultHost.intl.formatRelativeTime(locale, value, unit, encode(format));
	}
});
//#endregion
export { intl as t };

//# sourceMappingURL=intl-BsQ-mO8K.mjs.map