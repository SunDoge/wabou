import { Accessor } from "solid-js";
//#region src/i18n.d.ts
interface CompiledMessageOptions<Locale extends string> {
  locale?: Locale;
}
/** A framework-independent message function emitted by a compiler. */
type CompiledMessage<Inputs, Locale extends string, Output = string> = (inputs: Inputs, options?: CompiledMessageOptions<Locale>) => Output;
interface LocaleController<Locale extends string> {
  locale: Accessor<Locale>;
  set(locale: Locale): void;
  message<Inputs, Output>(message: CompiledMessage<Inputs, Locale, Output>, inputs: Inputs): Output;
}
interface LocaleOptions<Locale extends string> {
  onChange?: (locale: Locale) => void;
}
/**
 * Own native-app locale state while leaving message compilation to libraries
 * such as Paraglide. Locale selection is always explicit, so generated Web
 * URL, cookie, and local-storage strategies are unnecessary.
 */
declare function createLocale<const Locale extends string>(initialLocale: Locale, options?: LocaleOptions<Locale>): LocaleController<Locale>;
//#endregion
export { CompiledMessage, CompiledMessageOptions, LocaleController, LocaleOptions, createLocale };
//# sourceMappingURL=i18n.d.mts.map