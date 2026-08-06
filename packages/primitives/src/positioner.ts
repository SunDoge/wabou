import {
  type ComputePositionReturn,
  computePosition,
  type Middleware,
  type Placement,
  type Platform,
  type Strategy,
} from "@floating-ui/core";
import type {
  Host,
  LayoutRect as HostLayoutRect,
  LayoutTarget,
} from "@wabou/solid-renderer";

export type LayoutRect = HostLayoutRect;

export interface PositionPlatform<T> {
  getRect(target: T): LayoutRect | Promise<LayoutRect>;
  getClippingRect(target: T): LayoutRect | Promise<LayoutRect>;
  isRTL?(target: T): boolean | Promise<boolean>;
}

export interface ComputeFloatingPositionOptions<T> {
  platform: PositionPlatform<T>;
  placement?: Placement;
  strategy?: Strategy;
  middleware?: Array<Middleware | null | undefined | false>;
}

/**
 * Position two Wabou layout targets with Floating UI's renderer-independent
 * geometry engine. Measurement remains host-owned and is supplied explicitly;
 * no DOM-compatible Handle methods are required.
 */
export function computeFloatingPosition<T>(
  reference: T,
  floating: T,
  options: ComputeFloatingPositionOptions<T>,
): Promise<ComputePositionReturn> {
  const isRTL = options.platform.isRTL;
  const platform: Platform = {
    async getElementRects({ reference, floating }) {
      const [referenceRect, floatingRect] = await Promise.all([
        options.platform.getRect(reference as T),
        options.platform.getRect(floating as T),
      ]);
      return {
        reference: referenceRect,
        floating: {
          x: 0,
          y: 0,
          width: floatingRect.width,
          height: floatingRect.height,
        },
      };
    },
    getDimensions: (target) => options.platform.getRect(target as T),
    getClippingRect: ({ element }) =>
      options.platform.getClippingRect(element as T),
    isElement: () => true,
    isRTL: isRTL ? (target) => isRTL(target as T) : undefined,
  };

  return computePosition(reference, floating, {
    platform,
    placement: options.placement,
    strategy: options.strategy,
    middleware: options.middleware,
  });
}

export type ComputeHostFloatingPositionOptions = Omit<
  ComputeFloatingPositionOptions<LayoutTarget>,
  "platform"
>;

export class LayoutTargetUnavailableError extends Error {
  override readonly name = "LayoutTargetUnavailableError";
}

/** Position two native handles from a single coherent Host layout snapshot. */
export function computeHostFloatingPosition(
  reference: LayoutTarget,
  floating: LayoutTarget,
  host: { readonly layout: Pick<Host["layout"], "snapshot"> },
  options: ComputeHostFloatingPositionOptions = {},
): Promise<ComputePositionReturn> {
  const snapshot = host.layout.snapshot([reference, floating]);
  const nodes = new Map(snapshot.nodes.map((node) => [node.id, node]));
  const id = (target: LayoutTarget) =>
    typeof target === "number" ? target : target.id;
  const rect = (target: LayoutTarget) => {
    const targetId = id(target);
    const node = nodes.get(targetId);
    if (!node) {
      throw new LayoutTargetUnavailableError(
        `Layout target ${targetId} is not present in completed revision ${snapshot.revision}`,
      );
    }
    return node;
  };
  return computeFloatingPosition(reference, floating, {
    ...options,
    platform: {
      getRect: (target) => rect(target).rect,
      getClippingRect: (target) => rect(target).clip,
    },
  });
}

export type {
  ComputePositionReturn,
  Middleware,
  Placement,
  Strategy,
} from "@floating-ui/core";
export {
  arrow,
  autoPlacement,
  flip,
  offset,
  shift,
  size,
} from "@floating-ui/core";
