import { Text, View } from "../primitives/view.mjs";
import { PageViewport } from "./page.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/onboarding.tsx
/**
* Full-height first-run boundary with native scrolling and a readable measure.
* Content stays centered when it fits and remains reachable when it grows.
*/
function Onboarding(props) {
	const forwarded = omit(props, "class", "contentClass", "children");
	return createComponent$1(PageViewport, mergeProps(forwarded, {
		get role() {
			return props.role ?? "region";
		},
		get "class"() {
			return mergeClasses("h-full bg-canvas", props.class);
		},
		get contentClass() {
			return mergeClasses("min-h-full flex items-center justify-center", props.contentClass);
		},
		get children() {
			return createComponent$1(View, {
				class: "w-full max-w-2xl min-w-0 mx-auto px-8 py-12 flex flex-col gap-5",
				get children() {
					return props.children;
				}
			});
		}
	}));
}
function OnboardingHeader(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("w-full min-w-0 flex flex-row items-center gap-3", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function OnboardingHeading(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-w-0 flex-1 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function OnboardingTitle(props) {
	return createComponent$1(Text, mergeProps(props, {
		get role() {
			return props.role ?? "heading";
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-lg font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function OnboardingDescription(props) {
	return createComponent$1(Text, mergeProps(props, {
		get "class"() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function OnboardingFooter(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("w-full min-w-0 pt-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Onboarding, OnboardingDescription, OnboardingFooter, OnboardingHeader, OnboardingHeading, OnboardingTitle };

//# sourceMappingURL=onboarding.mjs.map