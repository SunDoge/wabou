import { a as RelativeTimeOptions, i as PluralOptions, n as DateFormatOptions, o as RelativeTimeUnit, r as NumberFormatOptions, s as intl, t as CalendarDateFields } from "./intl-C6JICTBS.mjs";
import { Accessor } from "solid-js";
//#region src/i18n.d.ts
/** Paraglide registry adapter backed by Wabou's native formatters. */
export declare function plural(locale: string, input: unknown, options?: PluralOptions): string;
/** Paraglide registry adapter backed by Wabou's native formatters. */
export declare function number(locale: string, input: unknown, options?: NumberFormatOptions): string;
export declare function icuPound(locale: string, input: unknown, options?: {
  offset?: unknown;
}): string;
/** Paraglide registry adapter backed by Wabou's native formatters. */
export declare function datetime(locale: string, input: unknown, options?: Parameters<typeof intl.formatDate>[1]): string;
/** Paraglide registry adapter backed by Wabou's native formatters. */
export declare function relativetime(locale: string, input: unknown, options: RelativeTimeOptions & {
  unit: RelativeTimeUnit;
}): string;
export interface CompiledMessageOptions<Locale extends string> {
  locale?: Locale;
}
/** A framework-independent message function emitted by a compiler. */
export type CompiledMessage<Inputs, Locale extends string, Output = string> = (inputs: Inputs, options?: CompiledMessageOptions<Locale>) => Output;
export interface LocaleController<Locale extends string> {
  locale: Accessor<Locale>;
  set(locale: Locale): void;
  message<Inputs, Output>(message: CompiledMessage<Inputs, Locale, Output>, inputs: Inputs): Output;
}
export interface LocaleOptions<Locale extends string> {
  onChange?: (locale: Locale) => void;
}
/**
 * Own native-app locale state while leaving message compilation to libraries
 * such as Paraglide. Locale selection is always explicit, so generated Web
 * URL, cookie, and local-storage strategies are unnecessary.
 */
export declare function createLocale<const Locale extends string>(initialLocale: Locale, options?: LocaleOptions<Locale>): LocaleController<Locale>;
//#endregion
export { type CalendarDateFields, type DateFormatOptions, type NumberFormatOptions, type PluralOptions, type RelativeTimeOptions, type RelativeTimeUnit, intl };
//# sourceMappingURL=i18n.d.mts.map