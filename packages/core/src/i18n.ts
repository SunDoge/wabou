import { type Accessor, createSignal, flush } from "solid-js";

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
