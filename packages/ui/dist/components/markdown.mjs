import { useReducedMotion } from "../animation/config.mjs";
import { createKeyframeAnimation } from "../animation.mjs";
import { RichText, RichTextSpan, Text, View } from "../primitives/view.mjs";
import { CodeBlock } from "./code-block.mjs";
import { MarkdownDocument } from "./markdown-model.mjs";
import { Separator } from "./separator.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, Match, Switch, createMemo, untrack } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
//#region src/components/markdown.tsx
function runClass(run) {
	return mergeClasses(run.style.strong && "font-semibold text-primary", run.style.emphasis && "italic text-primary", run.style.code && "font-mono font-normal text-primary", run.style.deleted && "text-muted", run.style.href && "font-medium text-accent");
}
function InlineMarkdown(props) {
	return createComponent$1(RichText, {
		get "class"() {
			return mergeClasses("min-w-0 whitespace-normal", props.variant === "conversation" ? "text-base leading-relaxed text-primary" : props.variant === "prompt" ? "text-sm leading-relaxed text-primary" : "text-base leading-relaxed text-secondary", props.class);
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
		get "class"() {
			return runClass(props.run);
		},
		get "data-motion"() {
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
			case 1: return "text-xl font-semibold tracking-tight text-primary whitespace-normal";
			case 2: return "text-lg font-semibold tracking-tight text-primary whitespace-normal";
			case 3: return "text-base font-semibold tracking-tight text-primary whitespace-normal";
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
		get "class"() {
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
		get "class"() {
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
							get "class"() {
								return mergeClasses("flex-none leading-relaxed text-secondary", props.variant === "prompt" ? "text-sm" : "text-base");
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
					get "class"() {
						return mergeClasses("min-w-0 flex flex-row border-b border-subtle", rowIndex() === 0 ? "bg-control" : "bg-surface");
					},
					get children() {
						return createComponent$1(For, {
							each: row,
							children: (runs, columnIndex) => createComponent$1(View, {
								get "class"() {
									return mergeClasses("min-w-0 flex-1", props.variant === "document" ? "px-3 py-2" : "px-2 py-1.5", columnIndex() + 1 < row.length && "border-r border-subtle");
								},
								get children() {
									return createComponent$1(InlineMarkdown, {
										runs,
										get variant() {
											return props.variant;
										},
										get "class"() {
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
	return createComponent$1(Switch, { get children() {
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
		get "aria-label"() {
			return props["aria-label"] ?? "Markdown";
		},
		get "class"() {
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
export { Markdown };

//# sourceMappingURL=markdown.mjs.map