import { $ as Svg, A as isSelected, B as createContainerMatch, C as OverlayPlaneProvider, Ct as useMotionConfig, D as Column, E as Center, F as createNetworkImageResource, G as CodeEditor, H as Button$1, I as createOwnedImageResource, J as PasswordInput$1, K as Icon, L as releaseImageResource, M as FORM_ERROR, N as createFormDraft, O as Row, P as createFileImageResource, Q as RichTextSpan, R as CollapsiblePresence, S as createTransitionPresence, St as MotionConfigProvider, U as Link, V as createMeasuredSize, W as createButton, X as PathBuilder, Y as Path, Z as RichText, _ as createRetainedItems, _t as createPulse, a as ScrollArea, at as translate2d$1, b as Spin, bt as createTransition, ct as createHover, dt as createAnimationFrame, et as Text, ft as animate, g as createNotifications, gt as createLoop, h as NotificationRegion, ht as createKeyframeAnimation, i as createScrollReset, it as rotate2d$1, j as toggleSelection, k as createKeyedSelection, lt as createFocus, mt as createInterpolation, n as createTabs, nt as TextInput, o as Popover$1, ot as createActive, pt as animateKeyframes, q as Image, r as createShortcuts, rt as View, st as createPress, t as primitives_exports, tt as TextArea, ut as createFocusWithin, v as Pulse, vt as createRotation, w as createOverlayLayer, wt as useReducedMotion, x as Modal, xt as normalizeSweepGeometry, y as Ripple, yt as createSweep, z as createPresence } from "./primitives-Bpim1abf.mjs";
import { rgba, useClipboard, useDialog, useFileDrop, useHost, useWindow } from "@wabou/core";
import { mergeClasses, rgba as rgba$1, scale2d, shadow } from "@wabou/core/style";
import { For, Match, Show, Switch as Switch$1, createComponent, createContext, createEffect, createMemo, createSignal, createUniqueId, flush, getOwner, omit, onCleanup, untrack, useContext } from "solid-js";
import { VirtualList, applyRef, createComponent as createComponent$1, createFps, memo, mergeProps, useHost as useHost$1 } from "@wabou/core/renderer";
import { P, match } from "ts-pattern";
import arrowDown from "lucide-static/icons/arrow-down.svg?raw";
import arrowLeft from "lucide-static/icons/arrow-left.svg?raw";
import arrowRight from "lucide-static/icons/arrow-right.svg?raw";
import arrowUp from "lucide-static/icons/arrow-up.svg?raw";
import check from "lucide-static/icons/check.svg?raw";
import copy from "lucide-static/icons/copy.svg?raw";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import { CalendarDate, endOfMonth, isSameDay, startOfMonth, startOfWeek } from "@internationalized/date";
import calendarIcon from "lucide-static/icons/calendar.svg?raw";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import fileUp from "lucide-static/icons/file-up.svg?raw";
import pencil from "lucide-static/icons/pencil.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
import { lexer } from "marked";
import remend from "remend";
import ellipsis from "lucide-static/icons/ellipsis.svg?raw";
import { NumberFormatter, NumberParser } from "@internationalized/number";
import plus from "lucide-static/icons/plus.svg?raw";
import { encode } from "uqr";
import star from "lucide-static/icons/star.svg?raw";
import sparkles from "lucide-static/icons/sparkles.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import checkCircle from "lucide-static/icons/circle-check.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import triangleAlert from "lucide-static/icons/triangle-alert.svg?raw";
import ellipsisCircle from "lucide-static/icons/circle-ellipsis.svg?raw";
import xCircle from "lucide-static/icons/circle-x.svg?raw";
import clock from "lucide-static/icons/clock-3.svg?raw";
import wrench from "lucide-static/icons/wrench.svg?raw";
import { createTable, functionalUpdate, getCoreRowModel, getFilteredRowModel, getSortedRowModel } from "@tanstack/table-core";
import { createMemoryHistory, createMemoryHistory as createMemoryHistory$1 } from "@tanstack/history";
import { BaseRootRoute, BaseRoute, RouterCore, notFound, redirect } from "@tanstack/router-core";
export * from "@wabou/core";
export * from "@wabou/core/i18n";
//#region src/components/badge.tsx
function badgeClass(variant = "default", weight = "medium", className) {
	const colors = match(variant).with("default", () => "bg-accent border-accent text-on-accent").with("secondary", () => "bg-control border-subtle text-primary").with("outline", () => "bg-transparent border-strong text-secondary").with("ghost", () => "bg-transparent border-transparent text-secondary").with("link", () => "bg-transparent border-transparent text-accent").with("success", () => "bg-success-surface border-success-primary text-success-primary").with("destructive", () => "bg-danger-surface border-danger text-danger-primary").exhaustive();
	return mergeClasses("flex-none overflow-hidden whitespace-nowrap rounded-full border px-2 py-0.5 text-xs", weight === "normal" ? "font-normal" : "font-medium", colors, className);
}
/** Compact status text with shadcn-compatible visual variants. */
function Badge(props) {
	const rest = omit(props, "variant", "weight", "class", "children");
	return createComponent$1(Text, mergeProps(rest, {
		get ["class"]() {
			return badgeClass(props.variant, props.weight, props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
//#region src/components/theme.ts
/**
* Geometry contract for Wabou's default desktop theme.
*
* Components consume these recipes instead of independently choosing height,
* padding, radius, and icon rhythm. The values intentionally favor desktop
* density over touch-first sizing.
*/
const componentsThemeContract = Object.freeze({
	controlHeight: Object.freeze({
		sm: 28,
		default: 32,
		lg: 40,
		icon: 32
	}),
	iconSize: Object.freeze({
		sm: 14,
		default: 16,
		lg: 18
	}),
	controlRadius: 8,
	containerRadius: 12,
	containerPadding: 20,
	sectionGap: 16
});
function componentsControlContentSize(size) {
	switch (size) {
		case "sm": return "h-7 px-2 gap-1.5 text-xs";
		case "lg": return "h-10 px-4 gap-2.5 text-base";
		case "icon": return "w-8 h-8 p-0 gap-0 text-sm";
		default: return "h-8 px-3 gap-2 text-sm";
	}
}
function componentsControlSize(size) {
	return `${componentsControlContentSize(size)} rounded-lg`;
}
/**
* Native elevation recipes adapted from gpui-component. Wabou and GPUI both
* pass standard deviation directly to their renderer, so these values should
* not use CSS's doubled blur radius. Floating surfaces also carry a subtle
* foreground-colored ring: black in light mode, white in dark mode.
*/
function componentsElevation(theme, elevation) {
	if (elevation === "raised") return [
		shadow({
			spread: 1,
			stdDev: 0,
			color: 13
		}),
		shadow({
			offsetY: 1,
			stdDev: 2,
			color: 31
		}),
		shadow({
			offsetY: 4,
			stdDev: 7,
			spread: -4,
			color: 20
		})
	];
	if (elevation === "floating") return [
		shadow({
			spread: 1,
			stdDev: 0,
			color: theme === "dark" ? 4294967071 : 20
		}),
		shadow({
			offsetY: 8,
			stdDev: 12,
			spread: -5,
			color: 36
		}),
		shadow({
			offsetY: 2,
			stdDev: 3,
			spread: -2,
			color: 31
		})
	];
	return [shadow({
		offsetY: 20,
		stdDev: 25,
		spread: -5,
		color: 26
	}), shadow({
		offsetY: 8,
		stdDev: 10,
		spread: -6,
		color: 26
	})];
}
const defaultTheme = { theme: () => "light" };
const ThemeContext = createContext(defaultTheme);
function ComponentsProvider(props) {
	return createComponent(ThemeContext, {
		value: { theme: () => props.theme ?? "light" },
		get children() {
			return props.children;
		}
	});
}
function useComponentsTheme() {
	return (getOwner() ? useContext(ThemeContext) : defaultTheme).theme;
}
//#endregion
//#region src/components/alert.tsx
const AlertContext = createContext({ variant: () => "default" });
function alertColors(variant) {
	return match(variant).with("default", () => ({
		container: "border-subtle bg-surface",
		title: "text-primary",
		description: "text-secondary"
	})).with("destructive", () => ({
		container: "border-danger bg-danger-surface",
		title: "text-danger-primary",
		description: "text-danger-primary"
	})).exhaustive();
}
/** A native status callout with shadcn-compatible compound composition. */
function Alert(props) {
	const variant = () => props.variant ?? "default";
	const colors = () => alertColors(variant());
	const forwarded = omit(props, "variant", "icon", "title", "class", "children");
	const content = () => props.title === void 0 ? props.children : [createComponent$1(AlertTitle, { get children() {
		return props.title;
	} }), memo(() => {
		return memo(() => {
			return props.children === void 0;
		})() ? null : createComponent$1(AlertDescription, { get children() {
			return props.children;
		} });
	})];
	return createComponent(AlertContext, {
		value: { variant },
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				role: "alert",
				get ["aria-label"]() {
					return props["aria-label"] ?? props.title;
				},
				get ["class"]() {
					return mergeClasses("w-full min-w-0 flex flex-row items-start gap-3 rounded-xl border p-4 shadow-xs", colors().container, props.class);
				},
				get children() {
					return [memo(() => {
						return memo(() => {
							return props.icon === void 0;
						})() ? null : createComponent$1(View, {
							class: "flex-none pt-0.5",
							get children() {
								return props.icon;
							}
						});
					}), createComponent$1(View, {
						class: "min-w-0 flex-1 flex flex-col gap-1",
						get children() {
							return content();
						}
					})];
				}
			}));
		}
	});
}
function AlertTitle(props) {
	const context = useContext(AlertContext);
	return createComponent$1(Text, mergeProps(props, { get ["class"]() {
		return mergeClasses("w-full min-w-0 text-sm font-semibold", alertColors(context.variant()).title, props.class);
	} }));
}
function AlertDescription(props) {
	const context = useContext(AlertContext);
	return createComponent$1(Text, mergeProps(props, { get ["class"]() {
		return mergeClasses("w-full min-w-0 whitespace-normal text-sm", alertColors(context.variant()).description, props.class);
	} }));
}
//#endregion
//#region src/components/button-group-context.ts
const ButtonGroupContext = createContext(null);
function useButtonGroupOrientation() {
	return useContext(ButtonGroupContext) ?? void 0;
}
//#endregion
//#region src/components/button.tsx
function buttonColors(variant, state) {
	const focus = state.focusVisible ? "border-focus" : "";
	const passiveBorder = (value) => match(value).with("outline", () => "border-strong").with(P.union("default", "secondary", "ghost", "destructive"), () => "border-transparent").exhaustive();
	return match({
		variant,
		pressed: state.pressed,
		hovered: state.hovered
	}).with({
		variant: "default",
		pressed: true
	}, () => mergeClasses("bg-accent-pressed border-transparent text-on-accent", focus)).with({
		variant: "default",
		hovered: true
	}, () => mergeClasses("bg-accent-hover border-transparent text-on-accent", focus)).with({ variant: "default" }, () => mergeClasses("bg-accent border-transparent text-on-accent", focus)).with({
		variant: "destructive",
		pressed: true
	}, () => mergeClasses("bg-danger-pressed border-transparent text-on-accent", focus)).with({
		variant: "destructive",
		hovered: true
	}, () => mergeClasses("bg-danger-hover border-transparent text-on-accent", focus)).with({ variant: "destructive" }, () => mergeClasses("bg-danger border-transparent text-on-accent", focus)).with({
		variant: "secondary",
		pressed: true
	}, () => mergeClasses("bg-control-pressed border-transparent text-primary", focus)).with({
		variant: "secondary",
		hovered: true
	}, () => mergeClasses("bg-control-hover border-transparent text-primary", focus)).with({ variant: "secondary" }, () => mergeClasses("bg-control border-transparent text-primary", focus)).with({ pressed: true }, ({ variant: value }) => mergeClasses("bg-control-pressed text-secondary", passiveBorder(value), focus)).with({ hovered: true }, ({ variant: value }) => mergeClasses("bg-control-hover text-secondary", passiveBorder(value), focus)).with({ variant: P.union("outline", "ghost") }, ({ variant: value }) => mergeClasses("bg-transparent text-secondary", passiveBorder(value), focus)).exhaustive();
}
function buttonSize(size) {
	return componentsControlSize(size);
}
function Button(props) {
	const local = props;
	const forwarded = omit(props, "variant", "size", "class", "style");
	const variant = () => local.variant ?? "default";
	const size = () => local.size ?? "default";
	const groupOrientation = useButtonGroupOrientation();
	return createComponent$1(Button$1, mergeProps(forwarded, {
		unstyled: true,
		class: (state) => mergeClasses("inline-flex flex-none whitespace-nowrap items-center justify-center border font-medium", buttonColors(variant(), state), buttonSize(size()), groupOrientation && "rounded-none border-transparent", local.class),
		style: (state) => ({
			"border-width": 1,
			opacity: state.disabled ? .45 : 1,
			...typeof local.style === "function" ? local.style(state) : local.style
		})
	}));
}
//#endregion
//#region src/components/dialog.tsx
function Dialog(props) {
	const theme = useComponentsTheme();
	return createComponent$1(Modal, mergeProps(props, {
		get motion() {
			return memo(() => {
				return props.motion === void 0;
			})() ? { fromScale: .98 } : props.motion;
		},
		get backdropClass() {
			return props.backdropClass;
		},
		get backdropStyle() {
			return {
				"background-color": rgba(51),
				...props.backdropStyle
			};
		},
		get contentClass() {
			return mergeClasses("w-[480px] max-w-full min-w-0 flex flex-col gap-4 rounded-xl border border-subtle bg-surface p-5", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "modal") : props.contentShadows;
		}
	}));
}
function DialogHeader(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogFooter(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("flex items-center justify-end gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogScrollBody(props) {
	return createComponent$1(ScrollArea, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-h-0 flex-1", props.class);
		},
		get contentClass() {
			return props.contentClass;
		},
		get children() {
			return props.children;
		}
	}));
}
function DialogTitle(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return mergeClasses("text-lg font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogDescription(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/components/alert-dialog.tsx
const AlertDialogContext = createContext();
function useAlertDialog() {
	const context = useContext(AlertDialogContext);
	if (!context) throw new Error("AlertDialog actions must be used inside AlertDialog");
	return context;
}
/**
* A blocking confirmation dialog. Backdrop dismissal is disabled by default so
* every close is an intentional cancel, confirmation, or Escape action.
*/
function AlertDialog(props) {
	return createComponent$1(Dialog, mergeProps(props, {
		contentRole: "alertdialog",
		get closeOnBackdrop() {
			return props.closeOnBackdrop ?? false;
		},
		children: (controls) => createComponent(AlertDialogContext, {
			value: controls,
			get children() {
				return typeof props.children === "function" ? props.children(controls) : props.children;
			}
		})
	}));
}
function closingHandler(close, handler) {
	return (event) => {
		handler?.(event);
		if (!event.defaultPrevented) close();
	};
}
function AlertDialogAction(props) {
	const dialog = useAlertDialog();
	return createComponent$1(Button, mergeProps(props, { get onClick() {
		return closingHandler(dialog.close, props.onClick);
	} }));
}
function AlertDialogCancel(props) {
	const dialog = useAlertDialog();
	return createComponent$1(Button, mergeProps(props, {
		get variant() {
			return props.variant ?? "outline";
		},
		get onClick() {
			return closingHandler(dialog.close, props.onClick);
		}
	}));
}
const AlertDialogHeader = DialogHeader;
const AlertDialogFooter = DialogFooter;
const AlertDialogTitle = DialogTitle;
const AlertDialogDescription = DialogDescription;
//#endregion
//#region src/components/aspect-ratio.tsx
function aspectRatioStyle(ratio, style) {
	const resolved = ratio ?? 1;
	if (!Number.isFinite(resolved) || resolved <= 0) throw new RangeError("AspectRatio ratio must be a finite positive number");
	return {
		...style,
		"aspect-ratio": resolved
	};
}
/** A native Taffy aspect-ratio constraint with explicit overflow ownership. */
function AspectRatio(props) {
	const rest = omit(props, "ratio", "style", "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		"data-wabou-owns": "clip",
		get ["class"]() {
			return mergeClasses("w-full min-w-0 overflow-hidden", props.class);
		},
		get style() {
			return aspectRatioStyle(props.ratio, props.style);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
//#region src/components/attachment.tsx
const AttachmentContext = createContext({
	state: () => "done",
	size: () => "default",
	orientation: () => "horizontal"
});
function attachmentClass(options) {
	const state = options.state ?? "done";
	const size = options.size ?? "default";
	const orientation = options.orientation ?? "horizontal";
	return mergeClasses("max-w-full min-w-0 flex-none flex border bg-surface text-primary", match(orientation).with("horizontal", () => "min-w-40 flex-row flex-wrap items-center").with("vertical", () => "w-28 flex-col items-stretch").exhaustive(), match(size).with("default", () => "gap-2 rounded-xl p-2 text-sm").with("sm", () => "gap-2 rounded-lg p-1.5 text-xs").with("xs", () => "gap-1.5 rounded-md p-1 text-xs").exhaustive(), match(state).with("idle", () => "border-strong").with("uploading", () => "border-focus").with("processing", () => "border-accent").with("error", () => "border-danger bg-danger-surface").with("done", () => "border-subtle").exhaustive(), options.class);
}
/** File/task summary anatomy adapted from shadcn without DOM data selectors. */
function Attachment(props) {
	const forwarded = omit(props, "state", "size", "orientation", "class", "children");
	const context = {
		state: () => props.state ?? "done",
		size: () => props.size ?? "default",
		orientation: () => props.orientation ?? "horizontal"
	};
	return createComponent$1(AttachmentContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get ["class"]() {
					return attachmentClass({
						state: context.state(),
						size: context.size(),
						orientation: context.orientation(),
						class: props.class
					});
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function attachmentMediaClass(variant, context, className) {
	const size = context.size();
	const orientation = context.orientation();
	const state = context.state();
	return mergeClasses("aspect-square flex-none overflow-hidden flex items-center justify-center rounded-lg", orientation === "vertical" ? "w-full" : match(size).with("default", () => "w-10").with("sm", () => "w-8").with("xs", () => "w-7").exhaustive(), state === "error" ? "bg-danger-surface text-danger-primary" : "bg-control text-primary", variant === "image" && state !== "done" && "opacity-60", className);
}
function AttachmentMedia(props) {
	const context = useContext(AttachmentContext);
	return createComponent$1(View, {
		get ["class"]() {
			return attachmentMediaClass(props.variant ?? "icon", context, props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function AttachmentContent(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("max-w-full min-w-0 flex-1 flex flex-col gap-0.5", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function AttachmentTitle(props) {
	return createComponent$1(Text, mergeProps(props, {
		get maxLines() {
			return props.maxLines ?? 1;
		},
		get ["class"]() {
			return mergeClasses("max-w-full min-w-0 font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function AttachmentDescription(props) {
	const context = useContext(AttachmentContext);
	return createComponent$1(Text, mergeProps(props, {
		get maxLines() {
			return props.maxLines ?? 1;
		},
		get ["class"]() {
			return mergeClasses("max-w-full min-w-0 text-xs", context.state() === "error" ? "text-danger-primary" : "text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function AttachmentActions(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("flex-none flex items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function AttachmentAction(props) {
	return createComponent$1(Button, mergeProps(props, {
		get variant() {
			return props.variant ?? "ghost";
		},
		get size() {
			return props.size ?? "sm";
		}
	}));
}
function attachmentGroupClass(className) {
	return mergeClasses("w-full min-w-0 overflow-x-auto overflow-y-hidden py-1 flex flex-row items-start gap-3", className);
}
function AttachmentGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return attachmentGroupClass(props.class);
		},
		get scrollbar() {
			return props.scrollbar ?? { visibility: "hidden" };
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
//#region src/components/avatar.tsx
function Avatar(props) {
	const size = () => match(props.size ?? "default").with("sm", () => "w-8 h-8 text-xs").with("default", () => "w-10 h-10 text-sm").with("lg", () => "w-12 h-12 text-base").exhaustive();
	return createComponent$1(Center, {
		role: "img",
		get ["aria-label"]() {
			return props.alt ?? props.fallback;
		},
		get ["class"]() {
			return mergeClasses("flex-none overflow-hidden rounded-full bg-control border border-subtle", size(), props.class);
		},
		get children() {
			return memo(() => {
				return !!props.image;
			})() ? createComponent$1(Image, {
				"aria-hidden": "true",
				get resource() {
					return props.image;
				},
				class: "w-full h-full"
			}) : createComponent$1(Text, {
				"aria-hidden": "true",
				class: "font-medium text-secondary",
				get children() {
					return props.fallback;
				}
			});
		}
	});
}
function AvatarGroup(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("flex items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function AvatarGroupCount(props) {
	return createComponent$1(Center, {
		get ["class"]() {
			return mergeClasses("w-10 h-10 flex-none rounded-full bg-control border border-subtle", props.class);
		},
		get children() {
			return createComponent$1(Text, {
				class: "text-xs font-medium text-muted",
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/components/button-group.tsx
/** A single bordered control surface composed from ordinary Wabou buttons. */
function ButtonGroup(props) {
	const orientation = () => props.orientation ?? "horizontal";
	const layout = () => match(orientation()).with("horizontal", () => "flex-row items-stretch").with("vertical", () => "flex-col items-stretch").exhaustive();
	return createComponent(ButtonGroupContext, {
		get value() {
			return orientation();
		},
		get children() {
			return createComponent$1(View, mergeProps(props, {
				role: "group",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["class"]() {
					return mergeClasses("min-w-0 flex gap-0 overflow-hidden rounded-lg border border-strong bg-surface shadow-xs", layout(), props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function ButtonGroupText(props) {
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-h-8 px-3 flex-none flex items-center whitespace-nowrap text-sm font-medium text-secondary bg-control", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ButtonGroupSeparator(props) {
	return createComponent$1(View, {
		role: "separator",
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("flex-none self-stretch bg-strong", match(props.orientation ?? "vertical").with("vertical", () => "w-px min-h-full").with("horizontal", () => "h-px min-w-full").exhaustive(), props.class);
		}
	});
}
//#endregion
//#region src/components/card.tsx
function Card(props) {
	const theme = useComponentsTheme();
	const rest = omit(props, "class", "children", "shadows");
	return createComponent$1(View, mergeProps(rest, {
		get ["class"]() {
			return mergeClasses("min-w-0 min-h-0 flex-none flex flex-col overflow-hidden rounded-xl border border-subtle bg-surface", props.class);
		},
		get shadows() {
			return memo(() => {
				return props.shadows === void 0;
			})() ? componentsElevation(theme(), "raised") : props.shadows;
		},
		get children() {
			return props.children;
		}
	}));
}
function CardHeader(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("relative min-w-0 flex flex-col gap-1.5 px-5 pt-5 pr-14", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardTitle(props) {
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-w-0 text-base font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardDescription(props) {
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
/** Top-end action slot owned by the relative CardHeader surface. */
function CardAction(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("absolute top-5 right-5 flex-none flex items-center justify-end", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardContent(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-w-0 min-h-0 flex flex-col gap-4 px-5 pt-4 pb-5", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardFooter(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-w-0 flex items-center gap-2 px-5 pb-5", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
//#region src/primitives/interactions/collection.ts
function createCollection(source) {
	const enabled = () => source().filter((item) => !item.disabled);
	const adjacent = (id, delta, loop) => {
		const items = enabled();
		if (items.length === 0) return void 0;
		const next = (id === void 0 ? -1 : items.findIndex((item) => item.id === id)) + delta;
		if (next >= 0 && next < items.length) return items[next];
		if (!loop) return void 0;
		return delta === 1 ? items[0] : items[items.length - 1];
	};
	return {
		items: source,
		find: (id) => source().find((item) => item.id === id),
		indexOf: (id) => source().findIndex((item) => item.id === id),
		first: () => enabled()[0],
		last: () => enabled().at(-1),
		next: (id, loop = false) => adjacent(id, 1, loop),
		previous: (id, loop = false) => adjacent(id, -1, loop)
	};
}
//#endregion
//#region src/primitives/interactions/machine.ts
function unchanged(state) {
	return {
		state,
		commands: []
	};
}
//#endregion
//#region src/primitives/interactions/state.ts
function createControllableState(options) {
	const [local, setLocal] = createSignal({ value: options.defaultValue }, { ownedWrite: true });
	const value = () => {
		const controlled = options.value();
		return controlled === void 0 ? local().value : controlled;
	};
	return {
		value,
		set(next) {
			if (options.disabled?.() || Object.is(value(), next)) return false;
			if (options.value() === void 0) setLocal({ value: next });
			options.onChange?.(next);
			return true;
		}
	};
}
//#endregion
//#region src/primitives/interactions/disclosure.ts
function updateDisclosure(state, event) {
	return match(event).with({ type: "DISABLED" }, ({ disabled }) => ({
		state: {
			...state,
			disabled,
			open: disabled ? false : state.open
		},
		commands: []
	})).with({ type: "OPEN" }, () => state.disabled || state.open ? unchanged(state) : {
		state: {
			...state,
			open: true
		},
		commands: []
	}).with({ type: "CLOSE" }, () => !state.open ? unchanged(state) : {
		state: {
			...state,
			open: false
		},
		commands: []
	}).with({ type: "TOGGLE" }, () => state.disabled ? unchanged(state) : {
		state: {
			...state,
			open: !state.open
		},
		commands: []
	}).exhaustive();
}
function createDisclosure(options = {}) {
	const controlled = createControllableState({
		value: options.open ?? (() => void 0),
		defaultValue: options.defaultOpen ?? false,
		disabled: options.disabled,
		onChange: options.onOpenChange
	});
	const set = (type) => {
		const result = updateDisclosure({
			open: controlled.value(),
			disabled: options.disabled?.() ?? false
		}, { type });
		return controlled.set(result.state.open);
	};
	return {
		open: controlled.value,
		disabled: () => options.disabled?.() ?? false,
		openDisclosure: () => set("OPEN"),
		close: () => set("CLOSE"),
		toggle: () => set("TOGGLE")
	};
}
//#endregion
//#region src/primitives/interactions/roving-focus.ts
function createRovingFocus(options = {}) {
	const items = [];
	const enabled = () => items.filter((item) => !item.disabled?.());
	return {
		register(item) {
			items.push(item);
			return () => {
				const index = items.indexOf(item);
				if (index >= 0) items.splice(index, 1);
			};
		},
		move(current, key) {
			const orientation = options.orientation?.() ?? "horizontal";
			const direction = match({
				orientation,
				key
			}).with({ key: "Home" }, () => "first").with({ key: "End" }, () => "last").with(P.union({
				orientation: "horizontal",
				key: "ArrowRight"
			}, {
				orientation: "vertical",
				key: "ArrowDown"
			}), () => "next").with(P.union({
				orientation: "horizontal",
				key: "ArrowLeft"
			}, {
				orientation: "vertical",
				key: "ArrowUp"
			}), () => "previous").otherwise(() => void 0);
			if (!direction) return false;
			const candidates = enabled();
			if (candidates.length === 0) return false;
			const index = candidates.findIndex((item) => item.id === current);
			const target = match(direction).with("first", () => candidates[0]).with("last", () => candidates.at(-1)).with("next", () => candidates[index + 1] ?? (options.loop === false ? void 0 : candidates[0])).with("previous", () => candidates[index - 1] ?? (options.loop === false ? void 0 : candidates.at(-1))).exhaustive();
			if (!target) return false;
			options.onMove?.(target.id);
			target.target.focus();
			return true;
		}
	};
}
//#endregion
//#region src/primitives/interactions/typeahead.ts
function createTypeahead(options = {}) {
	let keys = "";
	let timer;
	const collator = typeof Intl === "undefined" || typeof Intl.Collator !== "function" ? void 0 : new Intl.Collator(options.locale, {
		usage: "search",
		sensitivity: "base"
	});
	const reset = () => {
		keys = "";
		if (timer !== void 0) clearTimeout(timer);
		timer = void 0;
	};
	return {
		search(items, key, activeId) {
			if (key.length !== 1) return void 0;
			keys += key;
			if (timer !== void 0) clearTimeout(timer);
			timer = setTimeout(reset, options.timeout ?? 350);
			const query = keys.length > 1 && [...keys].every((value) => value === keys[0]) ? keys[0] : keys;
			const enabled = items.filter((item) => !item.disabled && item.textValue);
			const active = enabled.findIndex((item) => item.id === activeId);
			return [...enabled.slice(active + 1), ...enabled.slice(0, active + 1)].find((item) => {
				const prefix = item.textValue?.slice(0, query.length) ?? "";
				return collator ? collator.compare(prefix, query) === 0 : prefix.toLowerCase() === query.toLowerCase();
			});
		},
		reset
	};
}
//#endregion
//#region src/primitives/interactions/select.ts
function updateSelect(state, event, options) {
	const collection = createCollection(() => options.items);
	const openAt = (id) => ({
		state: {
			...state,
			open: true,
			highlighted: id
		},
		commands: [{ type: "FOCUS_CONTENT" }, ...id ? [{
			type: "SCROLL_TO_ITEM",
			id
		}] : []]
	});
	const move = (direction) => {
		const candidate = direction === "next" ? collection.next(state.highlighted, options.loop ?? true) : collection.previous(state.highlighted, options.loop ?? true);
		if (!candidate) return {
			state,
			commands: []
		};
		return {
			state: {
				...state,
				highlighted: candidate.id
			},
			commands: [{
				type: "SCROLL_TO_ITEM",
				id: candidate.id
			}]
		};
	};
	return match(event).with({ type: "OPEN" }, () => state.open ? {
		state,
		commands: []
	} : openAt(state.value ?? collection.first()?.id)).with({ type: "CLOSE" }, () => ({
		state: {
			...state,
			open: false,
			highlighted: void 0
		},
		commands: state.open ? [{ type: "FOCUS_TRIGGER" }] : []
	})).with({ type: "TOGGLE" }, () => state.open ? {
		state: {
			...state,
			open: false,
			highlighted: void 0
		},
		commands: [{ type: "FOCUS_TRIGGER" }]
	} : openAt(state.value ?? collection.first()?.id)).with({ type: "ARROW_DOWN" }, () => state.open ? move("next") : openAt(state.value ?? collection.first()?.id)).with({ type: "ARROW_UP" }, () => state.open ? move("previous") : openAt(state.value ?? collection.last()?.id)).with({ type: "HOME" }, () => openAt(collection.first()?.id)).with({ type: "END" }, () => openAt(collection.last()?.id)).with(P.union({ type: "HIGHLIGHT" }, { type: "TYPEAHEAD" }), ({ id }) => collection.find(id)?.disabled ? {
		state,
		commands: []
	} : event.type === "TYPEAHEAD" && !state.open ? openAt(id) : {
		state: {
			...state,
			highlighted: id
		},
		commands: [{
			type: "SCROLL_TO_ITEM",
			id
		}]
	}).with({ type: "SELECT" }, ({ id }) => {
		const selected = id ?? state.highlighted;
		if (!selected || collection.find(selected)?.disabled) return {
			state,
			commands: []
		};
		const close = options.closeOnSelect ?? true;
		return {
			state: {
				open: close ? false : state.open,
				value: selected,
				highlighted: close ? void 0 : selected
			},
			commands: close ? [{ type: "FOCUS_TRIGGER" }] : []
		};
	}).exhaustive();
}
function createSelectInteraction(options) {
	const value = createControllableState({
		value: options.value ?? (() => void 0),
		defaultValue: options.defaultValue,
		disabled: options.disabled,
		onChange: (next) => next && options.onValueChange?.(next)
	});
	const open = createControllableState({
		value: options.open ?? (() => void 0),
		defaultValue: options.defaultOpen ?? false,
		disabled: options.disabled,
		onChange: options.onOpenChange
	});
	const [highlighted, setHighlighted] = createSignal();
	const typeahead = createTypeahead();
	const state = () => ({
		open: open.value(),
		value: value.value(),
		highlighted: highlighted()
	});
	const send = (event) => {
		if (options.disabled?.()) return false;
		const result = updateSelect(state(), event, {
			items: options.items(),
			loop: options.loop
		});
		const previous = state();
		open.set(result.state.open);
		if (result.state.value !== void 0) value.set(result.state.value);
		setHighlighted(result.state.highlighted);
		for (const command of result.commands) options.execute?.(command);
		return previous.open !== result.state.open || previous.value !== result.state.value || previous.highlighted !== result.state.highlighted;
	};
	return {
		state,
		open: open.value,
		value: value.value,
		highlighted,
		send,
		typeahead(key) {
			const item = typeahead.search(options.items(), key, highlighted());
			return item ? send({
				type: "TYPEAHEAD",
				id: item.id
			}) : false;
		}
	};
}
//#endregion
//#region src/components/carousel.tsx
const CarouselContext = createContext();
function useCarousel() {
	const context = useContext(CarouselContext);
	if (!context) throw new Error("Carousel child must be used inside Carousel");
	return context;
}
function normalizeCarouselIndex(index, count, loop) {
	if (count <= 0) return 0;
	if (loop) return (Math.trunc(index) % count + count) % count;
	return Math.min(count - 1, Math.max(0, Math.trunc(index)));
}
/** A native snapping carousel with captured pointer dragging and keyboard navigation. */
function Carousel(props) {
	const items = [];
	const [revision, setRevision] = createSignal(0, { ownedWrite: true });
	const state = createControllableState({
		value: () => props.index,
		defaultValue: normalizeCarouselIndex(props.defaultIndex ?? 0, 1, false),
		onChange: props.onIndexChange
	});
	const count = () => {
		revision();
		return items.length;
	};
	const selectedIndex = () => normalizeCarouselIndex(state.value(), count(), props.loop ?? false);
	const scrollTo = (index) => {
		if (count() === 0) return;
		state.set(normalizeCarouselIndex(index, count(), props.loop ?? false));
	};
	const context = {
		orientation: () => props.orientation ?? "horizontal",
		selectedIndex,
		itemCount: count,
		canScrollPrevious: () => count() > 1 && ((props.loop ?? false) || selectedIndex() > 0),
		canScrollNext: () => count() > 1 && ((props.loop ?? false) || selectedIndex() < count() - 1),
		scrollPrevious: () => scrollTo(selectedIndex() - 1),
		scrollNext: () => scrollTo(selectedIndex() + 1),
		scrollTo,
		register(id) {
			items.push(id);
			setRevision((value) => value + 1);
			return () => {
				const index = items.indexOf(id);
				if (index >= 0) items.splice(index, 1);
				setRevision((value) => value + 1);
			};
		}
	};
	props.setApi?.(context);
	return createComponent(CarouselContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				role: "group",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["class"]() {
					return mergeClasses("relative min-w-0 min-h-0", props.class);
				},
				onKeyDown: (event) => {
					if (match({
						orientation: context.orientation(),
						key: event.key
					}).with({
						orientation: "horizontal",
						key: "ArrowLeft"
					}, () => {
						context.scrollPrevious();
						return true;
					}).with({
						orientation: "horizontal",
						key: "ArrowRight"
					}, () => {
						context.scrollNext();
						return true;
					}).with({
						orientation: "vertical",
						key: "ArrowUp"
					}, () => {
						context.scrollPrevious();
						return true;
					}).with({
						orientation: "vertical",
						key: "ArrowDown"
					}, () => {
						context.scrollNext();
						return true;
					}).otherwise(() => false)) event.preventDefault();
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function CarouselContent(props) {
	const carousel = useCarousel();
	const measured = createMeasuredSize();
	const reducedMotion = useReducedMotion();
	const [offset, setOffset] = createSignal(0, { ownedWrite: true });
	const [dragDelta, setDragDelta] = createSignal(0, { ownedWrite: true });
	const [dragging, setDragging] = createSignal(false, { ownedWrite: true });
	let startCoordinate = 0;
	let controls;
	const axisSize = () => carousel.orientation() === "horizontal" ? measured.width() : measured.height();
	const targetOffset = () => -carousel.selectedIndex() * axisSize();
	const coordinate = (event) => carousel.orientation() === "horizontal" ? event.clientX : event.clientY;
	const settle = () => {
		controls?.cancel();
		const target = targetOffset();
		if (reducedMotion() || Math.abs(target - offset()) < .5) {
			setOffset(target);
			return;
		}
		controls = animate(offset(), target, {
			duration: .28,
			ease: [
				.22,
				1,
				.36,
				1
			],
			onUpdate: setOffset
		});
	};
	createEffect(() => [targetOffset(), measured.measured()], () => {
		if (!dragging()) settle();
	});
	onCleanup(() => controls?.cancel());
	const stopDragging = (cancelled = false) => {
		if (!dragging()) return;
		const delta = dragDelta();
		const threshold = Math.max(12, axisSize() * (props.dragThreshold ?? .15));
		setOffset(offset() + delta);
		setDragDelta(0);
		setDragging(false);
		if (!cancelled && Math.abs(delta) >= threshold) {
			if (delta < 0) carousel.scrollNext();
			else carousel.scrollPrevious();
		}
		requestAnimationFrame(settle);
	};
	const forwarded = omit(props, "trackClass", "dragThreshold", "children", "class", "ref", "onPointerDown", "onPointerMove", "onPointerUp", "onPointerCancel");
	return createComponent$1(View, mergeProps(forwarded, {
		ref: (node) => {
			measured.ref(node);
			props.ref?.(node);
		},
		get ["class"]() {
			return mergeClasses("min-w-0 min-h-0 overflow-hidden", props.class);
		},
		onPointerDown: (event) => {
			if (event.button !== 0 || axisSize() <= 0) return;
			controls?.cancel();
			startCoordinate = coordinate(event);
			setDragDelta(0);
			setDragging(true);
			event.preventDefault();
			props.onPointerDown?.(event);
		},
		onPointerMove: (event) => {
			if (dragging() && event.buttons !== 0) setDragDelta(coordinate(event) - startCoordinate);
			props.onPointerMove?.(event);
		},
		onPointerUp: (event) => {
			stopDragging();
			props.onPointerUp?.(event);
		},
		onPointerCancel: (event) => {
			stopDragging(true);
			props.onPointerCancel?.(event);
		},
		get children() {
			return createComponent$1(View, {
				get ["class"]() {
					return mergeClasses("flex w-full h-full flex-none", carousel.orientation() === "horizontal" ? "flex-row" : "flex-col", props.trackClass);
				},
				get transform() {
					return memo(() => {
						return carousel.orientation() === "horizontal";
					})() ? translate2d$1(offset() + dragDelta(), 0) : translate2d$1(0, offset() + dragDelta());
				},
				get children() {
					return props.children;
				}
			});
		}
	}));
}
function CarouselItem(props) {
	const carousel = useCarousel();
	const id = createUniqueId();
	const unregister = carousel.register(id);
	onCleanup(unregister);
	return createComponent$1(View, mergeProps(props, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["class"]() {
			return mergeClasses("min-w-0 min-h-0 flex-none", carousel.orientation() === "horizontal" ? "w-full" : "h-full", props.class);
		}
	}));
}
function CarouselNavigationButton(props) {
	const carousel = useCarousel();
	const previous = props.direction === "previous";
	const forwarded = omit(props, "direction", "children", "onClick");
	return createComponent$1(Button, mergeProps(forwarded, {
		get ["aria-label"]() {
			return props["aria-label"] ?? (previous ? "Previous slide" : "Next slide");
		},
		get variant() {
			return props.variant ?? "outline";
		},
		get size() {
			return props.size ?? "icon";
		},
		get disabled() {
			return props.disabled ?? (previous ? !carousel.canScrollPrevious() : !carousel.canScrollNext());
		},
		onClick: (event) => {
			if (previous) carousel.scrollPrevious();
			else carousel.scrollNext();
			props.onClick?.(event);
		},
		get children() {
			return props.children ?? createComponent$1(Icon, {
				"aria-hidden": "true",
				get source() {
					return match({
						previous,
						orientation: carousel.orientation()
					}).with({
						orientation: "vertical",
						previous: true
					}, () => arrowUp).with({
						orientation: "vertical",
						previous: false
					}, () => arrowDown).with({ previous: true }, () => arrowLeft).otherwise(() => arrowRight);
				},
				size: 16
			});
		}
	}));
}
function CarouselPrevious(props) {
	return createComponent$1(CarouselNavigationButton, mergeProps(props, { direction: "previous" }));
}
function CarouselNext(props) {
	return createComponent$1(CarouselNavigationButton, mergeProps(props, { direction: "next" }));
}
//#endregion
//#region src/components/chart.tsx
const ChartContext = createContext({});
function ChartContainer(props) {
	return createComponent(ChartContext, {
		get value() {
			return props.config;
		},
		get children() {
			return createComponent$1(View, {
				role: "img",
				get ["aria-label"]() {
					return props.label;
				},
				get ["class"]() {
					return mergeClasses("relative min-w-0", props.class);
				},
				get style() {
					return props.style;
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function useChartConfig() {
	return useContext(ChartContext);
}
function ChartLegend(props) {
	const config = useChartConfig();
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("flex flex-row flex-wrap items-center gap-4", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return Object.values(config);
				},
				children: (series) => createComponent$1(View, {
					class: "flex flex-row items-center gap-2",
					get children() {
						return [createComponent$1(View, {
							"aria-hidden": "true",
							get ["class"]() {
								return mergeClasses("w-2.5 h-2.5 flex-none rounded-full", series.colorClass);
							}
						}), createComponent$1(Text, {
							class: "text-sm text-secondary",
							get children() {
								return series.label;
							}
						})];
					}
				})
			});
		}
	});
}
function ChartEmpty(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("h-48 items-center justify-center", props.class);
		},
		get children() {
			return createComponent$1(Text, {
				class: "text-sm text-muted",
				get children() {
					return props.message ?? "No chart data";
				}
			});
		}
	});
}
//#endregion
//#region src/components/copy-button.tsx
function CopyButton(props) {
	const clipboard = useClipboard();
	const [copied, setCopied] = createSignal(false);
	const forwarded = omit(props, "value", "idleLabel", "copiedLabel", "copiedChildren", "onCopied", "onCopyError");
	const copy = async () => {
		try {
			await clipboard.writeText(props.value);
			setCopied(true);
			props.onCopied?.();
		} catch (error) {
			props.onCopyError?.(error);
		}
	};
	return createComponent$1(Button, mergeProps(forwarded, {
		get ["aria-label"]() {
			return props["aria-label"] ?? "Copy";
		},
		onClick: copy,
		get children() {
			return memo(() => {
				return !!copied();
			})() ? props.copiedChildren ?? props.copiedLabel ?? "Copied" : props.children ?? props.idleLabel ?? "Copy";
		}
	}));
}
//#endregion
//#region src/components/code-block.tsx
function CodeBlock(props) {
	return createComponent$1(View, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Code block";
		},
		get ["class"]() {
			return mergeClasses("min-w-0 overflow-hidden rounded-xl border border-subtle bg-control", props.class);
		},
		get children() {
			return [createComponent$1(View, {
				class: "h-8 flex flex-row items-center justify-between gap-3 pl-3 pr-1 bg-control",
				get children() {
					return [createComponent$1(Text, {
						class: "min-w-0 font-mono text-xs text-muted",
						get children() {
							return props.language ?? "text";
						}
					}), memo(() => {
						return memo(() => {
							return props.copyable !== false;
						})() && createComponent$1(CopyButton, {
							get value() {
								return props.code;
							},
							variant: "ghost",
							size: "icon",
							class: "w-7 h-7 text-muted",
							get idleLabel() {
								return props.copyLabel;
							},
							get copiedLabel() {
								return props.copiedLabel;
							},
							get ["aria-label"]() {
								return props.copyLabel;
							},
							get copiedChildren() {
								return createComponent$1(Icon, {
									source: check,
									size: 13
								});
							},
							get children() {
								return createComponent$1(Icon, {
									source: copy,
									size: 13
								});
							}
						});
					})];
				}
			}), createComponent$1(Text, {
				class: "min-w-0 p-3 font-mono text-sm text-primary whitespace-normal",
				get children() {
					return props.code;
				}
			})];
		}
	});
}
//#endregion
//#region src/components/menu-state.ts
/** Resolve one keyboard move without coupling menu state to rendering. */
function moveMenuHighlight(items, current, move) {
	const enabled = items.filter((item) => !item.disabled);
	if (enabled.length === 0) return void 0;
	if (move === "first") return enabled[0].id;
	if (move === "last") return enabled.at(-1)?.id;
	const index = enabled.findIndex((item) => item.id === current);
	if (move === "next") return enabled[(index + 1) % enabled.length].id;
	return enabled[(index <= 0 ? enabled.length : index) - 1].id;
}
//#endregion
//#region src/components/command-state.ts
function filterCommandItems(items, query) {
	const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
	if (terms.length === 0) return [...items];
	return items.filter((item) => {
		const haystack = [item.label, ...item.keywords ?? []].join(" ").toLocaleLowerCase();
		return terms.every((term) => haystack.includes(term));
	});
}
function reconcileCommandHighlight(items, highlighted) {
	if (items.some((item) => item.id === highlighted && !item.disabled)) return highlighted;
	return moveMenuHighlight(items, void 0, "first");
}
//#endregion
//#region src/components/input.tsx
/** A plain-text input. Secrets must use `PasswordInput`. */
function Input(props) {
	const forwarded = omit(props, "chrome", "surfaceClass");
	return createComponent$1(TextInput, mergeProps(forwarded, {
		get ["data-wabou-owns"]() {
			return (props.chrome ?? "default") === "default" ? "surface native-editor" : "native-editor";
		},
		get ["class"]() {
			return mergeClasses("w-full text-primary", (props.chrome ?? "default") === "default" ? componentsControlSize("default") : componentsControlContentSize("default"), (props.chrome ?? "default") === "default" && mergeClasses("border border-subtle shadow-xs", props.surfaceClass ?? "bg-input"), props.disabled && "opacity-50", props.class);
		}
	}));
}
//#endregion
//#region src/components/command.tsx
/** Searchable command list whose filtering and keyboard behavior are host-independent. */
function Command(props) {
	const [uncontrolledQuery, setUncontrolledQuery] = createSignal(props.defaultQuery ?? "");
	const [highlighted, setHighlighted] = createSignal();
	const query = () => props.query ?? uncontrolledQuery();
	const filtered = createMemo(() => filterCommandItems(props.items, query()));
	createEffect(() => ({
		items: filtered(),
		highlighted: highlighted()
	}), ({ items, highlighted: current }) => {
		setHighlighted(reconcileCommandHighlight(items, current));
	});
	const setQuery = (next) => {
		if (props.query === void 0) setUncontrolledQuery(next);
		props.onQueryChange?.(next);
	};
	const select = (id) => {
		const item = filtered().find((candidate) => candidate.id === id);
		if (!item || item.disabled) return false;
		item.onSelect?.();
		props.onAction?.(item.id);
		return true;
	};
	const move = (direction) => {
		const next = moveMenuHighlight(filtered(), highlighted(), direction);
		if (next === void 0) return false;
		setHighlighted(next);
		return true;
	};
	const onKeyDown = (event) => {
		if (match(event.key).with("ArrowDown", () => move("next")).with("ArrowUp", () => move("previous")).with("Home", () => move("first")).with("End", () => move("last")).with("Enter", () => select(highlighted())).with("Escape", () => {
			props.onDismiss?.();
			return props.onDismiss !== void 0;
		}).otherwise(() => false)) event.preventDefault();
	};
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("min-w-0 flex flex-col gap-2", props.class);
		},
		get children() {
			return [createComponent$1(Input, {
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get value() {
					return query();
				},
				get placeholder() {
					return props.placeholder ?? "Type a command";
				},
				ref(r$) {
					var _ref$ = props.inputRef;
					typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : props.inputRef = r$;
				},
				onInput: (event) => setQuery(event.currentTarget.value),
				onKeyDown
			}), createComponent$1(View, {
				role: "listbox",
				get ["aria-label"]() {
					return `${props["aria-label"]} results`;
				},
				get ["aria-activedescendant"]() {
					return highlighted();
				},
				get ["class"]() {
					return mergeClasses("min-w-0 flex flex-col gap-1", props.listClass);
				},
				get children() {
					return memo(() => {
						return filtered().length === 0;
					})() ? createComponent$1(Text, {
						role: "status",
						class: "px-3 py-4 text-sm text-muted text-center",
						get children() {
							return props.emptyText ?? "No results found.";
						}
					}) : createComponent$1(For, {
						get each() {
							return filtered();
						},
						keyed: false,
						children: (item) => createComponent$1(View, {
							get id() {
								return item().id;
							},
							role: "option",
							get ["aria-label"]() {
								return item().label;
							},
							get ["aria-selected"]() {
								return highlighted() === item().id;
							},
							get ["aria-disabled"]() {
								return item().disabled;
							},
							get ["class"]() {
								return mergeClasses("min-h-9 px-3 py-1.5 flex flex-col justify-center rounded-lg", highlighted() === item().id ? "bg-control-hover text-primary" : "bg-transparent text-secondary");
							},
							get style() {
								return { opacity: item().disabled ? .45 : 1 };
							},
							onPointerMove: () => !item().disabled && setHighlighted(item().id),
							onClick: () => select(item().id),
							get children() {
								return createComponent$1(View, {
									class: "min-w-0 flex flex-row items-center justify-between gap-3",
									get children() {
										return [createComponent$1(View, {
											class: "min-w-0 flex-1 flex flex-col",
											get children() {
												return [createComponent$1(Text, {
													class: "min-w-0 truncate text-sm",
													get children() {
														return item().label;
													}
												}), memo(() => {
													return memo(() => {
														return !!item().description;
													})() ? createComponent$1(Text, {
														class: "min-w-0 truncate text-xs text-muted",
														get children() {
															return item().description;
														}
													}) : item().description;
												})];
											}
										}), memo(() => {
											return memo(() => {
												return !!item().shortcut;
											})() ? createComponent$1(Text, {
												"aria-hidden": "true",
												class: "flex-none rounded border border-subtle bg-surface px-1.5 py-0.5 text-xs text-muted",
												get children() {
													return item().shortcut;
												}
											}) : item().shortcut;
										})];
									}
								});
							}
						})
					});
				}
			})];
		}
	});
}
//#endregion
//#region src/components/combobox.tsx
/** A searchable single-value picker built from Popover and Command. */
function Combobox(props) {
	const theme = useComponentsTheme();
	const [uncontrolledValue, setUncontrolledValue] = createSignal(props.defaultValue);
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const [query, setQuery] = createSignal("");
	let trigger;
	let search;
	const value = () => props.value ?? uncontrolledValue();
	const open = () => props.open ?? uncontrolledOpen();
	const selected = () => props.options.find((option) => option.value === value());
	const setOpen = (next) => {
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next);
		if (next) {
			setQuery("");
			requestAnimationFrame(() => search?.focus());
		} else requestAnimationFrame(() => trigger?.focus());
	};
	const select = (id) => {
		const option = props.options.find((candidate) => candidate.id === id);
		if (!option || option.disabled) return;
		if (props.value === void 0) setUncontrolledValue(option.value);
		option.onSelect?.();
		props.onValueChange?.(option.value);
		setOpen(false);
	};
	return createComponent$1(Popover$1, {
		contentRole: "presentation",
		popupRole: "listbox",
		get open() {
			return open();
		},
		onOpenChange: setOpen,
		placement: "bottom-start",
		get contentClass() {
			return mergeClasses("w-72 p-2 rounded-xl border border-subtle bg-surface", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		trigger: (popover) => createComponent$1(Button$1, {
			unstyled: true,
			role: "combobox",
			get disabled() {
				return props.disabled;
			},
			get ["aria-label"]() {
				return props["aria-label"];
			},
			"aria-haspopup": "listbox",
			get ["aria-expanded"]() {
				return open();
			},
			get ["aria-valuetext"]() {
				return selected()?.label;
			},
			ref: (node) => {
				trigger = node;
				popover.ref(node);
			},
			class: (state) => mergeClasses("w-72 overflow-hidden justify-between border bg-input shadow-xs", componentsControlSize("default"), state.focused ? "border-focus" : "border-subtle", props.class),
			style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
			get onClick() {
				return popover.onClick;
			},
			get onKeyDown() {
				return popover.onKeyDown;
			},
			get children() {
				return [createComponent$1(Text, {
					get ["class"]() {
						return mergeClasses("min-w-0 flex-1 text-left truncate", selected() ? "text-primary" : "text-muted");
					},
					get children() {
						return selected()?.label ?? props.placeholder ?? "Select an option";
					}
				}), createComponent$1(Icon, {
					source: chevronDown,
					class: "flex-none text-muted",
					size: 16
				})];
			}
		}),
		get children() {
			return createComponent$1(Command, {
				get ["aria-label"]() {
					return `${props["aria-label"]} search`;
				},
				get query() {
					return query();
				},
				onQueryChange: setQuery,
				get placeholder() {
					return props.searchPlaceholder ?? "Search options";
				},
				get emptyText() {
					return props.emptyText;
				},
				get items() {
					return props.options;
				},
				onAction: select,
				onDismiss: () => setOpen(false),
				inputRef: (node) => search = node
			});
		}
	});
}
//#endregion
//#region src/components/config-editor.tsx
/**
* Configuration editor backed by DOM-free CodeMirror state and a controlled
* native viewport. It is intentionally not a general-purpose IDE editor.
*/
function ConfigEditor(props) {
	return createComponent$1(CodeEditor, mergeProps(props, {
		language: "json",
		get ["class"]() {
			return mergeClasses("min-h-48 w-full rounded-lg border border-strong bg-input text-primary", props.class);
		}
	}));
}
//#endregion
//#region src/components/dropdown-menu.tsx
/** A compact action menu with native focus, typeahead, and overlay routing. */
function DropdownMenu(props) {
	const theme = useComponentsTheme();
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const [highlighted, setHighlighted] = createSignal();
	const open = () => props.open ?? uncontrolledOpen();
	const typeahead = createTypeahead();
	let trigger;
	let content;
	let openEdge = "first";
	let wasOpen = false;
	const setOpen = (next, edge = "first") => {
		openEdge = edge;
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next);
	};
	createEffect(() => ({
		open: open(),
		items: props.items,
		restoreFocus: props.restoreFocus ?? true
	}), ({ open: isOpen, items, restoreFocus }) => {
		if (isOpen && !wasOpen) {
			setHighlighted(moveMenuHighlight(items, void 0, openEdge));
			requestAnimationFrame(() => content?.focus());
		} else if (!isOpen && wasOpen) {
			setHighlighted(void 0);
			typeahead.reset();
			if (restoreFocus) requestAnimationFrame(() => trigger?.focus());
		}
		wasOpen = isOpen;
	});
	const select = (id) => {
		const item = props.items.find((candidate) => candidate.id === id);
		if (!item || item.disabled) return false;
		item.onSelect?.();
		props.onAction?.(item.id);
		setOpen(false);
		return true;
	};
	const move = (direction) => {
		const next = moveMenuHighlight(props.items, highlighted(), direction);
		if (next === void 0) return false;
		setHighlighted(next);
		return true;
	};
	const handleMenuKey = (event) => {
		props.onContentKeyDown?.(event);
		if (event.defaultPrevented) return;
		if (match(event.key).with("ArrowDown", () => move("next")).with("ArrowUp", () => move("previous")).with("Home", () => move("first")).with("End", () => move("last")).with("Enter", () => select(highlighted())).with(" ", () => select(highlighted())).with("Escape", () => {
			setOpen(false);
			return true;
		}).otherwise((key) => {
			const item = typeahead.search(props.items, key, highlighted());
			if (item) setHighlighted(item.id);
			return item !== void 0;
		})) event.preventDefault();
	};
	return createComponent$1(Popover$1, {
		contentRole: "presentation",
		popupRole: "menu",
		get open() {
			return open();
		},
		onOpenChange: (next) => setOpen(next),
		get placement() {
			return props.placement ?? "bottom-end";
		},
		get restoreFocus() {
			return props.restoreFocus;
		},
		get outsidePointerStrategy() {
			return props.outsidePointerStrategy;
		},
		get anchorPoint() {
			return props.anchorPoint;
		},
		get contentClass() {
			return mergeClasses("w-56 p-1 flex flex-col gap-0.5 rounded-xl border border-subtle bg-surface", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		trigger: (popover) => props.trigger({
			ref: (node) => {
				trigger = node;
				popover.ref(node);
			},
			onClick: popover.onClick,
			onKeyDown: (event) => {
				popover.onKeyDown(event);
				if (match(event.key).with("ArrowDown", () => {
					setOpen(true, "first");
					return true;
				}).with("ArrowUp", () => {
					setOpen(true, "last");
					return true;
				}).otherwise(() => false)) event.preventDefault();
			},
			"aria-haspopup": "menu",
			get "aria-expanded"() {
				return open();
			}
		}),
		get children() {
			return createComponent$1(View, {
				ref: (node) => content = node,
				role: "menu",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				focusOrder: 0,
				class: "min-w-0 flex flex-col gap-0.5",
				onKeyDown: handleMenuKey,
				get children() {
					return createComponent$1(For, {
						get each() {
							return props.items;
						},
						keyed: false,
						children: (item) => [memo(() => {
							return memo(() => {
								return !!item().separatorBefore;
							})() ? createComponent$1(View, {
								role: "presentation",
								class: "h-px flex-none my-1 bg-control"
							}) : item().separatorBefore;
						}), createComponent$1(View, {
							role: "menuitem",
							get ["aria-label"]() {
								return item().label;
							},
							get ["aria-disabled"]() {
								return item().disabled;
							},
							get ["class"]() {
								return mergeClasses("w-full min-h-8 flex-none px-2 py-1.5 flex flex-col justify-center rounded-lg", highlighted() === item().id ? "bg-control-hover" : "bg-transparent", item().destructive ? "text-danger-primary" : "text-primary");
							},
							get style() {
								return { opacity: item().disabled ? .45 : 1 };
							},
							onPointerMove: () => !item().disabled && setHighlighted(item().id),
							onClick: () => select(item().id),
							get children() {
								return [createComponent$1(Text, {
									class: "text-sm",
									get children() {
										return item().label;
									}
								}), memo(() => {
									return memo(() => {
										return !!item().description;
									})() ? createComponent$1(Text, {
										class: "text-xs text-muted",
										get children() {
											return item().description;
										}
									}) : item().description;
								})];
							}
						})]
					});
				}
			});
		}
	});
}
//#endregion
//#region src/components/context-menu.tsx
/** An action menu anchored to the native secondary-click coordinate. */
function ContextMenu(props) {
	const [open, setOpen] = createSignal(false);
	const [point, setPoint] = createSignal();
	return createComponent$1(DropdownMenu, {
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get items() {
			return props.items;
		},
		get open() {
			return open();
		},
		onOpenChange: setOpen,
		get onAction() {
			return props.onAction;
		},
		anchorPoint: point,
		get contentClass() {
			return props.contentClass;
		},
		get contentShadows() {
			return props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		trigger: (menu) => props.trigger({
			ref: menu.ref,
			onContextMenu: (event) => {
				event.preventDefault();
				event.stopPropagation();
				setPoint({
					x: event.clientX,
					y: event.clientY
				});
				setOpen(true);
			},
			onKeyDown: (event) => {
				if (event.key === "ContextMenu" || event.key === "F10") {
					event.preventDefault();
					event.stopPropagation();
					setPoint(void 0);
					setOpen(true);
					return;
				}
				menu.onKeyDown(event);
			},
			"aria-haspopup": "menu",
			get "aria-expanded"() {
				return open();
			}
		})
	});
}
//#endregion
//#region src/components/table.tsx
/**
* A horizontally scrollable table surface.
*
* Wabou has no implicit HTML table layout. Columns align because every row
* uses the same flex-cell anatomy; applications can override individual cell
* widths with the usual flex and width utilities.
*/
function Table(props) {
	const rest = omit(props, "class", "contentClass", "children");
	return createComponent$1(View, mergeProps(rest, {
		role: "table",
		get ["class"]() {
			return mergeClasses("relative w-full min-w-0 overflow-x-auto overflow-y-hidden", props.class);
		},
		get children() {
			return createComponent$1(View, {
				get ["class"]() {
					return mergeClasses("w-full min-w-full flex-none flex flex-col text-sm", props.contentClass);
				},
				get children() {
					return props.children;
				}
			});
		}
	}));
}
function TableHeader(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Table header";
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex-none flex flex-col", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableBody(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Table body";
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex-none flex flex-col", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableFooter(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Table footer";
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex-none flex flex-col border-t border-subtle bg-control", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableRow(props) {
	const hover = createHover();
	const rest = omit(props, "class", "selected", "children", "onPointerEnter", "onPointerLeave");
	return createComponent$1(View, mergeProps(rest, {
		role: "row",
		get ["aria-selected"]() {
			return props.selected;
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 min-h-11 flex-none flex flex-row items-stretch border-b border-subtle", props.selected ? "bg-selected" : "bg-surface", hover.hovered() && !props.selected ? "bg-control-hover" : void 0, props.class);
		},
		onPointerEnter: (event) => {
			hover.bindings.onPointerEnter();
			props.onPointerEnter?.(event);
		},
		onPointerLeave: (event) => {
			hover.bindings.onPointerLeave();
			props.onPointerLeave?.(event);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableHead(props) {
	return createComponent$1(Text, mergeProps(props, {
		role: "columnheader",
		get ["class"]() {
			return mergeClasses("min-w-32 flex-1 px-3 flex items-center whitespace-nowrap text-xs font-medium text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableCell(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "cell",
		get ["class"]() {
			return mergeClasses("min-w-32 flex-1 px-3 flex items-center text-sm text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function TableCaption(props) {
	return createComponent$1(Text, mergeProps(props, {
		get role() {
			return props.role ?? "label";
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex-none px-3 py-3 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
//#region src/components/data-table.tsx
/** Shadcn-style table anatomy backed by the framework-agnostic TanStack core. */
function DataTable(props) {
	return createComponent$1(Table, {
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get children() {
			return [createComponent$1(TableHeader, { get children() {
				return createComponent$1(For, {
					get each() {
						return props.model.table.getHeaderGroups();
					},
					children: (headerGroup) => createComponent$1(TableRow, {
						class: "h-11 border-strong bg-control",
						get children() {
							return createComponent$1(For, {
								get each() {
									return headerGroup.headers;
								},
								children: (header) => {
									const sortable = () => header.column.getCanSort();
									const direction = () => props.model.sorting().find(({ id }) => id === header.column.id)?.desc ? "desc" : props.model.sorting().some(({ id }) => id === header.column.id) ? "asc" : false;
									const label = () => String(header.column.columnDef.header ?? header.id);
									return createComponent$1(Button$1, {
										unstyled: true,
										role: "columnheader",
										get ["aria-label"]() {
											return memo(() => {
												return !!sortable();
											})() ? `Sort by ${label()}` : label();
										},
										class: "flex-1 min-w-0 px-4 justify-start text-xs font-semibold text-secondary",
										get disabled() {
											return !sortable();
										},
										onClick: () => header.column.toggleSorting(),
										get children() {
											return [memo(() => {
												return label();
											}), createComponent$1(Text, {
												class: "ml-auto text-xs text-muted",
												get children() {
													return memo(() => {
														return direction() === "asc";
													})() ? "Asc" : direction() === "desc" ? "Desc" : "";
												}
											})];
										}
									});
								}
							});
						}
					})
				});
			} }), createComponent$1(TableBody, { get children() {
				return [createComponent$1(For, {
					get each() {
						return props.model.rows();
					},
					children: (row) => {
						const cells = () => createComponent$1(For, {
							get each() {
								return row.getVisibleCells();
							},
							children: (cell) => createComponent$1(TableCell, {
								class: "px-4",
								get children() {
									return props.renderCell?.({
										value: cell.getValue(),
										columnId: cell.column.id,
										row
									}) ?? createComponent$1(Text, {
										class: "w-full truncate text-sm text-primary",
										get children() {
											return String(cell.getValue() ?? "");
										}
									});
								}
							})
						});
						return createComponent$1(Show, {
							get when() {
								return props.selectable;
							},
							get fallback() {
								return createComponent$1(TableRow, {
									get ["aria-label"]() {
										return `Row ${row.id}`;
									},
									get children() {
										return cells();
									}
								});
							},
							get children() {
								return createComponent$1(Button$1, {
									unstyled: true,
									role: "row",
									get ["aria-label"]() {
										return `Select row ${row.id}`;
									},
									get ["aria-selected"]() {
										return props.model.rowSelection()[row.id] === true;
									},
									get selected() {
										return props.model.rowSelection()[row.id] === true;
									},
									get ["class"]() {
										return props.model.rowSelection()[row.id] === true ? "h-12 w-full flex border-b border-subtle bg-selected" : "h-12 w-full flex border-b border-subtle bg-surface";
									},
									onClick: () => row.toggleSelected(),
									get children() {
										return cells();
									}
								});
							}
						});
					}
				}), createComponent$1(Show, {
					get when() {
						return props.model.rows().length === 0;
					},
					get children() {
						return createComponent$1(View, {
							class: "h-24 flex items-center justify-center",
							get children() {
								return createComponent$1(Text, {
									class: "text-sm text-muted",
									get children() {
										return props.emptyMessage ?? "No results";
									}
								});
							}
						});
					}
				})];
			} })];
		}
	});
}
//#endregion
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
		const requested = props.locale ?? host.intl.locale();
		return Intl.DateTimeFormat.supportedLocalesOf([requested])[0] ?? "en";
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
		const firstWeekday = dayOfWeek(startOfWeek(visibleMonth(), currentLocale));
		const weekday = new Intl.DateTimeFormat(currentLocale, {
			weekday: "short",
			timeZone: "UTC"
		});
		return {
			first_weekday: firstWeekday,
			month_label: new Intl.DateTimeFormat(currentLocale, {
				year: "numeric",
				month: "long",
				timeZone: "UTC"
			}).format(Date.UTC(visibleMonth().year, visibleMonth().month - 1, 1)),
			weekday_labels: Array.from({ length: 7 }, (_, offset) => {
				const index = (firstWeekday + offset) % 7;
				return weekday.format(Date.UTC(2024, 0, 7 + index));
			})
		};
	});
	const dateFormatters = createMemo(() => ({
		medium: new Intl.DateTimeFormat(locale(), {
			dateStyle: "medium",
			timeZone: "UTC"
		}),
		full: new Intl.DateTimeFormat(locale(), {
			dateStyle: "full",
			timeZone: "UTC"
		})
	}));
	const formatDate = (date, style) => dateFormatters()[style].format(Date.UTC(date.year, date.month - 1, date.day));
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
		get ["aria-label"]() {
			return props["aria-label"] ?? "Calendar";
		},
		class: "w-72 p-3 flex flex-col gap-3",
		get children() {
			return [
				createComponent$1(View, {
					class: "h-8 flex items-center justify-between",
					get children() {
						return [
							createComponent$1(Button$1, {
								unstyled: true,
								get ["aria-label"]() {
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
							createComponent$1(Button$1, {
								unstyled: true,
								get ["aria-label"]() {
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
								return createComponent$1(Button$1, {
									ref: (node) => dayRefs.set(date().toString(), node),
									unstyled: true,
									get ["aria-label"]() {
										return formatDate(date(), "full");
									},
									get ["aria-selected"]() {
										return selected();
									},
									get ["aria-current"]() {
										return isSameDay(date(), systemToday()) ? "date" : void 0;
									},
									get focusOrder() {
										return isSameDay(date(), tabStop()) ? 0 : -1;
									},
									get disabled() {
										return disabled();
									},
									class: (state) => mergeClasses("w-8 h-8 rounded-md items-center justify-center text-sm", selected() ? "bg-accent text-on-accent" : state.hovered ? "bg-control-hover text-primary" : "bg-transparent text-primary", outside() && "text-muted"),
									get style() {
										return { opacity: disabled() ? .35 : 1 };
									},
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
						return createComponent$1(Button$1, {
							unstyled: true,
							get ["aria-label"]() {
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
		const requested = props.locale ?? host.intl.locale();
		return Intl.DateTimeFormat.supportedLocalesOf([requested])[0] ?? "en";
	};
	const formatted = () => {
		const date = value();
		return date ? new Intl.DateTimeFormat(locale(), {
			dateStyle: "medium",
			timeZone: "UTC"
		}).format(Date.UTC(date.year, date.month - 1, date.day)) : props.placeholder ?? "Pick a date";
	};
	const select = (date) => {
		if (props.value === void 0) setLocalValue(date);
		props.onValueChange?.(date);
		setOpen(false);
	};
	return createComponent$1(Popover$1, {
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get open() {
			return open();
		},
		onOpenChange: setOpen,
		placement: "bottom-start",
		contentClass: "rounded-xl border border-subtle bg-surface",
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		trigger: (trigger) => createComponent$1(Button$1, mergeProps({ unstyled: true }, trigger, {
			get ["aria-label"]() {
				return props["aria-label"];
			},
			get disabled() {
				return props.disabled;
			},
			get ["class"]() {
				return mergeClasses("w-72 justify-start border border-subtle bg-input shadow-xs", componentsControlSize("default"), props.class);
			},
			get children() {
				return [createComponent$1(Icon, {
					source: calendarIcon,
					class: "flex-none text-muted",
					size: 16
				}), createComponent$1(Text, {
					get ["class"]() {
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
				get ["aria-label"]() {
					return props["aria-label"];
				},
				onValueChange: select
			}));
		}
	});
}
//#endregion
//#region src/components/disclosure.tsx
function DisclosureIndicator(props) {
	const rotation = createTransition(() => props.open() ? Math.PI : 0, {
		duration: .2,
		ease: "easeOut",
		reducedMotion: props.reducedMotion
	});
	return createComponent$1(View, {
		class: "w-4 h-4 flex-none",
		get transform() {
			return rotate2d$1(rotation.value());
		},
		"aria-hidden": "true",
		get children() {
			return createComponent$1(Icon, {
				source: chevronDown,
				class: "text-muted",
				size: 16
			});
		}
	});
}
const CollapsibleContext = createContext();
const useCollapsible = () => {
	const value = useContext(CollapsibleContext);
	if (!value) throw new Error("Collapsible parts must be inside Collapsible");
	return value;
};
function Collapsible(props) {
	const inheritedReducedMotion = useReducedMotion();
	const state = createDisclosure({
		open: () => props.open,
		defaultOpen: props.defaultOpen,
		disabled: () => props.disabled ?? false,
		onOpenChange: props.onOpenChange
	});
	const context = {
		open: state.open,
		toggle: state.toggle,
		disabled: state.disabled,
		reducedMotion: () => props.reducedMotion ?? inheritedReducedMotion()
	};
	const rest = omit(props, "open", "defaultOpen", "disabled", "reducedMotion", "onOpenChange", "class", "children");
	return createComponent$1(CollapsibleContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				get ["class"]() {
					return mergeClasses("flex flex-col", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function CollapsibleTrigger(props) {
	const context = useCollapsible();
	const rest = omit(props, "children", "class", "onClick");
	return createComponent$1(Button$1, mergeProps(rest, {
		unstyled: true,
		get disabled() {
			return context.disabled() || props.disabled;
		},
		get ["aria-expanded"]() {
			return context.open();
		},
		get ["class"]() {
			return mergeClasses("w-full min-h-7 flex items-center justify-between gap-3", props.class);
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented) context.toggle();
		},
		get children() {
			return [memo(() => {
				return props.children;
			}), createComponent$1(DisclosureIndicator, {
				get open() {
					return context.open;
				},
				get reducedMotion() {
					return context.reducedMotion;
				}
			})];
		}
	}));
}
function CollapsibleContent(props) {
	const context = useCollapsible();
	const contentProps = omit(props, "children", "class", "style");
	return createComponent$1(CollapsiblePresence, {
		get open() {
			return context.open();
		},
		get reducedMotion() {
			return context.reducedMotion();
		},
		get contentClass() {
			return props.class;
		},
		contentProps,
		get contentStyle() {
			return props.style;
		},
		get children() {
			return props.children;
		}
	});
}
function nextAccordionValue(current, type, item, collapsible = false) {
	return toggleSelection(current, item, type, collapsible) ?? "";
}
const AccordionContext = createContext();
const AccordionItemContext = createContext();
const useAccordion = () => {
	const value = useContext(AccordionContext);
	if (!value) throw new Error("Accordion parts must be inside Accordion");
	return value;
};
const useAccordionItem = () => {
	const value = useContext(AccordionItemContext);
	if (!value) throw new Error("Accordion parts must be inside AccordionItem");
	return value;
};
function Accordion(props) {
	const inheritedReducedMotion = useReducedMotion();
	const type = () => props.type ?? "single";
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? (type() === "multiple" ? [] : ""),
		disabled: () => props.disabled ?? false,
		onChange: props.onValueChange
	});
	const roving = createRovingFocus({ orientation: () => "vertical" });
	const context = {
		active: (item) => isSelected(state.value(), item),
		toggle: (item) => {
			state.set(nextAccordionValue(state.value(), type(), item, props.collapsible));
		},
		disabled: () => props.disabled ?? false,
		reducedMotion: () => props.reducedMotion ?? inheritedReducedMotion(),
		register: (value, node, disabled) => roving.register({
			id: value,
			target: node,
			disabled
		}),
		move: roving.move
	};
	const rest = omit(props, "type", "value", "defaultValue", "collapsible", "disabled", "reducedMotion", "onValueChange", "class", "children");
	return createComponent$1(AccordionContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				get ["class"]() {
					return mergeClasses("flex flex-col", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function AccordionItem(props) {
	const rest = omit(props, "value", "disabled", "children", "class");
	return createComponent$1(AccordionItemContext, {
		get value() {
			return {
				value: props.value,
				disabled: () => props.disabled ?? false
			};
		},
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				get ["class"]() {
					return mergeClasses("flex flex-col border-b border-subtle", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function AccordionTrigger(props) {
	const root = useAccordion();
	const item = useAccordionItem();
	const open = () => root.active(item.value);
	const rest = omit(props, "children", "class", "ref", "onClick", "onKeyDown");
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button$1, mergeProps(rest, {
		unstyled: true,
		get disabled() {
			return root.disabled() || item.disabled() || props.disabled;
		},
		get ["aria-expanded"]() {
			return open();
		},
		get ["class"]() {
			return mergeClasses("w-full py-4 flex items-center justify-between gap-4", props.class);
		},
		ref: (node) => {
			unregister?.();
			unregister = root.register(item.value, node, () => root.disabled() || item.disabled() || (props.disabled ?? false));
			props.ref?.(node);
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented) root.toggle(item.value);
		},
		onKeyDown: (event) => {
			props.onKeyDown?.(event);
			if (!event.defaultPrevented && root.move(item.value, event.key)) event.preventDefault();
		},
		get children() {
			return [createComponent$1(Text, {
				class: "min-w-0 flex-1 whitespace-normal text-sm font-medium text-primary",
				get children() {
					return props.children;
				}
			}), createComponent$1(DisclosureIndicator, {
				open,
				get reducedMotion() {
					return root.reducedMotion;
				}
			})];
		}
	}));
}
function AccordionContent(props) {
	const root = useAccordion();
	const item = useAccordionItem();
	const contentProps = omit(props, "children", "class", "style");
	return createComponent$1(CollapsiblePresence, {
		get open() {
			return root.active(item.value);
		},
		get reducedMotion() {
			return root.reducedMotion();
		},
		get contentClass() {
			return mergeClasses("pb-4", props.class);
		},
		contentProps,
		get contentStyle() {
			return props.style;
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/components/diff-viewer.tsx
const defaultLabels = {
	filesChanged: (count) => `${count} file${count === 1 ? "" : "s"} changed`,
	additions: (count) => `${count} addition${count === 1 ? "" : "s"}`,
	deletions: (count) => `${count} deletion${count === 1 ? "" : "s"}`,
	empty: "No code changes.",
	technicalDetails: "Technical diff"
};
const statusClasses = {
	added: "bg-success-surface text-success-primary",
	modified: "bg-control text-secondary",
	deleted: "bg-danger-surface text-danger-primary",
	renamed: "bg-control text-secondary"
};
const statusLabels = {
	added: "A",
	modified: "M",
	deleted: "D",
	renamed: "R"
};
/**
* A progressive-disclosure code change viewer.
*
* The summary and file metadata are ordinary Wabou components. Unified patch
* text is mounted only after disclosure and uses the DOM-free CodeMirror
* document/native editor viewport for selection, copying, and large documents.
*/
function DiffViewer(props) {
	const labels = () => ({
		...defaultLabels,
		...props.labels
	});
	const additions = () => props.files.reduce((total, file) => total + file.additions, 0);
	const deletions = () => props.files.reduce((total, file) => total + file.deletions, 0);
	return createComponent$1(View, {
		role: "region",
		get ["aria-label"]() {
			return labels().technicalDetails;
		},
		get ["class"]() {
			return mergeClasses("min-w-0 flex flex-col rounded-xl border border-subtle bg-surface overflow-hidden", props.class);
		},
		get children() {
			return [createComponent$1(View, {
				class: "flex-none px-4 py-3 flex flex-row items-center justify-between gap-3 bg-surface-muted",
				get children() {
					return [createComponent$1(Text, {
						class: "font-medium",
						get children() {
							return labels().filesChanged(props.files.length);
						}
					}), createComponent$1(View, {
						class: "flex-none flex flex-row items-center gap-2",
						get children() {
							return [createComponent$1(Text, {
								class: "text-sm text-success-primary",
								get children() {
									return ["+", memo(() => {
										return additions();
									})];
								}
							}), createComponent$1(Text, {
								class: "text-sm text-danger-primary",
								get children() {
									return ["-", memo(() => {
										return deletions();
									})];
								}
							})];
						}
					})];
				}
			}), createComponent$1(Show, {
				get when() {
					return props.files.length > 0;
				},
				get fallback() {
					return createComponent$1(Text, {
						role: "status",
						class: "p-4 text-sm text-muted",
						get children() {
							return labels().empty;
						}
					});
				},
				get children() {
					return createComponent$1(Accordion, {
						type: "multiple",
						get defaultValue() {
							return props.defaultExpanded ?? [];
						},
						class: "min-w-0",
						get children() {
							return createComponent$1(For, {
								get each() {
									return props.files;
								},
								children: (file) => createComponent$1(AccordionItem, {
									get value() {
										return file.path;
									},
									class: "px-4",
									get children() {
										return [createComponent$1(AccordionTrigger, {
											get ["aria-label"]() {
												return file.path;
											},
											class: "py-3 gap-3",
											get children() {
												return createComponent$1(View, {
													class: "min-w-0 flex-1 flex flex-row items-center gap-3",
													get children() {
														return [
															createComponent$1(View, {
																"aria-hidden": "true",
																get ["class"]() {
																	return mergeClasses("w-6 h-6 flex-none rounded flex items-center justify-center", statusClasses[file.status]);
																},
																get children() {
																	return createComponent$1(Text, {
																		class: "text-xs font-semibold",
																		get children() {
																			return statusLabels[file.status];
																		}
																	});
																}
															}),
															createComponent$1(View, {
																class: "min-w-0 flex-1 flex flex-col items-start",
																get children() {
																	return [createComponent$1(Text, {
																		class: "max-w-full truncate text-sm",
																		get children() {
																			return file.path;
																		}
																	}), createComponent$1(Show, {
																		get when() {
																			return file.oldPath;
																		},
																		get children() {
																			return createComponent$1(Text, {
																				class: "max-w-full truncate text-xs text-muted",
																				get children() {
																					return file.oldPath;
																				}
																			});
																		}
																	})];
																}
															}),
															createComponent$1(View, {
																class: "flex-none flex flex-row items-center gap-2",
																get children() {
																	return [createComponent$1(Text, {
																		get ["aria-label"]() {
																			return labels().additions(file.additions);
																		},
																		class: "text-xs text-success-primary",
																		get children() {
																			return ["+", memo(() => {
																				return file.additions;
																			})];
																		}
																	}), createComponent$1(Text, {
																		get ["aria-label"]() {
																			return labels().deletions(file.deletions);
																		},
																		class: "text-xs text-danger-primary",
																		get children() {
																			return ["-", memo(() => {
																				return file.deletions;
																			})];
																		}
																	})];
																}
															})
														];
													}
												});
											}
										}), createComponent$1(AccordionContent, {
											class: "min-w-0",
											get children() {
												return createComponent$1(CodeEditor, {
													readOnly: true,
													get value() {
														return file.patch;
													},
													get ["aria-label"]() {
														return `${labels().technicalDetails}: ${file.path}`;
													},
													class: "h-64 w-full rounded-lg border border-strong bg-input text-primary"
												});
											}
										})];
									}
								})
							});
						}
					});
				}
			})];
		}
	});
}
//#endregion
//#region src/components/direction.tsx
const DirectionContext = createContext("ltr");
/** Own logical direction in JavaScript instead of relying on web inheritance. */
function DirectionProvider(props) {
	return createComponent(DirectionContext, {
		get value() {
			return props.dir;
		},
		get children() {
			return props.children;
		}
	});
}
function useDirection() {
	return useContext(DirectionContext);
}
function DirectionalRow(props) {
	const direction = () => useDirection();
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("flex", direction() === "rtl" ? "flex-row-reverse" : "flex-row", props.class);
	} }));
}
function DirectionalText(props) {
	const direction = () => useDirection();
	return createComponent$1(Text, mergeProps(props, { get ["class"]() {
		return mergeClasses(direction() === "rtl" ? "text-right" : "text-left", props.class);
	} }));
}
//#endregion
//#region src/components/directory-picker-state.ts
function directoryPickerOptions(value, options) {
	return {
		...options,
		directory: options?.directory ?? (value.trim() || void 0)
	};
}
//#endregion
//#region src/components/label.tsx
function resolveControl(control) {
	return typeof control === "function" ? control() : control;
}
/** Text label that forwards pointer activation to an explicit native control. */
function Label(props) {
	const rest = omit(props, "class", "children", "disabled", "control", "onClick");
	return createComponent$1(Text, mergeProps(rest, {
		role: "label",
		get ["aria-disabled"]() {
			return props.disabled;
		},
		get ["class"]() {
			return mergeClasses("min-w-0 text-sm font-medium text-primary", props.disabled ? "opacity-50" : "cursor-pointer", props.class);
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!props.disabled && !event.defaultPrevented) resolveControl(props.control)?.focus();
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
//#region src/components/forms.tsx
function fieldClass(orientation = "vertical", invalid = false, className) {
	const layout = match(orientation).with("vertical", () => "flex-col gap-2").with("horizontal", () => "flex-row items-start gap-4").exhaustive();
	return mergeClasses("w-full min-w-0 flex", layout, invalid && "text-danger-primary", className);
}
function Field(props) {
	return createComponent$1(View, {
		role: "group",
		get ["class"]() {
			return fieldClass(props.orientation, props.invalid ?? false, props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldSet(props) {
	return createComponent$1(View, {
		role: "group",
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex flex-col gap-6", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldLegend(props) {
	return createComponent$1(Text, {
		role: "heading",
		get ["class"]() {
			return mergeClasses("mb-1 font-medium text-primary", props.variant === "label" ? "text-sm" : "text-base", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldGroup(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("flex flex-col gap-4", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldLabel(props) {
	return createComponent$1(Label, props);
}
function FieldTitle(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return mergeClasses("text-sm font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldContent(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("min-w-0 flex-1 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldDescription(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-xs text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function uniqueFieldErrors(errors) {
	return [...new Set((errors ?? []).map((error) => error?.message).filter((message) => Boolean(message)))];
}
function fieldErrorLabel(explicit, children, messages) {
	if (explicit) return explicit;
	if (typeof children === "string") return children;
	return messages.length > 0 ? messages.join(" ") : void 0;
}
function FieldError(props) {
	const messages = () => uniqueFieldErrors(props.errors);
	const label = () => fieldErrorLabel(props["aria-label"], props.children, messages());
	return createComponent$1(View, {
		role: "alert",
		get ["aria-label"]() {
			return label();
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex flex-col gap-1", props.class);
		},
		get children() {
			return createComponent$1(Show, {
				get when() {
					return memo(() => {
						return props.children !== void 0;
					})() && props.children !== null;
				},
				get fallback() {
					return createComponent$1(For, {
						get each() {
							return messages();
						},
						children: (message) => createComponent$1(Text, {
							class: "w-full min-w-0 whitespace-normal text-xs text-danger-primary",
							children: message
						})
					});
				},
				get children() {
					return createComponent$1(Text, {
						class: "w-full min-w-0 whitespace-normal text-xs text-danger-primary",
						get children() {
							return props.children;
						}
					});
				}
			});
		}
	});
}
function FieldSeparator(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 h-5 flex items-center gap-2", props.class);
		},
		get children() {
			return [
				createComponent$1(View, {
					"aria-hidden": "true",
					class: "flex-1 min-w-0 h-px bg-subtle"
				}),
				createComponent$1(Show, {
					get when() {
						return memo(() => {
							return props.children !== void 0;
						})() && props.children !== null;
					},
					get children() {
						return createComponent$1(Text, {
							class: "flex-none text-xs text-muted",
							get children() {
								return props.children;
							}
						});
					}
				}),
				createComponent$1(View, {
					"aria-hidden": "true",
					class: "flex-1 min-w-0 h-px bg-subtle"
				})
			];
		}
	});
}
const InputGroupContext = createContext();
function useInputGroup() {
	return useContext(InputGroupContext);
}
function inputGroupClass(orientation, focused, invalid) {
	return mergeClasses("relative w-full min-w-0 flex rounded-lg border shadow-xs", orientation === "horizontal" ? "h-8 flex-row items-center" : "h-auto flex-col items-stretch", invalid ? "border-danger" : focused ? "border-focus" : "border-strong");
}
function InputGroup(props) {
	const focus = createFocusWithin();
	let control;
	const context = {
		registerControl(node) {
			control = node;
		},
		focusControl() {
			if (!props.disabled) control?.focus();
		}
	};
	const forwarded = omit(props, "children", "orientation", "invalid", "disabled", "surfaceClass", "class");
	return createComponent$1(InputGroupContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, () => {
				return focus.bindings;
			}, {
				get role() {
					return props.role ?? "group";
				},
				get ["aria-invalid"]() {
					return props.invalid;
				},
				get ["aria-disabled"]() {
					return props.disabled;
				},
				"data-wabou-owns": "surface focus-ring",
				get ["class"]() {
					return mergeClasses(inputGroupClass(props.orientation ?? "horizontal", focus.focusWithin(), props.invalid ?? false), props.surfaceClass ?? "bg-input", props.disabled && "opacity-50", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function InputGroupInput(props) {
	const group = useInputGroup();
	return createComponent$1(Input, mergeProps(props, {
		ref: (node) => {
			group?.registerControl(node);
			props.ref?.(node);
		},
		chrome: "none",
		get ["class"]() {
			return mergeClasses("h-full flex-1 min-w-0", props.class);
		}
	}));
}
function inputGroupAddonClass(align) {
	return match(align).with("inline-start", "inline-end", () => "h-full flex-none px-3 flex items-center justify-center gap-2 text-sm text-muted").with("block-start", "block-end", () => "w-full flex-none px-3 py-2 flex items-center justify-start gap-2 text-sm text-muted").exhaustive();
}
function InputGroupAddon(props) {
	const group = useInputGroup();
	const forwarded = omit(props, "align", "focusControl", "class", "onClick");
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return props.role ?? "group";
		},
		get ["class"]() {
			return mergeClasses(inputGroupAddonClass(props.align ?? "inline-start"), props.class);
		},
		onClick: (event) => {
			if (props.focusControl ?? true) group?.focusControl();
			props.onClick?.(event);
		}
	}));
}
function InputGroupText(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return mergeClasses("flex-none text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function InputGroupButton(props) {
	return createComponent$1(Button, mergeProps(props, {
		get size() {
			return props.size ?? "sm";
		},
		get variant() {
			return props.variant ?? "ghost";
		},
		get ["class"]() {
			return mergeClasses("mx-1", props.class);
		}
	}));
}
function InputGroupTextArea(props) {
	const group = useInputGroup();
	return createComponent$1(TextArea, mergeProps(props, {
		ref: (node) => {
			group?.registerControl(node);
			props.ref?.(node);
		},
		"data-wabou-owns": "native-editor",
		get ["class"]() {
			return mergeClasses("w-full h-24 px-3 py-2 text-sm text-primary", props.class);
		}
	}));
}
//#endregion
//#region src/components/directory-picker.tsx
/** A controlled path input paired with the operating system directory picker. */
function DirectoryPicker(props) {
	const nativeDialog = useDialog();
	const [pending, setPending] = createSignal(false);
	const local = props;
	const inputProps = omit(props, "value", "onValueChange", "dialogOptions", "browseLabel", "pendingLabel", "browseAriaLabel", "class", "inputClass", "buttonClass", "onBrowseError");
	async function browse() {
		if (pending() || inputProps.disabled) return;
		setPending(true);
		try {
			const selected = await nativeDialog.pickDirectory(directoryPickerOptions(local.value, local.dialogOptions));
			if (selected !== null) local.onValueChange(selected);
		} catch (error) {
			if (local.onBrowseError) local.onBrowseError(error);
			else throw error;
		} finally {
			setPending(false);
		}
	}
	return createComponent$1(InputGroup, {
		get disabled() {
			return Boolean(inputProps.disabled) || pending();
		},
		get ["class"]() {
			return local.class;
		},
		get children() {
			return [createComponent$1(InputGroupInput, mergeProps(inputProps, {
				get ["class"]() {
					return local.inputClass;
				},
				get value() {
					return local.value;
				},
				onInput: (event) => local.onValueChange(event.currentTarget.value)
			})), createComponent$1(InputGroupButton, {
				get ["class"]() {
					return mergeClasses("flex-none", local.buttonClass);
				},
				get disabled() {
					return Boolean(inputProps.disabled) || pending();
				},
				get ["aria-label"]() {
					return local.browseAriaLabel ?? local.browseLabel ?? "Browse directory";
				},
				onClick: () => void browse(),
				get children() {
					return [createComponent$1(Icon, {
						source: folder,
						size: 14
					}), memo(() => {
						return memo(() => {
							return !!pending();
						})() ? local.pendingLabel ?? "Opening…" : local.browseLabel ?? "Browse…";
					})];
				}
			})];
		}
	});
}
//#endregion
//#region src/components/display.tsx
const SPINNER_SOURCE = `<svg viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="3" opacity="0.25"/><path d="M 12 3 A 9 9 0 0 1 21 12" stroke="currentColor" stroke-width="3" stroke-linecap="round"/></svg>`;
function Skeleton(props) {
	const reducedMotion = useReducedMotion();
	const motionDisabled = () => props.animated === false || reducedMotion();
	const [width, setWidth] = createSignal(0, { ownedWrite: true });
	const measured = createMeasuredSize({ onChange: (size) => setWidth(size.width) });
	const sweep = createSweep({
		extent: width,
		itemRatio: .4,
		duration: 1.6,
		ease: "easeInOut",
		reducedMotion: motionDisabled,
		reducedValue: .5
	});
	return createComponent$1(View, {
		ref(r$) {
			var _ref$ = measured.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
		},
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("overflow-hidden rounded-md bg-control", props.class);
		},
		get children() {
			return createComponent$1(View, {
				class: "w-2/5 h-full flex-none bg-control-hover",
				get transform() {
					return sweep.transform();
				},
				get style() {
					return { opacity: motionDisabled() ? 0 : 1 };
				}
			});
		}
	});
}
function Spinner(props) {
	return createComponent$1(Spin, {
		role: "status",
		get ["aria-label"]() {
			return props.label ?? "Loading";
		},
		get ["class"]() {
			return mergeClasses("w-4 h-4 flex-none text-accent", props.class);
		},
		duration: .9,
		get children() {
			return createComponent$1(Svg, {
				"aria-hidden": "true",
				class: "w-full h-full",
				source: SPINNER_SOURCE
			});
		}
	});
}
function Kbd(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return mergeClasses("h-5 min-w-5 px-1 py-0.5 flex-none text-center rounded bg-control text-xs font-medium text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function KbdGroup(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("inline-flex items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/components/drawer.tsx
const DrawerContext = createContext();
function useDrawer() {
	const context = useContext(DrawerContext);
	if (!context) throw new Error("Drawer child must be used inside Drawer");
	return context;
}
const drawerGeometry = (direction) => match(direction).with("left", () => ({
	backdrop: {
		"align-items": "stretch",
		"justify-content": "flex-start"
	},
	content: "h-full w-[420px] max-w-[80%] rounded-xl border-r",
	motion: { fromX: -48 }
})).with("right", () => ({
	backdrop: {
		"align-items": "stretch",
		"justify-content": "flex-end"
	},
	content: "h-full w-[420px] max-w-[80%] rounded-xl border-l",
	motion: { fromX: 48 }
})).with("top", () => ({
	backdrop: {
		"align-items": "flex-start",
		"justify-content": "stretch"
	},
	content: "w-full max-h-[80%] rounded-xl border-b",
	motion: { fromY: -48 }
})).with("bottom", () => ({
	backdrop: {
		"align-items": "flex-end",
		"justify-content": "stretch"
	},
	content: "w-full max-h-[80%] rounded-xl border-t",
	motion: { fromY: 48 }
})).exhaustive();
function drawerDragOffset(direction, rawDelta) {
	const outwardSign = direction === "right" || direction === "bottom" ? 1 : -1;
	return outwardSign * Math.max(0, outwardSign * rawDelta);
}
function drawerShouldDismiss(offset, size, threshold) {
	const dismissDistance = size > 0 ? size * threshold : 80;
	return Math.abs(offset) >= dismissDistance;
}
/** A focus-isolated edge drawer with a captured native drag-to-dismiss gesture. */
function Drawer(props) {
	const theme = useComponentsTheme();
	const reducedMotion = useReducedMotion();
	const measured = createMeasuredSize();
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(untrack(() => props.defaultOpen ?? false));
	const [dragOffset, setDragOffset] = createSignal(0, { ownedWrite: true });
	const [dragging, setDragging] = createSignal(false, { ownedWrite: true });
	const direction = () => props.direction ?? "bottom";
	const open = () => props.open ?? uncontrolledOpen();
	const axisSize = () => direction() === "left" || direction() === "right" ? measured.width() : measured.height();
	let startCoordinate = 0;
	let snapControls;
	const setOpen = (next, reason) => {
		if (next) setDragOffset(0);
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next, reason);
	};
	const snapBack = () => {
		snapControls?.cancel();
		if (reducedMotion()) {
			setDragOffset(0);
			return;
		}
		snapControls = animate(dragOffset(), 0, {
			duration: .22,
			ease: [
				.22,
				1,
				.36,
				1
			],
			onUpdate: setDragOffset
		});
	};
	const finishDrag = (cancelled = false) => {
		if (!dragging()) return;
		setDragging(false);
		const threshold = Math.min(.9, Math.max(.05, props.dismissThreshold ?? .25));
		if (!cancelled && (props.dismissible ?? true) && drawerShouldDismiss(dragOffset(), axisSize(), threshold)) {
			setOpen(false, "drag");
			return;
		}
		snapBack();
	};
	const coordinate = (event) => direction() === "left" || direction() === "right" ? event.clientX : event.clientY;
	const context = {
		direction,
		close: () => setOpen(false, "programmatic"),
		onPointerDown(event) {
			if (event.button !== 0 || !(props.dismissible ?? true)) return;
			snapControls?.cancel();
			startCoordinate = coordinate(event);
			setDragging(true);
			event.preventDefault();
		},
		onPointerMove(event) {
			if (!dragging() || event.buttons === 0) return;
			setDragOffset(drawerDragOffset(direction(), coordinate(event) - startCoordinate));
		},
		onPointerUp: () => finishDrag(false),
		onPointerCancel: () => finishDrag(true)
	};
	const placement = () => drawerGeometry(direction());
	const transform = (base) => [
		base[0],
		base[1],
		base[2],
		base[3],
		base[4] + (direction() === "left" || direction() === "right" ? dragOffset() : 0),
		base[5] + (direction() === "top" || direction() === "bottom" ? dragOffset() : 0)
	];
	createEffect(open, (isOpen) => {
		if (isOpen) setDragOffset(0);
	});
	onCleanup(() => snapControls?.cancel());
	return createComponent(DrawerContext, {
		value: context,
		get children() {
			return createComponent$1(Modal, mergeProps(props, {
				get open() {
					return open();
				},
				onOpenChange: (next, reason) => setOpen(next, reason),
				get motion() {
					return {
						duration: .22,
						...placement().motion
					};
				},
				get backdropStyle() {
					return {
						"background-color": rgba(102),
						...placement().backdrop,
						...props.backdropStyle
					};
				},
				contentRef: (node) => {
					measured.ref(node);
					props.contentRef?.(node);
				},
				get contentClass() {
					return mergeClasses("relative min-w-0 min-h-0 flex flex-col border-subtle bg-surface", placement().content, props.contentClass);
				},
				contentTransform: transform,
				get contentShadows() {
					return memo(() => {
						return props.contentShadows === void 0;
					})() ? componentsElevation(theme(), "modal") : props.contentShadows;
				},
				get children() {
					return memo(() => {
						return typeof props.children === "function";
					})() ? props.children({ close: context.close }) : props.children;
				}
			}));
		}
	});
}
function DrawerHandle(props) {
	const drawer = useDrawer();
	const forwarded = omit(props, "class", "onPointerDown", "onPointerMove", "onPointerUp", "onPointerCancel", "onClick");
	const hitArea = () => match(drawer.direction()).with("left", () => "absolute right-0 top-0 w-8 h-full").with("right", () => "absolute left-0 top-0 w-8 h-full").with("top", "bottom", () => "w-full h-8").exhaustive();
	const indicator = () => match(drawer.direction()).with("left", "right", () => "w-1.5 h-20").with("top", "bottom", () => "w-20 h-1.5").exhaustive();
	return createComponent$1(View, mergeProps(forwarded, {
		role: "button",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Drag or click to close drawer";
		},
		get ["class"]() {
			return mergeClasses("flex flex-none items-center justify-center", hitArea(), props.class);
		},
		onPointerDown: (event) => {
			drawer.onPointerDown(event);
			props.onPointerDown?.(event);
		},
		onPointerMove: (event) => {
			drawer.onPointerMove(event);
			props.onPointerMove?.(event);
		},
		onPointerUp: (event) => {
			drawer.onPointerUp();
			props.onPointerUp?.(event);
		},
		onPointerCancel: (event) => {
			drawer.onPointerCancel();
			props.onPointerCancel?.(event);
		},
		onClick: (event) => {
			drawer.close();
			props.onClick?.(event);
		},
		get children() {
			return createComponent$1(View, {
				"aria-hidden": "true",
				get ["class"]() {
					return mergeClasses("pointer-events-none rounded-full bg-strong", indicator());
				}
			});
		}
	}));
}
function DrawerHeader(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("flex flex-col gap-1 p-5", props.class);
	} }));
}
function DrawerFooter(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("mt-auto flex flex-col gap-2 p-5", props.class);
	} }));
}
function DrawerTitle(props) {
	return createComponent$1(Text, mergeProps(props, { get ["class"]() {
		return mergeClasses("text-lg font-semibold text-primary", props.class);
	} }));
}
function DrawerDescription(props) {
	return createComponent$1(Text, mergeProps(props, { get ["class"]() {
		return mergeClasses("whitespace-normal text-sm text-muted", props.class);
	} }));
}
function DrawerClose(props) {
	const drawer = useDrawer();
	const forwarded = omit(props, "onClick");
	return createComponent$1(Button, mergeProps(forwarded, { onClick: (event) => {
		drawer.close();
		props.onClick?.(event);
	} }));
}
//#endregion
//#region src/components/drop-zone.tsx
/** Inclusive hit test in the logical coordinate space shared with native DnD. */
function pointInLayoutRect(point, rect) {
	return point.x >= rect.x && point.y >= rect.y && point.x <= rect.x + rect.width && point.y <= rect.y + rect.height;
}
/**
* A component-local target for native filesystem drops.
*
* Native backends publish window-relative coordinates, so the zone measures
* its completed native layout before claiming an event. Events without a
* position are deliberately ignored instead of being delivered ambiguously to
* every mounted zone.
*/
function DropZone(props) {
	const host = useHost();
	const [active, setActive] = createSignal(false);
	let target;
	const contains = (position) => {
		if (!position || !target) return false;
		const rect = host.layout.measure(target);
		return rect !== null && pointInLayoutRect(position, rect);
	};
	useFileDrop((event) => {
		if (props.disabled) {
			setActive(false);
			return;
		}
		if (event.phase === "left") {
			setActive(false);
			return;
		}
		const inside = contains(event.position);
		if (event.phase !== "dropped") {
			setActive(inside);
			return;
		}
		setActive(false);
		if (!inside) return;
		const accepted = props.accept ? event.paths.filter((path) => props.accept?.(path)) : event.paths;
		const rejected = props.accept ? event.paths.filter((path) => !props.accept?.(path)) : [];
		if (accepted.length > 0) props.onDrop(accepted);
		if (rejected.length > 0) props.onRejected?.(rejected);
	});
	createEffect(() => props.disabled, (disabled) => {
		if (disabled) setActive(false);
	});
	const surface = (isActive) => createComponent$1(View, mergeProps(props, {
		ref: (node) => {
			target = node;
		},
		role: "group",
		get ["aria-label"]() {
			return props.label ?? "Drop files";
		},
		get ["aria-disabled"]() {
			return props.disabled ? "true" : void 0;
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 min-h-36 px-6 py-5 flex flex-col items-center justify-center gap-3 rounded-xl border-2 text-center", isActive ? "border-accent bg-selected" : "border-strong bg-input", props.disabled && "opacity-50", props.class);
		},
		get children() {
			return [
				createComponent$1(View, {
					get ["class"]() {
						return mergeClasses("w-10 h-10 flex-none items-center justify-center rounded-lg", isActive ? "bg-accent text-on-accent" : "bg-control text-secondary");
					},
					get children() {
						return createComponent$1(Icon, {
							source: fileUp,
							size: 20
						});
					}
				}),
				createComponent$1(Text, {
					class: "text-sm font-semibold text-primary",
					get children() {
						return memo(() => {
							return !!active();
						})() ? props.activeLabel ?? "Release to add files" : props.label ?? "Drop files here";
					}
				}),
				createComponent$1(Text, {
					class: "max-w-md text-xs text-muted whitespace-normal",
					get children() {
						return props.description ?? "Drag files from the desktop into this area.";
					}
				})
			];
		}
	}));
	return createComponent$1(Show, {
		get when() {
			return active();
		},
		get fallback() {
			return surface(false);
		},
		get children() {
			return surface(true);
		}
	});
}
//#endregion
//#region src/components/empty.tsx
function emptyClass(variant = "surface", className) {
	return mergeClasses("w-full min-w-0 flex-1 p-8 items-center justify-center gap-6 text-center", variant === "surface" ? "min-h-64 rounded-xl border border-subtle bg-surface shadow-xs" : "min-h-0 bg-transparent", className);
}
/** A composable empty-state region based on shadcn's Empty anatomy. */
function Empty(props) {
	const rest = omit(props, "variant", "class", "children");
	return createComponent$1(Column, mergeProps(rest, {
		get ["class"]() {
			return emptyClass(props.variant, props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function EmptyHeader(props) {
	return createComponent$1(Column, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("w-full max-w-sm min-w-0 items-center gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function emptyMediaClass(variant = "default", className) {
	return mergeClasses("mb-2 flex-none flex items-center justify-center", match(variant).with("default", () => "bg-transparent").with("icon", () => mergeClasses("w-10 h-10 rounded-lg", "bg-control text-primary")).exhaustive(), className);
}
function EmptyMedia(props) {
	const rest = omit(props, "variant", "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		get ["class"]() {
			return emptyMediaClass(props.variant, props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function EmptyTitle(props) {
	return createComponent$1(Text, mergeProps(props, {
		get role() {
			return props.role ?? "heading";
		},
		get ["class"]() {
			return mergeClasses("text-lg font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function EmptyDescription(props) {
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-center text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function EmptyContent(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("w-full max-w-sm min-w-0 flex flex-col items-center gap-4 text-sm", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
//#region src/components/delayed-open.ts
/** Owns cancellable open/close timers independently from a rendered surface. */
function createDelayedOpenController(options) {
	let timer;
	const cancel = () => {
		if (timer !== void 0) clearTimeout(timer);
		timer = void 0;
	};
	const schedule = (open, delay) => {
		cancel();
		if (delay <= 0) {
			options.setOpen(open);
			return;
		}
		timer = setTimeout(() => {
			timer = void 0;
			options.setOpen(open);
		}, delay);
	};
	return {
		scheduleOpen: () => schedule(true, options.openDelay()),
		scheduleClose: () => schedule(false, options.closeDelay()),
		openNow: () => {
			cancel();
			options.setOpen(true);
		},
		closeNow: () => {
			cancel();
			options.setOpen(false);
		},
		cancel,
		dispose: cancel
	};
}
//#endregion
//#region src/components/hover-card.tsx
/** A preview surface that tolerates pointer travel between trigger and card. */
function HoverCard(props) {
	const theme = useComponentsTheme();
	const state = createControllableState({
		value: () => props.open,
		defaultValue: props.defaultOpen ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onOpenChange
	});
	const open = () => !(props.disabled ?? false) && state.value();
	const delay = createDelayedOpenController({
		openDelay: () => Math.max(0, props.openDelay ?? 400),
		closeDelay: () => Math.max(0, props.closeDelay ?? 200),
		setOpen: (next) => state.set(next)
	});
	onCleanup(delay.dispose);
	return createComponent$1(Popover$1, {
		get open() {
			return open();
		},
		onOpenChange: (next) => {
			if (!next) delay.closeNow();
		},
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get placement() {
			return props.placement ?? "bottom-start";
		},
		get offset() {
			return props.offset ?? 8;
		},
		closeOnEscape: true,
		restoreFocus: false,
		get contentClass() {
			return mergeClasses("min-w-56 max-w-sm min-h-0 p-4 flex flex-col gap-3 rounded-xl border border-subtle bg-surface", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion;
		},
		get onContentPointerEnter() {
			return delay.openNow;
		},
		get onContentPointerLeave() {
			return delay.scheduleClose;
		},
		get onContentFocusIn() {
			return delay.openNow;
		},
		get onContentFocusOut() {
			return delay.scheduleClose;
		},
		trigger: (popover) => {
			const trigger = {
				ref: popover.ref,
				onPointerEnter: delay.scheduleOpen,
				onPointerLeave: delay.scheduleClose,
				onFocus: delay.openNow,
				onBlur: delay.scheduleClose,
				onKeyDown: (event) => {
					if (event.key === "Escape") delay.closeNow();
				},
				"aria-haspopup": "dialog",
				get "aria-expanded"() {
					return open();
				}
			};
			return props.trigger(trigger);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/components/icon-frame.tsx
const frameSizeClass = (size) => match(size).with("sm", () => "w-8 h-8 rounded-md").with("default", () => "w-10 h-10 rounded-lg").with("lg", () => "w-12 h-12 rounded-lg").exhaustive();
const defaultIconSize = (size) => match(size).with("sm", () => 16).with("default", () => 20).with("lg", () => 23).exhaustive();
const variantClass = (variant) => match(variant).with("plain", () => "bg-transparent").with("muted", () => "bg-control text-secondary").with("selected", () => "bg-selected text-accent").with("solid", () => "bg-accent text-on-accent").exhaustive();
/**
* A square icon surface whose geometry is correct by construction.
*
* Use this for standalone icon tiles. Buttons and menu items already own their
* icon alignment and should continue to use `Icon` directly.
*/
function IconFrame(props) {
	const rest = omit(props, "source", "fill", "label", "size", "iconSize", "variant", "iconClass", "class");
	const size = () => props.size ?? "default";
	return createComponent$1(Center, mergeProps(rest, {
		role: "presentation",
		get ["class"]() {
			return mergeClasses("flex-none", frameSizeClass(size()), variantClass(props.variant ?? "plain"), props.class);
		},
		get children() {
			return createComponent$1(Icon, {
				get source() {
					return props.source;
				},
				get size() {
					return props.iconSize ?? defaultIconSize(size());
				},
				get fill() {
					return props.fill;
				},
				get label() {
					return props.label;
				},
				get ["class"]() {
					return props.iconClass;
				}
			});
		}
	}));
}
//#endregion
//#region src/components/image-list.tsx
function finitePositive(value, name) {
	if (!Number.isFinite(value) || value <= 0) throw new RangeError(`${name} must be finite and positive`);
	return value;
}
/**
* A virtualized, selectable image list for page strips, albums and file pickers.
* Resource creation remains owned by the caller; this component only borrows handles.
*/
function ImageList(props) {
	const config = untrack(() => ({
		items: props.items,
		getItemKey: props.getItemKey,
		getResource: props.getResource,
		getLabel: props.getLabel,
		getDescription: props.getDescription,
		viewportHeight: props.viewportHeight,
		overscan: props.overscan,
		onSelectionChange: props.onSelectionChange,
		onResourceReady: props.onResourceReady,
		onResourceError: props.onResourceError,
		accessibilityLabel: props.accessibilityLabel,
		class: props.class,
		renderTrailing: props.renderTrailing,
		renderThumbnail: props.renderThumbnail
	}));
	const { itemHeight, thumbnailWidth, thumbnailHeight } = untrack(() => ({
		itemHeight: finitePositive(props.itemHeight ?? 88, "itemHeight"),
		thumbnailWidth: finitePositive(props.thumbnailWidth ?? 56, "thumbnailWidth"),
		thumbnailHeight: finitePositive(props.thumbnailHeight ?? 72, "thumbnailHeight")
	}));
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("min-w-0 min-h-0 overflow-hidden", config.class);
		},
		get style() {
			return memo(() => {
				return config.viewportHeight === void 0;
			})() ? void 0 : { height: `${config.viewportHeight}px` };
		},
		get children() {
			return createComponent$1(VirtualList, {
				get items() {
					return config.items;
				},
				itemHeight,
				get viewportHeight() {
					return config.viewportHeight;
				},
				get overscan() {
					return config.overscan;
				},
				get getItemKey() {
					return config.getItemKey;
				},
				role: "listbox",
				get accessibilityLabel() {
					return config.accessibilityLabel ?? "Images";
				},
				class: "w-full h-full min-w-0 min-h-0",
				children: (item, index) => {
					const [failed, setFailed] = createSignal(false);
					const key = createMemo(() => config.getItemKey(item(), index()));
					const label = createMemo(() => config.getLabel(item(), index()));
					const resource = createMemo(() => config.getResource?.(item(), index()));
					const description = createMemo(() => config.getDescription?.(item(), index()));
					const selected = createMemo(() => props.selectedKey === key());
					return createComponent$1(Button$1, {
						unstyled: true,
						role: "option",
						get ["aria-label"]() {
							return label();
						},
						get ["aria-selected"]() {
							return selected();
						},
						get selected() {
							return selected();
						},
						class: (state) => mergeClasses("w-full h-full min-w-0 flex flex-row items-center gap-3 px-3 py-2 text-left", match({
							selected: selected(),
							hovered: state.hovered
						}).with({ selected: true }, () => "bg-selected text-primary").with({ hovered: true }, () => "bg-control text-primary").with(P._, () => "bg-transparent text-primary").exhaustive()),
						onClick: () => config.onSelectionChange?.(item(), index()),
						get children() {
							return [
								createComponent$1(View, {
									"aria-hidden": "true",
									class: "flex-none flex items-center justify-center overflow-hidden bg-control",
									style: {
										width: `${thumbnailWidth}px`,
										height: `${thumbnailHeight}px`
									},
									get children() {
										return config.renderThumbnail?.(item(), index()) ?? createComponent$1(Show, {
											get when() {
												return !failed();
											},
											get children() {
												return createComponent$1(Image, {
													get resource() {
														return resource();
													},
													get ["aria-label"]() {
														return label();
													},
													class: "w-full h-full",
													onResourceReady: (event) => config.onResourceReady?.(item(), index(), event),
													onResourceError: (event) => {
														setFailed(true);
														config.onResourceError?.(item(), index(), event);
													}
												});
											}
										});
									}
								}),
								createComponent$1(View, {
									class: "flex-1 min-w-0 flex flex-col gap-1",
									get children() {
										return [createComponent$1(Text, {
											maxLines: 1,
											class: "w-full min-w-0 font-medium",
											get children() {
												return label();
											}
										}), createComponent$1(Show, {
											get when() {
												return description();
											},
											children: (description) => createComponent$1(Text, {
												maxLines: 2,
												class: "w-full min-w-0 text-sm text-muted",
												get children() {
													return description();
												}
											})
										})];
									}
								}),
								memo(() => {
									return config.renderTrailing?.(item(), index());
								})
							];
						}
					});
				}
			});
		}
	});
}
//#endregion
//#region src/components/image-viewport.tsx
function positiveSize(size, name) {
	if (!Number.isFinite(size.width) || !Number.isFinite(size.height) || size.width <= 0 || size.height <= 0) throw new RangeError(`${name} width and height must be finite and positive`);
}
/** Deterministic contain + zoom + pan transform shared by paint and annotations. */
function imageViewportTransform(options) {
	positiveSize(options.viewport, "viewport");
	positiveSize(options.image, "image");
	const zoom = options.zoom ?? 1;
	if (!Number.isFinite(zoom) || zoom <= 0) throw new RangeError("image viewport zoom must be finite and positive");
	const pan = options.pan ?? {
		x: 0,
		y: 0
	};
	if (!Number.isFinite(pan.x) || !Number.isFinite(pan.y)) throw new RangeError("image viewport pan must be finite");
	const scale = Math.min(options.viewport.width / options.image.width, options.viewport.height / options.image.height) * zoom;
	const width = options.image.width * scale;
	const height = options.image.height * scale;
	const frame = {
		x: (options.viewport.width - width) / 2 + pan.x,
		y: (options.viewport.height - height) / 2 + pan.y,
		width,
		height
	};
	return {
		viewport: options.viewport,
		image: options.image,
		frame,
		scale,
		imageToViewport: (point) => ({
			x: frame.x + point.x * scale,
			y: frame.y + point.y * scale
		}),
		viewportToImage: (point) => ({
			x: (point.x - frame.x) / scale,
			y: (point.y - frame.y) / scale
		})
	};
}
const ImageViewportContext = createContext();
/** A clipped image-space viewport with one explicit, reusable coordinate model. */
function ImageViewport(props) {
	const measured = createMeasuredSize();
	const [intrinsicSize, setIntrinsicSize] = createSignal();
	const [localPan, setLocalPan] = createSignal({
		x: 0,
		y: 0
	});
	const [panDrag, setPanDrag] = createSignal();
	const pan = () => props.pan ?? localPan();
	const updatePan = (next) => {
		if (props.pan === void 0) setLocalPan(next);
		props.onPanChange?.(next);
	};
	const transform = createMemo(() => {
		const image = props.imageSize ?? intrinsicSize();
		if (!measured.measured() || measured.width() <= 0 || measured.height() <= 0) return null;
		if (!image) return null;
		return imageViewportTransform({
			viewport: {
				width: measured.width(),
				height: measured.height()
			},
			image,
			zoom: props.zoom,
			pan: pan()
		});
	});
	const frameStyle = () => {
		const frame = transform()?.frame;
		return frame ? {
			left: `${frame.x}px`,
			top: `${frame.y}px`,
			width: `${frame.width}px`,
			height: `${frame.height}px`
		} : {
			width: "0px",
			height: "0px"
		};
	};
	const rest = omit(props, "resource", "imageSize", "zoom", "pan", "pannable", "onPanChange", "media", "children", "imageLabel", "onResourceReady", "onResourceError");
	return createComponent$1(ImageViewportContext, {
		value: { transform },
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				ref(r$) {
					var _ref$ = measured.ref;
					typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
				},
				get role() {
					return props.role ?? "group";
				},
				get ["class"]() {
					return mergeClasses("relative min-w-0 min-h-0 overflow-hidden bg-control", props.class);
				},
				onPointerDown: (event) => {
					if (!props.pannable || event.button !== 0) return;
					event.preventDefault();
					setPanDrag({
						pointer: {
							x: event.clientX,
							y: event.clientY
						},
						pan: pan()
					});
				},
				onPointerMove: (event) => {
					const drag = panDrag();
					if (!drag) return;
					updatePan({
						x: drag.pan.x + event.clientX - drag.pointer.x,
						y: drag.pan.y + event.clientY - drag.pointer.y
					});
				},
				onPointerUp: () => setPanDrag(),
				onPointerCancel: () => setPanDrag(),
				get children() {
					return [createComponent$1(View, {
						"aria-hidden": "true",
						class: "absolute overflow-hidden bg-surface",
						get style() {
							return frameStyle();
						},
						get children() {
							return props.media ?? createComponent$1(Image, {
								get resource() {
									return props.resource;
								},
								get ["aria-label"]() {
									return props.imageLabel ?? "Image";
								},
								class: "w-full h-full",
								onResourceReady: (event) => {
									setIntrinsicSize({
										width: event.width,
										height: event.height
									});
									props.onResourceReady?.(event);
								},
								get onResourceError() {
									return props.onResourceError;
								}
							});
						}
					}), memo(() => {
						return props.children;
					})];
				}
			}));
		}
	});
}
function clampAnnotationRegion(region, image, minimumSize = 1) {
	positiveSize(image, "image");
	const min = Math.max(0, minimumSize);
	const x0 = Math.max(0, Math.min(image.width, region.x));
	const y0 = Math.max(0, Math.min(image.height, region.y));
	const x1 = Math.max(x0, Math.min(image.width, region.x + region.width));
	const y1 = Math.max(y0, Math.min(image.height, region.y + region.height));
	return {
		...region,
		x: Math.min(x0, Math.max(0, image.width - min)),
		y: Math.min(y0, Math.max(0, image.height - min)),
		width: Math.min(image.width - x0, Math.max(min, x1 - x0)),
		height: Math.min(image.height - y0, Math.max(min, y1 - y0))
	};
}
function draftRegion(id, start, end) {
	return {
		id,
		x: Math.min(start.x, end.x),
		y: Math.min(start.y, end.y),
		width: Math.abs(end.x - start.x),
		height: Math.abs(end.y - start.y)
	};
}
/** Editable image-space regions composed above an ImageViewport. */
function AnnotationLayer(props) {
	const viewport = useContext(ImageViewportContext);
	if (!viewport) throw new Error("AnnotationLayer must be placed inside ImageViewport");
	const [localSelected, setLocalSelected] = createSignal(null);
	const [interaction, setInteraction] = createSignal();
	let nextId = 1;
	const selected = () => props.selectedId === void 0 ? localSelected() : props.selectedId;
	const choose = (id) => {
		if (props.selectedId === void 0) setLocalSelected(id);
		props.onSelectedIdChange?.(id);
	};
	const point = (event) => {
		return viewport.transform()?.viewportToImage({
			x: event.offsetX,
			y: event.offsetY
		});
	};
	const displayedRegion = (region) => {
		const active = interaction();
		return active && active.kind !== "create" && active.id === region.id ? active.current : region;
	};
	const styleFor = (region) => {
		const transform = viewport.transform();
		if (!transform) return {
			width: "0px",
			height: "0px"
		};
		const origin = transform.imageToViewport(region);
		return {
			left: `${origin.x}px`,
			top: `${origin.y}px`,
			width: `${region.width * transform.scale}px`,
			height: `${region.height * transform.scale}px`
		};
	};
	const updateDrag = (event) => {
		const active = interaction();
		const transform = viewport.transform();
		if (!active || !transform) return;
		if (active.kind === "create") {
			const cursor = point(event);
			if (!cursor) return;
			setInteraction({
				...active,
				current: cursor
			});
			return;
		}
		const dx = (event.clientX - active.startPointer.x) / transform.scale;
		const dy = (event.clientY - active.startPointer.y) / transform.scale;
		const current = active.kind === "move" ? {
			...active.startRegion,
			x: Math.max(0, Math.min(transform.image.width - active.startRegion.width, active.startRegion.x + dx)),
			y: Math.max(0, Math.min(transform.image.height - active.startRegion.height, active.startRegion.y + dy))
		} : {
			...active.startRegion,
			width: active.startRegion.width + dx,
			height: active.startRegion.height + dy
		};
		setInteraction({
			...active,
			current: active.kind === "move" ? current : clampAnnotationRegion(current, transform.image, props.minimumSize ?? 8)
		});
	};
	const finish = () => {
		const active = interaction();
		const transform = viewport.transform();
		if (!active || !transform) return;
		if (active.kind === "create") {
			const created = clampAnnotationRegion(draftRegion(props.createRegionId?.() ?? `annotation-${nextId++}`, active.start, active.current), transform.image, props.minimumSize ?? 8);
			choose(created.id);
			props.onRegionsChange?.([...props.regions, created]);
		} else props.onRegionsChange?.(props.regions.map((region) => region.id === active.id ? active.current : region));
		setInteraction();
	};
	const beginRegion = (kind, region, event) => {
		if (event.button !== 0) return;
		event.preventDefault();
		event.stopPropagation();
		choose(region.id);
		setInteraction({
			kind,
			id: region.id,
			startPointer: {
				x: event.clientX,
				y: event.clientY
			},
			startRegion: region,
			current: region
		});
	};
	const draft = () => {
		const active = interaction();
		return active?.kind === "create" ? draftRegion("annotation-draft", active.start, active.current) : void 0;
	};
	const rest = omit(props, "regions", "selectedId", "minimumSize", "createRegionId", "interactionMode", "onRegionsChange", "onSelectedIdChange");
	return createComponent$1(View, mergeProps(rest, {
		get role() {
			return props.role ?? "group";
		},
		get ["class"]() {
			return mergeClasses("absolute inset-0", props.class);
		},
		get classList() {
			return { "pointer-events-none": props.interactionMode === "passthrough" };
		},
		onPointerDown: (event) => {
			if (event.button !== 0 || !viewport.transform()) return;
			const start = point(event);
			if (!start) return;
			event.preventDefault();
			choose(null);
			setInteraction({
				kind: "create",
				start,
				current: start
			});
		},
		onPointerMove: updateDrag,
		onPointerUp: finish,
		onPointerCancel: () => setInteraction(),
		get children() {
			return [createComponent$1(For, {
				get each() {
					return props.regions;
				},
				children: (region) => createComponent$1(View, {
					role: "button",
					get ["aria-label"]() {
						return region.label ?? `Annotation ${region.id}`;
					},
					get ["aria-pressed"]() {
						return selected() === region.id;
					},
					focusOrder: 0,
					get ["class"]() {
						return mergeClasses("absolute border-2 bg-transparent cursor-move", selected() === region.id ? "border-accent" : "border-strong");
					},
					get style() {
						return styleFor(displayedRegion(region));
					},
					onPointerDown: (event) => beginRegion("move", region, event),
					onPointerMove: updateDrag,
					onPointerUp: finish,
					onPointerCancel: () => setInteraction(),
					get children() {
						return createComponent$1(View, {
							role: "button",
							get ["aria-label"]() {
								return `Resize ${region.label ?? region.id}`;
							},
							class: "absolute w-7 h-7 flex items-center justify-center bg-transparent cursor-pointer",
							style: {
								right: "-8px",
								bottom: "-8px"
							},
							onPointerDown: (event) => beginRegion("resize", region, event),
							onPointerMove: updateDrag,
							onPointerUp: finish,
							onPointerCancel: () => setInteraction(),
							get children() {
								return createComponent$1(View, {
									"aria-hidden": "true",
									class: "w-3 h-3 rounded-sm border border-on-accent bg-accent pointer-events-none"
								});
							}
						});
					}
				})
			}), memo(() => {
				return memo(() => {
					return !!draft();
				})() ? createComponent$1(View, {
					"aria-hidden": "true",
					class: "absolute border-2 border-accent bg-transparent pointer-events-none",
					get style() {
						return styleFor(draft());
					}
				}) : draft();
			})];
		}
	}));
}
/** Read-only image-space overlays sharing the viewport's exact zoom and pan transform. */
function ImageOverlayLayer(props) {
	const viewport = useContext(ImageViewportContext);
	if (!viewport) throw new Error("ImageOverlayLayer must be placed inside ImageViewport");
	const styleFor = (item) => {
		const transform = viewport.transform();
		if (!transform) return {
			width: "0px",
			height: "0px"
		};
		const origin = transform.imageToViewport(item);
		return {
			left: `${origin.x}px`,
			top: `${origin.y}px`,
			width: `${item.width * transform.scale}px`,
			height: `${item.height * transform.scale}px`
		};
	};
	const rest = omit(props, "items", "children");
	return createComponent$1(View, mergeProps(rest, {
		get ["class"]() {
			return mergeClasses("absolute inset-0 pointer-events-none", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return props.items;
				},
				children: (item, index) => createComponent$1(View, {
					class: "absolute overflow-hidden",
					get style() {
						return styleFor(item);
					},
					get children() {
						return props.children(item, index);
					}
				})
			});
		}
	}));
}
//#endregion
//#region src/components/inline-edit.tsx
/** Compact rename interaction with explicit Enter, Escape, and blur behavior. */
function InlineEdit(props) {
	const [internal, setInternal] = createSignal(props.defaultValue ?? "");
	const value = () => props.value ?? internal();
	const [editing, setEditing] = createSignal(false);
	const [draft, setDraft] = createSignal(untrack(value));
	const begin = () => {
		if (props.disabled) return;
		setDraft(untrack(value));
		setEditing(true);
	};
	const commit = () => {
		if (!untrack(editing)) return;
		const next = untrack(draft);
		const previous = untrack(value);
		setEditing(false);
		if (props.value === void 0) setInternal(next);
		if (next !== previous) props.onValueChange?.(next);
		props.onCommit?.(next);
	};
	const cancel = () => {
		if (!untrack(editing)) return;
		setDraft(untrack(value));
		setEditing(false);
		props.onCancel?.();
	};
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("min-w-0", props.class);
		},
		get children() {
			return memo(() => {
				return !!editing();
			})() ? createComponent$1(Input, {
				ref: (node) => {
					node.focus();
				},
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["class"]() {
					return mergeClasses("min-w-0", props.inputClass);
				},
				get value() {
					return draft();
				},
				get placeholder() {
					return props.placeholder;
				},
				onInput: (event) => setDraft(event.currentTarget.value),
				onBlur: commit,
				onKeyDown: (event) => {
					if (event.key === "Enter") {
						event.preventDefault();
						commit();
					} else if (event.key === "Escape") {
						event.preventDefault();
						cancel();
					}
				}
			}) : createComponent$1(Button, {
				variant: "ghost",
				get disabled() {
					return props.disabled;
				},
				get ["aria-label"]() {
					return `Edit ${props["aria-label"]}`;
				},
				get ["class"]() {
					return mergeClasses("h-8 max-w-full min-w-0 px-2 justify-start gap-2", props.displayClass);
				},
				onClick: begin,
				get children() {
					return [createComponent$1(Text, {
						class: "min-w-0 flex-1 text-sm text-primary",
						maxLines: 1,
						get children() {
							return value() || props.placeholder || "Untitled";
						}
					}), createComponent$1(Icon, {
						"aria-hidden": "true",
						class: "flex-none text-muted",
						source: pencil,
						size: 14
					})];
				}
			});
		}
	});
}
//#endregion
//#region src/components/input-otp.tsx
function normalizeOtpValue(value, maxLength, allowed = /^[0-9]$/) {
	if (!Number.isInteger(maxLength) || maxLength <= 0) throw new RangeError("InputOTP maxLength must be a positive integer");
	return Array.from(value).filter((character) => {
		allowed.lastIndex = 0;
		return allowed.test(character);
	}).slice(0, maxLength).join("");
}
const InputOtpContext = createContext();
function requireInputOtp() {
	const context = useContext(InputOtpContext);
	if (!context) throw new Error("InputOTPSlot requires an InputOTP root");
	return context;
}
function InputOTP(props) {
	const initial = normalizeOtpValue(props.defaultValue ?? "", props.maxLength, props.allowed);
	const [internalValue, setInternalValue] = createSignal(initial);
	const [focused, setFocused] = createSignal(false);
	const value = () => normalizeOtpValue(props.value ?? internalValue(), props.maxLength, props.allowed);
	return createComponent$1(InputOtpContext, {
		value: {
			value,
			maxLength: () => props.maxLength,
			focused
		},
		get children() {
			return createComponent$1(View, {
				role: "group",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["class"]() {
					return mergeClasses("relative inline-flex flex-none items-center gap-2", props.disabled && "opacity-50", props.class);
				},
				get children() {
					return [memo(() => {
						return props.children;
					}), createComponent$1(TextInput, {
						ref(r$) {
							var _ref$ = props.ref;
							typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : props.ref = r$;
						},
						get ["aria-label"]() {
							return props["aria-label"];
						},
						get value() {
							return value();
						},
						get disabled() {
							return props.disabled;
						},
						get readOnly() {
							return props.readOnly;
						},
						get ["class"]() {
							return mergeClasses("absolute inset-0 w-full h-full z-10 opacity-0", props.inputClass);
						},
						onFocus: () => setFocused(true),
						onBlur: () => setFocused(false),
						onInput: (event) => {
							const previous = value();
							const next = normalizeOtpValue(event.currentTarget.value, props.maxLength, props.allowed);
							setInternalValue(next);
							props.onValueChange?.(next);
							if (next.length === props.maxLength && previous.length !== props.maxLength) props.onComplete?.(next);
						}
					})];
				}
			});
		}
	});
}
function InputOTPGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("flex flex-none items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function InputOTPSlot(props) {
	const context = requireInputOtp();
	const forwarded = omit(props, "index", "class");
	const character = () => context.value()[props.index];
	const active = () => context.focused() && props.index === Math.min(context.value().length, context.maxLength() - 1);
	return createComponent$1(View, mergeProps(forwarded, {
		get ["class"]() {
			return mergeClasses("relative w-9 h-9 flex-none flex items-center justify-center rounded-lg border bg-input text-sm text-primary shadow-xs", active() ? "border-focus" : "border-subtle", props.class);
		},
		get children() {
			return [createComponent$1(Show, {
				get when() {
					return character();
				},
				keyed: true,
				children: (value) => createComponent$1(Text, {
					class: "text-sm text-primary",
					children: value
				})
			}), createComponent$1(Show, {
				get when() {
					return memo(() => {
						return !!active();
					})() ? !character() : active();
				},
				get children() {
					return createComponent$1(View, {
						"aria-hidden": "true",
						class: "absolute w-px h-4 bg-primary pointer-events-none"
					});
				}
			})];
		}
	}));
}
function InputOTPSeparator(props) {
	return createComponent$1(View, mergeProps(props, {
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("w-5 h-9 flex-none flex items-center justify-center", props.class);
		},
		get children() {
			return createComponent$1(Icon, {
				source: minus,
				size: 14,
				class: "text-muted"
			});
		}
	}));
}
//#endregion
//#region src/components/item.tsx
function itemClass(variant = "default", size = "default", className) {
	const colors = match(variant).with("default", () => "border-transparent bg-transparent").with("outline", () => "border-subtle bg-transparent").with("muted", () => "border-transparent bg-control").exhaustive();
	const spacing = match(size).with("default", () => "gap-4 p-4").with("sm", () => "gap-2 px-4 py-3").exhaustive();
	return mergeClasses("w-full min-w-0 flex flex-row flex-wrap items-center rounded-lg border text-sm", colors, spacing, className);
}
/** A composable list row based on shadcn's Item anatomy. */
function Item(props) {
	const rest = omit(props, "variant", "size", "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		get role() {
			return props.role ?? "none";
		},
		get ["class"]() {
			return itemClass(props.variant, props.size, props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "group";
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex flex-col", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemSeparator(props) {
	return createComponent$1(View, {
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("w-full h-px flex-none bg-subtle", props.class);
		}
	});
}
function itemMediaClass(variant = "default", className) {
	return mergeClasses("flex-none flex items-center justify-center gap-2", match(variant).with("default", () => "bg-transparent").with("icon", () => "w-8 h-8 rounded-sm border border-subtle bg-control").with("image", () => "w-10 h-10 overflow-hidden rounded-sm").exhaustive(), className);
}
function ItemMedia(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return itemMediaClass(props.variant, props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function ItemContent(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("flex-1 min-w-0 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemTitle(props) {
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-w-0 text-sm font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemDescription(props) {
	return createComponent$1(Text, mergeProps(props, {
		get maxLines() {
			return props.maxLines ?? 2;
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemActions(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("flex-none flex items-center gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function ItemHeader(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex items-center justify-between gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
const ItemFooter = ItemHeader;
//#endregion
//#region src/components/layout.tsx
const ResponsiveGridContext = createContext();
/** Read the completed native size and active column count of the nearest grid. */
function useResponsiveGrid() {
	const context = useContext(ResponsiveGridContext);
	if (!context) throw new Error("useResponsiveGrid must be used inside ResponsiveGrid");
	return context;
}
const responsiveGridColumnClass = (columns) => match(columns).with(1, () => "grid-cols-1").with(2, () => "grid-cols-2").with(3, () => "grid-cols-3").with(4, () => "grid-cols-4").exhaustive();
function responsiveGridColumnCount(options) {
	const maxColumns = options.maxColumns ?? 4;
	if (!Number.isFinite(options.width) || options.width <= 0) return Math.min(options.initialColumns ?? 1, maxColumns);
	const gap = Math.max(0, options.gap ?? 16);
	const minColumnWidth = Math.max(1, options.minColumnWidth);
	return Math.min(maxColumns, Math.max(1, Math.floor((options.width + gap) / (minColumnWidth + gap))));
}
function responsiveGridRemainderCount(itemCount, columns) {
	const remainder = Math.max(0, Math.floor(itemCount)) % columns;
	return remainder === 0 ? 0 : columns - remainder;
}
/**
* A grid that responds to its own native content box instead of the window.
*
* This is important inside sidebars, split panes and dialogs: window media
* queries do not know how much width the component actually receives.
*/
function ResponsiveGrid(props) {
	const measured = createMeasuredSize();
	const columns = createMemo(() => responsiveGridColumnCount({
		width: measured.width(),
		minColumnWidth: props.minColumnWidth,
		gap: props.gap,
		maxColumns: props.maxColumns,
		initialColumns: props.initialColumns
	}));
	const rest = omit(props, "children", "minColumnWidth", "gap", "maxColumns", "initialColumns", "class", "ref");
	const state = {
		columns,
		width: measured.width,
		height: measured.height
	};
	return createComponent$1(ResponsiveGridContext, {
		value: state,
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				ref: (node) => {
					measured.ref(node);
					props.ref?.(node);
				},
				get style() {
					return {
						gap: props.gap ?? 16,
						...props.style
					};
				},
				get ["class"]() {
					return mergeClasses("w-full min-w-0 grid", responsiveGridColumnClass(columns()), props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
/** Fill the unused cells in the final row using the grid's measured columns. */
function ResponsiveGridRemainder(props) {
	const context = useResponsiveGrid();
	const cells = createMemo(() => Array.from({ length: responsiveGridRemainderCount(props.itemCount, context.columns()) }));
	return createComponent$1(For, {
		get each() {
			return cells();
		},
		children: () => createComponent$1(View, {
			"aria-hidden": true,
			get ["class"]() {
				return mergeClasses("min-w-0", props.class);
			}
		})
	});
}
/**
* A horizontal primary/aside boundary with explicit flex shrink semantics.
* Use `SplitPaneMain` for the elastic region and `SplitPaneAside` for a
* class-sized fixed rail. Both regions clip at their own boundary, so content
* cannot paint across the divider or a rounded parent clip.
*/
function SplitPane(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex flex-row overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function SplitPaneMain(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("flex-1 min-w-0 overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function SplitPaneAside(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("flex-none min-w-0 overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
const AdaptiveSplitPaneContext = createContext();
/**
* Master/detail layout whose detail region can move from an inline rail to a
* modal surface without changing the application's selection model.
*/
function AdaptiveSplitPane(props) {
	return createComponent$1(AdaptiveSplitPaneContext, {
		value: { compact: () => props.compact },
		get children() {
			return createComponent$1(SplitPane, {
				get ["class"]() {
					return props.class;
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function AdaptiveSplitPaneMain(props) {
	return createComponent$1(SplitPaneMain, {
		get ["class"]() {
			return props.class;
		},
		get children() {
			return props.children;
		}
	});
}
function AdaptiveSplitPaneDetail(props) {
	const context = useContext(AdaptiveSplitPaneContext);
	return createComponent$1(Show, {
		get when() {
			return context.compact();
		},
		get fallback() {
			return createComponent$1(SplitPaneAside, {
				get ["class"]() {
					return props.class;
				},
				get children() {
					return props.children;
				}
			});
		},
		get children() {
			return createComponent$1(Dialog, {
				get open() {
					return props.open;
				},
				onOpenChange: (open) => props.onOpenChange(open),
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get contentClass() {
					return mergeClasses("h-11/12 p-0 overflow-hidden", props.modalClass);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/components/markdown-model.ts
const LINK_REFERENCE_DEFINITION = /^ {0,3}\[[^\]\n]+\]:/m;
function sameRuns(left, right) {
	return left.length === right.length && left.every((run, index) => run.text === right[index]?.text && sameStyle(run.style, right[index]?.style ?? {}));
}
function reconcileRuns(previous, next) {
	const reconciled = [];
	let previousIndex = 0;
	let canReuse = true;
	for (const run of next) {
		let remaining = run.text;
		while (canReuse && remaining.length > 0) {
			const prior = previous[previousIndex];
			if (!prior || !sameStyle(prior.style, run.style) || !remaining.startsWith(prior.text)) {
				canReuse = false;
				break;
			}
			reconciled.push(prior);
			remaining = remaining.slice(prior.text.length);
			previousIndex += 1;
		}
		if (remaining) reconciled.push({
			text: remaining,
			style: run.style
		});
	}
	return previousIndex === previous.length && sameRuns(previous, reconciled) ? previous : reconciled;
}
function reconcileBlock(previous, next) {
	if (!previous || previous.kind !== next.kind) return next;
	if (sameBlock(previous, next)) return previous;
	switch (next.kind) {
		case "heading": {
			if (previous.kind !== "heading") return next;
			const runs = reconcileRuns(previous.runs, next.runs);
			return runs === previous.runs && previous.depth === next.depth ? previous : {
				...next,
				runs
			};
		}
		case "paragraph": {
			if (previous.kind !== "paragraph") return next;
			const runs = reconcileRuns(previous.runs, next.runs);
			return runs === previous.runs ? previous : {
				...next,
				runs
			};
		}
		case "blockquote": return {
			...next,
			blocks: previous.kind === "blockquote" ? reconcileMarkdownBlocks(previous.blocks, next.blocks) : next.blocks
		};
		case "list": return {
			...next,
			items: next.items.map((item, index) => ({
				...item,
				blocks: previous.kind === "list" ? reconcileMarkdownBlocks(previous.items[index]?.blocks ?? [], item.blocks) : item.blocks
			}))
		};
		case "table": return {
			...next,
			header: next.header.map((runs, index) => previous.kind === "table" ? reconcileRuns(previous.header[index] ?? [], runs) : runs),
			rows: next.rows.map((row, rowIndex) => row.map((runs, columnIndex) => previous.kind === "table" ? reconcileRuns(previous.rows[rowIndex]?.[columnIndex] ?? [], runs) : runs))
		};
		default: return next;
	}
}
function sameBlock(left, right) {
	if (left.kind !== right.kind) return false;
	switch (left.kind) {
		case "heading": return right.kind === "heading" && left.depth === right.depth && sameRuns(left.runs, right.runs);
		case "paragraph": return right.kind === "paragraph" && sameRuns(left.runs, right.runs);
		case "blockquote": return right.kind === "blockquote" && sameBlocks(left.blocks, right.blocks);
		case "list": return right.kind === "list" && left.ordered === right.ordered && left.start === right.start && left.items.length === right.items.length && left.items.every((item, index) => item.checked === right.items[index]?.checked && sameBlocks(item.blocks, right.items[index]?.blocks ?? []));
		case "table": return right.kind === "table" && left.align.length === right.align.length && left.align.every((align, index) => align === right.align[index]) && sameRunRows(left.header, right.header) && left.rows.length === right.rows.length && left.rows.every((row, index) => sameRunRows(row, right.rows[index] ?? []));
		case "code": return right.kind === "code" && left.code === right.code && left.language === right.language;
		case "rule": return right.kind === "rule";
		case "literal": return right.kind === "literal" && left.text === right.text;
	}
}
function sameRunRows(left, right) {
	return left.length === right.length && left.every((runs, index) => sameRuns(runs, right[index] ?? []));
}
function sameBlocks(left, right) {
	return left.length === right.length && left.every((block, index) => sameBlock(block, right[index]));
}
/**
* Preserve unchanged block identities across streaming parses. Solid can then
* retain the already-rendered native subtree while only replacing the block
* whose Markdown source is still growing.
*/
function reconcileMarkdownBlocks(previous, next) {
	let changed = previous.length !== next.length;
	const reconciled = next.map((block, index) => {
		const result = reconcileBlock(previous[index], block);
		if (result !== previous[index]) changed = true;
		return result;
	});
	return changed ? reconciled : previous;
}
function sameStyle(left, right) {
	return left.strong === right.strong && left.emphasis === right.emphasis && left.code === right.code && left.deleted === right.deleted && left.href === right.href;
}
function appendRun(runs, text, style) {
	if (!text) return;
	const previous = runs.at(-1);
	if (previous && sameStyle(previous.style, style)) previous.text += text;
	else runs.push({
		text,
		style
	});
}
function inlineRuns(tokens, inherited = {}) {
	const runs = [];
	for (const token of tokens) {
		const nested = (style) => {
			if ("tokens" in token && Array.isArray(token.tokens)) for (const run of inlineRuns(token.tokens, style)) appendRun(runs, run.text, run.style);
			else if ("text" in token && typeof token.text === "string") appendRun(runs, token.text, style);
		};
		switch (token.type) {
			case "strong":
				nested({
					...inherited,
					strong: true
				});
				break;
			case "em":
				nested({
					...inherited,
					emphasis: true
				});
				break;
			case "codespan":
				appendRun(runs, token.text, {
					...inherited,
					code: true
				});
				break;
			case "del":
				nested({
					...inherited,
					deleted: true
				});
				break;
			case "link":
				nested({
					...inherited,
					href: token.href
				});
				break;
			case "image":
				appendRun(runs, token.text || token.href, inherited);
				break;
			case "br":
				appendRun(runs, "\n", inherited);
				break;
			case "checkbox": break;
			default: nested(inherited);
		}
	}
	return runs;
}
function blocks(tokens) {
	const result = [];
	for (const token of tokens) switch (token.type) {
		case "space": break;
		case "heading": {
			const heading = token;
			result.push({
				kind: "heading",
				depth: heading.depth,
				runs: inlineRuns(heading.tokens)
			});
			break;
		}
		case "paragraph":
		case "text": {
			const text = token;
			result.push({
				kind: "paragraph",
				runs: inlineRuns(text.tokens ?? [text])
			});
			break;
		}
		case "blockquote":
			result.push({
				kind: "blockquote",
				blocks: blocks(token.tokens)
			});
			break;
		case "list": {
			const list = token;
			result.push({
				kind: "list",
				ordered: list.ordered,
				start: typeof list.start === "number" ? list.start : 1,
				items: list.items.map((item) => ({
					checked: typeof item.checked === "boolean" ? item.checked : void 0,
					blocks: blocks(item.tokens)
				}))
			});
			break;
		}
		case "table": {
			const table = token;
			result.push({
				kind: "table",
				align: table.align,
				header: table.header.map((cell) => inlineRuns(cell.tokens)),
				rows: table.rows.map((row) => row.map((cell) => inlineRuns(cell.tokens)))
			});
			break;
		}
		case "code":
			result.push({
				kind: "code",
				code: token.text,
				language: token.lang || void 0
			});
			break;
		case "hr":
			result.push({ kind: "rule" });
			break;
		case "html":
			result.push({
				kind: "literal",
				text: token.text
			});
			break;
		default: if ("tokens" in token && Array.isArray(token.tokens)) result.push(...blocks(token.tokens));
	}
	return result;
}
function parseWithOffsets(source) {
	const parsedBlocks = [];
	const parsedTokens = [];
	let offset = 0;
	for (const token of lexer(source, { gfm: true })) {
		const start = offset;
		offset += token.raw.length;
		const tokenBlocks = blocks([token]);
		parsedBlocks.push(...tokenBlocks);
		parsedTokens.push({
			start,
			end: offset,
			blockCount: tokenBlocks.length
		});
	}
	return {
		blocks: parsedBlocks,
		tokens: parsedTokens
	};
}
/**
* Stateful Markdown parse adapter for append-heavy streams.
*
* Like Waku's MarkdownView, it treats the last two top-level source tokens as
* volatile. Appends retain everything before that boundary and only lex the
* tail. Parser-specific tokens never escape this module.
*/
var MarkdownDocument = class {
	#source = "";
	#streaming = false;
	#blocks = [];
	#tokens = [];
	#lastParse = {
		incremental: false,
		parsedBytes: 0,
		sourceBytes: 0
	};
	get blocks() {
		return this.#blocks;
	}
	/** Diagnostics for benchmarks and streaming regression tests. */
	get lastParse() {
		return this.#lastParse;
	}
	setSource(source, streaming = false) {
		if (source === this.#source && streaming === this.#streaming) return this.#blocks;
		const displaySource = streaming ? remend(source) : source;
		const canAppend = streaming && this.#streaming && source.startsWith(this.#source) && !LINK_REFERENCE_DEFINITION.test(source) && this.#tokens.length > 2;
		let next;
		let parsedBytes = displaySource.length;
		let incremental = false;
		if (canAppend) {
			const stableTokenCount = this.#tokens.length - 2;
			const stableTokens = this.#tokens.slice(0, stableTokenCount);
			const boundary = stableTokens.at(-1)?.end ?? 0;
			if (boundary <= source.length && boundary <= displaySource.length) {
				const stableBlockCount = stableTokens.reduce((count, token) => count + token.blockCount, 0);
				const tail = parseWithOffsets(displaySource.slice(boundary));
				parsedBytes = displaySource.length - boundary;
				incremental = true;
				next = {
					blocks: [...this.#blocks.slice(0, stableBlockCount), ...tail.blocks],
					tokens: [...stableTokens, ...tail.tokens.map((token) => ({
						...token,
						start: token.start + boundary,
						end: token.end + boundary
					}))]
				};
			} else next = parseWithOffsets(displaySource);
		} else next = parseWithOffsets(displaySource);
		this.#blocks = reconcileMarkdownBlocks(this.#blocks, next.blocks);
		this.#tokens = next.tokens;
		this.#source = source;
		this.#streaming = streaming;
		this.#lastParse = {
			incremental,
			parsedBytes,
			sourceBytes: displaySource.length
		};
		return this.#blocks;
	}
};
//#endregion
//#region src/components/separator.tsx
/** A visual divider with an opt-in semantic separator contract. */
function Separator(props) {
	const orientation = () => props.orientation ?? "horizontal";
	const decorative = () => props.decorative ?? true;
	const dimensions = () => match(orientation()).with("horizontal", () => "h-px w-full").with("vertical", () => "w-px h-full").exhaustive();
	const rest = omit(props, "class", "decorative", "orientation");
	return createComponent$1(View, mergeProps(rest, {
		get role() {
			return decorative() ? "presentation" : "separator";
		},
		get ["aria-hidden"]() {
			return decorative() ? "true" : void 0;
		},
		get ["aria-orientation"]() {
			return memo(() => {
				return !!decorative();
			})() ? void 0 : orientation();
		},
		get ["class"]() {
			return mergeClasses("flex-none bg-subtle", dimensions(), props.class);
		}
	}));
}
//#endregion
//#region src/components/markdown.tsx
function runClass(run) {
	return mergeClasses(run.style.strong && "font-semibold text-primary", run.style.emphasis && "italic text-primary", run.style.code && "font-mono font-normal text-primary", run.style.deleted && "text-muted", run.style.href && "font-medium text-accent");
}
function InlineMarkdown(props) {
	return createComponent$1(RichText, {
		get ["class"]() {
			return mergeClasses("min-w-0 whitespace-normal", props.variant === "conversation" ? "text-sm leading-relaxed text-primary" : props.variant === "prompt" ? "text-sm leading-relaxed text-primary" : "text-base leading-relaxed text-secondary", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return props.runs;
				},
				children: (run) => createComponent$1(MarkdownSpan, {
					run,
					get reveal() {
						return props.animateRun?.(run) ?? false;
					}
				})
			});
		}
	});
}
function MarkdownSpan(props) {
	const reducedMotion = useReducedMotion();
	const reveal = untrack(() => props.reveal) ? createKeyframeAnimation([.72, 1], {
		duration: .12,
		ease: "easeOut",
		reducedMotion,
		reducedValue: 1
	}).value : () => 1;
	return createComponent$1(RichTextSpan, {
		get ["class"]() {
			return runClass(props.run);
		},
		get ["data-motion"]() {
			return props.reveal ? "markdown-stream-reveal" : void 0;
		},
		get style() {
			return { opacity: reveal() };
		},
		get children() {
			return props.run.text;
		}
	});
}
function Heading(props) {
	const className = createMemo(() => {
		if (props.variant === "prompt") return props.block.depth === 1 ? "text-base font-semibold tracking-tight text-primary whitespace-normal" : "text-sm font-semibold text-primary whitespace-normal";
		if (props.variant === "conversation") switch (props.block.depth) {
			case 1: return "text-lg font-semibold tracking-tight text-primary whitespace-normal";
			case 2: return "text-base font-semibold tracking-tight text-primary whitespace-normal";
			case 3: return "text-sm font-semibold tracking-tight text-primary whitespace-normal";
			default: return "text-sm font-semibold text-primary whitespace-normal";
		}
		switch (props.block.depth) {
			case 1: return "text-4xl font-bold text-primary";
			case 2: return "text-3xl font-semibold text-primary";
			case 3: return "text-2xl font-semibold text-primary";
			default: return "text-xl font-semibold text-primary";
		}
	});
	return createComponent$1(InlineMarkdown, {
		get runs() {
			return props.block.runs;
		},
		get variant() {
			return props.variant;
		},
		get ["class"]() {
			return className();
		},
		get animateRun() {
			return props.animateRun;
		}
	});
}
function MarkdownBlocks(props) {
	return createComponent$1(For, {
		get each() {
			return props.blocks;
		},
		keyed: false,
		children: (block) => createComponent$1(MarkdownBlock, {
			get block() {
				return block();
			},
			get variant() {
				return props.variant;
			},
			get animateRun() {
				return props.animateRun;
			}
		})
	});
}
function MarkdownList(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return props.variant === "conversation" || props.variant === "prompt" ? "flex flex-col gap-1.5" : "flex flex-col gap-2";
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return props.block.items;
				},
				children: (item, index) => createComponent$1(View, {
					class: "min-w-0 flex flex-row items-start gap-2",
					get children() {
						return [createComponent$1(Text, {
							"aria-hidden": "true",
							get ["class"]() {
								return mergeClasses("flex-none leading-relaxed text-secondary", props.variant === "document" ? "text-base" : "text-sm");
							},
							get children() {
								return memo(() => {
									return typeof item.checked === "boolean";
								})() ? item.checked ? "[x]" : "[ ]" : memo(() => {
									return !!props.block.ordered;
								})() ? `${props.block.start + index()}.` : "•";
							}
						}), createComponent$1(View, {
							class: "min-w-0 flex-1 flex flex-col gap-1.5",
							get children() {
								return createComponent$1(MarkdownBlocks, {
									get blocks() {
										return item.blocks;
									},
									get variant() {
										return props.variant;
									},
									get animateRun() {
										return props.animateRun;
									}
								});
							}
						})];
					}
				})
			});
		}
	});
}
function MarkdownTable(props) {
	const rows = () => [props.block.header, ...props.block.rows];
	return createComponent$1(View, {
		class: "min-w-0 overflow-hidden rounded-lg border border-subtle",
		get children() {
			return createComponent$1(For, {
				get each() {
					return rows();
				},
				children: (row, rowIndex) => createComponent$1(View, {
					get ["class"]() {
						return mergeClasses("min-w-0 flex flex-row border-b border-subtle", rowIndex() === 0 ? "bg-control" : "bg-surface");
					},
					get children() {
						return createComponent$1(For, {
							each: row,
							children: (runs, columnIndex) => createComponent$1(View, {
								get ["class"]() {
									return mergeClasses("min-w-0 flex-1", props.variant === "document" ? "px-3 py-2" : "px-2 py-1.5", columnIndex() + 1 < row.length && "border-r border-subtle");
								},
								get children() {
									return createComponent$1(InlineMarkdown, {
										runs,
										get variant() {
											return props.variant;
										},
										get ["class"]() {
											return mergeClasses(props.block.align[columnIndex()] === "center" && "text-center", props.block.align[columnIndex()] === "right" && "text-right");
										},
										get animateRun() {
											return props.animateRun;
										}
									});
								}
							})
						});
					}
				})
			});
		}
	});
}
function MarkdownBlock(props) {
	const heading = () => props.block.kind === "heading" ? props.block : void 0;
	const paragraph = () => props.block.kind === "paragraph" ? props.block : void 0;
	const blockquote = () => props.block.kind === "blockquote" ? props.block : void 0;
	const list = () => props.block.kind === "list" ? props.block : void 0;
	const table = () => props.block.kind === "table" ? props.block : void 0;
	const code = () => props.block.kind === "code" ? props.block : void 0;
	const literal = () => props.block.kind === "literal" ? props.block : void 0;
	return createComponent$1(Switch$1, { get children() {
		return [
			createComponent$1(Match, {
				get when() {
					return heading();
				},
				children: (block) => createComponent$1(Heading, {
					get block() {
						return block();
					},
					get variant() {
						return props.variant;
					},
					get animateRun() {
						return props.animateRun;
					}
				})
			}),
			createComponent$1(Match, {
				get when() {
					return paragraph();
				},
				children: (block) => createComponent$1(InlineMarkdown, {
					get runs() {
						return block().runs;
					},
					get variant() {
						return props.variant;
					},
					get animateRun() {
						return props.animateRun;
					}
				})
			}),
			createComponent$1(Match, {
				get when() {
					return blockquote();
				},
				children: (block) => createComponent$1(View, {
					class: "min-w-0 flex flex-row items-stretch gap-3",
					get children() {
						return [createComponent$1(View, {
							"aria-hidden": "true",
							class: "w-1 flex-none rounded-full bg-strong"
						}), createComponent$1(View, {
							class: "min-w-0 flex-1 flex flex-col gap-2",
							get children() {
								return createComponent$1(MarkdownBlocks, {
									get blocks() {
										return block().blocks;
									},
									get variant() {
										return props.variant;
									},
									get animateRun() {
										return props.animateRun;
									}
								});
							}
						})];
					}
				})
			}),
			createComponent$1(Match, {
				get when() {
					return list();
				},
				children: (block) => createComponent$1(MarkdownList, {
					get block() {
						return block();
					},
					get variant() {
						return props.variant;
					},
					get animateRun() {
						return props.animateRun;
					}
				})
			}),
			createComponent$1(Match, {
				get when() {
					return table();
				},
				children: (block) => createComponent$1(MarkdownTable, {
					get block() {
						return block();
					},
					get variant() {
						return props.variant;
					},
					get animateRun() {
						return props.animateRun;
					}
				})
			}),
			createComponent$1(Match, {
				get when() {
					return code();
				},
				children: (block) => createComponent$1(CodeBlock, {
					get code() {
						return block().code;
					},
					get language() {
						return block().language ?? "text";
					},
					copyLabel: "Copy code"
				})
			}),
			createComponent$1(Match, {
				get when() {
					return props.block.kind === "rule";
				},
				get children() {
					return createComponent$1(Separator, {});
				}
			}),
			createComponent$1(Match, {
				get when() {
					return literal();
				},
				children: (block) => createComponent$1(Text, {
					class: "text-base text-muted whitespace-normal leading-relaxed",
					get children() {
						return block().text;
					}
				})
			})
		];
	} });
}
function visitMarkdownRuns(blocks, visit) {
	for (const block of blocks) switch (block.kind) {
		case "heading":
		case "paragraph":
			block.runs.forEach(visit);
			break;
		case "blockquote":
			visitMarkdownRuns(block.blocks, visit);
			break;
		case "list":
			block.items.forEach((item) => {
				visitMarkdownRuns(item.blocks, visit);
			});
			break;
		case "table":
			block.header.forEach((runs) => {
				runs.forEach(visit);
			});
			block.rows.forEach((row) => {
				row.forEach((runs) => {
					runs.forEach(visit);
				});
			});
	}
}
/** Parses GFM in JavaScript and renders native Wabou components, without HTML or a DOM. */
function Markdown(props) {
	const document = new MarkdownDocument();
	let initialized = false;
	const knownRuns = /* @__PURE__ */ new WeakSet();
	const blocks = createMemo(() => {
		const parsed = document.setSource(props.source, props.streaming);
		if (!initialized) {
			visitMarkdownRuns(parsed, (run) => {
				knownRuns.add(run);
			});
			initialized = true;
		}
		return parsed;
	});
	const variant = () => props.variant ?? "document";
	const animateRun = (run) => {
		const reveal = untrack(() => props.streaming === true) && !knownRuns.has(run);
		knownRuns.add(run);
		return reveal;
	};
	return createComponent$1(View, {
		role: "region",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Markdown";
		},
		get ["class"]() {
			return mergeClasses("min-w-0 flex flex-col", variant() === "conversation" && "w-full", variant() === "document" ? "gap-4" : variant() === "prompt" ? "gap-2" : "gap-2.5", props.class);
		},
		get children() {
			return createComponent$1(MarkdownBlocks, {
				get blocks() {
					return blocks();
				},
				get variant() {
					return variant();
				},
				animateRun
			});
		}
	});
}
//#endregion
//#region src/components/toolbar.tsx
const ToolbarContext = createContext();
/** A compact command surface with one native tab stop and arrow navigation. */
function Toolbar(props) {
	const entries = [];
	const [activeId, setActiveId] = createSignal(void 0, { ownedWrite: true });
	const [registryVersion, setRegistryVersion] = createSignal(0, { ownedWrite: true });
	const orientation = () => props.orientation ?? "horizontal";
	const enabled = () => entries.filter((entry) => !entry.disabled());
	const roving = createRovingFocus({
		orientation,
		loop: props.loop,
		onMove: setActiveId
	});
	const context = {
		orientation,
		register(id, target, disabled) {
			const entry = {
				id,
				disabled
			};
			entries.push(entry);
			const unregisterRoving = roving.register({
				id,
				target,
				disabled
			});
			setRegistryVersion((version) => version + 1);
			return () => {
				unregisterRoving();
				const index = entries.indexOf(entry);
				if (index >= 0) entries.splice(index, 1);
				setRegistryVersion((version) => version + 1);
			};
		},
		activate: setActiveId,
		isTabStop(id) {
			registryVersion();
			const candidates = enabled();
			const active = activeId();
			return id === (candidates.some((entry) => entry.id === active) ? active : candidates[0]?.id);
		},
		move: roving.move
	};
	return createComponent(ToolbarContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				get role() {
					return props.role ?? "toolbar";
				},
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["aria-orientation"]() {
					return orientation();
				},
				get ["class"]() {
					return mergeClasses("flex-none flex items-center gap-1 rounded-lg border border-subtle bg-control p-1", match(orientation()).with("horizontal", () => "flex-row").with("vertical", () => "flex-col").exhaustive(), props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function ToolbarButton(props) {
	const toolbar = useContext(ToolbarContext);
	if (!toolbar) throw new Error("ToolbarButton must be used inside Toolbar");
	const id = createUniqueId();
	const forwarded = omit(props, "ref", "onFocus", "onKeyDown");
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button, mergeProps(forwarded, {
		get variant() {
			return props.variant ?? "ghost";
		},
		get size() {
			return props.size ?? "sm";
		},
		get focusOrder() {
			return toolbar.isTabStop(id) ? 0 : -1;
		},
		ref: (node) => {
			unregister?.();
			unregister = toolbar.register(id, node, () => props.disabled ?? false);
			props.ref?.(node);
		},
		onFocus: (event) => {
			toolbar.activate(id);
			props.onFocus?.(event);
		},
		onKeyDown: (event) => {
			props.onKeyDown?.(event);
			if (event.defaultPrevented) return;
			if (toolbar.move(id, event.key)) event.preventDefault();
		}
	}));
}
function ToolbarToggle(props) {
	const state = createControllableState({
		value: () => props.pressed,
		defaultValue: props.defaultPressed ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onPressedChange
	});
	const forwarded = omit(props, "pressed", "defaultPressed", "onPressedChange");
	return createComponent$1(ToolbarButton, mergeProps(forwarded, {
		get ["aria-pressed"]() {
			return state.value();
		},
		get variant() {
			return state.value() ? "secondary" : "ghost";
		},
		onClick: () => state.set(!state.value())
	}));
}
function ToolbarGroup(props) {
	const toolbar = useContext(ToolbarContext);
	if (!toolbar) throw new Error("ToolbarGroup must be used inside Toolbar");
	return createComponent$1(View, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["class"]() {
			return mergeClasses("flex items-center gap-0.5", toolbar.orientation() === "horizontal" ? "flex-row" : "flex-col", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function ToolbarSeparator(props) {
	const toolbar = useContext(ToolbarContext);
	if (!toolbar) throw new Error("ToolbarSeparator must be used inside Toolbar");
	return createComponent$1(View, {
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("flex-none bg-subtle", toolbar.orientation() === "horizontal" ? "w-px h-5" : "h-px w-5", props.class);
		}
	});
}
//#endregion
//#region src/components/menubar.tsx
const MenubarContext = createContext();
/** Persistent application menus with one tab stop and sibling menu switching. */
function Menubar(props) {
	const entries = [];
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? null,
		onChange: props.onValueChange
	});
	const enabled = () => entries.filter((entry) => !entry.disabled());
	const context = {
		openValue: state.value,
		setOpenValue: state.set,
		register(entry) {
			entries.push(entry);
			return () => {
				const index = entries.indexOf(entry);
				if (index >= 0) entries.splice(index, 1);
			};
		},
		moveOpen(value, direction) {
			const candidates = enabled();
			if (candidates.length === 0) return false;
			const index = candidates.findIndex((entry) => entry.value === value);
			const target = direction === "next" ? candidates[index + 1] ?? (props.loop === false ? void 0 : candidates[0]) : candidates[index - 1] ?? (props.loop === false ? void 0 : candidates.at(-1));
			if (!target) return false;
			state.set(target.value);
			target.target.focus();
			return true;
		}
	};
	return createComponent(MenubarContext, {
		value: context,
		get children() {
			return createComponent$1(Toolbar, {
				role: "menubar",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get loop() {
					return props.loop;
				},
				get ["class"]() {
					return props.class;
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function MenubarMenu(props) {
	const menubar = useContext(MenubarContext);
	if (!menubar) throw new Error("MenubarMenu must be used inside Menubar");
	let unregister;
	let closeOnPointerActivation;
	let switchedByHover = false;
	onCleanup(() => unregister?.());
	const handleMenuKey = (event) => {
		const direction = match(event.key).with("ArrowRight", () => "next").with("ArrowLeft", () => "previous").otherwise(() => void 0);
		if (!direction || !menubar.moveOpen(props.value, direction)) return;
		event.preventDefault();
	};
	return createComponent$1(DropdownMenu, {
		get ["aria-label"]() {
			return `${props.label} menu`;
		},
		get items() {
			return props.items;
		},
		get open() {
			return menubar.openValue() === props.value;
		},
		onOpenChange: (open) => menubar.setOpenValue(open ? props.value : null),
		get onAction() {
			return props.onAction;
		},
		get motion() {
			return props.motion;
		},
		placement: "bottom-start",
		outsidePointerStrategy: "passthrough",
		get restoreFocus() {
			return menubar.openValue() === null || menubar.openValue() === props.value;
		},
		onContentKeyDown: handleMenuKey,
		trigger: (trigger) => createComponent$1(ToolbarButton, mergeProps(trigger, {
			role: "menuitem",
			get ["aria-label"]() {
				return props.label;
			},
			get disabled() {
				return props.disabled;
			},
			ref: (node) => {
				unregister?.();
				unregister = menubar.register({
					value: props.value,
					target: node,
					disabled: () => props.disabled ?? false
				});
				trigger.ref(node);
			},
			onFocus: (event) => {
				if (event.payload.focusVisible && menubar.openValue() !== null) menubar.setOpenValue(props.value);
			},
			onPointerEnter: () => {
				if (menubar.openValue() !== null && menubar.openValue() !== props.value) {
					switchedByHover = true;
					menubar.setOpenValue(props.value);
				}
			},
			onPointerDown: () => {
				closeOnPointerActivation = menubar.openValue() === props.value && !switchedByHover;
				switchedByHover = false;
			},
			onPointerCancel: () => {
				closeOnPointerActivation = void 0;
			},
			onClick: (event) => {
				event.stopPropagation();
				const close = closeOnPointerActivation ?? menubar.openValue() === props.value;
				closeOnPointerActivation = void 0;
				menubar.setOpenValue(close ? null : props.value);
			},
			get children() {
				return props.children ?? props.label;
			}
		}))
	});
}
//#endregion
//#region src/components/message.tsx
const MessageContext = createContext({
	align: () => "start",
	interacting: () => false
});
const BubbleContext = createContext({
	align: () => "start",
	interacting: () => false,
	variant: () => "default"
});
function MessageGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "group";
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex flex-col gap-3", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function messageClass(align = "start", className) {
	return mergeClasses("relative w-full min-w-0 flex gap-2 text-sm", align === "end" ? "flex-row-reverse" : "flex-row", className);
}
function Message(props) {
	const hover = createHover();
	const focus = createFocusWithin();
	const forwarded = omit(props, "align", "class", "children", "onPointerEnter", "onPointerLeave", "onFocusIn", "onFocusOut");
	const context = {
		align: () => props.align ?? "start",
		interacting: () => hover.hovered() || focus.focusWithin()
	};
	return createComponent$1(MessageContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get role() {
					return props.role ?? "group";
				},
				get ["class"]() {
					return messageClass(context.align(), props.class);
				},
				onPointerEnter: (event) => {
					hover.bindings.onPointerEnter();
					props.onPointerEnter?.(event);
				},
				onPointerLeave: (event) => {
					hover.bindings.onPointerLeave();
					props.onPointerLeave?.(event);
				},
				onFocusIn: (event) => {
					focus.bindings.onFocusIn();
					props.onFocusIn?.(event);
				},
				onFocusOut: (event) => {
					focus.bindings.onFocusOut();
					props.onFocusOut?.(event);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function MessageAvatar(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("w-8 h-8 flex-none self-end overflow-hidden rounded-full bg-control flex items-center justify-center", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageContent(props) {
	const context = useContext(MessageContext);
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("flex-1 min-w-0 flex flex-col gap-2", context.align() === "end" ? "items-end" : "items-start", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageHeader(props) {
	const context = useContext(MessageContext);
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("max-w-full min-w-0 px-3 text-xs font-medium text-muted", context.align() === "end" && "text-right", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
const MessageFooter = MessageHeader;
function messageActionsClass(align = "start", className) {
	return mergeClasses("h-7 px-1 flex flex-row items-center gap-1", align === "end" ? "self-end justify-end" : "self-start justify-start", className);
}
/** Compact, consistently aligned actions belonging to one message. */
function MessageActions(props) {
	const context = useContext(MessageContext);
	const forwarded = omit(props, "align", "visibility", "class", "children");
	const interactionClass = () => props.visibility === "interaction" && !context.interacting() ? "opacity-0 pointer-events-none" : "opacity-100";
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return props.role ?? "toolbar";
		},
		get ["class"]() {
			return messageActionsClass(props.align ?? context.align(), mergeClasses(interactionClass(), props.class));
		},
		get children() {
			return props.children;
		}
	}));
}
function BubbleGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 grid gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function bubbleClass(variant = "default", align = "start", className) {
	return mergeClasses("relative min-w-0 flex flex-col gap-1", variant === "ghost" ? "max-w-full" : "max-w-4/5", align === "end" ? "self-end items-end" : "self-start items-start", className);
}
function Bubble(props) {
	const message = useContext(MessageContext);
	const forwarded = omit(props, "variant", "align", "class", "children");
	const context = {
		variant: () => props.variant ?? "default",
		align: () => props.align ?? message.align(),
		interacting: message.interacting
	};
	return createComponent$1(BubbleContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get ["class"]() {
					return bubbleClass(context.variant(), context.align(), props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function bubbleContentClass(variant, className) {
	const colors = match(variant).with("default", () => "border-transparent bg-accent text-on-accent").with("secondary", () => "border-transparent bg-control text-primary").with("muted", () => "border-transparent bg-control text-secondary").with("tinted", () => "border-transparent bg-selected text-primary").with("outline", () => "border-subtle bg-surface text-primary").with("ghost", () => "border-transparent bg-transparent text-primary").with("destructive", () => "border-danger bg-danger-surface text-danger-primary").exhaustive();
	return mergeClasses("max-w-full min-w-0 overflow-hidden rounded-xl border", variant === "ghost" ? "p-0" : "px-3 py-2", colors, className);
}
function BubbleContent(props) {
	const context = useContext(BubbleContext);
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return bubbleContentClass(context.variant(), props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BubbleReactions(props) {
	const bubble = useContext(BubbleContext);
	const side = () => props.side ?? "bottom";
	const align = () => props.align ?? bubble.align();
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("relative z-10 flex-none flex items-center justify-center gap-1 rounded-full bg-control px-1.5 py-0.5 text-sm", align() === "end" ? "self-end" : "self-start", props.class);
		},
		get transform() {
			return translate2d$1(0, side() === "top" ? 4 : -4);
		},
		get children() {
			return props.children;
		}
	});
}
function Marker(props) {
	const variant = () => props.variant ?? "default";
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 min-h-4 flex items-center gap-2 text-sm text-muted", variant() === "border" && "border-b border-subtle pb-2", props.class);
		},
		get children() {
			return [
				createComponent$1(Show, {
					get when() {
						return variant() === "separator";
					},
					get children() {
						return createComponent$1(View, {
							"aria-hidden": "true",
							class: "flex-1 min-w-0 h-px bg-subtle"
						});
					}
				}),
				memo(() => {
					return props.children;
				}),
				createComponent$1(Show, {
					get when() {
						return variant() === "separator";
					},
					get children() {
						return createComponent$1(View, {
							"aria-hidden": "true",
							class: "flex-1 min-w-0 h-px bg-subtle"
						});
					}
				})
			];
		}
	});
}
function MarkerIcon(props) {
	return createComponent$1(View, mergeProps(props, {
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("w-4 h-4 flex-none", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MarkerContent(props) {
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-w-0 whitespace-nowrap text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
//#region src/components/message-scroller.tsx
function messageScrollRange(contentHeight, viewportHeight) {
	return Math.max(0, contentHeight - viewportHeight);
}
function isMessageScrollNearEnd(scrollY, contentHeight, viewportHeight, threshold = 24) {
	return scrollY >= messageScrollRange(contentHeight, viewportHeight) - Math.max(0, threshold);
}
/** Smallest vertical delta that reveals a target without disturbing visible content. */
function messageScrollRevealDelta(viewport, target, margin = 12) {
	const inset = Math.max(0, margin);
	const visibleTop = viewport.y + inset;
	const visibleBottom = viewport.y + viewport.height - inset;
	if (target.y < visibleTop) return target.y - visibleTop;
	const targetBottom = target.y + target.height;
	if (targetBottom > visibleBottom) return targetBottom - visibleBottom;
	return 0;
}
/** Vertical delta that places a target at the viewport's reading start. */
function messageScrollStartDelta(viewport, target, margin = 12) {
	return target.y - (viewport.y + Math.max(0, margin));
}
const MessageScrollerContext = createContext(null);
function requireMessageScroller() {
	const context = useContext(MessageScrollerContext);
	if (!context) throw new Error("MessageScroller components require a MessageScroller root");
	return context;
}
function useMessageScroller() {
	return requireMessageScroller();
}
/** Pick the last conversation anchor that has crossed the reading line. */
function activeMessageAnchor(viewport, anchors, offset = 64) {
	if (anchors.length === 0) return void 0;
	const line = viewport.y + Math.min(Math.max(0, offset), viewport.height / 3);
	const ordered = [...anchors].sort((first, second) => first.rect.y === second.rect.y ? first.rect.x - second.rect.x : first.rect.y - second.rect.y);
	let active = ordered[0]?.id;
	for (const anchor of ordered) {
		if (anchor.rect.y > line) break;
		active = anchor.id;
	}
	return active;
}
function MessageScroller(props) {
	const host = useHost$1();
	const forwarded = omit(props, "followEnd", "endThreshold", "class", "children");
	const [scrollY, setScrollY] = createSignal(0);
	const [followingEnd, setFollowingEnd] = createSignal(props.followEnd ?? true);
	const [activeAnchor, setActiveAnchor] = createSignal();
	const threshold = () => Math.max(0, props.endThreshold ?? 24);
	let viewport;
	let endFrame;
	let anchorFrame;
	const anchors = /* @__PURE__ */ new Map();
	let measuredAnchors = [];
	const updateActiveFromScroll = (position = scrollY()) => {
		if (followingEnd() && isMessageScrollNearEnd(position, contentSize.height(), viewportSize.height(), threshold())) {
			const last = measuredAnchors.reduce((current, anchor) => !current || anchor.rect.y > current.rect.y ? anchor : current, void 0);
			setActiveAnchor(last?.id);
			return;
		}
		setActiveAnchor(activeMessageAnchor({
			x: 0,
			y: position,
			width: viewportSize.width(),
			height: viewportSize.height()
		}, measuredAnchors));
	};
	const measureAnchors = () => {
		if (!viewport || anchors.size === 0) {
			measuredAnchors = [];
			setActiveAnchor(void 0);
			return;
		}
		const targets = [...anchors.entries()];
		const snapshot = host.layout.snapshot([viewport, ...targets.map(([, node]) => node)]);
		const nodeByKey = new Map(snapshot.nodes.map((node) => [`${node.id.lo}:${node.id.hi}`, node]));
		const viewportMetrics = nodeByKey.get(`${viewport.id.lo}:${viewport.id.hi}`);
		if (!viewportMetrics) return;
		measuredAnchors = targets.flatMap(([id, node]) => {
			const metrics = nodeByKey.get(`${node.id.lo}:${node.id.hi}`);
			return metrics ? [{
				id,
				rect: {
					x: metrics.rect.x - viewportMetrics.rect.x,
					y: metrics.rect.y - viewportMetrics.rect.y + scrollY(),
					width: metrics.rect.width,
					height: metrics.rect.height
				}
			}] : [];
		});
		updateActiveFromScroll();
	};
	const scheduleAnchorMeasure = () => {
		if (anchorFrame !== void 0) cancelAnimationFrame(anchorFrame);
		anchorFrame = requestAnimationFrame(() => {
			anchorFrame = void 0;
			measureAnchors();
		});
	};
	const scheduleEnd = () => {
		if (!followingEnd() || !viewport) return;
		if (endFrame !== void 0) cancelAnimationFrame(endFrame);
		endFrame = requestAnimationFrame(() => {
			endFrame = void 0;
			viewport?.scrollTo({ top: Number.MAX_SAFE_INTEGER });
		});
	};
	const geometryChanged = () => {
		scheduleEnd();
		scheduleAnchorMeasure();
	};
	const viewportSize = createMeasuredSize({ onChange: geometryChanged });
	const contentSize = createMeasuredSize({ onChange: geometryChanged });
	const range = () => messageScrollRange(contentSize.height(), viewportSize.height());
	const nearEnd = () => isMessageScrollNearEnd(scrollY(), contentSize.height(), viewportSize.height(), threshold());
	const context = {
		followingEnd,
		canScrollStart: () => scrollY() > threshold(),
		canScrollEnd: () => range() > 0 && !nearEnd(),
		activeAnchor,
		scrollTo: (direction) => {
			setFollowingEnd(direction === "end");
			viewport?.scrollTo({ top: direction === "end" ? Number.MAX_SAFE_INTEGER : 0 });
		},
		scrollIntoView: (target, options) => {
			if (!viewport) return;
			const snapshot = host.layout.snapshot([viewport, target]);
			const viewportMetrics = snapshot.nodes.find((node) => node.id.lo === viewport?.id.lo && node.id.hi === viewport?.id.hi);
			const targetMetrics = snapshot.nodes.find((node) => node.id.lo === target.id.lo && node.id.hi === target.id.hi);
			if (!viewportMetrics || !targetMetrics) return;
			const delta = options?.align === "start" ? messageScrollStartDelta(viewportMetrics.rect, targetMetrics.rect, options.margin) : messageScrollRevealDelta(viewportMetrics.rect, targetMetrics.rect, options?.margin);
			if (delta === 0) return;
			setFollowingEnd(false);
			viewport.scrollBy({ top: delta });
		},
		scrollToAnchor: (anchor, options) => {
			const target = anchors.get(anchor);
			if (target) context.scrollIntoView(target, options);
		},
		setViewport: (node) => {
			viewport = node;
			viewportSize.ref(node);
			scheduleEnd();
			scheduleAnchorMeasure();
		},
		setContent: (node) => {
			contentSize.ref(node);
			scheduleAnchorMeasure();
		},
		handleScroll: (event) => {
			const next = Math.max(0, event.scrollY ?? 0);
			setScrollY(next);
			setFollowingEnd(isMessageScrollNearEnd(next, contentSize.height(), viewportSize.height(), threshold()));
			updateActiveFromScroll(next);
		},
		registerAnchor: (anchor, node) => {
			anchors.set(anchor, node);
			scheduleAnchorMeasure();
		},
		unregisterAnchor: (anchor, node) => {
			const current = anchors.get(anchor);
			if (current?.id.lo === node.id.lo && current.id.hi === node.id.hi) {
				anchors.delete(anchor);
				scheduleAnchorMeasure();
			}
		}
	};
	onCleanup(() => {
		if (endFrame !== void 0) cancelAnimationFrame(endFrame);
		if (anchorFrame !== void 0) cancelAnimationFrame(anchorFrame);
	});
	return createComponent$1(MessageScrollerContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get ["class"]() {
					return mergeClasses("relative w-full h-full min-w-0 min-h-0 flex flex-col overflow-hidden", props.class);
				},
				get transform() {
					return props.transform ?? translate2d$1(-16, 0);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function MessageScrollerViewport(props) {
	const context = requireMessageScroller();
	const forwarded = omit(props, "class", "children", "ref", "onScroll");
	return createComponent$1(View, mergeProps(forwarded, {
		ref: (node) => {
			context.setViewport(node);
			props.ref?.(node);
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 min-h-0 flex-1 overflow-x-hidden overflow-y-auto", props.class);
		},
		get scrollbar() {
			return props.scrollbar ?? { visibility: "auto" };
		},
		onScroll: (event) => {
			context.handleScroll(event);
			props.onScroll?.(event);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageScrollerContent(props) {
	const context = requireMessageScroller();
	const forwarded = omit(props, "class", "children", "ref");
	return createComponent$1(View, mergeProps(forwarded, {
		ref: (node) => {
			context.setContent(node);
			props.ref?.(node);
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 min-h-full flex-none flex flex-col gap-4", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageScrollerItem(props) {
	const context = useContext(MessageScrollerContext);
	const forwarded = omit(props, "anchor", "class", "children", "ref");
	let node;
	let registered;
	createEffect(() => props.anchor, (next) => {
		if (node && registered && registered !== next) {
			context?.unregisterAnchor(registered, node);
			registered = void 0;
		}
		if (node && next && next !== registered) {
			context?.registerAnchor(next, node);
			registered = next;
		}
	});
	onCleanup(() => {
		if (node && registered) context?.unregisterAnchor(registered, node);
	});
	return createComponent$1(View, mergeProps(forwarded, {
		ref: (handle) => {
			node = handle;
			const anchor = props.anchor;
			if (anchor) {
				context?.registerAnchor(anchor, handle);
				registered = anchor;
			}
			props.ref?.(handle);
		},
		get ["data-message-anchor"]() {
			return props.anchor;
		},
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex-none", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageScrollerButton(props) {
	const context = requireMessageScroller();
	const direction = () => props.direction ?? "end";
	const active = () => direction() === "end" ? context.canScrollEnd() : context.canScrollStart();
	const forwarded = omit(props, "direction", "class", "children", "onClick");
	const label = () => direction() === "end" ? "Scroll to end" : "Scroll to start";
	return createComponent$1(Show, {
		get when() {
			return active();
		},
		get children() {
			return createComponent$1(Button, mergeProps(forwarded, {
				get ["aria-label"]() {
					return props["aria-label"] ?? label();
				},
				get variant() {
					return props.variant ?? "secondary";
				},
				get size() {
					return props.size ?? "icon";
				},
				get ["class"]() {
					return mergeClasses("absolute z-10 left-1/2 flex-none rounded-full shadow-sm", direction() === "end" ? "bottom-3" : "top-3", props.class);
				},
				onClick: (event) => {
					context.scrollTo(direction());
					props.onClick?.(event);
				},
				get children() {
					return props.children ?? createComponent$1(Icon, {
						"aria-hidden": "true",
						source: arrowDown,
						size: 16,
						get transform() {
							return memo(() => {
								return direction() === "start";
							})() ? rotate2d$1(Math.PI) : void 0;
						}
					});
				}
			}));
		}
	});
}
//#endregion
//#region src/components/select-semantics.ts
/** Keep semantic ID references live for the same lifetime as the popup node. */
function selectControlsId(listboxId, open) {
	return open ? listboxId : void 0;
}
//#endregion
//#region src/components/select.tsx
const ITEM_HEIGHT = 40;
const VISIBLE_ITEMS = 6;
/** Shadcn-inspired single Select backed by Wabou-native interaction state. */
function Select(props) {
	const theme = useComponentsTheme();
	const id = createUniqueId();
	let trigger;
	let content;
	let viewport;
	let scrollTop = 0;
	const items = () => props.options.map((option) => ({
		id: option.value,
		textValue: option.label,
		disabled: option.disabled
	}));
	const execute = (command) => {
		match(command).with({ type: "FOCUS_TRIGGER" }, () => requestAnimationFrame(() => trigger?.focus())).with({ type: "FOCUS_CONTENT" }, () => requestAnimationFrame(() => content?.focus())).with({ type: "SCROLL_TO_ITEM" }, ({ id }) => {
			const index = props.options.findIndex((option) => option.value === id);
			if (index < 0) return;
			const firstVisible = Math.floor(scrollTop / ITEM_HEIGHT);
			const lastVisible = firstVisible + VISIBLE_ITEMS - 1;
			const nextTop = index < firstVisible ? index * ITEM_HEIGHT : index > lastVisible ? (index - VISIBLE_ITEMS + 1) * ITEM_HEIGHT : scrollTop;
			if (nextTop !== scrollTop) {
				scrollTop = nextTop;
				requestAnimationFrame(() => viewport?.scrollTo({ top: nextTop }));
			}
		}).exhaustive();
	};
	const interaction = createSelectInteraction({
		items,
		value: () => props.value,
		defaultValue: props.defaultValue,
		open: () => props.open,
		defaultOpen: props.defaultOpen,
		disabled: () => props.disabled ?? false,
		onValueChange: props.onValueChange,
		onOpenChange: props.onOpenChange,
		execute
	});
	const selected = () => props.options.find((option) => option.value === interaction.value());
	const handleKeyDown = (event) => {
		if (match(event.key).with("ArrowDown", () => interaction.send({ type: "ARROW_DOWN" })).with("ArrowUp", () => interaction.send({ type: "ARROW_UP" })).with("Home", () => interaction.send({ type: "HOME" })).with("End", () => interaction.send({ type: "END" })).with("Enter", () => interaction.send({ type: interaction.open() ? "SELECT" : "OPEN" })).with(" ", () => interaction.send({ type: interaction.open() ? "SELECT" : "OPEN" })).with("Escape", () => interaction.send({ type: "CLOSE" })).otherwise((key) => interaction.typeahead(key))) event.preventDefault();
	};
	return createComponent$1(Popover$1, {
		contentRole: "presentation",
		popupRole: "listbox",
		get open() {
			return interaction.open();
		},
		onOpenChange: (open) => {
			interaction.send({ type: open ? "OPEN" : "CLOSE" });
		},
		placement: "bottom-start",
		openOnPointerDown: true,
		get contentClass() {
			return mergeClasses("w-72 p-1 rounded-xl border border-subtle bg-surface", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		get motion() {
			return props.motion ?? false;
		},
		trigger: (popover) => createComponent$1(Button$1, {
			unstyled: true,
			role: "combobox",
			get disabled() {
				return props.disabled;
			},
			get ["aria-label"]() {
				return props["aria-label"];
			},
			"aria-haspopup": "listbox",
			get ["aria-expanded"]() {
				return interaction.open();
			},
			get ["aria-controls"]() {
				return selectControlsId(`${id}-listbox`, interaction.open());
			},
			get ["aria-valuetext"]() {
				return selected()?.label;
			},
			ref: (node) => {
				trigger = node;
				popover.ref(node);
			},
			class: (state) => mergeClasses("w-72 overflow-hidden justify-between border bg-input shadow-xs", componentsControlSize("default"), state.focused ? "border-focus" : "border-subtle", props.class),
			style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
			get onClick() {
				return popover.onClick;
			},
			get onPointerDown() {
				return popover.onPointerDown;
			},
			get onPointerCancel() {
				return popover.onPointerCancel;
			},
			onKeyDown: (event) => {
				popover.onKeyDown(event);
				handleKeyDown(event);
			},
			get children() {
				return [createComponent$1(Text, {
					get ["class"]() {
						return mergeClasses("min-w-0 flex-1 text-left truncate", selected() ? "text-primary" : "text-muted");
					},
					get children() {
						return selected()?.label ?? props.placeholder ?? "Select an option";
					}
				}), createComponent$1(Icon, {
					source: chevronDown,
					class: "flex-none text-muted",
					size: 16
				})];
			}
		}),
		get children() {
			return createComponent$1(ScrollArea, {
				ref: (node) => {
					viewport = node;
					scrollTop = 0;
				},
				class: "w-full flex-none",
				contentClass: "gap-1",
				get style() {
					return { height: `${Math.max(1, Math.min(props.options.length, VISIBLE_ITEMS)) * ITEM_HEIGHT - 4}px` };
				},
				onScroll: (event) => {
					scrollTop = event.scrollY ?? scrollTop;
				},
				get children() {
					return createComponent$1(View, {
						id: `${id}-listbox`,
						ref: (node) => content = node,
						role: "listbox",
						get ["aria-label"]() {
							return props["aria-label"];
						},
						get ["aria-activedescendant"]() {
							return memo(() => {
								return !!interaction.highlighted();
							})() ? `${id}-option-${interaction.highlighted()}` : void 0;
						},
						focusOrder: 0,
						class: "min-w-0 flex flex-col gap-1",
						onKeyDown: handleKeyDown,
						get children() {
							return createComponent$1(For, {
								get each() {
									return props.options;
								},
								keyed: false,
								children: (option) => {
									const selected = () => interaction.value() === option().value;
									const highlighted = () => interaction.highlighted() === option().value;
									return createComponent$1(View, {
										get id() {
											return `${id}-option-${option().value}`;
										},
										role: "option",
										get ["aria-selected"]() {
											return selected();
										},
										get ["aria-disabled"]() {
											return option().disabled;
										},
										get ["class"]() {
											return mergeClasses("w-full h-8 flex-none px-3 flex items-center justify-between gap-3 rounded-lg text-sm", highlighted() ? "bg-control-hover text-primary" : "bg-transparent text-secondary");
										},
										get style() {
											return { opacity: option().disabled ? .45 : 1 };
										},
										onPointerMove: () => interaction.send({
											type: "HIGHLIGHT",
											id: option().value
										}),
										onClick: () => interaction.send({
											type: "SELECT",
											id: option().value
										}),
										get children() {
											return [createComponent$1(Text, {
												class: "min-w-0 flex-1 text-sm truncate",
												get children() {
													return option().label;
												}
											}), createComponent$1(View, {
												"aria-hidden": "true",
												class: "w-4 h-4 flex-none",
												get children() {
													return memo(() => {
														return !!selected();
													})() ? createComponent$1(Icon, {
														source: check,
														class: "text-accent",
														size: 16
													}) : selected();
												}
											})];
										}
									});
								}
							});
						}
					});
				}
			});
		}
	});
}
//#endregion
//#region src/components/native-select.tsx
/**
* Compact Wabou-native select for ordinary forms.
*
* Unlike the composable Select skin, this deliberately fixes immediate motion
* and elevation so callers only own options, value, and form sizing.
*/
function NativeSelect(props) {
	return createComponent$1(Select, mergeProps(props, {
		motion: false,
		contentClass: "rounded-xl",
		contentShadows: null
	}));
}
//#endregion
//#region src/components/pagination-state.ts
function integerAtLeast(value, minimum) {
	return Math.max(minimum, Math.floor(Number.isFinite(value) ? value : minimum));
}
function normalizePageCount(count) {
	return integerAtLeast(count, 1);
}
function clampPage(page, count) {
	return Math.min(normalizePageCount(count), integerAtLeast(page, 1));
}
function range(start, end) {
	return Array.from({ length: Math.max(0, end - start + 1) }, (_, index) => start + index);
}
/**
* Produces a stable, 1-indexed page range with explicit start/end ellipses.
* A single hidden page is shown directly instead of being replaced by an
* ellipsis, which keeps every item actionable and avoids misleading gaps.
*/
function createPaginationRange(options) {
	const count = normalizePageCount(options.count);
	const page = clampPage(options.page, count);
	const siblings = integerAtLeast(options.siblingCount ?? 1, 0);
	const boundaries = integerAtLeast(options.boundaryCount ?? 1, 0);
	if (count <= boundaries * 2 + siblings * 2 + 3) return range(1, count);
	const left = Math.max(page - siblings, boundaries + 2);
	const right = Math.min(page + siblings, count - boundaries - 1);
	const start = range(1, boundaries);
	const middle = range(left, right);
	const end = range(count - boundaries + 1, count);
	const before = left === boundaries + 2 ? [boundaries + 1] : ["ellipsis-start"];
	const after = right === count - boundaries - 1 ? [count - boundaries] : ["ellipsis-end"];
	return [
		...start,
		...before,
		...middle,
		...after,
		...end
	];
}
//#endregion
//#region src/components/navigation.tsx
function Breadcrumb(props) {
	const rest = omit(props, "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Breadcrumb";
		},
		get ["class"]() {
			return mergeClasses("min-w-0", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BreadcrumbList(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-w-0 flex flex-wrap items-center gap-1.5 text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BreadcrumbItem(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-w-0 flex items-center gap-1.5", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BreadcrumbLink(props) {
	return createComponent$1(Button$1, mergeProps(props, {
		unstyled: true,
		role: "link",
		class: (state) => mergeClasses("min-w-0 rounded-sm text-sm text-secondary", state.hovered && "text-primary", state.focusVisible && "border border-focus", props.class)
	}));
}
function BreadcrumbPage(props) {
	const rest = omit(props, "class", "children");
	return createComponent$1(Text, mergeProps(rest, {
		role: "link",
		"aria-disabled": "true",
		"aria-current": "page",
		get ["class"]() {
			return mergeClasses("min-w-0 text-sm font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BreadcrumbSeparator(props) {
	const rest = omit(props, "class", "children");
	return createComponent$1(View, mergeProps(rest, {
		role: "presentation",
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("w-4 h-4 flex-none flex items-center justify-center text-muted", props.class);
		},
		get children() {
			return props.children ?? createComponent$1(Icon, {
				source: chevronRight,
				size: 14
			});
		}
	}));
}
function BreadcrumbEllipsis(props) {
	const rest = omit(props, "class");
	return createComponent$1(View, mergeProps(rest, {
		role: "presentation",
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("w-8 h-8 flex-none flex items-center justify-center text-muted", props.class);
		},
		get children() {
			return createComponent$1(Icon, {
				source: ellipsis,
				size: 16
			});
		}
	}));
}
const PaginationContext = createContext({
	managed: false,
	count: () => 1,
	page: () => 1,
	disabled: () => false,
	select: () => {}
});
function Pagination(props) {
	const state = createControllableState({
		value: () => props.page,
		defaultValue: props.defaultPage ?? 1,
		onChange: props.onPageChange
	});
	const count = () => normalizePageCount(props.count ?? 1);
	const page = () => clampPage(state.value(), count());
	const context = {
		managed: true,
		count,
		page,
		disabled: () => props.disabled ?? false,
		select: (next) => {
			if (props.count === void 0 || context.disabled()) return;
			state.set(clampPage(next, count()));
		}
	};
	const content = () => createComponent$1(View, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Pagination";
		},
		get ["aria-disabled"]() {
			return context.disabled() || void 0;
		},
		get ["class"]() {
			return mergeClasses("flex items-center", props.class);
		},
		get children() {
			return props.children;
		}
	});
	return props.count === void 0 ? content() : createComponent(PaginationContext, {
		value: context,
		get children() {
			return content();
		}
	});
}
function PaginationContent(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("flex items-center gap-1", props.class);
	} }));
}
function PaginationItem(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("flex-none", props.class);
	} }));
}
function PaginationLink(props) {
	const context = useContext(PaginationContext);
	const forwarded = omit(props, "active", "page");
	const active = () => props.active ?? (props.page !== void 0 && context.managed && context.page() === props.page);
	return createComponent$1(Button, mergeProps(forwarded, {
		role: "link",
		size: "icon",
		get variant() {
			return active() ? "outline" : "ghost";
		},
		get selected() {
			return active();
		},
		get ["aria-current"]() {
			return active() ? "page" : void 0;
		},
		get ["aria-label"]() {
			return props["aria-label"] ?? (props.page === void 0 ? void 0 : `Page ${props.page}`);
		},
		get disabled() {
			return props.disabled ?? (context.managed && context.disabled());
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented && props.page !== void 0) context.select(props.page);
		}
	}));
}
function PaginationEllipsis(props) {
	return createComponent$1(Text, {
		"aria-hidden": true,
		get ["class"]() {
			return mergeClasses("w-8 text-center text-muted", props.class);
		},
		children: "..."
	});
}
function PaginationItems(props) {
	const context = useContext(PaginationContext);
	if (!context.managed) throw new Error("PaginationItems must be used inside a managed Pagination");
	const items = () => createPaginationRange({
		count: context.count(),
		page: context.page(),
		siblingCount: props.siblingCount,
		boundaryCount: props.boundaryCount
	});
	return createComponent$1(For, {
		get each() {
			return items();
		},
		children: (item) => typeof item === "number" ? props.renderItem?.(item) ?? createComponent$1(PaginationItem, { get children() {
			return createComponent$1(PaginationLink, {
				page: item,
				get children() {
					return String(item);
				}
			});
		} }) : props.renderEllipsis?.(item === "ellipsis-start" ? "start" : "end") ?? createComponent$1(PaginationEllipsis, {})
	});
}
function PaginationPrevious(props) {
	const context = useContext(PaginationContext);
	return createComponent$1(Button, mergeProps(props, {
		variant: "ghost",
		size: "sm",
		get disabled() {
			return props.disabled ?? (context.managed ? context.disabled() || context.page() <= 1 : false);
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented && context.managed) context.select(context.page() - 1);
		},
		get children() {
			return props.children ?? "Previous";
		}
	}));
}
function PaginationNext(props) {
	const context = useContext(PaginationContext);
	return createComponent$1(Button, mergeProps(props, {
		variant: "ghost",
		size: "sm",
		get disabled() {
			return props.disabled ?? (context.managed ? context.disabled() || context.page() >= context.count() : false);
		},
		onClick: (event) => {
			props.onClick?.(event);
			if (!event.defaultPrevented && context.managed) context.select(context.page() + 1);
		},
		get children() {
			return props.children ?? "Next";
		}
	}));
}
//#endregion
//#region src/components/popover.tsx
/** A ready-to-use floating surface backed by Wabou's collision-aware overlay. */
function Popover(props) {
	const theme = useComponentsTheme();
	return createComponent$1(Popover$1, mergeProps(props, {
		get contentClass() {
			return mergeClasses("min-w-48 max-w-sm min-h-0 p-4 flex flex-col gap-3 rounded-xl border border-subtle bg-surface", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		}
	}));
}
function PopoverHeader(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("min-w-0 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function PopoverTitle(props) {
	return createComponent$1(Text, {
		role: "heading",
		get ["class"]() {
			return mergeClasses("whitespace-normal text-sm font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function PopoverDescription(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return mergeClasses("min-w-0 whitespace-normal text-xs text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function PopoverFooter(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("min-w-0 flex items-center justify-end gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/components/navigation-menu.tsx
const NavigationMenuContext = createContext();
const NavigationItemContext = createContext();
function requireNavigationMenu() {
	const context = useContext(NavigationMenuContext);
	if (!context) throw new Error("NavigationMenu child requires a root");
	return context;
}
function requireNavigationItem() {
	const context = useContext(NavigationItemContext);
	if (!context) throw new Error("NavigationMenu child requires an item");
	return context;
}
function NavigationMenu(props) {
	const entries = [];
	const contents = /* @__PURE__ */ new Map();
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? null,
		onChange: props.onValueChange
	});
	const enabled = () => entries.filter((entry) => !entry.disabled());
	const context = {
		openValue: state.value,
		setOpenValue: state.set,
		registerTrigger(entry) {
			entries.push(entry);
			return () => {
				const index = entries.indexOf(entry);
				if (index >= 0) entries.splice(index, 1);
			};
		},
		registerContent(value, render) {
			contents.set(value, render);
			return () => {
				if (contents.get(value) !== render) return;
				contents.delete(value);
			};
		},
		content() {
			const value = state.value();
			return value ? contents.get(value)?.() : void 0;
		},
		move(value, direction) {
			const candidates = enabled();
			if (candidates.length === 0) return false;
			const index = candidates.findIndex((entry) => entry.value === value);
			const target = direction === "next" ? candidates[index + 1] ?? (props.loop === false ? void 0 : candidates[0]) : candidates[index - 1] ?? (props.loop === false ? void 0 : candidates.at(-1));
			if (!target) return false;
			target.target.focus();
			if (state.value() !== null) state.set(target.value);
			return true;
		}
	};
	return createComponent(NavigationMenuContext, {
		value: context,
		get children() {
			return createComponent$1(Popover, {
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get open() {
					return state.value() !== null;
				},
				onOpenChange: (open) => {
					if (!open) state.set(null);
				},
				placement: "bottom-start",
				outsidePointerStrategy: "passthrough",
				get contentClass() {
					return mergeClasses("w-[520px] max-w-full min-w-0 overflow-hidden rounded-xl p-2", props.viewportClass);
				},
				trigger: (trigger) => createComponent$1(View, {
					ref(r$) {
						var _ref$ = trigger.ref;
						typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : trigger.ref = r$;
					},
					role: "group",
					get ["aria-label"]() {
						return props["aria-label"];
					},
					get ["class"]() {
						return mergeClasses("relative inline-flex flex-none items-center justify-center", props.class);
					},
					get children() {
						return props.children;
					}
				}),
				get children() {
					return context.content();
				}
			});
		}
	});
}
function NavigationMenuList(props) {
	return createComponent$1(View, {
		role: "menubar",
		get ["class"]() {
			return mergeClasses("flex flex-none items-center justify-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function NavigationMenuItem(props) {
	const context = {
		value: props.value,
		disabled: () => props.disabled ?? false
	};
	return createComponent$1(NavigationItemContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				get ["class"]() {
					return mergeClasses("relative flex-none", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function navigationMenuTriggerClass(open, className) {
	return mergeClasses("h-8 px-3 inline-flex flex-none items-center justify-center gap-1 rounded-lg border-transparent text-sm font-medium", open ? "bg-selected text-primary" : "bg-transparent text-secondary", className);
}
function NavigationMenuTrigger(props) {
	const menu = requireNavigationMenu();
	const item = requireNavigationItem();
	let unregister;
	onCleanup(() => unregister?.());
	const open = () => menu.openValue() === item.value;
	const forwarded = omit(props, "class", "children", "ref", "onClick");
	return createComponent$1(Button, mergeProps(forwarded, {
		role: "menuitem",
		"aria-haspopup": "dialog",
		get ["aria-expanded"]() {
			return open();
		},
		get disabled() {
			return props.disabled ?? item.disabled();
		},
		ref: (node) => {
			unregister?.();
			unregister = menu.registerTrigger({
				value: item.value,
				target: node,
				disabled: () => props.disabled ?? item.disabled()
			});
			props.ref?.(node);
		},
		variant: "ghost",
		get ["class"]() {
			return navigationMenuTriggerClass(open(), props.class);
		},
		onPointerEnter: () => {
			if (menu.openValue() !== null) menu.setOpenValue(item.value);
		},
		onKeyDown: (event) => {
			if (event.key === "ArrowDown") {
				menu.setOpenValue(item.value);
				event.preventDefault();
			} else if (event.key === "ArrowRight" || event.key === "ArrowLeft") {
				menu.move(item.value, event.key === "ArrowRight" ? "next" : "previous");
				event.preventDefault();
			}
			props.onKeyDown?.(event);
		},
		onClick: (event) => {
			menu.setOpenValue(open() ? null : item.value);
			props.onClick?.(event);
		},
		get children() {
			return [memo(() => {
				return props.children;
			}), createComponent$1(Icon, {
				"aria-hidden": "true",
				source: chevronDown,
				size: 14,
				class: "text-muted",
				get transform() {
					return memo(() => {
						return !!open();
					})() ? rotate2d$1(Math.PI) : void 0;
				}
			})];
		}
	}));
}
function NavigationMenuContent(props) {
	const menu = requireNavigationMenu();
	const item = requireNavigationItem();
	const forwarded = omit(props, "class", "children");
	const render = () => createComponent$1(View, mergeProps(forwarded, {
		role: "group",
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
	const unregister = menu.registerContent(item.value, render);
	onCleanup(() => unregister?.());
}
function NavigationMenuLink(props) {
	const menu = requireNavigationMenu();
	const forwarded = omit(props, "active", "closeOnSelect", "class", "onClick");
	return createComponent$1(Button, mergeProps(forwarded, {
		role: "link",
		variant: "ghost",
		get ["class"]() {
			return mergeClasses("w-full h-auto min-w-0 flex flex-col items-start gap-1 rounded-lg p-2 text-left", props.active && "bg-selected", props.class);
		},
		onClick: (event) => {
			if (props.closeOnSelect ?? true) menu.setOpenValue(null);
			props.onClick?.(event);
		},
		get children() {
			return createComponent$1(View, {
				class: "w-full min-w-0 flex flex-col items-start gap-1",
				get children() {
					return props.children;
				}
			});
		}
	}));
}
function NavigationMenuIndicator(props) {
	return createComponent$1(View, mergeProps(props, {
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("absolute left-1/2 bottom-0 w-2 h-0.5 rounded-full bg-accent", props.class);
		}
	}));
}
/** The shared Popover content already is the native viewport. */
function NavigationMenuViewport() {}
//#endregion
//#region src/components/range.ts
function finiteOr(value, fallback) {
	return value !== void 0 && Number.isFinite(value) ? value : fallback;
}
function normalizeRange(minValue, maxValue, stepValue) {
	const min = finiteOr(minValue, 0);
	const max = Math.max(min, finiteOr(maxValue, 100));
	const candidateStep = finiteOr(stepValue, 1);
	return {
		min,
		max,
		step: candidateStep > 0 ? candidateStep : 1
	};
}
function decimalPlaces(value) {
	const [coefficient, exponentText] = String(value).toLowerCase().split("e");
	const fractionLength = coefficient.split(".")[1]?.length ?? 0;
	const exponent = exponentText === void 0 ? 0 : Number(exponentText);
	return Math.max(0, Math.min(100, fractionLength - exponent));
}
//#endregion
//#region src/components/number-field-state.ts
function normalizeNumberFieldRange(minValue, maxValue, stepValue, largeStepValue) {
	const min = finiteOr(minValue, Number.NEGATIVE_INFINITY);
	const max = Math.max(min, finiteOr(maxValue, Number.POSITIVE_INFINITY));
	const candidateStep = finiteOr(stepValue, 1);
	const step = candidateStep > 0 ? candidateStep : 1;
	const candidateLargeStep = finiteOr(largeStepValue, step * 10);
	return {
		min,
		max,
		step,
		largeStep: candidateLargeStep > 0 ? candidateLargeStep : step * 10
	};
}
function clampNumberFieldValue(value, range) {
	return Math.max(range.min, Math.min(range.max, value));
}
function addNumberFieldStep(value, amount, range) {
	const precision = Math.max(decimalPlaces(value), decimalPlaces(amount));
	return clampNumberFieldValue(Number((value + amount).toFixed(precision)), range);
}
function numberFieldValueFromEmpty(direction, range) {
	if (direction > 0 && Number.isFinite(range.min)) return range.min;
	if (direction < 0 && Number.isFinite(range.max)) return range.max;
	return clampNumberFieldValue(0, range);
}
//#endregion
//#region src/components/number-field.tsx
/** Locale-aware numeric input with explicit native stepping semantics. */
function NumberField(props) {
	const host = useHost();
	const forwarded = omit(props, "value", "defaultValue", "min", "max", "step", "largeStep", "locale", "formatOptions", "placeholder", "changeOnWheel", "onValueChange", "class", "inputClass", "incrementLabel", "decrementLabel");
	const range = () => normalizeNumberFieldRange(props.min, props.max, props.step, props.largeStep);
	const locale = () => props.locale ?? host.intl.locale();
	const parser = createMemo(() => new NumberParser(locale(), props.formatOptions));
	const formatter = createMemo(() => new NumberFormatter(locale(), props.formatOptions));
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? null,
		disabled: () => Boolean(props.disabled || props.readOnly),
		onChange: props.onValueChange
	});
	const formattedValue = () => {
		const value = state.value();
		return value === null ? "" : formatter().format(value);
	};
	const [focused, setFocused] = createSignal(false);
	const [draft, setDraft] = createSignal(untrack(formattedValue));
	createEffect(() => ({
		focused: focused(),
		value: formattedValue()
	}), ({ focused: isFocused, value }) => {
		if (!isFocused) setDraft(value);
	});
	const update = (next) => {
		if (props.disabled || props.readOnly) return false;
		const normalized = next === null ? null : clampNumberFieldValue(next, range());
		const changed = state.set(normalized);
		setDraft(formattedValue());
		return changed;
	};
	const commitDraft = () => {
		const value = draft().trim();
		if (!value) {
			update(null);
			return;
		}
		const parsed = parser().parse(value);
		if (Number.isFinite(parsed)) update(parsed);
		else setDraft(formattedValue());
	};
	const changeBy = (direction, amount) => {
		const current = state.value();
		const next = current === null ? numberFieldValueFromEmpty(direction, range()) : addNumberFieldStep(current, direction * amount, range());
		update(next);
	};
	const canDecrement = () => {
		const value = state.value();
		return !props.disabled && !props.readOnly && (value === null || value > range().min);
	};
	const canIncrement = () => {
		const value = state.value();
		return !props.disabled && !props.readOnly && (value === null || value < range().max);
	};
	return createComponent$1(InputGroup, {
		get ["class"]() {
			return props.class;
		},
		get children() {
			return [
				createComponent$1(InputGroupButton, {
					size: "icon",
					class: "w-7 h-7 mx-0.5",
					get disabled() {
						return !canDecrement();
					},
					get ["aria-label"]() {
						return props.decrementLabel ?? `Decrease ${props["aria-label"]}`;
					},
					onClick: () => changeBy(-1, range().step),
					get children() {
						return createComponent$1(Icon, {
							source: minus,
							"aria-hidden": "true",
							size: 14
						});
					}
				}),
				createComponent$1(InputGroupInput, mergeProps(forwarded, {
					role: "spinbutton",
					get ["aria-label"]() {
						return props["aria-label"];
					},
					get ["aria-valuemin"]() {
						return memo(() => {
							return !!Number.isFinite(range().min);
						})() ? range().min : void 0;
					},
					get ["aria-valuemax"]() {
						return memo(() => {
							return !!Number.isFinite(range().max);
						})() ? range().max : void 0;
					},
					get ["aria-valuenow"]() {
						return state.value() ?? void 0;
					},
					get ["aria-valuetext"]() {
						return memo(() => {
							return state.value() === null;
						})() ? void 0 : formattedValue();
					},
					get value() {
						return draft();
					},
					get placeholder() {
						return props.placeholder;
					},
					get ["class"]() {
						return props.inputClass;
					},
					onFocus: (event) => {
						setFocused(true);
						props.onFocus?.(event);
					},
					onBlur: (event) => {
						commitDraft();
						setFocused(false);
						props.onBlur?.(event);
					},
					onInput: (event) => {
						const next = event.currentTarget.value;
						setDraft(next);
						if (!parser().isValidPartialNumber(next, range().min, range().max)) return;
						const parsed = parser().parse(next);
						if (Number.isFinite(parsed) && parsed >= range().min && parsed <= range().max) state.set(parsed);
					},
					onKeyDown: (event) => {
						props.onKeyDown?.(event);
						if (event.defaultPrevented || props.disabled || props.readOnly) return;
						if (match(event.key).with("ArrowUp", () => changeBy(1, range().step)).with("ArrowDown", () => changeBy(-1, range().step)).with("PageUp", () => changeBy(1, range().largeStep)).with("PageDown", () => changeBy(-1, range().largeStep)).with("Home", () => Number.isFinite(range().min) ? update(range().min) : false).with("End", () => Number.isFinite(range().max) ? update(range().max) : false).otherwise(() => false)) event.preventDefault();
					},
					onWheel: (event) => {
						props.onWheel?.(event);
						if (event.defaultPrevented || !props.changeOnWheel || !focused() || props.disabled || props.readOnly || event.deltaY === 0) return;
						changeBy(event.deltaY < 0 ? 1 : -1, range().step);
						event.preventDefault();
					}
				})),
				createComponent$1(InputGroupButton, {
					size: "icon",
					class: "w-7 h-7 mx-0.5",
					get disabled() {
						return !canIncrement();
					},
					get ["aria-label"]() {
						return props.incrementLabel ?? `Increase ${props["aria-label"]}`;
					},
					onClick: () => changeBy(1, range().step),
					get children() {
						return createComponent$1(Icon, {
							source: plus,
							"aria-hidden": "true",
							size: 14
						});
					}
				})
			];
		}
	});
}
//#endregion
//#region src/components/page.tsx
const pageViewportClass = (className) => mergeClasses("min-w-0 min-h-0 flex-1", className);
const pageViewportContentClass = (className) => mergeClasses("w-full", className);
/**
* A full-height application page boundary.
*
* This composes native scrolling with an explicitly sized content wrapper and
* optional identity-based scroll reset. Page implementations can therefore
* focus on their own layout instead of reconstructing flex/overflow rules.
*/
function PageViewport(props) {
	let viewport;
	createScrollReset({
		target: () => viewport,
		key: () => props.resetKey
	});
	return createComponent$1(ScrollArea, {
		get ["class"]() {
			return pageViewportClass(props.class);
		},
		get contentClass() {
			return pageViewportContentClass(props.contentClass);
		},
		get style() {
			return props.style;
		},
		get scrollbar() {
			return props.scrollbar;
		},
		get onScroll() {
			return props.onScroll;
		},
		ref: (node) => {
			viewport = node;
			props.ref?.(node);
		},
		get children() {
			return props.children;
		}
	});
}
const pageHeaderClass = (className, stacked = false) => mergeClasses("min-w-0 min-h-12 flex-none flex justify-between gap-4", stacked ? "flex-col items-stretch" : "flex-row items-center", className);
const pageHeaderTitleClass = () => "whitespace-nowrap text-2xl font-semibold text-primary";
const pageHeaderDescriptionClass = (stacked = false) => mergeClasses("text-sm text-secondary", stacked ? "whitespace-normal" : "truncate");
/** Consistent page title, supporting text and trailing application actions. */
function PageHeader(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return pageHeaderClass(props.class, props.stacked);
		},
		get children() {
			return [createComponent$1(View, {
				class: "min-w-0 flex-1 flex flex-row items-center gap-3",
				get children() {
					return [createComponent$1(View, {
						class: "min-w-0 flex flex-col gap-1",
						get children() {
							return [createComponent$1(Text, {
								role: "heading",
								get ["class"]() {
									return pageHeaderTitleClass();
								},
								get children() {
									return props.title;
								}
							}), createComponent$1(Show, {
								get when() {
									return props.description;
								},
								children: (description) => createComponent$1(Text, {
									get ["class"]() {
										return pageHeaderDescriptionClass(props.stacked);
									},
									get children() {
										return description();
									}
								})
							})];
						}
					}), memo(() => {
						return props.titleAdornment;
					})];
				}
			}), createComponent$1(Show, {
				get when() {
					return props.actions;
				},
				children: (actions) => createComponent$1(View, {
					class: "flex-none flex flex-row items-center gap-2",
					get classList() {
						return { "w-full": props.stacked };
					},
					get children() {
						return actions();
					}
				})
			})];
		}
	});
}
//#endregion
//#region src/components/progress.tsx
const ProgressContext = createContext();
function useProgressContext() {
	const context = useContext(ProgressContext);
	if (!context) throw new Error("Progress parts must be used inside ProgressRoot");
	return context;
}
function normalizeProgressValue(value, minValue, maxValue) {
	const min = finiteOr(minValue, 0);
	const requestedMax = finiteOr(maxValue, 100);
	const max = requestedMax > min ? requestedMax : min + 1;
	const normalizedValue = Math.max(min, Math.min(max, finiteOr(value, min)));
	return {
		value: normalizedValue,
		min,
		max,
		percent: (normalizedValue - min) / (max - min) * 100
	};
}
/** Semantic progress state with explicit, composable visual parts. */
function ProgressRoot(props) {
	const [trackWidth, setTrackWidth] = createSignal(0, { ownedWrite: true });
	const forwarded = omit(props, "value", "minValue", "maxValue", "indeterminate", "label", "getValueLabel", "children", "class");
	const details = () => normalizeProgressValue(props.value, props.minValue, props.maxValue);
	const indeterminate = () => props.indeterminate ?? false;
	const defaultValueLabel = () => `${Math.round(details().percent)} percent`;
	const context = {
		value: () => details().value,
		min: () => details().min,
		max: () => details().max,
		percent: () => details().percent,
		indeterminate,
		label: () => props.label ?? "Progress",
		valueLabel: () => indeterminate() ? void 0 : props.getValueLabel?.(details()) ?? defaultValueLabel(),
		trackWidth,
		setTrackWidth
	};
	return createComponent$1(ProgressContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				role: "progressbar",
				get ["aria-label"]() {
					return context.label();
				},
				get ["aria-valuemin"]() {
					return context.min();
				},
				get ["aria-valuemax"]() {
					return context.max();
				},
				get ["aria-valuenow"]() {
					return memo(() => {
						return !!indeterminate();
					})() ? void 0 : context.value();
				},
				get ["aria-valuetext"]() {
					return context.valueLabel();
				},
				get ["class"]() {
					return mergeClasses("w-full min-w-0 flex flex-col gap-2", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function ProgressTrack(props) {
	const context = useProgressContext();
	const measured = createMeasuredSize({ onChange: ({ width }) => context.setTrackWidth(width) });
	return createComponent$1(View, mergeProps(props, {
		ref: (node) => {
			measured.ref(node);
			props.ref?.(node);
		},
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("w-full h-2 flex-none overflow-hidden rounded-full bg-control", props.class);
		}
	}));
}
function IndeterminateProgressFill(props) {
	const context = useProgressContext();
	const reducedMotion = useReducedMotion();
	const sweep = createSweep({
		extent: context.trackWidth,
		itemRatio: .4,
		duration: 1.35,
		ease: "easeInOut",
		reducedMotion,
		reducedValue: .5
	});
	return createComponent$1(View, mergeProps(props, {
		"aria-hidden": "true",
		get ["class"]() {
			return mergeClasses("w-2/5 h-full rounded-full bg-accent", props.class);
		},
		get transform() {
			return sweep.transform();
		}
	}));
}
function ProgressFill(props) {
	const context = useProgressContext();
	return createComponent$1(Show, {
		get when() {
			return !context.indeterminate();
		},
		get fallback() {
			return createComponent$1(IndeterminateProgressFill, props);
		},
		get children() {
			return createComponent$1(View, mergeProps(props, {
				"aria-hidden": "true",
				get ["class"]() {
					return mergeClasses("h-full rounded-full bg-accent", props.class);
				},
				get style() {
					return {
						width: `${context.percent()}%`,
						...props.style
					};
				}
			}));
		}
	});
}
function ProgressLabel(props) {
	const context = useProgressContext();
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-w-0 text-sm text-secondary", props.class);
		},
		get children() {
			return props.children ?? context.label();
		}
	}));
}
function ProgressValueLabel(props) {
	const context = useProgressContext();
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("flex-none text-sm font-mono text-muted", props.class);
		},
		get children() {
			return props.children ?? context.valueLabel() ?? "In progress";
		}
	}));
}
/** Compact progress bar; use ProgressRoot and parts for custom composition. */
function Progress(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(ProgressRoot, mergeProps(forwarded, { get children() {
		return createComponent$1(ProgressTrack, {
			get ["class"]() {
				return props.class;
			},
			get children() {
				return createComponent$1(ProgressFill, {});
			}
		});
	} }));
}
//#endregion
//#region src/components/prompt-suggestion.tsx
/** Container-responsive starter actions for empty conversations and assistants. */
function PromptSuggestions(props) {
	const rest = omit(props, "minColumnWidth", "maxColumns", "gap", "class");
	return createComponent$1(ResponsiveGrid, mergeProps(rest, {
		get minColumnWidth() {
			return props.minColumnWidth ?? 176;
		},
		get maxColumns() {
			return props.maxColumns ?? 3;
		},
		initialColumns: 1,
		get gap() {
			return props.gap ?? 8;
		},
		get ["class"]() {
			return mergeClasses("w-full", props.class);
		}
	}));
}
/** One explicit prompt choice; the application remains responsible for its payload. */
function PromptSuggestion(props) {
	const rest = omit(props, "title", "description", "icon", "class");
	return createComponent$1(Button, mergeProps(rest, {
		get ["aria-label"]() {
			return props["aria-label"] ?? props.title;
		},
		get variant() {
			return props.variant ?? "outline";
		},
		get ["class"]() {
			return mergeClasses("w-full h-auto min-h-14 min-w-0 items-start justify-start gap-2 border-subtle bg-transparent p-3 text-left shadow-none", props.class);
		},
		get children() {
			return [createComponent$1(Show, {
				get when() {
					return props.icon;
				},
				children: (source) => createComponent$1(Icon, {
					get source() {
						return source();
					},
					size: 15,
					class: "flex-none text-accent"
				})
			}), createComponent$1(View, {
				class: "min-w-0 flex-1 flex flex-col gap-1",
				get children() {
					return [createComponent$1(Text, {
						class: "whitespace-normal text-sm font-medium text-primary",
						get children() {
							return props.title;
						}
					}), createComponent$1(Show, {
						get when() {
							return props.description;
						},
						children: (description) => createComponent$1(Text, {
							class: "whitespace-normal text-xs text-secondary",
							get children() {
								return description();
							}
						})
					})];
				}
			})];
		}
	}));
}
//#endregion
//#region src/components/property-list.tsx
function PropertyList(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "table",
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex flex-col overflow-hidden rounded-xl border border-subtle bg-surface", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
/** Compact two-column data intended for inspectors and settings summaries. */
function PropertyRow(props) {
	return createComponent$1(View, {
		role: "row",
		get ["aria-label"]() {
			return props.name;
		},
		get ["class"]() {
			return mergeClasses("min-h-10 w-full min-w-0 px-3 py-2 flex flex-row items-center gap-3 border-b border-subtle", props.class);
		},
		get children() {
			return [createComponent$1(Text, {
				role: "cell",
				get ["class"]() {
					return mergeClasses("w-48 flex-none text-xs font-mono text-accent whitespace-nowrap", props.nameClass);
				},
				get children() {
					return props.name;
				}
			}), createComponent$1(Text, {
				role: "cell",
				get ["class"]() {
					return mergeClasses("min-w-0 flex-1 text-xs text-muted whitespace-normal", props.valueClass);
				},
				get children() {
					return props.value;
				}
			})];
		}
	});
}
//#endregion
//#region src/components/qr-code.tsx
function positive(name, value) {
	if (!Number.isFinite(value) || value <= 0) throw new RangeError(`${name} must be a positive finite number`);
	return value;
}
function quietZone(value) {
	const resolved = value ?? 4;
	if (!Number.isInteger(resolved) || resolved < 0) throw new RangeError("QRCode quietZone must be a non-negative integer");
	return resolved;
}
/** Encode with uqr while keeping its renderer-independent matrix contract. */
function encodeQrCode(value, errorCorrection = "M") {
	const encoded = encode(value, {
		border: 0,
		ecc: errorCorrection
	});
	return {
		size: encoded.size,
		data: encoded.data
	};
}
function rectangle(path, x, y, width, height) {
	path.moveTo(x, y).lineTo(x + width, y).lineTo(x + width, y + height).lineTo(x, y + height).close();
}
/**
* Convert consecutive dark modules into one retained Vello path. Horizontal
* runs keep the bridge traffic and native scene node count independent of the
* number of QR modules.
*/
function qrCodePath(matrix, renderedSize, quiet, foreground = 255) {
	positive("QRCode size", renderedSize);
	const border = quietZone(quiet);
	const moduleSize = renderedSize / (matrix.size + border * 2);
	const path = new PathBuilder();
	for (let row = 0; row < matrix.size; row++) {
		const modules = matrix.data[row];
		for (let column = 0; column < matrix.size;) {
			if (!modules[column]) {
				column++;
				continue;
			}
			const start = column;
			while (column < matrix.size && modules[column]) column++;
			rectangle(path, (border + start) * moduleSize, (border + row) * moduleSize, (column - start) * moduleSize, moduleSize);
		}
	}
	return path.build({ fill: foreground });
}
/** A QR encoder from the JS ecosystem rendered as one native vector path. */
function QRCode(props) {
	const rest = omit(props, "value", "size", "errorCorrection", "quietZone", "foreground", "background", "style", "class");
	const renderedSize = () => positive("QRCode size", props.size ?? 180);
	const matrix = createMemo(() => encodeQrCode(props.value, props.errorCorrection));
	const source = createMemo(() => qrCodePath(matrix(), renderedSize(), quietZone(props.quietZone), props.foreground));
	return createComponent$1(View, mergeProps(rest, {
		role: "img",
		get ["aria-label"]() {
			return props["aria-label"] ?? "QR code";
		},
		get ["class"]() {
			return mergeClasses("relative flex-none overflow-hidden", props.class);
		},
		get style() {
			return {
				...props.style,
				width: renderedSize(),
				height: renderedSize(),
				"background-color": props.background ?? rgba$1(4294967295)
			};
		},
		get children() {
			return createComponent$1(Path, {
				"aria-hidden": "true",
				class: "absolute inset-0",
				get style() {
					return {
						width: renderedSize(),
						height: renderedSize()
					};
				},
				get source() {
					return source();
				}
			});
		}
	}));
}
//#endregion
//#region src/components/rating-state.ts
function normalizeRatingMax(max) {
	if (max === void 0 || !Number.isFinite(max)) return 5;
	return Math.max(1, Math.min(20, Math.floor(max)));
}
function clampRatingValue(value, max) {
	if (value === void 0 || !Number.isFinite(value)) return 0;
	return Math.max(0, Math.min(normalizeRatingMax(max), Math.round(value)));
}
function ratingLabel(value) {
	return `${value} ${value === 1 ? "star" : "stars"}`;
}
//#endregion
//#region src/components/rating.tsx
function RatingIcon(props) {
	const reducedMotion = useReducedMotion();
	const emphasis = createTransition(() => props.highlighted ? 1 : 0, {
		duration: .12,
		ease: "easeOut",
		reducedMotion
	});
	return createComponent$1(Icon, {
		"aria-hidden": "true",
		source: star,
		get size() {
			return props.size;
		},
		get fill() {
			return props.highlighted ? "currentColor" : "none";
		},
		get ["class"]() {
			return props.highlighted ? "text-accent" : "text-muted";
		},
		get transform() {
			return scale2d(.9 + emphasis.value() * .1);
		}
	});
}
function Rating(props) {
	const max = () => normalizeRatingMax(props.max);
	const normalize = (value) => clampRatingValue(value, max());
	const state = createControllableState({
		value: () => props.value === void 0 ? void 0 : normalize(props.value),
		defaultValue: normalize(props.defaultValue),
		disabled: () => (props.disabled ?? false) || (props.readOnly ?? false),
		onChange: props.onValueChange
	});
	const value = () => normalize(state.value());
	const [preview, setPreview] = createSignal();
	let previewGeneration = 0;
	const previewItem = (item) => {
		previewGeneration++;
		setPreview(item);
	};
	const clearPreviewAfterPointerDispatch = () => {
		const generation = ++previewGeneration;
		queueMicrotask(() => {
			if (generation === previewGeneration) setPreview(void 0);
		});
	};
	const shownValue = () => preview() ?? value();
	const disabled = () => props.disabled ?? false;
	const inert = () => disabled() || (props.readOnly ?? false);
	const items = () => Array.from({ length: max() }, (_, index) => index + 1);
	const select = (next) => {
		if (inert()) return;
		const normalized = normalize(next);
		state.set(props.allowClear && normalized === value() ? 0 : normalized);
	};
	const roving = createRovingFocus({
		orientation: () => "horizontal",
		onMove: (id) => select(Number(id))
	});
	return createComponent$1(View, {
		role: "radiogroup",
		get ["aria-label"]() {
			return props.label;
		},
		get ["aria-disabled"]() {
			return disabled() || void 0;
		},
		"aria-orientation": "horizontal",
		get ["class"]() {
			return mergeClasses("flex flex-col items-start gap-1.5", props.class);
		},
		get style() {
			return { opacity: disabled() ? .45 : 1 };
		},
		get children() {
			return createComponent$1(View, {
				class: "flex flex-row items-center gap-0.5",
				get children() {
					return createComponent$1(For, {
						get each() {
							return items();
						},
						children: (item) => {
							const checked = () => value() === item;
							const highlighted = () => item <= shownValue();
							let unregister;
							onCleanup(() => unregister?.());
							return createComponent$1(Button$1, {
								unstyled: true,
								role: "radio",
								get ["aria-label"]() {
									return ratingLabel(item);
								},
								get ["aria-checked"]() {
									return checked();
								},
								get selected() {
									return checked();
								},
								get disabled() {
									return disabled();
								},
								get focusOrder() {
									return memo(() => {
										return !!disabled();
									})() ? -1 : checked() || value() === 0 && item === 1 ? 0 : -1;
								},
								ref: (node) => {
									unregister?.();
									unregister = roving.register({
										id: String(item),
										target: node,
										disabled: inert
									});
								},
								class: (buttonState) => mergeClasses("w-8 h-8 items-center justify-center rounded-md border border-transparent", match({
									focused: buttonState.focusVisible,
									hovered: buttonState.hovered
								}).with({ focused: true }, () => "border-focus bg-control").with({ hovered: true }, () => "bg-control-hover").with({
									focused: false,
									hovered: false
								}, () => "bg-transparent").exhaustive()),
								onPointerEnter: () => !inert() && previewItem(item),
								onPointerLeave: clearPreviewAfterPointerDispatch,
								onClick: () => select(item),
								onKeyDown: (event) => {
									if (match(event.key).with(P.union("Home", "End", "ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"), () => roving.move(String(item), event.key)).otherwise(() => false)) event.preventDefault();
								},
								get children() {
									return createComponent$1(RatingIcon, {
										get highlighted() {
											return highlighted();
										},
										get size() {
											return props.size ?? 20;
										}
									});
								}
							});
						}
					});
				}
			});
		}
	});
}
//#endregion
//#region src/components/reasoning.tsx
/** Quiet disclosure surface for model reasoning or intermediate thought. */
function Reasoning(props) {
	const rest = omit(props, "class");
	return createComponent$1(Collapsible, mergeProps(rest, { get ["class"]() {
		return mergeClasses("w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface-muted", props.class);
	} }));
}
function ReasoningTrigger(props) {
	const label = () => props.label ?? (props.streaming ? "Thinking" : "Reasoning");
	const rest = omit(props, "label", "streaming", "class", "aria-label");
	return createComponent$1(CollapsibleTrigger, mergeProps(rest, {
		get ["aria-label"]() {
			return props["aria-label"] ?? label();
		},
		get ["class"]() {
			return mergeClasses("min-h-9 px-3 py-1.5 text-left", props.class);
		},
		get children() {
			return [
				createComponent$1(Icon, {
					source: sparkles,
					size: 13,
					class: "flex-none text-muted"
				}),
				createComponent$1(Text, {
					class: "min-w-0 flex-1 text-sm font-medium text-secondary",
					get children() {
						return label();
					}
				}),
				memo(() => {
					return memo(() => {
						return !!props.streaming;
					})() ? createComponent$1(Pulse, {
						"aria-hidden": "true",
						class: "w-1.5 h-1.5 flex-none rounded-full bg-accent",
						from: .3,
						to: 1,
						duration: .8
					}) : props.streaming;
				})
			];
		}
	}));
}
function ReasoningContent(props) {
	const rest = omit(props, "class");
	return createComponent$1(CollapsibleContent, mergeProps(rest, { get ["class"]() {
		return mergeClasses("min-w-0 border-t border-subtle px-3 py-2", props.class);
	} }));
}
//#endregion
//#region src/components/resizable.tsx
function finitePercentage(value, name) {
	if (!Number.isFinite(value) || value < 0 || value > 100) throw new RangeError(`${name} must be a finite percentage from 0 to 100`);
	return value;
}
function validateResizableSizes(panels, sizes) {
	if (panels.length < 2) throw new RangeError("resizable panels require at least two definitions");
	const ids = /* @__PURE__ */ new Set();
	const candidates = panels.map((panel) => {
		if (!panel.id || ids.has(panel.id)) throw new Error(`resizable panel id must be unique: ${panel.id}`);
		ids.add(panel.id);
		const min = finitePercentage(panel.minSize ?? 0, `${panel.id}.minSize`);
		const max = finitePercentage(panel.maxSize ?? 100, `${panel.id}.maxSize`);
		if (min > max) throw new RangeError(`${panel.id}.minSize exceeds maxSize`);
		const candidate = finitePercentage(sizes?.[panel.id] ?? panel.defaultSize, `${panel.id}.size`);
		if (candidate < min || candidate > max) throw new RangeError(`${panel.id}.size is outside its min/max range`);
		return candidate;
	});
	const total = candidates.reduce((sum, value) => sum + value, 0);
	if (Math.abs(total - 100) > .001) throw new RangeError(`resizable panel sizes must total 100; received ${total}`);
	return Object.fromEntries(panels.map((panel, index) => [panel.id, candidates[index]]));
}
function createResizablePanelState(options) {
	const defaults = validateResizableSizes(options.panels, options.defaultValue);
	const definitions = new Map(options.panels.map((panel) => [panel.id, panel]));
	const state = createControllableState({
		value: () => {
			const value = options.value?.();
			return value === void 0 ? void 0 : validateResizableSizes(options.panels, value);
		},
		defaultValue: defaults,
		onChange: options.onValueChange
	});
	const requirePanel = (id) => {
		const panel = definitions.get(id);
		if (!panel) throw new Error(`unknown resizable panel: ${id}`);
		return panel;
	};
	const size = (id) => {
		requirePanel(id);
		const value = state.value()[id];
		if (!Number.isFinite(value)) throw new Error(`missing resizable panel size: ${id}`);
		return value;
	};
	const pairRange = (before, after) => {
		const beforePanel = requirePanel(before);
		const afterPanel = requirePanel(after);
		if (before === after) throw new Error("resizable handle requires two panels");
		const pair = size(before) + size(after);
		return {
			min: Math.max(beforePanel.minSize ?? 0, pair - (afterPanel.maxSize ?? 100)),
			max: Math.min(beforePanel.maxSize ?? 100, pair - (afterPanel.minSize ?? 0))
		};
	};
	const resizePair = (before, after, beforeSize) => {
		const range = pairRange(before, after);
		const pair = size(before) + size(after);
		const nextBefore = Math.max(range.min, Math.min(range.max, beforeSize));
		const nextAfter = pair - nextBefore;
		const current = state.value();
		if (current[before] === nextBefore && current[after] === nextAfter) return false;
		return state.set({
			...current,
			[before]: nextBefore,
			[after]: nextAfter
		});
	};
	return {
		panels: options.panels,
		sizes: state.value,
		size,
		pairRange,
		resizePair,
		resetPair(before, after) {
			const beforeDefault = defaults[before];
			const afterDefault = defaults[after];
			if (beforeDefault === void 0 || afterDefault === void 0) {
				pairRange(before, after);
				return false;
			}
			const pair = size(before) + size(after);
			return resizePair(before, after, pair * (beforeDefault / (beforeDefault + afterDefault)));
		}
	};
}
const ResizableContext = createContext();
function useResizable() {
	const context = useContext(ResizableContext);
	if (!context) throw new Error("Resizable parts must be inside ResizablePanelGroup");
	return context;
}
function ResizablePanelGroup(props) {
	let measuredWidth = 0;
	let measuredHeight = 0;
	const measured = createMeasuredSize({ onChange(size) {
		measuredWidth = size.width;
		measuredHeight = size.height;
	} });
	const state = createResizablePanelState({
		panels: props.panels,
		value: () => props.value,
		defaultValue: props.defaultValue,
		onValueChange: props.onValueChange
	});
	const direction = () => props.direction ?? "horizontal";
	return createComponent$1(ResizableContext, {
		value: {
			direction,
			state,
			axisSize: () => direction() === "horizontal" ? measuredWidth : measuredHeight
		},
		get children() {
			return createComponent$1(View, {
				ref(r$) {
					var _ref$ = measured.ref;
					typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
				},
				role: "group",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["class"]() {
					return mergeClasses("w-full h-full min-w-0 min-h-0 flex overflow-hidden", direction() === "horizontal" ? "flex-row" : "flex-col", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function ResizablePanel(props) {
	const context = useResizable();
	const style = createMemo(() => context.direction() === "horizontal" ? { width: `${context.state.size(props.id)}%` } : { height: `${context.state.size(props.id)}%` });
	return createComponent$1(View, {
		role: "group",
		get ["aria-label"]() {
			return props.id;
		},
		get ["class"]() {
			return mergeClasses("min-w-0 min-h-0 flex-none overflow-hidden", props.class);
		},
		get style() {
			return style();
		},
		get children() {
			return props.children;
		}
	});
}
function ResizableHandle(props) {
	const context = useResizable();
	const [dragging, setDragging] = createSignal(false);
	const [hovered, setHovered] = createSignal(false);
	let startCoordinate = 0;
	let startSize = 0;
	const coordinate = (event) => context.direction() === "horizontal" ? event.clientX : event.clientY;
	const range = () => context.state.pairRange(props.before, props.after);
	const moveTo = (value) => context.state.resizePair(props.before, props.after, value);
	const onPointerDown = (event) => {
		if (event.button !== 0) return;
		event.preventDefault();
		startCoordinate = coordinate(event);
		startSize = context.state.size(props.before);
		setDragging(true);
	};
	const onPointerMove = (event) => {
		if (!dragging() || event.buttons === 0) return;
		const axisSize = context.axisSize();
		if (axisSize <= 0) return;
		moveTo(startSize + (coordinate(event) - startCoordinate) / axisSize * 100);
	};
	const stopDragging = () => setDragging(false);
	const onKeyDown = (event) => {
		const step = Math.max(.1, props.keyboardStep ?? 2);
		const current = context.state.size(props.before);
		const next = event.key === "Home" ? range().min : event.key === "End" ? range().max : event.key === (context.direction() === "horizontal" ? "ArrowLeft" : "ArrowUp") ? current - step : event.key === (context.direction() === "horizontal" ? "ArrowRight" : "ArrowDown") ? current + step : void 0;
		if (next === void 0) return;
		event.preventDefault();
		moveTo(next);
	};
	return createComponent$1(View, {
		role: "separator",
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-valuemin"]() {
			return range().min;
		},
		get ["aria-valuemax"]() {
			return range().max;
		},
		get ["aria-valuenow"]() {
			return context.state.size(props.before);
		},
		get ["aria-valuetext"]() {
			return `${Math.round(context.state.size(props.before))} percent`;
		},
		focusOrder: 0,
		get ["class"]() {
			return mergeClasses("flex-none rounded-sm", context.direction() === "horizontal" ? "w-2 h-full" : "w-full h-2", dragging() || hovered() ? "bg-accent" : "bg-control", props.class);
		},
		onPointerEnter: () => setHovered(true),
		onPointerLeave: () => setHovered(false),
		onPointerDown,
		onPointerMove,
		onPointerUp: stopDragging,
		onPointerCancel: stopDragging,
		onDblClick: () => context.state.resetPair(props.before, props.after),
		onKeyDown
	});
}
//#endregion
//#region src/components/search-field.tsx
/** A native search input with consistent clear, Escape, and submit behavior. */
function SearchField(props) {
	const forwarded = omit(props, "value", "defaultValue", "onValueChange", "onSearch", "onClear", "clearLabel", "class", "surfaceClass", "inputClass", "inputRef");
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? "",
		onChange: props.onValueChange
	});
	let input;
	const clear = () => {
		if (!state.value() || props.disabled || props.readOnly) return false;
		if (!state.set("")) return false;
		props.onClear?.();
		input?.focus();
		return true;
	};
	return createComponent$1(InputGroup, {
		get ["class"]() {
			return props.class;
		},
		get surfaceClass() {
			return props.surfaceClass;
		},
		get children() {
			return [
				createComponent$1(View, {
					"aria-hidden": "true",
					class: "flex-none pl-2.5 flex items-center text-muted",
					get children() {
						return createComponent$1(Icon, {
							source: search,
							size: 15
						});
					}
				}),
				createComponent$1(InputGroupInput, mergeProps(forwarded, {
					ref: (node) => {
						input = node;
						props.inputRef?.(node);
					},
					get value() {
						return state.value();
					},
					get ["class"]() {
						return props.inputClass;
					},
					onInput: (event) => state.set(event.currentTarget.value),
					onKeyDown: (event) => {
						props.onKeyDown?.(event);
						if (event.key === "Escape" && clear()) event.preventDefault();
						if (event.key === "Enter") props.onSearch?.(state.value());
					}
				})),
				createComponent$1(Show, {
					get when() {
						return memo(() => {
							return !!(state.value() && !props.disabled);
						})() ? !props.readOnly : state.value() && !props.disabled;
					},
					get children() {
						return createComponent$1(InputGroupButton, {
							size: "icon",
							get ["aria-label"]() {
								return props.clearLabel ?? "Clear search";
							},
							onClick: clear,
							get children() {
								return createComponent$1(Icon, {
									source: x,
									"aria-hidden": "true",
									size: 15
								});
							}
						});
					}
				})
			];
		}
	});
}
//#endregion
//#region src/components/selection.tsx
const SELECTION_INDICATOR_CLASS = "w-5 h-5 flex-none border";
function Checkbox(props) {
	const state = createControllableState({
		value: () => props.checked,
		defaultValue: props.defaultChecked ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onCheckedChange
	});
	const checked = state.value;
	const toggle = () => {
		state.set(!checked());
	};
	const boxColors = () => match({
		checked: checked(),
		indeterminate: !!props.indeterminate
	}).with({ checked: true }, () => "bg-accent border-accent text-on-accent").with({ indeterminate: true }, () => "bg-accent border-accent text-on-accent").otherwise(() => "bg-input border-strong text-primary");
	const ariaChecked = () => match({
		checked: checked(),
		indeterminate: !!props.indeterminate
	}).with({ indeterminate: true }, () => "mixed").otherwise(({ checked }) => checked);
	const indicator = () => match({
		checked: checked(),
		indeterminate: !!props.indeterminate
	}).with({ indeterminate: true }, () => minus).with({ checked: true }, () => check).otherwise(() => void 0);
	return createComponent$1(Button$1, {
		unstyled: true,
		role: "checkbox",
		get disabled() {
			return props.disabled;
		},
		get ["aria-label"]() {
			return props["aria-label"] ?? props.label;
		},
		get ["aria-checked"]() {
			return ariaChecked();
		},
		get selected() {
			return checked();
		},
		class: (buttonState) => mergeClasses("min-h-7 px-1 items-center gap-2 rounded-md border border-transparent", buttonState.hovered && "bg-control-hover", buttonState.focusVisible && "border-focus", props.class),
		style: (buttonState) => ({ opacity: buttonState.disabled ? .45 : 1 }),
		onClick: toggle,
		get children() {
			return [createComponent$1(Center, {
				"aria-hidden": "true",
				get ["class"]() {
					return mergeClasses(SELECTION_INDICATOR_CLASS, "rounded text-xs font-bold", boxColors());
				},
				get children() {
					return memo(() => {
						return !!indicator();
					})() ? createComponent$1(Icon, {
						get source() {
							return indicator();
						},
						size: 14,
						class: "text-on-accent"
					}) : indicator();
				}
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent$1(Text, {
					class: "text-sm text-secondary",
					get children() {
						return props.label;
					}
				}) : props.label;
			})];
		}
	});
}
const RadioContext = createContext();
function RadioGroup(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		disabled: () => props.disabled ?? false,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const value = state.value;
	const select = (next) => {
		state.set(next);
	};
	const roving = createRovingFocus({
		orientation: () => "vertical",
		onMove: select
	});
	return createComponent(RadioContext, {
		value: {
			value,
			select,
			disabled: () => props.disabled ?? false,
			register: (id, target, disabled) => roving.register({
				id,
				target,
				disabled
			}),
			move: roving.move
		},
		get children() {
			return createComponent$1(View, {
				role: "radiogroup",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["class"]() {
					return mergeClasses("flex flex-col gap-3", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function RadioGroupItem(props) {
	const group = useContext(RadioContext);
	if (!group) throw new Error("RadioGroupItem must be used inside RadioGroup");
	const checked = () => group.value() === props.value;
	const disabled = () => group.disabled() || (props.disabled ?? false);
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button$1, {
		unstyled: true,
		role: "radio",
		get disabled() {
			return disabled();
		},
		get selected() {
			return checked();
		},
		get ["aria-label"]() {
			return props.label;
		},
		get ["aria-checked"]() {
			return checked();
		},
		ref: (node) => {
			unregister?.();
			unregister = group.register(props.value, node, disabled);
		},
		class: (buttonState) => mergeClasses("min-h-7 px-1 items-center gap-2 rounded-md border border-transparent", buttonState.hovered && "bg-control-hover", buttonState.focusVisible && "border-focus", props.class),
		style: (buttonState) => ({ opacity: buttonState.disabled ? .45 : 1 }),
		onClick: () => group.select(props.value),
		onKeyDown: (event) => {
			if (group.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return [createComponent$1(Center, {
				"aria-hidden": "true",
				get ["class"]() {
					return mergeClasses(SELECTION_INDICATOR_CLASS, "rounded-full bg-input", match(checked()).with(true, () => "border-accent").with(false, () => "border-strong").exhaustive());
				},
				get children() {
					return memo(() => {
						return !!checked();
					})() ? createComponent$1(View, { class: "w-2.5 h-2.5 rounded-full bg-accent" }) : checked();
				}
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent$1(Text, {
					class: "text-sm text-secondary",
					get children() {
						return props.label;
					}
				}) : props.label;
			})];
		}
	});
}
function Toggle(props) {
	const state = createControllableState({
		value: () => props.pressed,
		defaultValue: props.defaultPressed ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onPressedChange
	});
	const pressed = state.value;
	const toggle = () => {
		state.set(!pressed());
	};
	const size = () => match(props.size ?? "default").with("sm", () => "h-6 min-w-6 px-2 text-xs").with("default", () => "h-8 min-w-8 px-2.5 text-sm").with("lg", () => "h-10 min-w-10 px-3 text-sm").exhaustive();
	const colors = (state) => match({
		selected: pressed(),
		hovered: state.hovered
	}).with({ selected: true }, () => "bg-selected border-accent text-primary").with({ hovered: true }, () => "bg-control-hover text-primary").otherwise(() => "bg-transparent text-secondary");
	return createComponent$1(Button$1, {
		unstyled: true,
		get disabled() {
			return props.disabled;
		},
		get selected() {
			return pressed();
		},
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-pressed"]() {
			return pressed();
		},
		class: (state) => mergeClasses("items-center justify-center rounded-md border font-medium", size(), colors(state), match(props.variant ?? "default").with("outline", () => "border-strong").with("default", () => "border-transparent").exhaustive(), state.focusVisible && "border-focus", props.class),
		style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
		onClick: toggle,
		get children() {
			return props.children;
		}
	});
}
const ToggleGroupContext = createContext();
function nextToggleGroupValue(current, value, type) {
	if (type === "single") return current === value ? "" : value;
	const values = Array.isArray(current) ? current : [];
	return values.includes(value) ? values.filter((candidate) => candidate !== value) : [...values, value];
}
/** Shadcn-style single-value toggle group with native roving focus. */
function ToggleGroup(props) {
	const entries = [];
	const [activeValue, setActiveValue] = createSignal(void 0, { ownedWrite: true });
	const [registryVersion, setRegistryVersion] = createSignal(0, { ownedWrite: true });
	const type = () => props.type ?? "single";
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? (props.type === "multiple" ? [] : ""),
		disabled: () => props.disabled ?? false,
		onChange: (value) => {
			if (props.type === "multiple") props.onValueChange?.(Array.isArray(value) ? value : []);
			else props.onValueChange?.(typeof value === "string" ? value : "");
		}
	});
	const activateFromKeyboard = (value) => {
		setActiveValue(value);
		if (type() === "single" && state.value() !== value) state.set(value);
	};
	const roving = createRovingFocus({
		orientation: () => "horizontal",
		loop: props.loop,
		onMove: activateFromKeyboard
	});
	const context = {
		selected(value) {
			const current = state.value();
			return Array.isArray(current) ? current.includes(value) : current === value;
		},
		disabled: () => props.disabled ?? false,
		toggle: (value) => state.set(nextToggleGroupValue(state.value(), value, type())),
		register(value, node, disabled) {
			const entry = {
				value,
				disabled
			};
			entries.push(entry);
			const unregisterRoving = roving.register({
				id: value,
				target: node,
				disabled
			});
			setRegistryVersion((version) => version + 1);
			return () => {
				unregisterRoving();
				const index = entries.indexOf(entry);
				if (index >= 0) entries.splice(index, 1);
				setRegistryVersion((version) => version + 1);
			};
		},
		activate: setActiveValue,
		isTabStop(value) {
			registryVersion();
			const enabled = entries.filter((entry) => !entry.disabled());
			const active = activeValue();
			return value === (enabled.some((entry) => entry.value === active) ? active : enabled.find((entry) => context.selected(entry.value))?.value ?? enabled[0]?.value);
		},
		move: roving.move,
		variant: () => props.variant ?? "default",
		size: () => props.size ?? "default"
	};
	return createComponent(ToggleGroupContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				role: "group",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["class"]() {
					return mergeClasses("flex flex-row items-center rounded-md bg-transparent", match(props.spacing ?? 0).with(0, () => "gap-0").with(1, () => "gap-1").with(2, () => "gap-2").exhaustive(), props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function ToggleGroupItem(props) {
	const group = useContext(ToggleGroupContext);
	if (!group) throw new Error("ToggleGroupItem must be used inside ToggleGroup");
	const selected = () => group.selected(props.value);
	const disabled = () => group.disabled() || (props.disabled ?? false);
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button$1, {
		unstyled: true,
		get disabled() {
			return disabled();
		},
		get selected() {
			return selected();
		},
		get ["aria-pressed"]() {
			return selected();
		},
		get focusOrder() {
			return group.isTabStop(props.value) ? 0 : -1;
		},
		ref: (node) => {
			unregister?.();
			unregister = group.register(props.value, node, disabled);
		},
		class: (state) => mergeClasses("h-7 flex-1 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium", match(props.size ?? group.size()).with("sm", () => "h-6 px-2 text-xs").with("default", () => "h-8 px-3 text-sm").with("lg", () => "h-10 px-4 text-sm").exhaustive(), match({
			selected: selected(),
			accent: props.variant === "accent",
			hovered: state.hovered
		}).with({
			selected: true,
			accent: true
		}, () => "bg-accent text-on-accent").with({ selected: true }, () => "bg-selected text-primary").with({ hovered: true }, () => "bg-control-hover text-primary").otherwise(() => "bg-transparent text-muted"), (props.variant ?? group.variant()) === "outline" && "border-strong", state.focusVisible && "border-focus", props.class),
		style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
		onFocus: () => group.activate(props.value),
		onClick: () => group.toggle(props.value),
		onKeyDown: (event) => {
			if (group.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/components/sheet.tsx
const geometry = (side) => match(side).with("left", () => ({
	backdrop: {
		"align-items": "stretch",
		"justify-content": "flex-start"
	},
	content: "h-full w-[400px] max-w-full border-r"
})).with("right", () => ({
	backdrop: {
		"align-items": "stretch",
		"justify-content": "flex-end"
	},
	content: "h-full w-[400px] max-w-full border-l"
})).with("top", () => ({
	backdrop: {
		"align-items": "flex-start",
		"justify-content": "stretch"
	},
	content: "w-full max-h-[80%] border-b"
})).with("bottom", () => ({
	backdrop: {
		"align-items": "flex-end",
		"justify-content": "stretch"
	},
	content: "w-full max-h-[80%] border-t"
})).exhaustive();
const sheetMotion = (side) => match(side).with("left", () => ({
	duration: .18,
	fromX: -32
})).with("right", () => ({
	duration: .18,
	fromX: 32
})).with("top", () => ({
	duration: .18,
	fromY: -32
})).with("bottom", () => ({
	duration: .18,
	fromY: 32
})).exhaustive();
/** A modal edge panel that shares native focus isolation with Dialog. */
function Sheet(props) {
	const theme = useComponentsTheme();
	const side = () => props.side ?? "right";
	const placement = () => geometry(side());
	return createComponent$1(Modal, mergeProps(props, {
		get motion() {
			return memo(() => {
				return props.motion === void 0;
			})() ? sheetMotion(side()) : props.motion;
		},
		get backdropStyle() {
			return {
				"background-color": rgba(51),
				...placement().backdrop,
				...props.backdropStyle
			};
		},
		get contentClass() {
			return mergeClasses("min-w-0 min-h-0 flex flex-col gap-4 border-subtle bg-surface p-5", placement().content, props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "modal") : props.contentShadows;
		}
	}));
}
//#endregion
//#region src/components/shortcut-recorder.tsx
const MOD_SHIFT = 1;
const MOD_CONTROL = 2;
const MOD_ALT = 4;
const MOD_META = 8;
const MODIFIER_KEYS = /* @__PURE__ */ new Set([
	"Shift",
	"Control",
	"Alt",
	"Meta"
]);
function shortcutFromKeyEvent(event) {
	if (MODIFIER_KEYS.has(event.key)) return void 0;
	const mods = event.mods ?? 0;
	const parts = [];
	if (event.primary) parts.push("Primary");
	if (!event.primary && (mods & MOD_CONTROL) !== 0) parts.push("Control");
	if ((mods & MOD_ALT) !== 0) parts.push("Alt");
	if ((mods & MOD_SHIFT) !== 0) parts.push("Shift");
	if (!event.primary && (mods & MOD_META) !== 0) parts.push("Meta");
	const key = match(event.key).with(" ", () => "Space").with("ArrowUp", () => "Up").with("ArrowDown", () => "Down").with("ArrowLeft", () => "Left").with("ArrowRight", () => "Right").otherwise((value) => value.length === 1 ? value.toUpperCase() : value);
	parts.push(key);
	return {
		chord: parts.join("+"),
		parts
	};
}
function ShortcutRecorder(props) {
	const [local, setLocal] = createSignal(props.defaultValue ?? "");
	const [recording, setRecording] = createSignal(false);
	const value = () => props.value ?? local();
	const setValue = (next) => {
		if (props.value === void 0) setLocal(next);
		props.onValueChange?.(next);
	};
	const handleKeyDown = (event) => {
		if (!recording()) return;
		event.preventDefault();
		if (event.key === "Escape") {
			setRecording(false);
			return;
		}
		if (event.key === "Backspace" || event.key === "Delete") {
			setValue("");
			setRecording(false);
			return;
		}
		const shortcut = shortcutFromKeyEvent(event);
		if (!shortcut) return;
		setValue(shortcut.chord);
		setRecording(false);
	};
	const parts = () => value().split("+").filter(Boolean);
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("min-w-0 flex flex-col gap-2", props.class);
		},
		get children() {
			return [memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent$1(Text, {
					class: "text-sm font-medium text-primary",
					get children() {
						return props.label;
					}
				}) : props.label;
			}), createComponent$1(Button$1, {
				unstyled: true,
				role: "button",
				get ["aria-label"]() {
					return props.label ?? "Keyboard shortcut";
				},
				get ["aria-pressed"]() {
					return recording();
				},
				get disabled() {
					return props.disabled;
				},
				class: (state) => mergeClasses("h-10 min-w-48 px-3 flex flex-row items-center gap-1 rounded-lg border bg-input", recording() || state.focusVisible ? "border-focus" : "border-strong"),
				onClick: () => setRecording(true),
				onKeyDown: handleKeyDown,
				get children() {
					return memo(() => {
						return !!recording();
					})() ? createComponent$1(Text, {
						class: "text-sm text-muted",
						children: "Press a shortcut…"
					}) : memo(() => {
						return parts().length > 0;
					})() ? createComponent$1(For, {
						get each() {
							return parts();
						},
						children: (part) => createComponent$1(Kbd, { children: part })
					}) : createComponent$1(Text, {
						class: "text-sm text-muted",
						get children() {
							return props.placeholder ?? "Record shortcut";
						}
					});
				}
			})];
		}
	});
}
//#endregion
//#region src/components/workbench-style.ts
const workbenchClass = (className) => mergeClasses("w-full h-full min-w-0 min-h-0 flex flex-row overflow-hidden bg-canvas text-primary", className);
const workbenchSidebarClass = (className) => mergeClasses("w-64 flex-none border-r border-subtle", className);
const workbenchMainClass = (className) => mergeClasses("min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden bg-canvas", className);
const workbenchHeaderClass = (className) => mergeClasses("w-full h-12 min-w-0 flex-none px-4 flex flex-row items-center gap-3 border-b border-subtle bg-surface", className);
const workbenchContentClass = (className) => mergeClasses("min-w-0 min-h-0 flex-1 flex flex-col overflow-hidden bg-canvas", className);
/** Shared readable column for workbench transcripts, editors and composers. */
const workbenchContentColumnClass = (className) => mergeClasses("w-full max-w-4xl mx-auto min-w-0", className);
const workbenchFooterClass = (className) => mergeClasses("min-w-0 flex-none border-t border-subtle bg-surface", className);
//#endregion
//#region src/components/sidebar.tsx
const SidebarMenuContext = createContext({
	managed: false,
	value: () => void 0,
	select: () => {}
});
/**
* Filter grouped sidebar data without taking ownership of routing or identity.
* Group labels participate in matching so a query can reveal a whole section.
*/
function filterSidebarGroups(groups, query, searchableText) {
	const needle = query.trim().toLowerCase();
	if (!needle) return groups.map((group) => ({
		...group,
		items: [...group.items]
	}));
	return groups.flatMap((group) => {
		const items = group.label.toLowerCase().includes(needle) ? [...group.items] : group.items.filter((item) => searchableText(item).toLowerCase().includes(needle));
		return items.length === 0 ? [] : [{
			...group,
			items
		}];
	});
}
/** Structural application sidebar. State, routing and width remain explicit. */
function Sidebar(props) {
	const theme = useComponentsTheme();
	const forwarded = omit(props, "class", "elevation", "shadows");
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return props.role ?? "group";
		},
		get ["class"]() {
			return mergeClasses("h-full min-h-0 flex-none flex flex-col overflow-hidden bg-surface-muted", props.class);
		},
		get shadows() {
			return memo(() => {
				return !!(props.shadows === void 0 && props.elevation);
			})() ? componentsElevation(theme(), props.elevation) : props.shadows;
		}
	}));
}
function SidebarHeader(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return workbenchHeaderClass(props.class);
	} }));
}
function SidebarSearch(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, {
		class: "flex-none p-2 border-b border-subtle bg-surface",
		get children() {
			return createComponent$1(SearchField, mergeProps(forwarded, {
				get placeholder() {
					return props.placeholder ?? "Search";
				},
				get ["class"]() {
					return mergeClasses("w-full", props.class);
				}
			}));
		}
	});
}
/** The only scrolling region in a standard sidebar; header/footer stay fixed. */
function SidebarContent(props) {
	return createComponent$1(ScrollArea, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("min-h-0 flex-1", props.class);
		},
		get contentClass() {
			return mergeClasses("px-3 py-4", props.contentClass);
		}
	}));
}
function SidebarGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "group";
		},
		get ["class"]() {
			return mergeClasses("flex-none flex flex-col gap-1 mb-5", props.class);
		}
	}));
}
function SidebarGroupLabel(props) {
	return createComponent$1(Text, mergeProps(props, { get ["class"]() {
		return mergeClasses("px-2 py-1.5 text-xs font-medium text-muted", props.class);
	} }));
}
/**
* Single-selection navigation scope for sidebar destinations.
* Buttons without a value remain actions and never become selected items.
*/
function SidebarMenu(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const forwarded = omit(props, "value", "defaultValue", "onValueChange", "class", "children");
	return createComponent(SidebarMenuContext, {
		value: {
			managed: true,
			value: state.value,
			select: (value) => state.set(value)
		},
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get role() {
					return props.role ?? "group";
				},
				get ["class"]() {
					return mergeClasses("min-w-0 flex flex-col gap-1", props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
/** Consistent navigation row; applications still own activation and routing. */
function SidebarMenuButton(props) {
	const menu = useContext(SidebarMenuContext);
	const forwarded = omit(props, "class", "value", "selected", "onClick");
	const selected = () => props.value !== void 0 && menu.managed ? menu.value() === props.value : props.selected ?? false;
	return createComponent$1(Button$1, mergeProps(forwarded, {
		unstyled: true,
		get selected() {
			return selected();
		},
		get ["aria-selected"]() {
			return selected();
		},
		class: (state) => mergeClasses("w-full min-w-0 h-8 px-3 justify-start gap-2.5 rounded-lg border border-transparent text-sm", state.pressed ? "bg-control-pressed text-primary" : state.hovered ? "bg-control-hover text-primary" : state.selected ? "border-subtle bg-surface text-primary shadow-xs" : "bg-transparent text-secondary", state.focusVisible && "border-focus", props.class),
		onClick: (event) => {
			if (props.value !== void 0 && menu.managed) menu.select(props.value);
			props.onClick?.(event);
		}
	}));
}
function SidebarEmpty(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("px-3 py-6 flex flex-col items-center gap-1", props.class);
		},
		get children() {
			return [createComponent$1(Text, {
				role: "status",
				class: "text-sm text-secondary",
				get children() {
					return props.title ?? "No results found";
				}
			}), memo(() => {
				return memo(() => {
					return !!props.description;
				})() ? createComponent$1(Text, {
					class: "text-xs text-muted",
					get children() {
						return props.description;
					}
				}) : null;
			})];
		}
	});
}
function SidebarFooter(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("flex-none border-t border-subtle bg-surface", props.class);
	} }));
}
//#endregion
//#region src/components/slider.tsx
function Slider(props) {
	const range = () => normalizeRange(props.min, props.max, props.step);
	const min = () => range().min;
	const max = () => range().max;
	const step = () => range().step;
	const clamp = (value) => Math.max(min(), Math.min(max(), finiteOr(value, min())));
	const snap = (value) => {
		const stepped = min() + Math.round((clamp(value) - min()) / step()) * step();
		const precision = decimalPlaces(step());
		return clamp(Number(stepped.toFixed(precision)));
	};
	const [local, setLocal] = createSignal(snap(props.defaultValue ?? min()));
	const value = () => snap(props.value ?? local());
	const ratio = () => max() === min() ? 0 : (value() - min()) / (max() - min());
	const measured = createMeasuredSize();
	const [dragging, setDragging] = createSignal(false);
	const [focused, setFocused] = createSignal(false);
	const update = (next) => {
		if (props.disabled) return;
		const normalized = snap(next);
		const changed = normalized !== value();
		if (props.value === void 0) setLocal(normalized);
		if (changed) props.onValueChange?.(normalized);
	};
	const updateFromPointer = (event) => {
		const width = measured.width();
		if (width <= 0) return;
		event.preventDefault();
		update(min() + Math.max(0, Math.min(width, event.offsetX)) / width * (max() - min()));
	};
	const changeBy = (amount) => update(value() + amount);
	const onKeyDown = (event) => {
		if (props.disabled) return;
		if (event.key === "ArrowLeft" || event.key === "ArrowDown") changeBy(-step());
		else if (event.key === "ArrowRight" || event.key === "ArrowUp") changeBy(step());
		else if (event.key === "PageDown") changeBy(-step() * 10);
		else if (event.key === "PageUp") changeBy(step() * 10);
		else if (event.key === "Home") update(min());
		else if (event.key === "End") update(max());
		else return;
		event.preventDefault();
	};
	return createComponent$1(View, {
		ref(r$) {
			var _ref$ = measured.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
		},
		role: "slider",
		get ["aria-label"]() {
			return props.label;
		},
		get ["aria-valuemin"]() {
			return min();
		},
		get ["aria-valuemax"]() {
			return max();
		},
		get ["aria-valuenow"]() {
			return value();
		},
		get ["aria-valuetext"]() {
			return props.valueText?.(value()) ?? String(value());
		},
		get ["aria-disabled"]() {
			return props.disabled;
		},
		get focusOrder() {
			return props.disabled ? -1 : 0;
		},
		get ["class"]() {
			return mergeClasses("h-7 relative flex items-center", props.disabled ? "cursor-not-allowed" : "cursor-pointer", props.class);
		},
		onFocus: () => setFocused(true),
		onBlur: () => {
			setFocused(false);
			setDragging(false);
		},
		onPointerDown: (event) => {
			setDragging(true);
			updateFromPointer(event);
		},
		onPointerMove: (event) => {
			if (dragging() && event.buttons !== 0) updateFromPointer(event);
		},
		onPointerUp: (event) => {
			if (dragging()) updateFromPointer(event);
			setDragging(false);
		},
		onPointerCancel: () => setDragging(false),
		onKeyDown,
		get style() {
			return { opacity: props.disabled ? .45 : 1 };
		},
		get children() {
			return [createComponent$1(View, {
				"aria-hidden": "true",
				class: "w-full h-1.5 overflow-hidden rounded-full border border-subtle bg-control",
				get children() {
					return createComponent$1(View, {
						class: "h-full rounded-full bg-accent",
						get style() {
							return { width: `${ratio() * 100}%` };
						}
					});
				}
			}), createComponent$1(View, {
				"aria-hidden": "true",
				get ["class"]() {
					return mergeClasses("w-4 h-4 absolute rounded-full border bg-surface shadow-xs", focused() || dragging() ? "border-focus" : "border-strong");
				},
				get style() {
					return {
						left: `${ratio() * Math.max(0, measured.width() - 16)}px`,
						top: "6px"
					};
				}
			})];
		}
	});
}
//#endregion
//#region src/components/split-button.tsx
/**
* Desktop split action: the label always performs the primary command while
* the adjacent arrow owns alternative commands and their keyboard behavior.
*/
function SplitButton(props) {
	const menuLabel = () => props.menuLabel ?? `${props.label} alternatives`;
	return createComponent$1(ButtonGroup, {
		get ["aria-label"]() {
			return props.label;
		},
		get ["class"]() {
			return props.class;
		},
		get children() {
			return [
				createComponent$1(Button, {
					get variant() {
						return props.variant;
					},
					get size() {
						return props.size;
					},
					get disabled() {
						return props.disabled;
					},
					get onClick() {
						return props.onClick;
					},
					class: "flex-1",
					get children() {
						return props.label;
					}
				}),
				createComponent$1(ButtonGroupSeparator, {}),
				createComponent$1(DropdownMenu, {
					get ["aria-label"]() {
						return menuLabel();
					},
					get items() {
						return props.items;
					},
					get onAction() {
						return props.onAction;
					},
					trigger: (trigger) => createComponent$1(Button, mergeProps(trigger, {
						get ["aria-label"]() {
							return menuLabel();
						},
						get variant() {
							return props.variant;
						},
						size: "icon",
						get disabled() {
							return props.disabled;
						},
						get children() {
							return createComponent$1(Icon, {
								source: chevronDown,
								size: 14
							});
						}
					}))
				})
			];
		}
	});
}
//#endregion
//#region src/components/stat-card.tsx
function StatCard(props) {
	return createComponent$1(Card, {
		role: "group",
		get ["aria-label"]() {
			return props.label;
		},
		get ["class"]() {
			return props.class;
		},
		get children() {
			return createComponent$1(CardContent, {
				class: "gap-2",
				get children() {
					return [
						createComponent$1(View, {
							class: "flex flex-row items-center justify-between gap-3",
							get children() {
								return [createComponent$1(Text, {
									class: "min-w-0 text-xs font-medium text-muted",
									get children() {
										return props.label;
									}
								}), createComponent$1(View, {
									"aria-hidden": "true",
									get ["class"]() {
										return mergeClasses("w-2 h-2 rounded-full bg-accent", props.indicatorClass);
									}
								})];
							}
						}),
						createComponent$1(Text, {
							class: "text-3xl font-semibold text-primary",
							get children() {
								return props.value;
							}
						}),
						memo(() => {
							return memo(() => {
								return !!(props.description || props.trend);
							})() ? createComponent$1(View, {
								class: "min-w-0 flex flex-row items-center gap-2",
								get children() {
									return [memo(() => {
										return memo(() => {
											return !!props.trend;
										})() ? createComponent$1(Text, {
											class: "flex-none text-xs font-medium text-success-primary",
											get children() {
												return props.trend;
											}
										}) : props.trend;
									}), memo(() => {
										return memo(() => {
											return !!props.description;
										})() ? createComponent$1(Text, {
											class: "min-w-0 text-xs text-muted whitespace-normal",
											get children() {
												return props.description;
											}
										}) : props.description;
									})];
								}
							}) : props.description || props.trend;
						})
					];
				}
			});
		}
	});
}
//#endregion
//#region src/components/status-bar.tsx
/** Persistent low-emphasis application state at the bottom of a desktop window. */
function StatusBar(props) {
	return createComponent$1(View, mergeProps(props, {
		role: "status",
		get ["class"]() {
			return mergeClasses("h-7 w-full min-w-0 flex-none flex flex-row items-center gap-1 px-2 border-t border-subtle bg-control", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function StatusBarItem(props) {
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return mergeClasses("h-full min-w-0 px-1.5 flex items-center text-xs text-muted whitespace-nowrap", props.grow ? "flex-1" : "flex-none", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function StatusBarSeparator(props) {
	return createComponent$1(View, {
		role: "separator",
		"aria-orientation": "vertical",
		get ["class"]() {
			return mergeClasses("w-px h-4 flex-none bg-subtle", props.class);
		}
	});
}
//#endregion
//#region src/components/stepper.tsx
function Stepper(props) {
	const [local, setLocal] = createSignal(props.defaultValue ?? props.steps[0]?.id ?? "");
	const value = () => props.value ?? local();
	const currentIndex = () => Math.max(0, props.steps.findIndex((step) => step.id === value()));
	const select = (id) => {
		if (props.value === void 0) setLocal(id);
		props.onValueChange?.(id);
	};
	return createComponent$1(View, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Progress steps";
		},
		get ["class"]() {
			return mergeClasses("min-w-0 flex flex-row items-start", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return props.steps;
				},
				children: (step, index) => createComponent$1(View, {
					class: "min-w-0 flex-1 flex flex-row items-start",
					get children() {
						return [createComponent$1(View, {
							class: "min-w-0 flex-1 items-center gap-2",
							get children() {
								return [
									createComponent$1(Button, {
										get ["aria-label"]() {
											return `Go to ${step.label}`;
										},
										get variant() {
											return index() <= currentIndex() ? "default" : "outline";
										},
										size: "icon",
										onClick: () => select(step.id),
										get children() {
											return String(index() + 1);
										}
									}),
									createComponent$1(Text, {
										class: "text-sm font-medium text-primary",
										get children() {
											return step.label;
										}
									}),
									memo(() => {
										return memo(() => {
											return !!step.description;
										})() ? createComponent$1(Text, {
											class: "text-xs text-muted whitespace-normal",
											get children() {
												return step.description;
											}
										}) : step.description;
									})
								];
							}
						}), memo(() => {
							return memo(() => {
								return index() < props.steps.length - 1;
							})() && createComponent$1(View, { get ["class"]() {
								return mergeClasses("h-px flex-1 mt-4", index() < currentIndex() ? "bg-accent" : "bg-subtle");
							} });
						})];
					}
				})
			});
		}
	});
}
//#endregion
//#region src/components/tabs.tsx
const orientationClass = (orientation, horizontal, vertical) => match(orientation).with("horizontal", () => horizontal).with("vertical", () => vertical).exhaustive();
const TabsContext = createContext();
function Tabs(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const value = state.value;
	const select = (next) => {
		state.set(next);
	};
	const roving = createRovingFocus({
		orientation: () => props.orientation ?? "horizontal",
		onMove: select
	});
	const context = {
		value,
		orientation: () => props.orientation ?? "horizontal",
		select,
		register: (next, node, disabled) => {
			const unregister = roving.register({
				id: next,
				target: node,
				disabled
			});
			if (value() === void 0) select(next);
			return unregister;
		},
		move: roving.move
	};
	return createComponent(TabsContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				get ["class"]() {
					return mergeClasses("flex gap-3", orientationClass(context.orientation(), "flex-col", "flex-row"), props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function TabsList(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsList must be used inside Tabs");
	return createComponent$1(View, {
		role: "tablist",
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-orientation"]() {
			return context.orientation();
		},
		get ["class"]() {
			return memo(() => {
				return !!props.unstyled;
			})() ? props.class : mergeClasses("flex-none flex items-center gap-1", orientationClass(context.orientation(), "flex-row", "flex-col"), match(props.variant ?? "default").with("default", () => "p-0.5 rounded-lg bg-control").with("line", () => "bg-transparent").exhaustive(), props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function TabsTrigger(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsTrigger must be used inside Tabs");
	const selected = () => context.value() === props.value;
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent$1(Button$1, {
		unstyled: true,
		role: "tab",
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get disabled() {
			return props.disabled;
		},
		get selected() {
			return selected();
		},
		get ["aria-selected"]() {
			return selected();
		},
		ref: (node) => {
			unregister?.();
			unregister = context.register(props.value, node, () => props.disabled ?? false);
		},
		class: (state) => props.unstyled ? typeof props.class === "function" ? props.class(state) : props.class ?? "" : mergeClasses("h-7 px-3 items-center justify-center rounded-md border border-transparent text-sm font-medium", match({
			selected: selected(),
			hovered: state.hovered
		}).with({ selected: true }, () => "bg-surface text-primary shadow-xs").with({ hovered: true }, () => "bg-control-hover text-primary").otherwise(() => "bg-transparent text-muted"), state.focusVisible && "border-focus", typeof props.class === "function" ? props.class(state) : props.class),
		style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
		onClick: () => context.select(props.value),
		onKeyDown: (event) => {
			if (context.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return createComponent$1(Text, {
				class: "text-sm font-medium",
				get children() {
					return props.children;
				}
			});
		}
	});
}
function TabsContent(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsContent must be used inside Tabs");
	return createComponent$1(Show, {
		get when() {
			return context.value() === props.value;
		},
		get children() {
			return createComponent$1(View, {
				role: "tabpanel",
				get ["class"]() {
					return mergeClasses("flex-1", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/components/timeline.tsx
function Timeline(props) {
	return createComponent$1(View, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Timeline";
		},
		get ["class"]() {
			return mergeClasses("min-w-0 flex flex-col", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return props.items;
				},
				children: (item, index) => createComponent$1(View, {
					class: "min-w-0 flex flex-row gap-3",
					get children() {
						return [createComponent$1(View, {
							class: "w-4 flex-none flex flex-col items-center",
							get children() {
								return [createComponent$1(View, { get ["class"]() {
									return mergeClasses("w-3 h-3 flex-none rounded-full border", item.status === "complete" ? "border-accent bg-accent" : item.status === "current" ? "border-accent bg-surface" : "border-subtle bg-control");
								} }), memo(() => {
									return memo(() => {
										return index() < props.items.length - 1;
									})() && createComponent$1(View, { class: "w-px min-h-8 flex-1 bg-subtle" });
								})];
							}
						}), createComponent$1(View, {
							class: "min-w-0 flex-1 pb-5",
							get children() {
								return [createComponent$1(View, {
									class: "flex flex-row justify-between gap-3",
									get children() {
										return [createComponent$1(Text, {
											class: "min-w-0 font-medium text-primary",
											get children() {
												return item.title;
											}
										}), memo(() => {
											return memo(() => {
												return !!item.time;
											})() ? createComponent$1(Text, {
												class: "flex-none text-xs text-muted",
												get children() {
													return item.time;
												}
											}) : item.time;
										})];
									}
								}), memo(() => {
									return memo(() => {
										return !!item.description;
									})() ? createComponent$1(Text, {
										class: "mt-1 text-sm text-muted whitespace-normal",
										get children() {
											return item.description;
										}
									}) : item.description;
								})];
							}
						})];
					}
				})
			});
		}
	});
}
//#endregion
//#region src/components/title-bar.tsx
function windowFrameBackdropClassList(maximized, rounded = true) {
	return { "p-3": rounded && !maximized };
}
function windowFrameClientClassList(maximized, rounded = true, classList = {}) {
	const decorated = rounded && !maximized;
	return {
		...classList,
		"rounded-xl": decorated,
		border: decorated,
		"border-subtle": decorated,
		"overflow-hidden": decorated
	};
}
/** Two restrained client-decoration layers sized to fit the 12px backdrop. */
function windowFrameShadows(theme) {
	const ambient = theme === "dark" ? 77 : 36;
	const contact = theme === "dark" ? 82 : 46;
	return [shadow({
		offsetY: 2,
		spread: -1,
		stdDev: 3,
		color: ambient,
		radius: 12
	}), shadow({
		offsetY: 1,
		spread: 0,
		stdDev: 1.5,
		color: contact,
		radius: 12
	})];
}
/**
* Root frame for an application-owned title bar and window chrome.
*
* Rounded outer corners require the native window to preserve alpha and the
* Rust host to clear with a transparent base color. Maximized windows are
* intentionally square so their content reaches every display edge.
*/
function WindowFrame(props) {
	const window = useWindow();
	const theme = useComponentsTheme();
	const decorated = () => props.rounded !== false && !window.maximized();
	return createComponent$1(View, {
		class: "w-full h-full bg-transparent",
		get classList() {
			return windowFrameBackdropClassList(window.maximized(), props.rounded !== false);
		},
		get children() {
			return createComponent$1(View, mergeProps(props, {
				get ["class"]() {
					return mergeClasses("w-full h-full", props.class);
				},
				get classList() {
					return windowFrameClientClassList(window.maximized(), props.rounded !== false, props.classList);
				},
				get shadows() {
					return memo(() => {
						return props.shadows !== void 0;
					})() ? props.shadows : memo(() => {
						return !!decorated();
					})() ? windowFrameShadows(theme()) : null;
				}
			}));
		}
	});
}
const titleBarClass = "border-b border-subtle";
const titleBarLayoutStyle = {
	display: "flex",
	"flex-direction": "row",
	"align-items": "center",
	"flex-shrink": 0,
	height: "40px"
};
const titleBarDragRegionLayoutStyle = {
	display: "flex",
	"flex-direction": "row",
	"align-items": "center",
	"flex-grow": 1,
	"flex-shrink": 1,
	"flex-basis": "0%",
	height: "100%"
};
/** Layout shell for an application-owned title bar. */
function TitleBar(props) {
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return mergeClasses(titleBarClass, props.class);
		},
		get style() {
			return {
				...titleBarLayoutStyle,
				...props.style
			};
		}
	}));
}
/** Explicit non-interactive region that moves the native window. */
function TitleBarDragRegion(props) {
	const window = useWindow();
	return createComponent$1(View, mergeProps(props, {
		get ["class"]() {
			return props.class;
		},
		get style() {
			return {
				...titleBarDragRegionLayoutStyle,
				...props.style
			};
		},
		onPointerDown: (event) => {
			if (event.button === 0) window.startDragging();
		},
		onDblClick: () => window.setMaximized(!window.maximized())
	}));
}
//#endregion
//#region src/components/toast.tsx
const treatment = (variant) => match(variant).with("default", () => ({
	icon: info,
	color: "text-accent"
})).with("success", () => ({
	icon: checkCircle,
	color: "text-success-primary"
})).with("warning", () => ({
	icon: triangleAlert,
	color: "text-accent"
})).with("destructive", () => ({
	icon: triangleAlert,
	color: "text-danger-primary"
})).exhaustive();
function ToastContent(props) {
	const theme = useComponentsTheme();
	const style = () => treatment(props.input.variant ?? "default");
	return createComponent$1(View, {
		class: "w-full min-w-0 flex items-start gap-3 rounded-xl border border-subtle bg-surface p-4",
		get shadows() {
			return componentsElevation(theme(), "floating");
		},
		get children() {
			return [
				createComponent$1(Icon, {
					get source() {
						return style().icon;
					},
					get ["class"]() {
						return mergeClasses("flex-none mt-0.5", style().color);
					},
					size: 18
				}),
				createComponent$1(View, {
					class: "min-w-0 flex-1 flex flex-col gap-1",
					get children() {
						return [
							createComponent$1(Text, {
								class: "text-sm font-medium text-primary",
								get children() {
									return props.input.title;
								}
							}),
							memo(() => {
								return memo(() => {
									return !!props.input.description;
								})() ? createComponent$1(Text, {
									class: "text-sm whitespace-normal text-muted",
									get children() {
										return props.input.description;
									}
								}) : props.input.description;
							}),
							memo(() => {
								return memo(() => {
									return !!props.input.action;
								})() ? createComponent$1(View, {
									class: "pt-1 flex items-center",
									get children() {
										return createComponent$1(Button, {
											size: "sm",
											variant: "outline",
											onClick: () => {
												props.input.action?.onAction();
												if (props.input.action?.dismiss !== false) props.dismiss();
											},
											get children() {
												return props.input.action.label;
											}
										});
									}
								}) : props.input.action;
							})
						];
					}
				}),
				createComponent$1(Button, {
					size: "icon",
					variant: "ghost",
					get ["aria-label"]() {
						return `Dismiss ${props.input.title}`;
					},
					class: "w-7 h-7 flex-none",
					get onClick() {
						return props.dismiss;
					},
					get children() {
						return createComponent$1(Icon, {
							source: x,
							size: 14
						});
					}
				})
			];
		}
	});
}
/** Create an owner-scoped, styled toast queue over NotificationRegion. */
function createToasts(options = {}) {
	const notifications = createNotifications(options);
	const show = (input) => notifications.show({
		"aria-label": input.title,
		priority: input.variant === "destructive" ? "assertive" : "polite",
		duration: input.duration,
		onDismiss: input.onDismiss,
		content: (controls) => createComponent$1(ToastContent, {
			input,
			get dismiss() {
				return controls.dismiss;
			}
		})
	});
	return {
		notifications,
		show,
		success: (title, input = {}) => show({
			...input,
			title,
			variant: "success"
		}),
		warning: (title, input = {}) => show({
			...input,
			title,
			variant: "warning"
		}),
		error: (title, input = {}) => show({
			...input,
			title,
			variant: "destructive"
		}),
		dismiss: (id) => notifications.dismiss(id, "dismiss"),
		clear: notifications.clear
	};
}
/** Render a non-blocking stack of styled toasts on the floating plane. */
function Toaster(props) {
	return createComponent$1(NotificationRegion, {
		get notifications() {
			return props.toasts.notifications;
		},
		get placement() {
			return props.placement ?? "bottom-end";
		},
		get ["class"]() {
			return props.class;
		},
		get itemClass() {
			return mergeClasses("w-96 max-w-full", props.itemClass);
		},
		get motion() {
			return memo(() => {
				return props.motion === void 0;
			})() ? { fromY: (props.placement ?? "bottom-end").startsWith("bottom") ? 12 : -12 } : props.motion;
		}
	});
}
//#endregion
//#region src/components/tool.tsx
/** Composable disclosure root for an AI or automation tool invocation. */
function Tool(props) {
	const rest = omit(props, "class");
	return createComponent$1(Collapsible, mergeProps(rest, { get ["class"]() {
		return mergeClasses("w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface shadow-xs", props.class);
	} }));
}
function statusPresentation(status) {
	return match(status).with("pending", () => ({
		icon: ellipsisCircle,
		label: "Pending",
		variant: "secondary"
	})).with("running", () => ({
		icon: clock,
		label: "Running",
		variant: "secondary"
	})).with("success", () => ({
		icon: checkCircle,
		label: "Completed",
		variant: "success"
	})).with("failed", () => ({
		icon: xCircle,
		label: "Failed",
		variant: "destructive"
	})).exhaustive();
}
function toolHeaderLabel(title, summary, status) {
	return [
		title,
		summary,
		status ? statusPresentation(status).label : void 0
	].filter(Boolean).join(": ");
}
/** Stable title, summary, status and disclosure geometry for one tool call. */
function ToolHeader(props) {
	const presentation = () => props.status ? statusPresentation(props.status) : void 0;
	const rest = omit(props, "title", "summary", "status", "icon", "class", "aria-label");
	return createComponent$1(CollapsibleTrigger, mergeProps(rest, {
		get ["aria-label"]() {
			return props["aria-label"] ?? toolHeaderLabel(props.title, props.summary, props.status);
		},
		get ["class"]() {
			return mergeClasses("min-h-10 px-3 py-1.5 bg-control text-left", props.class);
		},
		get children() {
			return createComponent$1(View, {
				class: "min-w-0 flex-1 flex flex-row items-center gap-2",
				get children() {
					return [
						createComponent$1(Icon, {
							get source() {
								return props.icon ?? wrench;
							},
							size: 14,
							class: "flex-none text-muted"
						}),
						createComponent$1(Text, {
							class: "flex-none text-sm font-semibold text-primary",
							get children() {
								return props.title;
							}
						}),
						memo(() => {
							return memo(() => {
								return !!props.summary;
							})() ? createComponent$1(Text, {
								class: "min-w-0 flex-1 truncate text-xs text-muted",
								get children() {
									return props.summary;
								}
							}) : props.summary;
						}),
						memo(() => {
							return memo(() => {
								return !!presentation();
							})() ? createComponent$1(Badge, {
								get variant() {
									return presentation()?.variant;
								},
								weight: "normal",
								class: "flex flex-row items-center gap-1",
								get children() {
									return [createComponent$1(Icon, {
										get source() {
											return presentation()?.icon ?? wrench;
										},
										size: 11
									}), memo(() => {
										return presentation()?.label;
									})];
								}
							}) : presentation();
						})
					];
				}
			});
		}
	}));
}
function ToolContent(props) {
	const rest = omit(props, "class");
	return createComponent$1(CollapsibleContent, mergeProps(rest, { get ["class"]() {
		return mergeClasses("min-w-0 border-t border-subtle bg-surface", props.class);
	} }));
}
/** Labelled code payload used for tool parameters, results and errors. */
function ToolCodeSection(props) {
	const rest = omit(props, "code", "label", "language", "copyable", "class", "codeClass", "codeProps");
	return createComponent$1(View, mergeProps(rest, {
		get ["class"]() {
			return mergeClasses("min-w-0 p-3 gap-2", props.class);
		},
		get children() {
			return [createComponent$1(Text, {
				class: "text-xs font-medium tracking-wide text-muted",
				get children() {
					return props.label;
				}
			}), createComponent$1(CodeBlock, mergeProps(() => {
				return props.codeProps;
			}, {
				get code() {
					return props.code;
				},
				get language() {
					return props.language;
				},
				get copyable() {
					return props.copyable;
				},
				get ["class"]() {
					return mergeClasses("rounded-md", props.codeClass);
				}
			}))];
		}
	}));
}
function ToolInput(props) {
	return createComponent$1(ToolCodeSection, mergeProps(props, { get label() {
		return props.label ?? "Parameters";
	} }));
}
function ToolOutput(props) {
	const rest = omit(props, "error");
	return createComponent$1(ToolCodeSection, mergeProps(rest, {
		get label() {
			return props.label ?? (props.error ? "Error" : "Result");
		},
		get ["class"]() {
			return mergeClasses("border-t border-subtle", props.error ? "bg-danger-surface" : void 0, props.class);
		}
	}));
}
//#endregion
//#region src/components/tooltip.tsx
let tooltipId = 0;
/** A delayed, non-interactive label for pointer and keyboard focus targets. */
function Tooltip(props) {
	const theme = useComponentsTheme();
	const id = `wabou-tooltip-${++tooltipId}`;
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const open = () => !props.disabled && (props.open ?? uncontrolledOpen());
	const setOpen = (next) => {
		if (props.disabled) next = false;
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next);
	};
	const delay = createDelayedOpenController({
		openDelay: () => props.openDelay ?? 500,
		closeDelay: () => props.closeDelay ?? 80,
		setOpen
	});
	onCleanup(delay.dispose);
	return createComponent$1(Popover$1, {
		get open() {
			return open();
		},
		onOpenChange: (next) => !next && delay.closeNow(),
		get placement() {
			return props.placement ?? "top";
		},
		get offset() {
			return props.offset ?? 8;
		},
		contentRole: "presentation",
		popupRole: "tooltip",
		outsidePointerStrategy: "passthrough",
		contentInteractionBlocked: true,
		closeOnEscape: true,
		restoreFocus: false,
		get contentClass() {
			return mergeClasses("max-w-xs rounded-lg border border-subtle bg-surface px-2 py-1.5", props.contentClass);
		},
		get contentShadows() {
			return componentsElevation(theme(), "floating");
		},
		get motion() {
			return props.motion;
		},
		trigger: (popover) => props.trigger({
			ref: popover.ref,
			onPointerEnter: delay.scheduleOpen,
			onPointerLeave: delay.scheduleClose,
			onFocus: delay.openNow,
			onBlur: delay.closeNow,
			onKeyDown: (event) => {
				if (event.key === "Escape") delay.closeNow();
			}
		}),
		get children() {
			return createComponent$1(Text, {
				id,
				role: "tooltip",
				class: "whitespace-normal text-xs text-primary",
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/components/tree-view.tsx
/** Validates a nested tree once and provides deterministic visible traversal. */
function createTreeModel(nodes) {
	const byId = /* @__PURE__ */ new Map();
	const parents = /* @__PURE__ */ new Map();
	const entries = /* @__PURE__ */ new Map();
	const visit = (items, parentId) => {
		items.forEach((node, index) => {
			if (!node.id) throw new Error("tree node id must not be empty");
			if (byId.has(node.id)) throw new Error(`tree node id must be unique: ${node.id}`);
			if (!node.label) throw new Error(`tree node label must not be empty: ${node.id}`);
			byId.set(node.id, node);
			parents.set(node.id, parentId);
			entries.set(node.id, {
				node,
				parentId,
				level: parentId === null ? 1 : (entries.get(parentId)?.level ?? 0) + 1,
				position: index + 1,
				setSize: items.length
			});
			if (node.children?.length) visit(node.children, node.id);
		});
	};
	visit(nodes, null);
	const isBranch = (id) => (byId.get(id)?.children?.length ?? 0) > 0;
	return {
		get: (id) => byId.get(id),
		parent: (id) => parents.get(id),
		firstChild: (id) => byId.get(id)?.children?.[0]?.id,
		isBranch,
		visible(expandedIds) {
			const expanded = new Set(expandedIds);
			const result = [];
			const flatten = (items) => {
				items.forEach((node) => {
					const entry = entries.get(node.id);
					if (!entry) throw new Error(`missing tree model entry: ${node.id}`);
					result.push(entry);
					if (node.children?.length && expanded.has(node.id)) flatten(node.children);
				});
			};
			flatten(nodes);
			return result;
		}
	};
}
function validateExpandedIds(model, ids) {
	const unique = /* @__PURE__ */ new Set();
	for (const id of ids) {
		if (!model.get(id)) throw new Error(`unknown expanded tree node: ${id}`);
		if (!model.isBranch(id)) throw new Error(`tree leaf cannot be expanded: ${id}`);
		unique.add(id);
	}
	return [...unique];
}
/** A single-select tree with explicit data, expansion, and native focus routing. */
function TreeView(props) {
	const initialModel = createTreeModel(props.items);
	const model = createMemo(() => createTreeModel(props.items));
	const expandedState = createControllableState({
		value: () => props.expandedIds === void 0 ? void 0 : validateExpandedIds(model(), props.expandedIds),
		defaultValue: validateExpandedIds(initialModel, props.defaultExpandedIds ?? []),
		onChange: props.onExpandedChange
	});
	const selectedState = createControllableState({
		value: () => props.selectedId,
		defaultValue: props.defaultSelectedId ?? null,
		onChange: props.onSelectedChange
	});
	const [activeId, setActiveId] = createSignal(void 0, { ownedWrite: true });
	const handles = /* @__PURE__ */ new Map();
	const expanded = () => expandedState.value();
	const visible = createMemo(() => model().visible(expanded()));
	const enabledVisible = () => visible().filter(({ node }) => !node.disabled);
	const isExpanded = (id) => expanded().includes(id);
	const isSelected = (id) => selectedState.value() === id;
	const tabStop = () => {
		const candidates = enabledVisible();
		const active = activeId();
		if (active && candidates.some(({ node }) => node.id === active)) return active;
		const selected = selectedState.value();
		if (selected && candidates.some(({ node }) => node.id === selected)) return selected;
		return candidates[0]?.node.id;
	};
	const focus = (id) => {
		if (!id || model().get(id)?.disabled) return false;
		setActiveId(id);
		handles.get(id)?.focus();
		return true;
	};
	const setExpanded = (id, next) => {
		if (!model().isBranch(id)) return false;
		const current = expanded();
		if (current.includes(id) === next) return false;
		return expandedState.set(next ? [...current, id] : current.filter((candidate) => candidate !== id));
	};
	const select = (node) => {
		if (node.disabled) return;
		selectedState.set(node.id);
	};
	const activate = (node) => {
		select(node);
		if (model().isBranch(node.id)) setExpanded(node.id, !isExpanded(node.id));
	};
	const moveLinear = (id, key) => {
		const candidates = enabledVisible();
		const index = candidates.findIndex(({ node }) => node.id === id);
		const target = key === "Home" ? candidates[0] : key === "End" ? candidates.at(-1) : key === "ArrowDown" ? candidates[index + 1] : key === "ArrowUp" ? candidates[index - 1] : void 0;
		return focus(target?.node.id);
	};
	const handleKey = (item, event) => {
		const { id } = item.node;
		let handled = false;
		if ([
			"ArrowUp",
			"ArrowDown",
			"Home",
			"End"
		].includes(event.key)) handled = moveLinear(id, event.key);
		else if (event.key === "ArrowRight" && model().isBranch(id)) handled = isExpanded(id) ? focus(model().firstChild(id)) : setExpanded(id, true);
		else if (event.key === "ArrowLeft") handled = isExpanded(id) ? setExpanded(id, false) : focus(item.parentId ?? void 0);
		if (handled) event.preventDefault();
	};
	return createComponent$1(View, {
		role: "tree",
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["class"]() {
			return mergeClasses("min-w-0 flex flex-col gap-0.5", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return visible();
				},
				children: (item) => {
					const branch = () => model().isBranch(item.node.id);
					return createComponent$1(Button$1, {
						unstyled: true,
						ref: (node) => handles.set(item.node.id, node),
						role: "treeitem",
						get ["aria-label"]() {
							return item.node.label;
						},
						get ["aria-expanded"]() {
							return memo(() => {
								return !!branch();
							})() ? isExpanded(item.node.id) : void 0;
						},
						get ["aria-selected"]() {
							return isSelected(item.node.id);
						},
						get selected() {
							return isSelected(item.node.id);
						},
						get disabled() {
							return item.node.disabled;
						},
						get focusOrder() {
							return tabStop() === item.node.id ? 0 : -1;
						},
						class: (state) => mergeClasses("w-full h-8 min-w-0 pr-2 items-center gap-2 rounded-md text-sm", state.selected ? "bg-selected text-primary" : state.hovered ? "bg-control-hover text-primary" : "bg-transparent text-secondary", props.itemClass),
						get style() {
							return { "padding-left": `${8 + (item.level - 1) * 20}px` };
						},
						onFocus: () => setActiveId(item.node.id),
						onClick: () => activate(item.node),
						onKeyDown: (event) => handleKey(item, event),
						get children() {
							return [memo(() => {
								return memo(() => {
									return !!branch();
								})() ? createComponent$1(Icon, {
									"aria-hidden": "true",
									get source() {
										return isExpanded(item.node.id) ? chevronDown : chevronRight;
									},
									size: 14,
									class: "flex-none text-muted"
								}) : createComponent$1(View, {
									"aria-hidden": "true",
									class: "w-3.5 h-3.5 flex-none"
								});
							}), memo(() => {
								return memo(() => {
									return !!props.renderItem;
								})() ? props.renderItem(item.node, {
									expanded: isExpanded(item.node.id),
									selected: isSelected(item.node.id),
									level: item.level
								}) : createComponent$1(Text, {
									maxLines: 1,
									class: "min-w-0 flex-1 text-sm",
									get children() {
										return item.node.label;
									}
								});
							})];
						}
					});
				}
			});
		}
	});
}
//#endregion
//#region src/components/typography.tsx
function styledText(props, className) {
	return createComponent$1(Text, mergeProps(props, { get ["class"]() {
		return mergeClasses(className, props.class);
	} }));
}
const TypographyH1 = (props) => styledText(props, "text-4xl font-bold text-primary whitespace-normal");
const TypographyH2 = (props) => styledText(props, "text-3xl font-semibold text-primary whitespace-normal");
const TypographyH3 = (props) => styledText(props, "text-2xl font-semibold text-primary whitespace-normal");
const TypographyH4 = (props) => styledText(props, "text-xl font-semibold text-primary whitespace-normal");
const TypographyP = (props) => styledText(props, "text-base leading-relaxed text-secondary whitespace-normal");
const TypographyLead = (props) => styledText(props, "text-xl leading-relaxed text-muted whitespace-normal");
const TypographyLarge = (props) => styledText(props, "text-lg font-semibold text-primary whitespace-normal");
const TypographySmall = (props) => styledText(props, "text-sm font-medium text-secondary whitespace-normal");
const TypographyMuted = (props) => styledText(props, "text-sm text-muted whitespace-normal");
const TypographyInlineCode = (props) => styledText(props, "px-1.5 py-0.5 rounded bg-control font-mono text-sm font-medium text-primary");
function TypographyBlockquote(props) {
	return createComponent$1(View, {
		class: "flex flex-row items-stretch gap-4",
		get children() {
			return [createComponent$1(View, {
				"aria-hidden": "true",
				class: "w-1 flex-none rounded-full bg-strong"
			}), memo(() => {
				return styledText(props, "min-w-0 flex-1 text-base leading-relaxed text-secondary whitespace-normal");
			})];
		}
	});
}
function TypographyList(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return mergeClasses("flex flex-col gap-2", props.class);
	} }));
}
function TypographyListItem(props) {
	return createComponent$1(View, {
		class: "min-w-0 flex flex-row items-start gap-2",
		get children() {
			return [createComponent$1(Text, {
				"aria-hidden": "true",
				class: "flex-none text-secondary",
				children: "•"
			}), memo(() => {
				return styledText(props, "min-w-0 flex-1 text-base text-secondary whitespace-normal");
			})];
		}
	});
}
//#endregion
//#region src/components/workbench.tsx
/** Full-window desktop application boundary with explicit shrink semantics. */
function Workbench(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get ["class"]() {
		return workbenchClass(props.class);
	} }));
}
/** Fixed-width navigation rail paired with a {@link WorkbenchMain}. */
function WorkbenchSidebar(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(Sidebar, mergeProps(forwarded, { get ["class"]() {
		return workbenchSidebarClass(props.class);
	} }));
}
/** The resizable application column beside the navigation rail. */
function WorkbenchMain(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get ["class"]() {
		return workbenchMainClass(props.class);
	} }));
}
/** Shared 48px chrome row for both sidebar and content headers. */
function WorkbenchHeader(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get ["class"]() {
		return workbenchHeaderClass(props.class);
	} }));
}
/** Bounded application content. Add a ScrollArea inside when scrolling is needed. */
function WorkbenchContent(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get ["class"]() {
		return workbenchContentClass(props.class);
	} }));
}
/** A centered 896px desktop content column that still shrinks with its pane. */
function WorkbenchContentColumn(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get ["class"]() {
		return workbenchContentColumnClass(props.class);
	} }));
}
/** Fixed chrome below the workbench content, such as a composer or status bar. */
function WorkbenchFooter(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get ["class"]() {
		return workbenchFooterClass(props.class);
	} }));
}
//#endregion
//#region src/components/index.tsx
/** Live host frame-rate indicator with sensible performance thresholds. */
function Fps(props) {
	const measured = props.value === void 0 ? createFps() : () => props.value ?? 0;
	const value = () => Math.max(0, Math.round(measured()));
	const variant = () => match(value()).with(0, () => "outline").with(P.when((fps) => fps >= (props.goodAt ?? 55)), () => "success").with(P.when((fps) => fps < (props.warningBelow ?? 30)), () => "destructive").otherwise(() => "secondary");
	return createComponent$1(Badge, {
		get variant() {
			return variant();
		},
		weight: "normal",
		get ["class"]() {
			return mergeClasses("font-mono", props.class);
		},
		get children() {
			return [memo(() => {
				return value();
			}), memo(() => {
				return memo(() => {
					return props.label === "";
				})() ? "" : ` ${props.label ?? "fps"}`;
			})];
		}
	});
}
/** A native secret input whose value never crosses into JavaScript. */
function PasswordInput(props) {
	return createComponent$1(PasswordInput$1, mergeProps(props, { get ["class"]() {
		return mergeClasses("w-full border shadow-xs", componentsControlSize("default"), "border-subtle bg-input text-primary", props.disabled && "opacity-50", props.class);
	} }));
}
function TextArea$1(props) {
	const forwarded = omit(props, "chrome", "surfaceClass");
	return createComponent$1(TextArea, mergeProps(forwarded, {
		get ["data-wabou-owns"]() {
			return (props.chrome ?? "default") === "default" ? "surface native-editor" : "native-editor";
		},
		get ["class"]() {
			return mergeClasses("h-24 w-full px-3 py-2 text-sm text-primary", (props.chrome ?? "default") === "default" && mergeClasses("rounded-lg border border-subtle shadow-xs", props.surfaceClass ?? "bg-input"), props.disabled && "opacity-50", props.class);
		}
	}));
}
function switchColors(checked, state) {
	return match({
		checked,
		pressed: state.pressed,
		hovered: state.hovered
	}).with({
		checked: true,
		pressed: true
	}, () => "bg-accent-pressed").with({
		checked: true,
		hovered: true
	}, () => "bg-accent-hover").with({ checked: true }, () => "bg-accent").with({
		checked: false,
		pressed: true
	}, () => "bg-control-pressed").with({
		checked: false,
		hovered: true
	}, () => "bg-control-hover").with({ checked: false }, () => "bg-control").exhaustive();
}
function Switch(props) {
	const [local, setLocal] = createSignal(props.defaultChecked ?? false);
	const checked = () => props.checked ?? local();
	const reducedMotion = useReducedMotion();
	const movement = createTransition(() => checked() ? 20 : 0, {
		duration: .18,
		ease: [
			.22,
			1,
			.36,
			1
		],
		reducedMotion
	});
	const toggle = () => {
		if (props.disabled) return;
		const next = !checked();
		if (props.checked === void 0) setLocal(next);
		props.onCheckedChange?.(next);
	};
	return createComponent$1(View, {
		get ["class"]() {
			return mergeClasses("w-full min-w-0 flex items-center gap-3", props.class);
		},
		get children() {
			return [createComponent$1(Button$1, {
				unstyled: true,
				role: "switch",
				get disabled() {
					return props.disabled;
				},
				get ["aria-label"]() {
					return props["aria-label"] ?? props.label;
				},
				get ["aria-checked"]() {
					return checked();
				},
				class: (state) => mergeClasses("w-11 h-6 flex-none rounded-full p-0.5", switchColors(checked(), state), state.focused && "border border-focus"),
				style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
				onClick: toggle,
				get children() {
					return createComponent$1(View, {
						"aria-hidden": "true",
						class: "w-5 h-5 rounded-full bg-on-accent",
						get transform() {
							return translate2d$1(movement.value(), 0);
						}
					});
				}
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent$1(Text, {
					class: "min-w-0 flex-1 whitespace-normal text-sm text-secondary",
					get children() {
						return props.label;
					}
				}) : props.label;
			})];
		}
	});
}
//#endregion
//#region src/integrations/standard-schema.ts
function isPromiseLike(value) {
	return typeof value.then === "function";
}
function issueKey(value, issue) {
	const segment = issue.path?.[0];
	const candidate = typeof segment === "object" && segment !== null && "key" in segment ? segment.key : segment;
	return (typeof candidate === "string" || typeof candidate === "number" || typeof candidate === "symbol") && Reflect.has(value, candidate) ? candidate : FORM_ERROR;
}
/**
* Adapt any synchronous Standard Schema V1 implementation (including Valibot,
* Zod, and ArkType) to `createFormDraft` without coupling Wabou to its API.
*/
function createStandardSchemaValidator(schema) {
	return (value) => {
		const result = schema["~standard"].validate(value);
		if (isPromiseLike(result)) throw new TypeError("createFormDraft requires synchronous validation; validate asynchronous schemas before submission");
		if (!result.issues) return {};
		const errors = {};
		for (const issue of result.issues) {
			const key = issueKey(value, issue);
			errors[key] ??= issue.message;
		}
		return errors;
	};
}
//#endregion
//#region src/integrations/tanstack-table.ts
function access(value) {
	return typeof value === "function" ? value() : value;
}
/**
* Solid's reactive ownership around TanStack Table's DOM-independent core.
*
* Wabou deliberately owns no duplicate sorting, filtering, or selection state
* machine here. Applications retain the native renderer and component layer,
* while TanStack owns the mature data model.
*/
function createTanStackDataTable(options) {
	const [sorting, setSorting] = createSignal(options.initialSorting ?? []);
	const [globalFilter, setGlobalFilter] = createSignal(options.initialGlobalFilter ?? "");
	const [rowSelection, setRowSelection] = createSignal(options.initialRowSelection ?? {});
	const table = createTable({
		data: [...access(options.data)],
		columns: [...options.columns],
		state: {},
		onStateChange: () => {},
		renderFallbackValue: "—",
		getRowId: options.getRowId,
		enableRowSelection: options.enableRowSelection,
		getCoreRowModel: getCoreRowModel(),
		getFilteredRowModel: getFilteredRowModel(),
		getSortedRowModel: getSortedRowModel(),
		onSortingChange: (updater) => setSorting((value) => functionalUpdate(updater, value)),
		onGlobalFilterChange: (updater) => setGlobalFilter((value) => functionalUpdate(updater, value)),
		onRowSelectionChange: (updater) => setRowSelection((value) => functionalUpdate(updater, value))
	});
	return {
		table,
		rows: createMemo(() => {
			table.setOptions((current) => ({
				...current,
				data: [...access(options.data)],
				columns: [...options.columns],
				state: {
					...table.initialState,
					sorting: sorting(),
					globalFilter: globalFilter(),
					rowSelection: rowSelection()
				}
			}));
			return table.getRowModel().rows;
		}),
		sorting,
		setSorting,
		globalFilter,
		setGlobalFilter,
		rowSelection,
		setRowSelection,
		selectedCount: createMemo(() => Object.values(rowSelection()).filter(Boolean).length)
	};
}
//#endregion
//#region src/router/data.tsx
globalThis.scrollTo ??= () => {};
function createMutableStore(initial) {
	const [read, write] = createSignal(() => initial, { ownedWrite: true });
	return {
		get: read,
		set(next) {
			const value = typeof next === "function" ? next(untrack(read)) : next;
			write(() => value);
		}
	};
}
function createReadonlyStore(read) {
	return { get: read };
}
const solidStores = () => ({
	createMutableStore,
	createReadonlyStore,
	batch: flush
});
/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
function createDataRouter(options) {
	const history = options.history ?? createMemoryHistory$1({ initialEntries: ["/"] });
	const router = new RouterCore({
		isServer: false,
		origin: "wabou://app",
		...options,
		history
	}, solidStores);
	router.startTransition = async (commit) => {
		flush(commit);
		return true;
	};
	return router;
}
const DataRouterContext = createContext();
function requireDataRouter() {
	if (!getOwner()) throw new Error("Wabou data-router hooks must be used inside <RouterProvider>");
	const router = useContext(DataRouterContext);
	if (!router) throw new Error("Wabou data-router hooks must be used inside <RouterProvider>");
	return router;
}
function matchView(router, index) {
	const match = router.state.matches[index];
	if (!match) return void 0;
	const options = router.routesById[match.routeId].options;
	return match.status === "error" ? options.errorComponent : match.status === "notFound" ? options.notFoundComponent : match.status === "pending" ? options.pendingComponent : options.component;
}
/** Preserve a matched component while its route and selected view are stable. */
function RouteMatch(props) {
	const match = () => props.router.state.matches[props.index];
	let outlet;
	return createComponent(Show, {
		get when() {
			return matchView(props.router, props.index);
		},
		keyed: true,
		children: (view) => createComponent(view, {
			get error() {
				return match()?.error;
			},
			get children() {
				if (outlet === void 0) outlet = createComponent(RouteOutlet, {
					router: props.router,
					index: props.index + 1
				});
				return outlet;
			}
		})
	});
}
/** Key only the route level that changed instead of rebuilding all matches. */
function RouteOutlet(props) {
	return createComponent(Show, {
		get when() {
			return props.router.state.matches[props.index]?.routeId;
		},
		keyed: true,
		get fallback() {
			return props.fallback ?? null;
		},
		children: (_routeId) => createComponent(RouteMatch, {
			router: props.router,
			index: props.index
		})
	});
}
/** Own router lifecycle and render its current native component branch. */
function RouterProvider(props) {
	const router = untrack(() => props.router);
	let disposed = false;
	let loadScheduled = false;
	const scheduleLoad = () => {
		if (disposed || loadScheduled) return;
		loadScheduled = true;
		Promise.resolve().then(async () => {
			loadScheduled = false;
			if (!disposed) await router.load();
		}).catch((error) => {
			console.error(`[wabou-router] route load failed: ${String(error)}`);
		});
	};
	const unsubscribe = router.history.subscribe(scheduleLoad);
	onCleanup(() => {
		disposed = true;
		unsubscribe();
	});
	scheduleLoad();
	return createComponent(DataRouterContext, {
		value: router,
		get children() {
			return createComponent(RouteOutlet, {
				router,
				index: 0,
				get fallback() {
					return props.fallback;
				}
			});
		}
	});
}
function useRouter() {
	return requireDataRouter();
}
function useRouterState(selector) {
	const router = requireDataRouter();
	return createMemo(() => selector(router));
}
function useNavigate() {
	return requireDataRouter().navigate;
}
function useLocation() {
	const router = requireDataRouter();
	return createMemo(() => router.state.location);
}
/**
* Reactively report whether a native router destination is active.
*
* This delegates path, base-path, parameter, and trailing-slash behavior to
* Router Core instead of duplicating pathname comparisons in navigation UI.
*/
function useRouteActive(to, options = {}) {
	const router = requireDataRouter();
	const exact = options.exact ?? to === "/";
	return createMemo(() => router.matchRoute({ to }, {
		fuzzy: !exact,
		includeSearch: options.includeSearch ?? false,
		pending: options.pending
	}) !== false);
}
function useParams() {
	const router = requireDataRouter();
	return createMemo(() => router.state.matches.at(-1)?.params ?? {});
}
function useLoaderData() {
	const router = requireDataRouter();
	return createMemo(() => router.state.matches.at(-1)?.loaderData);
}
//#endregion
export { Accordion, AccordionContent, AccordionItem, AccordionTrigger, AdaptiveSplitPane, AdaptiveSplitPaneDetail, AdaptiveSplitPaneMain, Alert, AlertDescription, AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertTitle, AnnotationLayer, AspectRatio, Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentGroup, AttachmentMedia, AttachmentTitle, Avatar, AvatarGroup, AvatarGroupCount, Badge, BaseRootRoute, BaseRoute, Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator, Bubble, BubbleContent, BubbleGroup, BubbleReactions, Button, ButtonGroup, ButtonGroupSeparator, ButtonGroupText, Calendar, CalendarDate, Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious, Center, ChartContainer, ChartEmpty, ChartLegend, Checkbox, CodeBlock, CodeEditor, Collapsible, CollapsibleContent, CollapsiblePresence, CollapsibleTrigger, Column, Combobox, Command, ComponentsProvider, ConfigEditor, ContextMenu, CopyButton, DataTable, DatePicker, Dialog, DialogDescription, DialogDescription as SheetDescription, DialogFooter, DialogFooter as SheetFooter, DialogHeader, DialogHeader as SheetHeader, DialogScrollBody, DialogScrollBody as SheetScrollBody, DialogTitle, DialogTitle as SheetTitle, DiffViewer, DirectionProvider, DirectionalRow, DirectionalText, DirectoryPicker, Drawer, DrawerClose, DrawerDescription, DrawerFooter, DrawerHandle, DrawerHeader, DrawerTitle, DropZone, DropdownMenu, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, FORM_ERROR, Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend, FieldSeparator, FieldSet, FieldTitle, Fps, HoverCard, Icon, IconFrame, Image, ImageList, ImageOverlayLayer, ImageViewport, InlineEdit, Input, InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextArea, InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot, Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia, ItemSeparator, ItemTitle, Kbd, KbdGroup, Label, Markdown, Marker, MarkerContent, MarkerIcon, Menubar, MenubarMenu, Message, MessageActions, MessageAvatar, MessageContent, MessageFooter, MessageGroup, MessageHeader, MessageScroller, MessageScrollerButton, MessageScrollerContent, MessageScrollerItem, MessageScrollerViewport, Modal, MotionConfigProvider, NativeSelect, NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem, NavigationMenuLink, NavigationMenuList, NavigationMenuTrigger, NavigationMenuViewport, NotificationRegion, NumberField, OverlayPlaneProvider, PageHeader, PageViewport, Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationItems, PaginationLink, PaginationNext, PaginationPrevious, PasswordInput, Path, PathBuilder, Popover, PopoverDescription, PopoverFooter, PopoverHeader, PopoverTitle, Button$1 as PrimitiveButton, Link as PrimitiveLink, PasswordInput$1 as PrimitivePasswordInput, Popover$1 as PrimitivePopover, TextArea as PrimitiveTextArea, TextInput as PrimitiveTextInput, Progress, ProgressFill, ProgressLabel, ProgressRoot, ProgressTrack, ProgressValueLabel, PromptSuggestion, PromptSuggestions, PropertyList, PropertyRow, Pulse, QRCode, RadioGroup, RadioGroupItem, Rating, Reasoning, ReasoningContent, ReasoningTrigger, ResizableHandle, ResizablePanel, ResizablePanelGroup, ResponsiveGrid, ResponsiveGridRemainder, RichText, RichTextSpan, Ripple, RouterProvider, Row, ScrollArea, SearchField, Select, Separator, Sheet, ShortcutRecorder, Sidebar, SidebarContent, SidebarEmpty, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuButton, SidebarSearch, Skeleton, Slider, Spin, Spinner, SplitButton, SplitPane, SplitPaneAside, SplitPaneMain, StatCard, StatusBar, StatusBarItem, StatusBarSeparator, Stepper, Svg, Switch, Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow, Tabs, TabsContent, TabsList, TabsTrigger, Text, TextArea$1 as TextArea, Timeline, TitleBar, TitleBarDragRegion, Toaster, Toggle, ToggleGroup, ToggleGroupItem, Tool, ToolCodeSection, ToolContent, ToolHeader, ToolInput, ToolOutput, Toolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator, ToolbarToggle, Tooltip, TreeView, TypographyBlockquote, TypographyH1, TypographyH2, TypographyH3, TypographyH4, TypographyInlineCode, TypographyLarge, TypographyLead, TypographyList, TypographyListItem, TypographyMuted, TypographyP, TypographySmall, View, WindowFrame, Workbench, WorkbenchContent, WorkbenchContentColumn, WorkbenchFooter, WorkbenchHeader, WorkbenchMain, WorkbenchSidebar, activeMessageAnchor, alertColors, animate, animateKeyframes, aspectRatioStyle, attachmentClass, attachmentGroupClass, attachmentMediaClass, badgeClass, bubbleClass, bubbleContentClass, clampAnnotationRegion, clampPage, clampRatingValue, componentsControlSize, componentsElevation, componentsThemeContract, createActive, createAnimationFrame, createButton, createContainerMatch, createDataRouter, createDelayedOpenController, createDelayedOpenController as createTooltipDelayController, createFileImageResource, createFocus, createFocusWithin, createFormDraft, createHover, createInterpolation, createKeyedSelection, createKeyframeAnimation, createLoop, createMeasuredSize, createMemoryHistory, createNetworkImageResource, createNotifications, createOverlayLayer, createOwnedImageResource, createPaginationRange, createPresence, createPress, createPulse, createResizablePanelState, createRetainedItems, createRotation, createScrollReset, createShortcuts, createStandardSchemaValidator, createSweep, createTabs, createTanStackDataTable, createToasts, createTransition, createTransitionPresence, createTreeModel, drawerDragOffset, drawerShouldDismiss, emptyClass, emptyMediaClass, encodeQrCode, fieldClass, fieldErrorLabel, filterCommandItems, filterSidebarGroups, imageViewportTransform, inputGroupAddonClass, inputGroupClass, isMessageScrollNearEnd, itemClass, itemMediaClass, messageActionsClass, messageClass, messageScrollRange, messageScrollRevealDelta, messageScrollStartDelta, moveMenuHighlight, navigationMenuTriggerClass, nextAccordionValue, normalizeCarouselIndex, normalizeOtpValue, normalizePageCount, normalizeProgressValue, normalizeRatingMax, normalizeSweepGeometry, notFound, pageHeaderClass, pageHeaderDescriptionClass, pageHeaderTitleClass, pageViewportClass, pageViewportContentClass, pointInLayoutRect, primitives_exports as primitives, qrCodePath, ratingLabel, reconcileCommandHighlight, redirect, releaseImageResource, responsiveGridColumnCount, responsiveGridRemainderCount, shortcutFromKeyEvent, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, toolHeaderLabel, uniqueFieldErrors, useChartConfig, useComponentsTheme, useDirection, useLoaderData, useLocation, useMessageScroller, useMotionConfig, useNavigate, useParams, useReducedMotion, useResponsiveGrid, useRouteActive, useRouter, useRouterState, validateResizableSizes, windowFrameBackdropClassList, windowFrameClientClassList, windowFrameShadows, workbenchClass, workbenchContentClass, workbenchContentColumnClass, workbenchFooterClass, workbenchHeaderClass, workbenchMainClass, workbenchSidebarClass };

//# sourceMappingURL=index.mjs.map