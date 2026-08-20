import {
  CalendarDate,
  endOfMonth,
  isSameDay,
  startOfMonth,
} from "@internationalized/date";
import { useHost } from "@wabou/core";
import type { Handle } from "@wabou/core/renderer";
import type { Shadow } from "@wabou/core/style";
import calendarIcon from "lucide-static/icons/calendar.svg?raw";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import { createMemo, createSignal, For, type JSX, untrack } from "solid-js";
import {
  Button as HeadlessButton,
  Icon,
  Popover,
  Text,
  View,
} from "../primitives";
import { join } from "./class-names";
import { componentsElevation, useComponentsTheme } from "./theme";

function dayOfWeek(value: CalendarDate): number {
  return new Date(Date.UTC(value.year, value.month - 1, value.day)).getUTCDay();
}

export interface CalendarProps {
  value?: CalendarDate;
  defaultValue?: CalendarDate;
  minValue?: CalendarDate;
  maxValue?: CalendarDate;
  disabled?: boolean;
  isDateUnavailable?: (date: CalendarDate) => boolean;
  locale?: string;
  labels?: Partial<CalendarLabels>;
  "aria-label"?: string;
  onValueChange?: (value: CalendarDate) => void;
}

export interface CalendarLabels {
  previousMonth: string;
  nextMonth: string;
  today: string;
  selectToday: string;
}

const DEFAULT_LABELS: CalendarLabels = {
  previousMonth: "Previous month",
  nextMonth: "Next month",
  today: "Today",
  selectToday: "Select today",
};

