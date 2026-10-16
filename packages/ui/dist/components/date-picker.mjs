import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { Popover } from "../primitives/popover.mjs";
import { componentsControlSize, componentsDisabledInteractiveClass, componentsElevation, componentsSurfaceClass, useComponentsTheme } from "./theme.mjs";
import { useHost } from "@wabou/core";
import { mergeClasses } from "@wabou/core/style";
import { For, createMemo, createSignal, untrack } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { CalendarDate, endOfMonth, isSameDay, startOfMonth } from "@internationalized/date";
import calendarIcon from "lucide-static/icons/calendar.svg?raw";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
//#region src/components/date-picker.tsx
function dayOfWeek(value) {
	return new Date(Date.UTC(value.year, value.month - 1, value.day)).getUTCDay();
}
const DEFAULT_LABELS = {
	previousMonth: "Previous month",
	nextMonth: "Next month",
	today: "Today",
	selectToday: "Select today"
};
/** A Wabou-native calendar using @internationalized/date for date arithmetic. */
function Calendar(props) {
	const host = useHost();
	const systemToday = () => {
		const value = host.intl.today();
		return new CalendarDate(value.year, value.month, value.day);
	};
	const locale = () => {
		return props.locale ?? host.intl.locale();
	};
	const labels = () => ({
		...DEFAULT_LABELS,
		...props.labels
	});
	const initial = untrack(() => props.value ?? props.defaultValue) ?? systemToday();
	const [localValue, setLocalValue] = createSignal(initial);
	const [visibleMonth, setVisibleMonth] = createSignal(startOfMonth(initial));
	const [focusedDate, setFocusedDate] = createSignal(initial);
	const value = () => props.value ?? localValue();
	const dayRefs = /* @__PURE__ */ new Map();
	const monthInfo = createMemo(() => {
		const currentLocale = locale();
		const firstWeekday = host.intl.firstWeekday(currentLocale);
		return {
			first_weekday: firstWeekday,
			month_label: host.intl.formatDate(currentLocale, Date.UTC(visibleMonth().year, visibleMonth().month - 1, 1), JSON.stringify({
				year: "numeric",
				month: "long",
				timeZone: "UTC"
			})),
			weekday_labels: Array.from({ length: 7 }, (_, offset) => {
				const index = (firstWeekday + offset) % 7;
				return host.intl.formatDate(currentLocale, Date.UTC(2024, 0, 7 + index), JSON.stringify({
					weekday: "short",
					timeZone: "UTC"
				}));
			})
		};
	});
	const formatDate = (date, style) => host.intl.formatDate(locale(), Date.UTC(date.year, date.month - 1, date.day), JSON.stringify({
		dateStyle: style,
		timeZone: "UTC"
	}));
	const days = () => {
		const first = startOfMonth(visibleMonth());
		const offset = (dayOfWeek(first) - monthInfo().first_weekday + 7) % 7;
		const gridStart = first.subtract({ days: offset });
		return Array.from({ length: 42 }, (_, index) => gridStart.add({ days: index }));
	};
	const unavailable = (date) => props.disabled || props.minValue !== void 0 && date.compare(props.minValue) < 0 || props.maxValue !== void 0 && date.compare(props.maxValue) > 0 || props.isDateUnavailable?.(date) === true;
	const canShowMonth = (month) => (props.minValue === void 0 || endOfMonth(month).compare(props.minValue) >= 0) && (props.maxValue === void 0 || month.compare(props.maxValue) <= 0);
	const select = (date) => {
		if (unavailable(date)) return;
		if (props.value === void 0) setLocalValue(date);
		setVisibleMonth(startOfMonth(date));
		setFocusedDate(date);
		props.onValueChange?.(date);
	};
	const focusDate = (date) => {
		setFocusedDate(date);
		if (date.month !== visibleMonth().month || date.year !== visibleMonth().year) setVisibleMonth(startOfMonth(date));
		requestAnimationFrame(() => dayRefs.get(date.toString())?.focus());
	};
	const tabStop = () => {
		const focused = focusedDate();
		if (focused.year === visibleMonth().year && focused.month === visibleMonth().month && !unavailable(focused)) return focused;
		let candidate = startOfMonth(visibleMonth());
		while (candidate.month === visibleMonth().month) {
			if (!unavailable(candidate)) return candidate;
			candidate = candidate.add({ days: 1 });
		}
		return startOfMonth(visibleMonth());
	};
	const focusAvailable = (date, step) => {
		let candidate = date;
		for (let attempts = 0; attempts < 366; attempts++) {
			if (!unavailable(candidate)) {
				focusDate(candidate);
				return;
			}
			candidate = candidate.add({ days: step });
		}
	};
	const handleKeyDown = (event, date) => {
		let next;
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
			next = date.subtract({ days: (dayOfWeek(date) - monthInfo().first_weekday + 7) % 7 });
			step = -1;
		} else if (event.key === "End") next = date.add({ days: 6 - (dayOfWeek(date) - monthInfo().first_weekday + 7) % 7 });
		else if (event.key === "PageUp") {
			next = date.subtract({ months: 1 });
			step = -1;
		} else if (event.key === "PageDown") next = date.add({ months: 1 });
		else if (event.key === "Enter" || event.key === " ") select(date);
		else return;
		event.preventDefault();
		if (next) focusAvailable(next, step);
	};
	return createComponent$1(View, {
		get "aria-label"() {
			return props["aria-label"] ?? "Calendar";
		},
		class: "w-72 p-3 flex flex-col gap-3",
		get children() {
			return [
				createComponent$1(View, {
					class: "h-8 flex items-center justify-between",
					get children() {
						return [
							createComponent$1(Button, {
								unstyled: true,
								get "aria-label"() {
									return labels().previousMonth;
								},
								get disabled() {
									return props.disabled || !canShowMonth(visibleMonth().subtract({ months: 1 }));
								},
								class: "w-8 h-8 rounded-md items-center justify-center",
								onClick: () => setVisibleMonth((month) => month.subtract({ months: 1 })),
								get children() {
									return createComponent$1(Icon, {
										source: chevronLeft,
										size: 16
									});
								}
							}),
							createComponent$1(Text, {
								class: "font-medium text-sm text-primary",
								get children() {
									return monthInfo().month_label;
								}
							}),
							createComponent$1(Button, {
								unstyled: true,
								get "aria-label"() {
									return labels().nextMonth;
								},
								get disabled() {
									return props.disabled || !canShowMonth(visibleMonth().add({ months: 1 }));
								},
								class: "w-8 h-8 rounded-md items-center justify-center",
								onClick: () => setVisibleMonth((month) => month.add({ months: 1 })),
								get children() {
									return createComponent$1(Icon, {
										source: chevronRight,
										size: 16
									});
								}
							})
						];
					}
				}),
				createComponent$1(View, {
					class: "w-64 flex flex-wrap gap-1",
					get children() {
						return [createComponent$1(For, {
							get each() {
								return monthInfo().weekday_labels;
							},
							children: (day) => createComponent$1(Text, {
								class: "w-8 h-7 flex items-center justify-center text-xs text-muted",
								children: day
							})
						}), createComponent$1(For, {
							get each() {
								return days();
							},
							keyed: false,
							children: (date) => {
								const selected = () => isSameDay(date(), value());
								const outside = () => date().month !== visibleMonth().month;
								const disabled = () => unavailable(date());
								return createComponent$1(Button, {
									ref: (node) => dayRefs.set(date().toString(), node),
									unstyled: true,
									get "aria-label"() {
										return formatDate(date(), "full");
									},
									get "aria-selected"() {
										return selected();
									},
									get "aria-current"() {
										return isSameDay(date(), systemToday()) ? "date" : void 0;
									},
									get focusOrder() {
										return isSameDay(date(), tabStop()) ? 0 : -1;
									},
									get disabled() {
										return disabled();
									},
									class: (state) => mergeClasses("w-8 h-8 rounded-md items-center justify-center text-sm", selected() ? "bg-accent text-on-accent" : !state.disabled && state.hovered ? "bg-control-hover text-primary" : "bg-transparent text-primary", outside() && "text-muted", componentsDisabledInteractiveClass(state.disabled)),
									onClick: () => select(date()),
									onKeyDown: (event) => handleKeyDown(event, date()),
									get children() {
										return createComponent$1(Text, { get children() {
											return date().day;
										} });
									}
								});
							}
						})];
					}
				}),
				createComponent$1(View, {
					class: "pt-2 flex items-center border-t border-subtle",
					get children() {
						return createComponent$1(Button, {
							unstyled: true,
							get "aria-label"() {
								return labels().selectToday;
							},
							class: "h-8 px-2 rounded-md text-sm text-accent",
							onClick: () => select(systemToday()),
							get children() {
								return labels().today;
							}
						});
					}
				})
			];
		}
	});
}
/** A shadcn-inspired date picker composed from Wabou Popover and Calendar. */
function DatePicker(props) {
	const host = useHost();
	const theme = useComponentsTheme();
	const [localValue, setLocalValue] = createSignal(props.defaultValue);
	const [localOpen, setLocalOpen] = createSignal(props.defaultOpen ?? false);
	const open = () => props.open ?? localOpen();
	const setOpen = (next) => {
		if (props.open === void 0) setLocalOpen(next);
		props.onOpenChange?.(next);
	};
	const value = () => props.value ?? localValue();
	const locale = () => {
		return props.locale ?? host.intl.locale();
	};
	const formatted = () => {
		const date = value();
		return date ? host.intl.formatDate(locale(), Date.UTC(date.year, date.month - 1, date.day), JSON.stringify({
			dateStyle: "medium",
			timeZone: "UTC"
		})) : props.placeholder ?? "Pick a date";
	};
	const select = (date) => {
		if (props.value === void 0) setLocalValue(date);
		props.onValueChange?.(date);
		setOpen(false);
	};
	return createComponent$1(Popover, {
		get "aria-label"() {
			return props["aria-label"];
		},
		get open() {
			return open();
		},
		onOpenChange: setOpen,
		placement: "bottom-start",
		get contentClass() {
			return componentsSurfaceClass("floating");
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		trigger: (trigger) => createComponent$1(Button, mergeProps({ unstyled: true }, trigger, {
			get "aria-label"() {
				return props["aria-label"];
			},
			get disabled() {
				return props.disabled;
			},
			get "class"() {
				return mergeClasses("w-72 justify-start border border-subtle bg-input shadow-xs", componentsControlSize("default"), props.class);
			},
			get children() {
				return [createComponent$1(Icon, {
					source: calendarIcon,
					class: "flex-none text-muted",
					size: 16
				}), createComponent$1(Text, {
					get "class"() {
						return value() ? "text-primary" : "text-muted";
					},
					get children() {
						return formatted();
					}
				})];
			}
		})),
		get children() {
			return createComponent$1(Calendar, mergeProps(props, {
				get value() {
					return value();
				},
				get "aria-label"() {
					return props["aria-label"];
				},
				onValueChange: select
			}));
		}
	});
}
//#endregion
export { Calendar, CalendarDate, DatePicker };

//# sourceMappingURL=date-picker.mjs.map