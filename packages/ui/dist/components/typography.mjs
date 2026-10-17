import { Text, View } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/typography.tsx
function styledText(props, className) {
	return createComponent(Text, mergeProps(props, { get "class"() {
		return mergeClasses(className, props.class);
	} }));
}
const TypographyH1 = (props) => styledText(props, "text-4xl leading-tight font-bold tracking-tight text-primary whitespace-normal");
const TypographyH2 = (props) => styledText(props, "text-3xl leading-tight font-semibold tracking-tight text-primary whitespace-normal");
const TypographyH3 = (props) => styledText(props, "text-2xl leading-tight font-semibold tracking-tight text-primary whitespace-normal");
const TypographyH4 = (props) => styledText(props, "text-xl leading-normal font-semibold text-primary whitespace-normal");
const TypographyP = (props) => styledText(props, "text-base leading-relaxed text-secondary whitespace-normal");
const TypographyLead = (props) => styledText(props, "text-xl leading-normal text-muted whitespace-normal");
const TypographyLarge = (props) => styledText(props, "text-lg font-semibold text-primary whitespace-normal");
const TypographySmall = (props) => styledText(props, "text-sm font-medium text-secondary whitespace-normal");
const TypographyMuted = (props) => styledText(props, "text-sm text-muted whitespace-normal");
const TypographyInlineCode = (props) => styledText(props, "px-1.5 py-0.5 rounded bg-control font-mono text-sm font-medium text-primary");
function TypographyBlockquote(props) {
	return createComponent(View, {
		class: "flex flex-row items-stretch gap-4",
		get children() {
			return [createComponent(View, {
				"aria-hidden": "true",
				class: "w-1 flex-none rounded-full bg-strong"
			}), memo(() => {
				return styledText(props, "min-w-0 flex-1 text-base leading-relaxed text-secondary whitespace-normal");
			})];
		}
	});
}
function TypographyList(props) {
	return createComponent(View, mergeProps(props, { get "class"() {
		return mergeClasses("flex flex-col gap-2", props.class);
	} }));
}
function TypographyListItem(props) {
	return createComponent(View, {
		class: "min-w-0 flex flex-row items-start gap-2",
		get children() {
			return [createComponent(Text, {
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
export { TypographyBlockquote, TypographyH1, TypographyH2, TypographyH3, TypographyH4, TypographyInlineCode, TypographyLarge, TypographyLead, TypographyList, TypographyListItem, TypographyMuted, TypographyP, TypographySmall };

//# sourceMappingURL=typography.mjs.map