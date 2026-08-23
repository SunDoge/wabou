import type { WabouPointerEvent } from "@wabou/core/renderer";
import type { WabouStyle } from "@wabou/core/style";
import {
  createContext,
  createMemo,
  createSignal,
  For,
  type JSX,
  omit,
  useContext,
} from "solid-js";
import {
  createMeasuredSize,
  Image,
  type ImageResourceErrorEvent,
  type ImageResourceHandle,
  type ImageResourceReadyEvent,
  View,
  type ViewProps,
} from "../primitives";
import { join } from "./class-names";

export interface ImageViewportSize {
  width: number;
  height: number;
}

export interface ImageViewportPoint {
  x: number;
  y: number;
}

export interface ImageViewportRect extends ImageViewportPoint {
  width: number;
  height: number;
}

export interface ImageViewportTransform {
  readonly viewport: ImageViewportSize;
  readonly image: ImageViewportSize;
  readonly frame: ImageViewportRect;
  readonly scale: number;
  imageToViewport(point: ImageViewportPoint): ImageViewportPoint;
  viewportToImage(point: ImageViewportPoint): ImageViewportPoint;
}

function positiveSize(size: ImageViewportSize, name: string) {
  if (
    !Number.isFinite(size.width) ||
    !Number.isFinite(size.height) ||
    size.width <= 0 ||
    size.height <= 0
  ) {
    throw new RangeError(
      `${name} width and height must be finite and positive`,
    );
  }
}

/** Deterministic contain + zoom + pan transform shared by paint and annotations. */
export function imageViewportTransform(options: {
  viewport: ImageViewportSize;
  image: ImageViewportSize;
  zoom?: number;
  pan?: ImageViewportPoint;
}): ImageViewportTransform {
  positiveSize(options.viewport, "viewport");
  positiveSize(options.image, "image");
  const zoom = options.zoom ?? 1;
  if (!Number.isFinite(zoom) || zoom <= 0)
    throw new RangeError("image viewport zoom must be finite and positive");
  const pan = options.pan ?? { x: 0, y: 0 };
  if (!Number.isFinite(pan.x) || !Number.isFinite(pan.y))
    throw new RangeError("image viewport pan must be finite");
  const scale =
    Math.min(
      options.viewport.width / options.image.width,
      options.viewport.height / options.image.height,
    ) * zoom;
  const width = options.image.width * scale;
  const height = options.image.height * scale;
  const frame = {
    x: (options.viewport.width - width) / 2 + pan.x,
    y: (options.viewport.height - height) / 2 + pan.y,
    width,
    height,
  };
  return {
    viewport: options.viewport,
    image: options.image,
    frame,
    scale,
    imageToViewport: (point) => ({
      x: frame.x + point.x * scale,
      y: frame.y + point.y * scale,
    }),
    viewportToImage: (point) => ({
      x: (point.x - frame.x) / scale,
      y: (point.y - frame.y) / scale,
    }),
  };
}

interface ImageViewportContextValue {
  transform(): ImageViewportTransform | null;
}

const ImageViewportContext = createContext<ImageViewportContextValue>();

export interface ImageViewportProps
  extends Omit<ViewProps, "children" | "ref" | "onWheel"> {
  resource?: ImageResourceHandle;
  /** Intrinsic image size in image pixels. */
  imageSize?: ImageViewportSize;
  zoom?: number;
  pan?: ImageViewportPoint;
  /** Optional replacement for the native Image, useful for generated media. */
  media?: JSX.Element;
  children?: JSX.Element;
  imageLabel?: string;
  onResourceReady?: (event: ImageResourceReadyEvent) => void;
  onResourceError?: (event: ImageResourceErrorEvent) => void;
}

