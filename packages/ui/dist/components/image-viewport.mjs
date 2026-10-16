import { Image, View } from "../primitives/view.mjs";
import { createMeasuredSize } from "../primitives/measure.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, createContext, createMemo, createSignal, omit, useContext } from "solid-js";
import { applyRef, createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/image-viewport.tsx
function positiveSize(size, name) {
	if (!Number.isFinite(size.width) || !Number.isFinite(size.height) || size.width <= 0 || size.height <= 0) throw new RangeError(`${name} width and height must be finite and positive`);
}
/** Deterministic contain + zoom + pan transform shared by paint and annotations. */
function imageViewportTransform(options) {
	positiveSize(options.viewport, "viewport");
	positiveSize(options.image, "image");
	const zoom = options.zoom ?? 1;
	if (!Number.isFinite(zoom) || zoom <= 0) throw new RangeError("image viewport zoom must be finite and positive");
	const pan = options.pan ?? {
		x: 0,
		y: 0
	};
	if (!Number.isFinite(pan.x) || !Number.isFinite(pan.y)) throw new RangeError("image viewport pan must be finite");
	const scale = Math.min(options.viewport.width / options.image.width, options.viewport.height / options.image.height) * zoom;
	const width = options.image.width * scale;
	const height = options.image.height * scale;
	const frame = {
		x: (options.viewport.width - width) / 2 + pan.x,
		y: (options.viewport.height - height) / 2 + pan.y,
		width,
		height
	};
	return {
		viewport: options.viewport,
		image: options.image,
		frame,
		scale,
		imageToViewport: (point) => ({
			x: frame.x + point.x * scale,
			y: frame.y + point.y * scale
		}),
		viewportToImage: (point) => ({
			x: (point.x - frame.x) / scale,
			y: (point.y - frame.y) / scale
		})
	};
}
const ImageViewportContext = createContext();
/** A clipped image-space viewport with one explicit, reusable coordinate model. */
function ImageViewport(props) {
	const measured = createMeasuredSize();
	const [intrinsicSize, setIntrinsicSize] = createSignal();
	const [localPan, setLocalPan] = createSignal({
		x: 0,
		y: 0
	});
	const [panDrag, setPanDrag] = createSignal();
	const pan = () => props.pan ?? localPan();
	const updatePan = (next) => {
		if (props.pan === void 0) setLocalPan(next);
		props.onPanChange?.(next);
	};
	const transform = createMemo(() => {
		const image = props.imageSize ?? intrinsicSize();
		if (!measured.measured() || measured.width() <= 0 || measured.height() <= 0) return null;
		if (!image) return null;
		return imageViewportTransform({
			viewport: {
				width: measured.width(),
				height: measured.height()
			},
			image,
			zoom: props.zoom,
			pan: pan()
		});
	});
	const frameStyle = () => {
		const frame = transform()?.frame;
		return frame ? {
			left: `${frame.x}px`,
			top: `${frame.y}px`,
			width: `${frame.width}px`,
			height: `${frame.height}px`
		} : {
			width: "0px",
			height: "0px"
		};
	};
	const rest = omit(props, "resource", "imageSize", "zoom", "pan", "pannable", "onPanChange", "media", "children", "imageLabel", "onResourceReady", "onResourceError");
	return createComponent$1(ImageViewportContext, {
		value: { transform },
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				projectionBoundary: true,
				ref(r$) {
					var _ref$ = measured.ref;
					typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
				},
				get role() {
					return props.role ?? "group";
				},
				get "class"() {
					return mergeClasses("relative min-w-0 min-h-0 overflow-hidden bg-control", props.class);
				},
				onPointerDown: (event) => {
					if (!props.pannable || event.button !== 0) return;
					event.preventDefault();
					setPanDrag({
						pointer: {
							x: event.clientX,
							y: event.clientY
						},
						pan: pan()
					});
				},
				onPointerMove: (event) => {
					const drag = panDrag();
					if (!drag) return;
					updatePan({
						x: drag.pan.x + event.clientX - drag.pointer.x,
						y: drag.pan.y + event.clientY - drag.pointer.y
					});
				},
				onPointerUp: () => setPanDrag(),
				onPointerCancel: () => setPanDrag(),
				get children() {
					return [createComponent$1(View, {
						"aria-hidden": "true",
						class: "absolute overflow-hidden bg-surface",
						get style() {
							return frameStyle();
						},
						get children() {
							return props.media ?? createComponent$1(Image, {
								get resource() {
									return props.resource;
								},
								get "aria-label"() {
									return props.imageLabel ?? "Image";
								},
								class: "w-full h-full",
								onResourceReady: (event) => {
									setIntrinsicSize({
										width: event.width,
										height: event.height
									});
									props.onResourceReady?.(event);
								},
								get onResourceError() {
									return props.onResourceError;
								}
							});
						}
					}), memo(() => {
						return props.children;
					})];
				}
			}));
		}
	});
}
function clampAnnotationRegion(region, image, minimumSize = 1) {
	positiveSize(image, "image");
	const min = Math.max(0, minimumSize);
	const x0 = Math.max(0, Math.min(image.width, region.x));
	const y0 = Math.max(0, Math.min(image.height, region.y));
	const x1 = Math.max(x0, Math.min(image.width, region.x + region.width));
	const y1 = Math.max(y0, Math.min(image.height, region.y + region.height));
	return {
		...region,
		x: Math.min(x0, Math.max(0, image.width - min)),
		y: Math.min(y0, Math.max(0, image.height - min)),
		width: Math.min(image.width - x0, Math.max(min, x1 - x0)),
		height: Math.min(image.height - y0, Math.max(min, y1 - y0))
	};
}
function draftRegion(id, start, end) {
	return {
		id,
		x: Math.min(start.x, end.x),
		y: Math.min(start.y, end.y),
		width: Math.abs(end.x - start.x),
		height: Math.abs(end.y - start.y)
	};
}
/** Editable image-space regions composed above an ImageViewport. */
function AnnotationLayer(props) {
	const viewport = useContext(ImageViewportContext);
	if (!viewport) throw new Error("AnnotationLayer must be placed inside ImageViewport");
	const [localSelected, setLocalSelected] = createSignal(null);
	const [interaction, setInteraction] = createSignal();
	let nextId = 1;
	const selected = () => props.selectedId === void 0 ? localSelected() : props.selectedId;
	const choose = (id) => {
		if (props.selectedId === void 0) setLocalSelected(id);
		props.onSelectedIdChange?.(id);
	};
	const point = (event) => {
		return viewport.transform()?.viewportToImage({
			x: event.offsetX,
			y: event.offsetY
		});
	};
	const displayedRegion = (region) => {
		const active = interaction();
		return active && active.kind !== "create" && active.id === region.id ? active.current : region;
	};
	const styleFor = (region) => {
		const transform = viewport.transform();
		if (!transform) return {
			width: "0px",
			height: "0px"
		};
		const origin = transform.imageToViewport(region);
		return {
			left: `${origin.x}px`,
			top: `${origin.y}px`,
			width: `${region.width * transform.scale}px`,
			height: `${region.height * transform.scale}px`
		};
	};
	const updateDrag = (event) => {
		const active = interaction();
		const transform = viewport.transform();
		if (!active || !transform) return;
		if (active.kind === "create") {
			const cursor = point(event);
			if (!cursor) return;
			setInteraction({
				...active,
				current: cursor
			});
			return;
		}
		const dx = (event.clientX - active.startPointer.x) / transform.scale;
		const dy = (event.clientY - active.startPointer.y) / transform.scale;
		const current = active.kind === "move" ? {
			...active.startRegion,
			x: Math.max(0, Math.min(transform.image.width - active.startRegion.width, active.startRegion.x + dx)),
			y: Math.max(0, Math.min(transform.image.height - active.startRegion.height, active.startRegion.y + dy))
		} : {
			...active.startRegion,
			width: active.startRegion.width + dx,
			height: active.startRegion.height + dy
		};
		setInteraction({
			...active,
			current: active.kind === "move" ? current : clampAnnotationRegion(current, transform.image, props.minimumSize ?? 8)
		});
	};
	const finish = () => {
		const active = interaction();
		const transform = viewport.transform();
		if (!active || !transform) return;
		if (active.kind === "create") {
			const created = clampAnnotationRegion(draftRegion(props.createRegionId?.() ?? `annotation-${nextId++}`, active.start, active.current), transform.image, props.minimumSize ?? 8);
			choose(created.id);
			props.onRegionsChange?.([...props.regions, created]);
		} else props.onRegionsChange?.(props.regions.map((region) => region.id === active.id ? active.current : region));
		setInteraction();
	};
	const beginRegion = (kind, region, event) => {
		if (event.button !== 0) return;
		event.preventDefault();
		event.stopPropagation();
		choose(region.id);
		setInteraction({
			kind,
			id: region.id,
			startPointer: {
				x: event.clientX,
				y: event.clientY
			},
			startRegion: region,
			current: region
		});
	};
	const draft = () => {
		const active = interaction();
		return active?.kind === "create" ? draftRegion("annotation-draft", active.start, active.current) : void 0;
	};
	const rest = omit(props, "regions", "selectedId", "minimumSize", "createRegionId", "interactionMode", "onRegionsChange", "onSelectedIdChange");
	return createComponent$1(View, mergeProps(rest, {
		get role() {
			return props.role ?? "group";
		},
		get "class"() {
			return mergeClasses("absolute inset-0", props.class);
		},
		get classList() {
			return { "pointer-events-none": props.interactionMode === "passthrough" };
		},
		onPointerDown: (event) => {
			if (event.button !== 0 || !viewport.transform()) return;
			const start = point(event);
			if (!start) return;
			event.preventDefault();
			choose(null);
			setInteraction({
				kind: "create",
				start,
				current: start
			});
		},
		onPointerMove: updateDrag,
		onPointerUp: finish,
		onPointerCancel: () => setInteraction(),
		get children() {
			return [createComponent$1(For, {
				get each() {
					return props.regions;
				},
				children: (region) => createComponent$1(View, {
					role: "button",
					get "aria-label"() {
						return region.label ?? `Annotation ${region.id}`;
					},
					get "aria-pressed"() {
						return selected() === region.id;
					},
					focusOrder: 0,
					get "class"() {
						return mergeClasses("absolute border-2 bg-transparent cursor-move", selected() === region.id ? "border-accent" : "border-strong");
					},
					get style() {
						return styleFor(displayedRegion(region));
					},
					onPointerDown: (event) => beginRegion("move", region, event),
					onPointerMove: updateDrag,
					onPointerUp: finish,
					onPointerCancel: () => setInteraction(),
					get children() {
						return createComponent$1(View, {
							role: "button",
							get "aria-label"() {
								return `Resize ${region.label ?? region.id}`;
							},
							class: "absolute w-7 h-7 flex items-center justify-center bg-transparent cursor-pointer",
							style: {
								right: "-8px",
								bottom: "-8px"
							},
							onPointerDown: (event) => beginRegion("resize", region, event),
							onPointerMove: updateDrag,
							onPointerUp: finish,
							onPointerCancel: () => setInteraction(),
							get children() {
								return createComponent$1(View, {
									"aria-hidden": "true",
									class: "w-3 h-3 rounded-sm border border-on-accent bg-accent pointer-events-none"
								});
							}
						});
					}
				})
			}), memo(() => {
				return (() => {
					const rect = draft();
					return rect && createComponent$1(View, {
						"aria-hidden": "true",
						class: "absolute border-2 border-accent bg-transparent pointer-events-none",
						get style() {
							return styleFor(rect);
						}
					});
				})();
			})];
		}
	}));
}
/** Read-only image-space overlays sharing the viewport's exact zoom and pan transform. */
function ImageOverlayLayer(props) {
	const viewport = useContext(ImageViewportContext);
	if (!viewport) throw new Error("ImageOverlayLayer must be placed inside ImageViewport");
	const styleFor = (item) => {
		const transform = viewport.transform();
		if (!transform) return {
			width: "0px",
			height: "0px"
		};
		const origin = transform.imageToViewport(item);
		return {
			left: `${origin.x}px`,
			top: `${origin.y}px`,
			width: `${item.width * transform.scale}px`,
			height: `${item.height * transform.scale}px`
		};
	};
	const rest = omit(props, "items", "children");
	return createComponent$1(View, mergeProps(rest, {
		get "class"() {
			return mergeClasses("absolute inset-0 pointer-events-none", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return props.items;
				},
				children: (item, index) => createComponent$1(View, {
					class: "absolute overflow-hidden",
					get style() {
						return styleFor(item);
					},
					get children() {
						return props.children(item, index);
					}
				})
			});
		}
	}));
}
//#endregion
export { AnnotationLayer, ImageOverlayLayer, ImageViewport, clampAnnotationRegion, imageViewportTransform };

//# sourceMappingURL=image-viewport.mjs.map