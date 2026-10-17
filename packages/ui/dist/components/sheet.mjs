import { Modal } from "../primitives/modal.mjs";
import { componentsElevation, useComponentsTheme } from "./theme.mjs";
import { DialogDescription, DialogFooter, DialogHeader, DialogScrollBody, DialogTitle } from "./dialog.mjs";
import { rgba } from "@wabou/core";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
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
	duration: .22,
	ease: "easeOut",
	fromX: -400
})).with("right", () => ({
	duration: .22,
	ease: "easeOut",
	fromX: 400
})).with("top", () => ({
	duration: .22,
	ease: "easeOut",
	fromY: -320
})).with("bottom", () => ({
	duration: .22,
	ease: "easeOut",
	fromY: 320
})).exhaustive();
/** A modal edge panel that shares native focus isolation with Dialog. */
function Sheet(props) {
	const theme = useComponentsTheme();
	const side = () => props.side ?? "right";
	const placement = () => geometry(side());
	return createComponent(Modal, mergeProps(props, {
		backdropFade: false,
		contentFade: false,
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
export { Sheet, DialogDescription as SheetDescription, DialogFooter as SheetFooter, DialogHeader as SheetHeader, DialogScrollBody as SheetScrollBody, DialogTitle as SheetTitle };

//# sourceMappingURL=sheet.mjs.map