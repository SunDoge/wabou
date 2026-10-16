//#region src/glue/intl.d.ts
interface CalendarDateFields {
  year: number;
  month: number;
  day: number;
}
interface NumberFormatOptions {
  style?: "decimal" | "percent" | "currency";
  currency?: "USD" | "EUR" | "GBP" | "JPY" | "CNY";
  minimumFractionDigits?: number;
  maximumFractionDigits?: number;
  useGrouping?: boolean;
}
interface DateFormatOptions {
  dateStyle?: "short" | "medium" | "long" | "full";
  year?: "numeric";
  month?: "long";
  weekday?: "short";
  timeZone?: string;
}
interface PluralOptions {
  type?: "cardinal" | "ordinal";
  offset?: number;
}
type RelativeTimeUnit = "year" | "quarter" | "month" | "week" | "day" | "hour" | "minute" | "second";
interface RelativeTimeOptions {
  numeric?: "always" | "auto";
}
/** Small native localization surface. It deliberately does not emulate ECMA-402. */
declare const intl: Readonly<{
  locale(): string;
  timeZone(): string;
  today(): CalendarDateFields;
  firstWeekday(locale?: string): number;
  formatNumber(value: number, options?: NumberFormatOptions & {
    locale?: string;
  }): string;
  parseNumber(input: string, options?: NumberFormatOptions & {
    locale?: string;
  }): number;
  pluralCategory(value: number, options?: PluralOptions & {
    locale?: string;
  }): string;
  formatDate(input: string | number | Date, options?: DateFormatOptions & {
    locale?: string;
  }): string;
  formatRelativeTime(value: number, unit: RelativeTimeUnit, options?: RelativeTimeOptions & {
    locale?: string;
  }): string;
}>;
//#endregion
export { RelativeTimeOptions as a, PluralOptions as i, DateFormatOptions as n, RelativeTimeUnit as o, NumberFormatOptions as r, intl as s, CalendarDateFields as t };
//# sourceMappingURL=intl-C6JICTBS.d.mts.map