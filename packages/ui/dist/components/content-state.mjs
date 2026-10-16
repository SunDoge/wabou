import { Text, View } from "../primitives/view.mjs";
import { Spinner } from "./display.mjs";
import { Button } from "./button.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Match, Switch, omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/content-state.tsx
/** Mutually exclusive loading, empty, or error state for a bounded region. */
function ContentState(props) {
	const forwarded = omit(props, "state", "title", "description", "action", "renderMedia", "renderAction", "class");
	const error = () => props.state === "error";
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return error() ? "alert" : "status";
		},
		get "aria-label"() {
			return props["aria-label"] ?? props.title;
		},
		get "class"() {
			return mergeClasses("w-full h-full min-w-0 min-h-0 flex-1 p-6 flex flex-col items-center justify-center gap-3 text-center", error() ? "text-danger-primary" : "text-secondary", props.class);
		},
		get children() {
			return [
				memo(() => {
					return props.renderMedia?.() ?? (props.state === "loading" ? createComponent$1(Spinner, { decorative: true }) : null);
				}),
				createComponent$1(View, {
					class: "w-full max-w-sm min-w-0 flex flex-col items-center gap-1",
					get children() {
						return [createComponent$1(Text, {
							role: "heading",
							get "class"() {
								return mergeClasses("w-full min-w-0 whitespace-normal text-sm font-medium", error() ? "text-danger-primary" : "text-primary");
							},
							get children() {
								return props.title;
							}
						}), memo(() => {
							return memo(() => {
								return props.description === void 0;
							})() ? null : createComponent$1(Text, {
								get "class"() {
									return mergeClasses("w-full min-w-0 whitespace-normal text-xs", error() ? "text-danger-primary" : "text-muted");
								},
								get children() {
									return props.description;
								}
							});
						})];
					}
				}),
				memo(() => {
					return props.renderAction?.() ?? (props.action ? createComponent$1(Button, {
						size: "sm",
						variant: "outline",
						get "aria-label"() {
							return props.action.label;
						},
						get onClick() {
							return props.action.onAction;
						},
						get children() {
							return props.action.label;
						}
					}) : null);
				})
			];
		}
	}));
}
/**
* Mutually exclusive async resource boundary.
*
* Existing content remains mounted during a background refresh. This avoids
* replacing a useful inspector or list with a loading spinner every time its
* resource is refreshed.
*/
function ResourceBoundary(props) {
	const forwarded = omit(props, "loading", "error", "hasContent", "loadingTitle", "errorTitle", "emptyTitle", "loadingDescription", "emptyDescription", "retryLabel", "onRetry", "renderContent", "renderEmptyMedia", "renderErrorMedia", "class");
	const failed = () => props.error !== void 0 && props.error !== null;
	return createComponent$1(View, mergeProps(forwarded, {
		get "class"() {
			return mergeClasses("w-full min-w-0 min-h-0 flex-1 flex flex-col", props.class);
		},
		get children() {
			return createComponent$1(Switch, { get children() {
				return [
					createComponent$1(Match, {
						get when() {
							return failed();
						},
						get children() {
							return createComponent$1(ContentState, {
								state: "error",
								get title() {
									return props.errorTitle;
								},
								get description() {
									return String(props.error);
								},
								get renderMedia() {
									return props.renderErrorMedia;
								},
								get action() {
									return memo(() => {
										return !!props.onRetry;
									})() ? {
										label: props.retryLabel ?? "Try again",
										onAction: props.onRetry
									} : void 0;
								}
							});
						}
					}),
					createComponent$1(Match, {
						get when() {
							return memo(() => {
								return !!props.loading;
							})() ? !props.hasContent : props.loading;
						},
						get children() {
							return createComponent$1(ContentState, {
								state: "loading",
								get title() {
									return props.loadingTitle;
								},
								get description() {
									return props.loadingDescription;
								}
							});
						}
					}),
					createComponent$1(Match, {
						get when() {
							return !props.hasContent;
						},
						get children() {
							return createComponent$1(ContentState, {
								state: "empty",
								get title() {
									return props.emptyTitle;
								},
								get description() {
									return props.emptyDescription;
								},
								get renderMedia() {
									return props.renderEmptyMedia;
								}
							});
						}
					}),
					createComponent$1(Match, {
						when: true,
						get children() {
							return props.renderContent();
						}
					})
				];
			} });
		}
	}));
}
//#endregion
export { ContentState, ResourceBoundary };

//# sourceMappingURL=content-state.mjs.map