/** A clipped image-space viewport with one explicit, reusable coordinate model. */
export function ImageViewport(props: ImageViewportProps): JSX.Element {
  const measured = createMeasuredSize();
  const [intrinsicSize, setIntrinsicSize] = createSignal<ImageViewportSize>();
  const transform = createMemo(() => {
    const image = props.imageSize ?? intrinsicSize();
    if (!measured.measured() || measured.width() <= 0 || measured.height() <= 0)
      return null;
    if (!image) return null;
    return imageViewportTransform({
      viewport: { width: measured.width(), height: measured.height() },
      image,
      zoom: props.zoom,
      pan: props.pan,
    });
  });
  const frameStyle = (): WabouStyle => {
    const frame = transform()?.frame;
    return frame
      ? {
          left: `${frame.x}px`,
          top: `${frame.y}px`,
          width: `${frame.width}px`,
          height: `${frame.height}px`,
        }
      : { width: "0px", height: "0px" };
  };
  const rest = omit(
    props,
    "resource",
    "imageSize",
    "zoom",
    "pan",
    "media",
    "children",
    "imageLabel",
    "onResourceReady",
    "onResourceError",
  );
  return (
    <ImageViewportContext value={{ transform }}>
      <View
        {...rest}
        ref={measured.ref}
        role={props.role ?? "group"}
        class={join(
          "relative min-w-0 min-h-0 overflow-hidden bg-control",
          props.class,
        )}
      >
        <View
          aria-hidden="true"
          class="absolute overflow-hidden bg-surface"
          style={frameStyle()}
        >
          {props.media ?? (
            <Image
              resource={props.resource}
              aria-label={props.imageLabel ?? "Image"}
              class="w-full h-full"
              onResourceReady={(event) => {
                setIntrinsicSize({ width: event.width, height: event.height });
                props.onResourceReady?.(event);
              }}
              onResourceError={props.onResourceError}
            />
          )}
        </View>
        {props.children}
      </View>
    </ImageViewportContext>
  );
}

export interface AnnotationRegion extends ImageViewportRect {
  id: string;
  label?: string;
}

export function clampAnnotationRegion(
  region: AnnotationRegion,
  image: ImageViewportSize,
  minimumSize = 1,
): AnnotationRegion {
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
    height: Math.min(image.height - y0, Math.max(min, y1 - y0)),
  };
}

type AnnotationInteraction =
  | {
      kind: "create";
      start: ImageViewportPoint;
      current: ImageViewportPoint;
    }
  | {
      kind: "move" | "resize";
      id: string;
      startPointer: ImageViewportPoint;
      startRegion: AnnotationRegion;
      current: AnnotationRegion;
    };

export interface AnnotationLayerProps
  extends Omit<
    ViewProps,
    | "children"
    | "onPointerDown"
    | "onPointerMove"
    | "onPointerUp"
    | "onPointerCancel"
  > {
  regions: readonly AnnotationRegion[];
  selectedId?: string | null;
  minimumSize?: number;
  createRegionId?: () => string;
  onRegionsChange?: (regions: readonly AnnotationRegion[]) => void;
  onSelectedIdChange?: (id: string | null) => void;
}

function draftRegion(
  id: string,
  start: ImageViewportPoint,
  end: ImageViewportPoint,
): AnnotationRegion {
  return {
    id,
    x: Math.min(start.x, end.x),
    y: Math.min(start.y, end.y),
    width: Math.abs(end.x - start.x),
    height: Math.abs(end.y - start.y),
  };
}

