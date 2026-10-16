import { type Accessor, createSignal, flush } from "solid-js";
import {
  intl,
  type NumberFormatOptions,
  type PluralOptions,
  type RelativeTimeOptions,
  type RelativeTimeUnit,
} from "./glue/intl";

export {
  type CalendarDateFields,
  type DateFormatOptions,
  intl,
  type NumberFormatOptions,
  type PluralOptions,
  type RelativeTimeOptions,
  type RelativeTimeUnit,
} from "./glue/intl";

/** Paraglide registry adapter backed by Wabou's native formatters. */
export function plural(
  locale: string,
  input: unknown,
  options?: PluralOptions,
): string {
  return intl.pluralCategory(Number(input), { locale, ...options });
}

/** Paraglide registry adapter backed by Wabou's native formatters. */
export function number(
  locale: string,
  input: unknown,
  options?: NumberFormatOptions,
): string {
  return intl.formatNumber(Number(input), { locale, ...options });
}

export function icuPound(
  locale: string,
  input: unknown,
  options?: { offset?: unknown },
): string {
  const value = typeof input === "number" ? input : Number(input);
  if (!Number.isFinite(value)) return String(input);
  return number(locale, value - Number(options?.offset ?? 0));
}

/** Paraglide registry adapter backed by Wabou's native formatters. */
export function datetime(
  locale: string,
  input: unknown,
  options?: Parameters<typeof intl.formatDate>[1],
): string {
  return intl.formatDate(input as string | number | Date, {
    locale,
    ...options,
  });
}

/** Paraglide registry adapter backed by Wabou's native formatters. */
export function relativetime(
  locale: string,
  input: unknown,
  options: RelativeTimeOptions & { unit: RelativeTimeUnit },
): string {
  const { unit, ...format } = options;
  return intl.formatRelativeTime(Number(input), unit, { locale, ...format });
}

export interface CompiledMessageOptions<Locale extends string> {
  locale?: Locale;
}

/** A framework-independent message function emitted by a compiler. */
export type CompiledMessage<Inputs, Locale extends string, Output = string> = (
  inputs: Inputs,
  options?: CompiledMessageOptions<Locale>,
) => Output;

export interface LocaleController<Locale extends string> {
  locale: Accessor<Locale>;
  set(locale: Locale): void;
  message<Inputs, Output>(
    message: CompiledMessage<Inputs, Locale, Output>,
    inputs: Inputs,
  ): Output;
}

export interface LocaleOptions<Locale extends string> {
  onChange?: (locale: Locale) => void;
}

/**
 * Own native-app locale state while leaving message compilation to libraries
 * such as Paraglide. Locale selection is always explicit, so generated Web
 * URL, cookie, and local-storage strategies are unnecessary.
 */
export function createLocale<const Locale extends string>(
  initialLocale: Locale,
  options: LocaleOptions<Locale> = {},
): LocaleController<Locale> {
  let current = initialLocale;
  let flushScheduled = false;
  const [tracked, write] = createSignal<Locale>(() => current);
  const locale: Accessor<Locale> = () => {
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
        void Promise.resolve().then(() => {
          flushScheduled = false;
          flush();
        });
      }
      options.onChange?.(next);
    },
    message(message, inputs) {
      return message(inputs, { locale: locale() });
    },
  };
}
