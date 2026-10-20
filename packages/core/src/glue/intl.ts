import { defaultHost } from "../renderer";

export interface CalendarDateFields {
  year: number;
  month: number;
  day: number;
}

export interface NumberFormatOptions {
  style?: "decimal" | "percent" | "currency";
  currency?: "USD" | "EUR" | "GBP" | "JPY" | "CNY";
  minimumFractionDigits?: number;
  maximumFractionDigits?: number;
  useGrouping?: boolean;
}

export interface DateFormatOptions {
  dateStyle?: "short" | "medium" | "long" | "full";
  year?: "numeric";
  month?: "long";
  weekday?: "short";
  timeZone?: string;
}

export interface PluralOptions {
  type?: "cardinal" | "ordinal";
  offset?: number;
}

export type RelativeTimeUnit =
  | "year"
  | "quarter"
  | "month"
  | "week"
  | "day"
  | "hour"
  | "minute"
  | "second";

export interface RelativeTimeOptions {
  numeric?: "always" | "auto";
}

function encode(options: object | undefined): string {
  return JSON.stringify(options ?? {});
}

function epochMillis(input: string | number | Date): number {
  if (typeof input === "number") return input;
  if (input instanceof Date) return input.getTime();
  return new Date(input).getTime();
}

/** Small native localization surface. It deliberately does not emulate ECMA-402. */
export const intl = /* @__PURE__ */ Object.freeze({
  locale(): string {
    return defaultHost.intl.locale();
  },
  timeZone(): string {
    return defaultHost.intl.timeZone();
  },
  today(): CalendarDateFields {
    return defaultHost.intl.today();
  },
  firstWeekday(locale = intl.locale()): number {
    return defaultHost.intl.firstWeekday(locale);
  },
  formatNumber(
    value: number,
    options?: NumberFormatOptions & { locale?: string },
  ): string {
    const { locale = intl.locale(), ...format } = options ?? {};
    return defaultHost.intl.formatNumber(locale, value, encode(format));
  },
  parseNumber(
    input: string,
    options?: NumberFormatOptions & { locale?: string },
  ): number {
    const { locale = intl.locale(), ...format } = options ?? {};
    return defaultHost.intl.parseNumber(locale, input, encode(format));
  },
  pluralCategory(
    value: number,
    options?: PluralOptions & { locale?: string },
  ): string {
    const { locale = intl.locale(), ...format } = options ?? {};
    return defaultHost.intl.pluralCategory(locale, value, encode(format));
  },
  formatDate(
    input: string | number | Date,
    options?: DateFormatOptions & { locale?: string },
  ): string {
    const { locale = intl.locale(), ...format } = options ?? {};
    return defaultHost.intl.formatDate(
      locale,
      epochMillis(input),
      encode(format),
    );
  },
  formatRelativeTime(
    value: number,
    unit: RelativeTimeUnit,
    options?: RelativeTimeOptions & { locale?: string },
  ): string {
    const { locale = intl.locale(), ...format } = options ?? {};
    return defaultHost.intl.formatRelativeTime(
      locale,
      value,
      unit,
      encode(format),
    );
  },
});
