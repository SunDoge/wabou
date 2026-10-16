import { Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { Modal } from "../primitives/modal.mjs";
import { useDevelopmentDiagnostics } from "./development-diagnostics.mjs";
import { rgba } from "@wabou/core";
import { Show } from "solid-js";
import { createComponent as createComponent$1 } from "@wabou/core/renderer";
//#region src/components/dev-server-error.tsx
function diagnosticLocation(diagnostic) {
	const file = diagnostic.loc?.file ?? diagnostic.id;
	if (!file) return diagnostic.plugin ?? "Vite";
	const line = diagnostic.loc?.line;
	const column = diagnostic.loc?.column;
	return `${file}${line ? `:${line}${column ? `:${column}` : ""}` : ""}${diagnostic.plugin ? ` · ${diagnostic.plugin}` : ""}`;
}
/** Native equivalent of Vite's browser error overlay. */
function DevServerErrorOverlay() {
	const { diagnostic, dismiss } = useDevelopmentDiagnostics();
	return createComponent$1(Modal, {
		get open() {
			return diagnostic() !== void 0;
		},
		onOpenChange: (open) => {
			if (!open) dismiss();
		},
		"aria-label": "Development build failed",
		contentRole: "alertdialog",
		closeOnBackdrop: false,
		get backdropStyle() {
			return { "background-color": rgba(253176524) };
		},
		contentClass: "w-[720px] max-w-full max-h-[640px] min-w-0 rounded-xl border border-danger bg-surface p-5 gap-4",
		get children() {
			return createComponent$1(Show, {
				get when() {
					return diagnostic();
				},
				keyed: true,
				children: (error) => [
					createComponent$1(View, {
						class: "min-w-0 gap-1",
						get children() {
							return [createComponent$1(Text, {
								class: "text-lg font-semibold text-danger-primary",
								get children() {
									return error.plugin === "Rust" ? "Rust could not update the app" : "JavaScript could not update the app";
								}
							}), createComponent$1(Text, {
								class: "text-xs text-muted whitespace-normal",
								get children() {
									return diagnosticLocation(error);
								}
							})];
						}
					}),
					createComponent$1(Text, {
						class: "text-sm font-medium text-primary whitespace-normal",
						get children() {
							return error.message;
						}
					}),
					createComponent$1(Show, {
						get when() {
							return error.frame ?? error.stack;
						},
						keyed: true,
						children: (details) => createComponent$1(View, {
							class: "min-w-0 max-h-80 overflow-y-auto rounded-lg bg-control p-3",
							get children() {
								return createComponent$1(Text, {
									class: "font-mono text-xs text-secondary whitespace-pre-wrap",
									children: details
								});
							}
						})
					}),
					createComponent$1(View, {
						class: "flex flex-row items-center justify-between gap-4",
						get children() {
							return [createComponent$1(Text, {
								class: "min-w-0 flex-1 text-xs text-muted whitespace-normal",
								children: "The last working UI is still running. Save a valid update to retry automatically."
							}), createComponent$1(Button, {
								"aria-label": "Dismiss development error",
								class: "flex-none h-8 px-3 rounded-lg bg-control text-sm text-primary",
								onClick: () => dismiss(),
								children: "Dismiss"
							})];
						}
					})
				]
			});
		}
	});
}
//#endregion
export { DevServerErrorOverlay };

//# sourceMappingURL=dev-server-error.mjs.map