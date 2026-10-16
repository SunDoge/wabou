import { createFocusWithin } from "../primitives/focus.mjs";
import { Text, TextArea, View } from "../primitives/view.mjs";
import { componentsDisabledControlClass } from "./theme.mjs";
import { Button } from "./button.mjs";
import { Input } from "./input.mjs";
import { Label } from "./label.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, Show, createContext, omit, useContext } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/forms.tsx
const FieldContext = createContext({
	orientation: () => "vertical",
	invalid: () => false,
	required: () => false
});
function fieldClass(orientation = "vertical", invalid = false, className) {
	const layout = match(orientation).with("vertical", () => "flex-col gap-2").with("horizontal", () => "flex-row items-start gap-4").exhaustive();
	return mergeClasses("w-full min-w-0 flex", layout, invalid && "text-danger-primary", className);
}
function Field(props) {
	const context = {
		orientation: () => props.orientation ?? "vertical",
		invalid: () => props.invalid ?? false,
		required: () => props.required ?? false
	};
	return createComponent$1(FieldContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
				role: "group",
				get "aria-orientation"() {
					return context.orientation();
				},
				get "aria-invalid"() {
					return context.invalid();
				},
				get "aria-required"() {
					return context.required();
				},
				get "class"() {
					return fieldClass(context.orientation(), context.invalid(), props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function FieldSet(props) {
	return createComponent$1(View, {
		role: "group",
		get "class"() {
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
		get "class"() {
			return mergeClasses("mb-1 font-medium text-primary", props.variant === "label" ? "text-sm" : "text-base", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldGroup(props) {
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("flex flex-col gap-4", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldLabel(props) {
	const field = useContext(FieldContext);
	const forwarded = omit(props, "class", "children");
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("min-w-0 flex flex-row items-center gap-1", field.orientation() === "horizontal" ? "w-36 flex-none" : "w-full");
		},
		get children() {
			return [createComponent$1(Label, mergeProps(forwarded, {
				get "class"() {
					return mergeClasses("min-w-0 flex-1", field.invalid() && "text-danger-primary", props.class);
				},
				get children() {
					return props.children;
				}
			})), createComponent$1(Show, {
				get when() {
					return field.required();
				},
				get children() {
					return createComponent$1(Text, {
						"aria-hidden": "true",
						class: "flex-none text-sm text-danger-primary",
						children: "*"
					});
				}
			})];
		}
	});
}
/**
* A complete native field whose visible label always focuses its control.
* This avoids repeating ad-hoc Handle plumbing in every settings surface.
*/
function LabeledField(props) {
	let control;
	const errors = () => uniqueFieldErrors(props.errors);
	return createComponent$1(Field, {
		get invalid() {
			return props.invalid ?? errors().length > 0;
		},
		get required() {
			return props.required;
		},
		get "class"() {
			return props.class;
		},
		get children() {
			return [
				createComponent$1(FieldLabel, {
					get disabled() {
						return props.disabled;
					},
					control: () => control,
					get children() {
						return props.label;
					}
				}),
				memo(() => {
					return props.renderControl((node) => {
						control = node;
						props.controlRef?.(node);
					});
				}),
				createComponent$1(Show, {
					get when() {
						return memo(() => {
							return props.description !== void 0;
						})() && props.description !== null;
					},
					get children() {
						return createComponent$1(FieldDescription, { get children() {
							return props.description;
						} });
					}
				}),
				createComponent$1(Show, {
					get when() {
						return errors().length > 0;
					},
					get children() {
						return createComponent$1(FieldError, { get errors() {
							return props.errors;
						} });
					}
				})
			];
		}
	});
}
function FieldTitle(props) {
	return createComponent$1(Text, {
		get "class"() {
			return mergeClasses("text-sm font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldContent(props) {
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("min-w-0 flex-1 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldDescription(props) {
	return createComponent$1(Text, {
		get "class"() {
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
		get "aria-label"() {
			return label();
		},
		get "class"() {
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
		get "class"() {
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
function inputGroupClass(orientation, focused, invalid, variant = "default") {
	return mergeClasses("relative w-full min-w-0 flex rounded-lg border", variant === "default" ? "shadow-xs" : "shadow-none", orientation === "horizontal" ? "h-8 flex-row items-center" : "h-auto flex-col items-stretch", invalid ? "border-danger" : focused ? "border-focus" : variant === "quiet" ? "border-transparent" : "border-strong");
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
	const forwarded = omit(props, "children", "orientation", "variant", "invalid", "disabled", "surfaceClass", "class");
	return createComponent$1(InputGroupContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, () => {
				return focus.bindings;
			}, {
				get role() {
					return props.role ?? "group";
				},
				get "aria-invalid"() {
					return props.invalid;
				},
				get "aria-disabled"() {
					return props.disabled;
				},
				"data-wabou-owns": "surface focus-ring",
				get "class"() {
					return mergeClasses(inputGroupClass(props.orientation ?? "horizontal", focus.focusWithin(), props.invalid ?? false, props.variant ?? "default"), props.surfaceClass ?? (props.variant === "quiet" ? "bg-transparent" : "bg-input"), props.class, componentsDisabledControlClass(props.disabled ?? false));
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
		get "class"() {
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
		get "class"() {
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
		get "class"() {
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
		get "class"() {
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
		get "class"() {
			return mergeClasses("w-full h-24 px-3 py-2 text-sm text-primary", props.class);
		}
	}));
}
//#endregion
export { Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend, FieldSeparator, FieldSet, FieldTitle, InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextArea, LabeledField, fieldClass, fieldErrorLabel, inputGroupAddonClass, inputGroupClass, uniqueFieldErrors };

//# sourceMappingURL=forms.mjs.map