/** A Wabou-native calendar using @internationalized/date for date arithmetic. */
export function Calendar(props: CalendarProps): JSX.Element {
  const host = useHost();
  const systemToday = () => {
    const value = host.intl.today();
    return new CalendarDate(value.year, value.month, value.day);
  };
  const locale = () => {
    const requested = props.locale ?? host.intl.locale();
    return Intl.DateTimeFormat.supportedLocalesOf([requested])[0] ?? "en";
  };
  const labels = (): CalendarLabels => ({
    ...DEFAULT_LABELS,
    ...props.labels,
  });
  const initial =
    untrack(() => props.value ?? props.defaultValue) ?? systemToday();
  const [localValue, setLocalValue] = createSignal(initial);
  const [visibleMonth, setVisibleMonth] = createSignal(startOfMonth(initial));
  const [focusedDate, setFocusedDate] = createSignal(initial);
  const value = () => props.value ?? localValue();
  const dayRefs = new Map<string, Handle>();

  const monthInfo = createMemo(() => {
    const currentLocale = locale();
    const week = (
      new Intl.Locale(currentLocale) as Intl.Locale & {
        getWeekInfo(): {
          firstDay: number;
          weekend: number[];
          minimalDays: number;
        };
      }
    ).getWeekInfo();
    const firstWeekday = week.firstDay % 7;
    const weekday = new Intl.DateTimeFormat(currentLocale, {
      weekday: "short",
      timeZone: "UTC",
    });
    return {
      first_weekday: firstWeekday,
      month_label: new Intl.DateTimeFormat(currentLocale, {
        year: "numeric",
        month: "long",
        timeZone: "UTC",
      }).format(Date.UTC(visibleMonth().year, visibleMonth().month - 1, 1)),
      weekday_labels: Array.from({ length: 7 }, (_, offset) => {
        const index = (firstWeekday + offset) % 7;
        return weekday.format(Date.UTC(2024, 0, 7 + index));
      }),
    };
  });
  const dateFormatters = createMemo(() => ({
    medium: new Intl.DateTimeFormat(locale(), {
      dateStyle: "medium",
      timeZone: "UTC",
    }),
    full: new Intl.DateTimeFormat(locale(), {
      dateStyle: "full",
      timeZone: "UTC",
    }),
  }));
  const formatDate = (date: CalendarDate, style: "medium" | "full") =>
    dateFormatters()[style].format(
      Date.UTC(date.year, date.month - 1, date.day),
    );
  const days = () => {
    const first = startOfMonth(visibleMonth());
    const offset = (dayOfWeek(first) - monthInfo().first_weekday + 7) % 7;
    const gridStart = first.subtract({ days: offset });
    return Array.from({ length: 42 }, (_, index) =>
      gridStart.add({ days: index }),
    );
  };
  const unavailable = (date: CalendarDate) =>
    props.disabled ||
    (props.minValue !== undefined && date.compare(props.minValue) < 0) ||
    (props.maxValue !== undefined && date.compare(props.maxValue) > 0) ||
    props.isDateUnavailable?.(date) === true;
  const canShowMonth = (month: CalendarDate) =>
    (props.minValue === undefined ||
      endOfMonth(month).compare(props.minValue) >= 0) &&
    (props.maxValue === undefined || month.compare(props.maxValue) <= 0);
  const select = (date: CalendarDate) => {
    if (unavailable(date)) return;
    if (props.value === undefined) setLocalValue(date);
    setVisibleMonth(startOfMonth(date));
    setFocusedDate(date);
    props.onValueChange?.(date);
  };
  const focusDate = (date: CalendarDate) => {
    setFocusedDate(date);
    if (
      date.month !== visibleMonth().month ||
      date.year !== visibleMonth().year
    ) {
      setVisibleMonth(startOfMonth(date));
    }
    requestAnimationFrame(() => dayRefs.get(date.toString())?.focus());
  };
  const tabStop = () => {
    const focused = focusedDate();
    if (
      focused.year === visibleMonth().year &&
      focused.month === visibleMonth().month &&
      !unavailable(focused)
    ) {
      return focused;
    }
    let candidate = startOfMonth(visibleMonth());
    while (candidate.month === visibleMonth().month) {
      if (!unavailable(candidate)) return candidate;
      candidate = candidate.add({ days: 1 });
    }
    return startOfMonth(visibleMonth());
  };
  const focusAvailable = (date: CalendarDate, step: number) => {
    let candidate = date;
    for (let attempts = 0; attempts < 366; attempts++) {
      if (!unavailable(candidate)) {
        focusDate(candidate);
        return;
      }
      candidate = candidate.add({ days: step });
    }
  };
  const handleKeyDown = (
    event: { key: string; preventDefault(): void },
    date: CalendarDate,
  ) => {
    let next: CalendarDate | undefined;
    let step = 1;
    if (event.key === "ArrowLeft") {
      next = date.subtract({ days: 1 });
      step = -1;
    } else if (event.key === "ArrowRight") next = date.add({ days: 1 });
    else if (event.key === "ArrowUp") {
      next = date.subtract({ days: 7 });
      step = -1;
    } else if (event.key === "ArrowDown") next = date.add({ days: 7 });
    else if (event.key === "Home") {
      next = date.subtract({
        days: (dayOfWeek(date) - monthInfo().first_weekday + 7) % 7,
      });
      step = -1;
    } else if (event.key === "End")
      next = date.add({
        days: 6 - ((dayOfWeek(date) - monthInfo().first_weekday + 7) % 7),
      });
    else if (event.key === "PageUp") {
      next = date.subtract({ months: 1 });
      step = -1;
    } else if (event.key === "PageDown") next = date.add({ months: 1 });
    else if (event.key === "Enter" || event.key === " ") select(date);
    else return;
    event.preventDefault();
    if (next) focusAvailable(next, step);
  };

  return (
    <View
      aria-label={props["aria-label"] ?? "Calendar"}
      class="w-72 p-3 flex flex-col gap-3"
    >
      <View class="h-8 flex items-center justify-between">
        <HeadlessButton
          unstyled
          aria-label={labels().previousMonth}
          disabled={
            props.disabled ||
            !canShowMonth(visibleMonth().subtract({ months: 1 }))
          }
          class="w-8 h-8 rounded-md items-center justify-center"
          onClick={() =>
            setVisibleMonth((month) => month.subtract({ months: 1 }))
          }
        >
          <Icon source={chevronLeft} size={16} />
        </HeadlessButton>
        <Text class="font-medium text-sm text-primary">
          {monthInfo().month_label}
        </Text>
        <HeadlessButton
          unstyled
          aria-label={labels().nextMonth}
          disabled={
            props.disabled || !canShowMonth(visibleMonth().add({ months: 1 }))
          }
          class="w-8 h-8 rounded-md items-center justify-center"
          onClick={() => setVisibleMonth((month) => month.add({ months: 1 }))}
        >
          <Icon source={chevronRight} size={16} />
        </HeadlessButton>
      </View>
      <View class="w-64 flex flex-wrap gap-1">
        <For each={monthInfo().weekday_labels}>
          {(day) => (
            <Text class="w-8 h-7 flex items-center justify-center text-xs text-muted">
              {day}
            </Text>
          )}
        </For>
        <For each={days()} keyed={false}>
          {(date) => {
            const selected = () => isSameDay(date(), value());
            const outside = () => date().month !== visibleMonth().month;
            const disabled = () => unavailable(date());
            return (
              <HeadlessButton
                ref={(node) => dayRefs.set(date().toString(), node)}
                unstyled
                aria-label={formatDate(date(), "full")}
                aria-selected={selected()}
                aria-current={
                  isSameDay(date(), systemToday()) ? "date" : undefined
                }
                focusOrder={isSameDay(date(), tabStop()) ? 0 : -1}
                disabled={disabled()}
                class={(state) =>
                  join(
                    "w-8 h-8 rounded-md items-center justify-center text-sm",
                    selected()
                      ? "bg-accent text-on-accent"
                      : state.hovered
                        ? "bg-control-hover text-primary"
                        : "bg-transparent text-primary",
                    outside() && "text-muted",
                  )
                }
                style={{ opacity: disabled() ? 0.35 : 1 }}
                onClick={() => select(date())}
                onKeyDown={(event) => handleKeyDown(event, date())}
              >
                <Text>{date().day}</Text>
              </HeadlessButton>
            );
          }}
        </For>
      </View>
      <View class="pt-2 flex items-center border-t border-subtle">
        <HeadlessButton
          unstyled
          aria-label={labels().selectToday}
          class="h-8 px-2 rounded-md text-sm text-accent"
          onClick={() => select(systemToday())}
        >
          {labels().today}
        </HeadlessButton>
      </View>
    </View>
  );
}

