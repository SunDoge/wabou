import { Editor, Text, View } from "../primitives/view.mjs";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "./disclosure.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, Show } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
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
* text is mounted only after disclosure and uses Wabou's native editor viewport
* for selection, copying, and large documents.
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
		get "aria-label"() {
			return labels().technicalDetails;
		},
		get "class"() {
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
											get "aria-label"() {
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
																get "class"() {
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
																		get "aria-label"() {
																			return labels().additions(file.additions);
																		},
																		class: "text-xs text-success-primary",
																		get children() {
																			return ["+", memo(() => {
																				return file.additions;
																			})];
																		}
																	}), createComponent$1(Text, {
																		get "aria-label"() {
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
												return createComponent$1(Editor, {
													readOnly: true,
													language: "diff",
													get value() {
														return file.patch;
													},
													get "aria-label"() {
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
export { DiffViewer };

//# sourceMappingURL=diff-viewer.mjs.map