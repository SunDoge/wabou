import { Icon, Text, TextInput, View } from "../primitives/view.mjs";
import { componentsDisabledInteractiveClass } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, createContext, createSignal, omit, useContext } from "solid-js";
import { applyRef, createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import minus from "lucide-static/icons/minus.svg?raw";
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
				get "aria-label"() {
					return props["aria-label"];
				},
				get "class"() {
					return mergeClasses("relative inline-flex flex-none items-center gap-2", props.class, componentsDisabledInteractiveClass(props.disabled ?? false));
				},
				get children() {
					return [memo(() => {
						return props.children;
					}), createComponent$1(TextInput, {
						ref(r$) {
							var _ref$ = props.ref;
							typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : props.ref = r$;
						},
						get "aria-label"() {
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
						get "class"() {
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
		get "class"() {
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
		get "class"() {
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
		get "class"() {
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
export { InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot, normalizeOtpValue };

//# sourceMappingURL=input-otp.mjs.map