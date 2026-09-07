import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { NativeWidget, type NativeWidgetProps } from "../primitives";

export interface ShaderLayerConfig {
  /** WGSL defining `wabou_effect`, or a complete module when `module` is true. */
  source: string;
  /** Accept a complete single-pass module exposing `vs_main` and `fs_main`. */
  module?: boolean;
  /** Up to 256 scalar values exposed through `wabou.values` or binding zero. */
  values?: readonly number[];
  /** Initial animation time in seconds. */
  time?: number;
  /** Native animation clock multiplier. */
  speed?: number;
  /** Whether the native frame clock advances time. */
  animated?: boolean;
  /** Temporarily freeze the native frame clock. */
  paused?: boolean;
}

export interface ShaderLayerProps
  extends Omit<
      NativeWidgetProps<ShaderLayerConfig>,
      "tag" | "config" | "children"
    >,
    ShaderLayerConfig {
  class?: string;
}

/** A layout-aware custom WGSL surface rendered and animated by the Vello host. */
export function ShaderLayer(props: ShaderLayerProps): JSX.Element {
  const forwarded = omit(
    props,
    "source",
    "module",
    "values",
    "time",
    "speed",
    "animated",
    "paused",
    "class",
  );
  const config = (): ShaderLayerConfig => ({
    source: props.source,
    module: props.module ?? false,
    values: props.values ?? [],
    time: props.time ?? 0,
    speed: props.speed ?? 1,
    animated: props.animated ?? true,
    paused: props.paused ?? false,
  });
  return (
    <NativeWidget
      {...forwarded}
      tag="shader-layer"
      role={props.role ?? "img"}
      aria-label={props["aria-label"] ?? "Custom GPU effect"}
      class={mergeClasses("overflow-hidden", props.class)}
      config={config()}
    />
  );
}
