import { Image, Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, createMemo, createSignal, untrack } from "solid-js";
import { VirtualList, createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import { P, match } from "ts-pattern";
//#region src/components/image-list.tsx
function finitePositive(value, name) {
	if (!Number.isFinite(value) || value <= 0) throw new RangeError(`${name} must be finite and positive`);
	return value;
}
/**
* A virtualized, selectable image list for page strips, albums and file pickers.
* Resource creation remains owned by the caller; this component only borrows handles.
*/
function ImageList(props) {
	const config = untrack(() => ({
		items: props.items,
		getItemKey: props.getItemKey,
		getResource: props.getResource,
		getLabel: props.getLabel,
		getDescription: props.getDescription,
		viewportHeight: props.viewportHeight,
		onSelectionChange: props.onSelectionChange,
		onResourceReady: props.onResourceReady,
		onResourceError: props.onResourceError,
		accessibilityLabel: props.accessibilityLabel,
		class: props.class,
		renderTrailing: props.renderTrailing,
		renderThumbnail: props.renderThumbnail
	}));
	const { itemHeight, thumbnailWidth, thumbnailHeight } = untrack(() => ({
		itemHeight: finitePositive(props.itemHeight ?? 88, "itemHeight"),
		thumbnailWidth: finitePositive(props.thumbnailWidth ?? 56, "thumbnailWidth"),
		thumbnailHeight: finitePositive(props.thumbnailHeight ?? 72, "thumbnailHeight")
	}));
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("min-w-0 min-h-0 overflow-hidden", config.class);
		},
		get style() {
			return memo(() => {
				return config.viewportHeight === void 0;
			})() ? void 0 : { height: `${config.viewportHeight}px` };
		},
		get children() {
			return createComponent$1(VirtualList, {
				get items() {
					return config.items;
				},
				itemHeight,
				get viewportHeight() {
					return config.viewportHeight;
				},
				get getItemKey() {
					return config.getItemKey;
				},
				role: "listbox",
				get accessibilityLabel() {
					return config.accessibilityLabel ?? "Images";
				},
				class: "w-full h-full min-w-0 min-h-0",
				children: (item, index) => {
					const [failed, setFailed] = createSignal(false);
					const key = createMemo(() => config.getItemKey(item(), index()));
					const label = createMemo(() => config.getLabel(item(), index()));
					const resource = createMemo(() => config.getResource?.(item(), index()));
					const description = createMemo(() => config.getDescription?.(item(), index()));
					const selected = createMemo(() => props.selectedKey === key());
					return createComponent$1(Button, {
						unstyled: true,
						role: "option",
						get "aria-label"() {
							return label();
						},
						get "aria-selected"() {
							return selected();
						},
						get selected() {
							return selected();
						},
						class: (state) => mergeClasses("w-full h-full min-w-0 flex flex-row items-center gap-3 px-3 py-2 text-left", match({
							selected: selected(),
							hovered: state.hovered
						}).with({ selected: true }, () => "bg-selected text-primary").with({ hovered: true }, () => "bg-control text-primary").with(P._, () => "bg-transparent text-primary").exhaustive()),
						onClick: () => config.onSelectionChange?.(item(), index()),
						get children() {
							return [
								createComponent$1(View, {
									"aria-hidden": "true",
									class: "flex-none flex items-center justify-center overflow-hidden bg-control",
									style: {
										width: `${thumbnailWidth}px`,
										height: `${thumbnailHeight}px`
									},
									get children() {
										return config.renderThumbnail?.(item(), index()) ?? createComponent$1(Show, {
											get when() {
												return !failed();
											},
											get children() {
												return createComponent$1(Image, {
													get resource() {
														return resource();
													},
													get "aria-label"() {
														return label();
													},
													class: "w-full h-full",
													onResourceReady: (event) => config.onResourceReady?.(item(), index(), event),
													onResourceError: (event) => {
														setFailed(true);
														config.onResourceError?.(item(), index(), event);
													}
												});
											}
										});
									}
								}),
								createComponent$1(View, {
									class: "flex-1 min-w-0 flex flex-col gap-1",
									get children() {
										return [createComponent$1(Text, {
											maxLines: 1,
											class: "w-full min-w-0 font-medium",
											get children() {
												return label();
											}
										}), createComponent$1(Show, {
											get when() {
												return description();
											},
											children: (description) => createComponent$1(Text, {
												maxLines: 2,
												class: "w-full min-w-0 text-sm text-muted",
												get children() {
													return description();
												}
											})
										})];
									}
								}),
								memo(() => {
									return config.renderTrailing?.(item(), index());
								})
							];
						}
					});
				}
			});
		}
	});
}
//#endregion
export { ImageList };

//# sourceMappingURL=image-list.mjs.map