/** Editable image-space regions composed above an ImageViewport. */
export function AnnotationLayer(props: AnnotationLayerProps): JSX.Element {
  const viewport = useContext(ImageViewportContext);
  if (!viewport)
    throw new Error("AnnotationLayer must be placed inside ImageViewport");
  const [localSelected, setLocalSelected] = createSignal<string | null>(null);
  const [interaction, setInteraction] = createSignal<AnnotationInteraction>();
  let nextId = 1;
  const selected = () =>
    props.selectedId === undefined ? localSelected() : props.selectedId;
  const choose = (id: string | null) => {
    if (props.selectedId === undefined) setLocalSelected(id);
    props.onSelectedIdChange?.(id);
  };
  const point = (event: WabouPointerEvent) => {
    const transform = viewport.transform();
    return transform?.viewportToImage({
      x: event.offsetX,
      y: event.offsetY,
    });
  };
  const displayedRegion = (region: AnnotationRegion) => {
    const active = interaction();
    return active && active.kind !== "create" && active.id === region.id
      ? active.current
      : region;
  };
  const styleFor = (region: AnnotationRegion): WabouStyle => {
    const transform = viewport.transform();
    if (!transform) return { width: "0px", height: "0px" };
    const origin = transform.imageToViewport(region);
    return {
      left: `${origin.x}px`,
      top: `${origin.y}px`,
      width: `${region.width * transform.scale}px`,
      height: `${region.height * transform.scale}px`,
    };
  };
  const updateDrag = (event: WabouPointerEvent) => {
    if (event.buttons === 0) return;
    const active = interaction();
    const transform = viewport.transform();
    if (!active || !transform) return;
    if (active.kind === "create") {
      const cursor = point(event);
      if (!cursor) return;
      setInteraction({ ...active, current: cursor });
      return;
    }
    const dx = (event.clientX - active.startPointer.x) / transform.scale;
    const dy = (event.clientY - active.startPointer.y) / transform.scale;
    const current =
      active.kind === "move"
        ? {
            ...active.startRegion,
            x: Math.max(
              0,
              Math.min(
                transform.image.width - active.startRegion.width,
                active.startRegion.x + dx,
              ),
            ),
            y: Math.max(
              0,
              Math.min(
                transform.image.height - active.startRegion.height,
                active.startRegion.y + dy,
              ),
            ),
          }
        : {
            ...active.startRegion,
            width: active.startRegion.width + dx,
            height: active.startRegion.height + dy,
          };
    setInteraction({
      ...active,
      current:
        active.kind === "move"
          ? current
          : clampAnnotationRegion(
              current,
              transform.image,
              props.minimumSize ?? 8,
            ),
    });
  };
  const finish = () => {
    const active = interaction();
    const transform = viewport.transform();
    if (!active || !transform) return;
    if (active.kind === "create") {
      const created = clampAnnotationRegion(
        draftRegion(
          props.createRegionId?.() ?? `annotation-${nextId++}`,
          active.start,
          active.current,
        ),
        transform.image,
        props.minimumSize ?? 8,
      );
      choose(created.id);
      props.onRegionsChange?.([...props.regions, created]);
    } else {
      props.onRegionsChange?.(
        props.regions.map((region) =>
          region.id === active.id ? active.current : region,
        ),
      );
    }
    setInteraction();
  };
  const beginRegion = (
    kind: "move" | "resize",
    region: AnnotationRegion,
    event: WabouPointerEvent,
  ) => {
    if (event.button !== 0) return;
    event.preventDefault();
    event.stopPropagation();
    choose(region.id);
    setInteraction({
      kind,
      id: region.id,
      startPointer: { x: event.clientX, y: event.clientY },
      startRegion: region,
      current: region,
    });
  };
  const draft = () => {
    const active = interaction();
    return active?.kind === "create"
      ? draftRegion("annotation-draft", active.start, active.current)
      : undefined;
  };
  const rest = omit(
    props,
    "regions",
    "selectedId",
    "minimumSize",
    "createRegionId",
    "onRegionsChange",
    "onSelectedIdChange",
  );
  return (
    <View
      {...rest}
      role={props.role ?? "group"}
      class={join("absolute inset-0", props.class)}
      onPointerDown={(event) => {
        if (event.button !== 0 || !viewport.transform()) return;
        const start = point(event);
        if (!start) return;
        event.preventDefault();
        choose(null);
        setInteraction({ kind: "create", start, current: start });
      }}
      onPointerMove={updateDrag}
      onPointerUp={finish}
      onPointerCancel={() => setInteraction()}
    >
      <For each={props.regions}>
        {(region) => (
          <View
            role="button"
            aria-label={region.label ?? `Annotation ${region.id}`}
            aria-pressed={selected() === region.id}
            focusOrder={0}
            class={join(
              "absolute border-2 bg-transparent cursor-move",
              selected() === region.id ? "border-accent" : "border-strong",
            )}
            style={styleFor(displayedRegion(region))}
            onPointerDown={(event) => beginRegion("move", region, event)}
            onPointerMove={updateDrag}
            onPointerUp={finish}
            onPointerCancel={() => setInteraction()}
          >
            <View
              role="button"
              aria-label={`Resize ${region.label ?? region.id}`}
              class="absolute w-3 h-3 rounded-sm border border-on-accent bg-accent cursor-pointer"
              style={{ right: "0px", bottom: "0px" }}
              onPointerDown={(event) => beginRegion("resize", region, event)}
              onPointerMove={updateDrag}
              onPointerUp={finish}
              onPointerCancel={() => setInteraction()}
            />
          </View>
        )}
      </For>
      {draft() && (
        <View
          aria-hidden="true"
          class="absolute border-2 border-accent bg-transparent pointer-events-none"
          style={styleFor(draft()!)}
        />
      )}
    </View>
  );
}
