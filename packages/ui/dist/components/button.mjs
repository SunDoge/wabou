import { Button as Button$1 } from "../primitives/button.mjs";
import { buttonGroupItemCorners, useButtonGroupItem } from "./button-group-context.mjs";
import { Spinner } from "./display.mjs";
import { componentsControlContentSize, componentsControlSize, componentsDisabledControlClass } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { P, match } from "ts-pattern";
//#region src/components/button.tsx
function buttonColors(variant, state, visuallyDisabled = false) {
	if (visuallyDisabled) return "bg-surface-muted border-subtle text-muted";
	const focus = state.focusVisible ? "border-focus" : "";
	const active = state.pressed || state.selected;
	const passiveBorder = (value) => match(value).with("outline", () => "border-strong").with(P.union("default", "secondary", "ghost", "destructive"), () => "border-transparent").exhaustive();
	return match({
		variant,
		pressed: active,
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
	}, () => mergeClasses("bg-danger-pressed border-transparent text-on-danger", focus)).with({
		variant: "destructive",
		hovered: true
	}, () => mergeClasses("bg-danger-hover border-transparent text-on-danger", focus)).with({ variant: "destructive" }, () => mergeClasses("bg-danger border-transparent text-on-danger", focus)).with({
		variant: "secondary",
		pressed: true
	}, () => mergeClasses("bg-control-pressed border-transparent text-primary", focus)).with({
		variant: "secondary",
		hovered: true
	}, () => mergeClasses("bg-control-hover border-transparent text-primary", focus)).with({ variant: "secondary" }, () => mergeClasses("bg-control border-transparent text-primary", focus)).with({ pressed: true }, ({ variant: value }) => mergeClasses("bg-control-pressed text-secondary", passiveBorder(value), focus)).with({ hovered: true }, ({ variant: value }) => mergeClasses("bg-control-hover text-secondary", passiveBorder(value), focus)).with({ variant: P.union("outline", "ghost") }, ({ variant: value }) => mergeClasses("bg-transparent text-secondary", passiveBorder(value), focus)).exhaustive();
}
function buttonSize(size, grouped) {
	return grouped ? componentsControlContentSize(size) : componentsControlSize(size);
}
function buttonSpinnerColor(variant) {
	return match(variant).with("default", () => "text-on-accent").with("destructive", () => "text-on-danger").with("secondary", () => "text-primary").with("outline", "ghost", () => "text-secondary").exhaustive();
}
function Button(props) {
	const local = props;
	const forwarded = omit(props, "variant", "size", "class", "style", "loading", "loadingLabel", "children");
	const groupItem = useButtonGroupItem();
	const variant = () => local.variant ?? groupItem?.variant() ?? "default";
	const size = () => local.size ?? groupItem?.size() ?? "default";
	const visuallyDisabled = () => local.disabled || groupItem?.disabled() || false;
	const disabled = () => local.disabled || local.loading || groupItem?.disabled() || false;
	return createComponent$1(Button$1, mergeProps(forwarded, {
		get disabled() {
			return disabled();
		},
		get "aria-busy"() {
			return local.loading ?? false;
		},
		unstyled: true,
		class: (state) => mergeClasses("inline-flex flex-none overflow-hidden whitespace-nowrap items-center justify-center border font-medium", buttonColors(variant(), state, visuallyDisabled()), buttonSize(size(), groupItem !== void 0), groupItem && buttonGroupItemCorners(groupItem), groupItem && "border-transparent", local.class, componentsDisabledControlClass(visuallyDisabled())),
		style: (state) => ({
			"border-width": 1,
			...visuallyDisabled() ? {} : { opacity: 1 },
			...typeof local.style === "function" ? local.style(state) : local.style
		}),
		get children() {
			return createComponent$1(Show, {
				get when() {
					return local.loading;
				},
				get fallback() {
					return local.children;
				},
				get children() {
					return [createComponent$1(Spinner, {
						get label() {
							return local.loadingLabel ?? "Loading";
						},
						get "class"() {
							return buttonSpinnerColor(variant());
						}
					}), memo(() => {
						return local.loadingLabel ?? local.children;
					})];
				}
			});
		}
	}));
}
//#endregion
export { Button };

//# sourceMappingURL=button.mjs.map