export interface DatePickerProps extends Omit<CalendarProps, "aria-label"> {
  "aria-label": string;
  placeholder?: string;
  class?: string;
  contentShadows?: readonly Shadow[] | null;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  onValueChange?: (value: CalendarDate) => void;
}

/** A shadcn-inspired date picker composed from Wabou Popover and Calendar. */
export function DatePicker(props: DatePickerProps): JSX.Element {
  const host = useHost();
  const theme = useComponentsTheme();
  const [localValue, setLocalValue] = createSignal(props.defaultValue);
  const [localOpen, setLocalOpen] = createSignal(props.defaultOpen ?? false);
  const open = () => props.open ?? localOpen();
  const setOpen = (next: boolean) => {
    if (props.open === undefined) setLocalOpen(next);
    props.onOpenChange?.(next);
  };
  const value = () => props.value ?? localValue();
  const locale = () => {
    const requested = props.locale ?? host.intl.locale();
    return Intl.DateTimeFormat.supportedLocalesOf([requested])[0] ?? "en";
  };
  const formatted = () => {
    const date = value();
    return date
      ? new Intl.DateTimeFormat(locale(), {
          dateStyle: "medium",
          timeZone: "UTC",
        }).format(Date.UTC(date.year, date.month - 1, date.day))
      : (props.placeholder ?? "Pick a date");
  };
  const select = (date: CalendarDate) => {
    if (props.value === undefined) setLocalValue(date);
    props.onValueChange?.(date);
    setOpen(false);
  };

  return (
    <Popover
      aria-label={props["aria-label"]}
      open={open()}
      onOpenChange={setOpen}
      placement="bottom-start"
      contentClass="rounded-lg border border-subtle bg-surface"
      contentShadows={
        props.contentShadows === undefined
          ? componentsElevation(theme(), "floating")
          : props.contentShadows
      }
      trigger={(trigger) => (
        <HeadlessButton
          unstyled
          {...trigger}
          aria-label={props["aria-label"]}
          disabled={props.disabled}
          class={join(
            "w-72 h-8 px-3 justify-start gap-2 rounded-md border border-subtle bg-input text-sm shadow-xs",
            props.class,
          )}
        >
          <Icon source={calendarIcon} class="flex-none text-muted" size={16} />
          <Text class={value() ? "text-primary" : "text-muted"}>
            {formatted()}
          </Text>
        </HeadlessButton>
      )}
    >
      <Calendar
        {...props}
        value={value()}
        aria-label={props["aria-label"]}
        onValueChange={select}
      />
    </Popover>
  );
}

export { CalendarDate };
