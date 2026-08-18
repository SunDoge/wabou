import { defaultHost } from "../renderer";

export interface CalendarDateFields {
  year: number;
  month: number;
  day: number;
}

/**
 * Operating-system locale facts. Standards-compatible formatting is installed
 * separately by the FormatJS-backed Intl polyfill.
 */
export const intl = Object.freeze({
  locale(): string {
    return defaultHost.intl.locale();
  },
  timeZone(): string {
    return defaultHost.intl.timeZone();
  },
  today(): CalendarDateFields {
    return defaultHost.intl.today();
  },
});
