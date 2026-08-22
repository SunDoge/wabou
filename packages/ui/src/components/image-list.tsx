import { VirtualList } from "@wabou/core/renderer";
import { createMemo, createSignal, type JSX, Show, untrack } from "solid-js";
import { match, P } from "ts-pattern";
import {
  Image,
  type ImageResourceErrorEvent,
  type ImageResourceReadyEvent,
  type ImageSource,
  Button as PrimitiveButton,
  Text,
  View,
} from "../primitives";

export interface ImageListProps<T> {
  /** Reactive backing collection. Only visible rows mount their Image nodes. */
  items: () => readonly T[];
  getItemKey: (item: T, index: number) => string | number;
  getSource: (item: T, index: number) => ImageSource;
  getLabel: (item: T, index: number) => string;
  getDescription?: (item: T, index: number) => string | undefined;
  itemHeight?: number;
  thumbnailWidth?: number;
  thumbnailHeight?: number;
  viewportHeight?: number;
  overscan?: number;
  selectedKey?: string | number;
  onSelectionChange?: (item: T, index: number) => void;
  onResourceReady?: (
    item: T,
    index: number,
    event: ImageResourceReadyEvent,
  ) => void;
  onResourceError?: (
    item: T,
    index: number,
    event: ImageResourceErrorEvent,
  ) => void;
  accessibilityLabel?: string;
  class?: string;
  renderTrailing?: (item: T, index: number) => JSX.Element;
  /** Optional generated thumbnail; bypasses Image resource loading for this row. */
  renderThumbnail?: (item: T, index: number) => JSX.Element;
}

function finitePositive(value: number, name: string): number {
  if (!Number.isFinite(value) || value <= 0)
    throw new RangeError(`${name} must be finite and positive`);
  return value;
}

/**
 * A virtualized, selectable image list for page strips, albums and file pickers.
 * Resource loading remains owned by Image; this component owns list lifecycle.
 */
export function ImageList<T>(props: ImageListProps<T>): JSX.Element {
  const config = untrack(() => ({
    items: props.items,
    getItemKey: props.getItemKey,
    getSource: props.getSource,
    getLabel: props.getLabel,
    getDescription: props.getDescription,
    viewportHeight: props.viewportHeight,
    overscan: props.overscan,
    onSelectionChange: props.onSelectionChange,
    onResourceReady: props.onResourceReady,
    onResourceError: props.onResourceError,
    accessibilityLabel: props.accessibilityLabel,
    class: props.class,
    renderTrailing: props.renderTrailing,
    renderThumbnail: props.renderThumbnail,
  }));
  const { itemHeight, thumbnailWidth, thumbnailHeight } = untrack(() => ({
    itemHeight: finitePositive(props.itemHeight ?? 88, "itemHeight"),
    thumbnailWidth: finitePositive(
      props.thumbnailWidth ?? 56,
      "thumbnailWidth",
    ),
    thumbnailHeight: finitePositive(
      props.thumbnailHeight ?? 72,
      "thumbnailHeight",
    ),
  }));

  return (
    <View
      class={config.class}
      classList={{
        "min-w-0": true,
        "min-h-0": true,
        "overflow-hidden": true,
      }}
      style={
        config.viewportHeight === undefined
          ? undefined
          : { height: `${config.viewportHeight}px` }
      }
    >
      <VirtualList
        items={config.items}
        itemHeight={itemHeight}
        viewportHeight={config.viewportHeight}
        overscan={config.overscan}
        getItemKey={config.getItemKey}
        role="listbox"
        accessibilityLabel={config.accessibilityLabel ?? "Images"}
        class="w-full h-full min-w-0 min-h-0"
      >
        {(item, index) => {
          const [failed, setFailed] = createSignal(false);
          const key = createMemo(() => config.getItemKey(item(), index()));
          const label = createMemo(() => config.getLabel(item(), index()));
          const source = createMemo(() => config.getSource(item(), index()));
          const description = createMemo(() =>
            config.getDescription?.(item(), index()),
          );
          const selected = createMemo(() => props.selectedKey === key());
          return (
            <PrimitiveButton
              role="option"
              aria-label={label()}
              aria-selected={selected()}
              class={(state) =>
                match({ selected: selected(), hovered: state.hovered })
                  .with({ selected: true }, () => "bg-selected text-primary")
                  .with({ hovered: true }, () => "bg-control text-primary")
                  .with(P._, () => "bg-transparent text-primary")
                  .exhaustive()
              }
              classList={{
                "w-full": true,
                "h-full": true,
                "min-w-0": true,
                flex: true,
                "flex-row": true,
                "items-center": true,
                "gap-3": true,
                "px-3": true,
                "py-2": true,
                "rounded-md": true,
                "text-left": true,
              }}
              onClick={() => config.onSelectionChange?.(item(), index())}
            >
              <View
                aria-hidden="true"
                class="flex-none flex items-center justify-center overflow-hidden rounded-sm bg-control"
                style={{
                  width: `${thumbnailWidth}px`,
                  height: `${thumbnailHeight}px`,
                }}
              >
                {config.renderThumbnail?.(item(), index()) ?? (
                  <Show when={!failed()}>
                    <Image
                      source={source()}
                      aria-label={label()}
                      class="w-full h-full"
                      onResourceReady={(event) =>
                        config.onResourceReady?.(item(), index(), event)
                      }
                      onResourceError={(event) => {
                        setFailed(true);
                        config.onResourceError?.(item(), index(), event);
                      }}
                    />
                  </Show>
                )}
              </View>
              <View class="flex-1 min-w-0 flex flex-col gap-1">
                <Text maxLines={1} class="w-full min-w-0 font-medium">
                  {label()}
                </Text>
                <Show when={description()}>
                  {(description) => (
                    <Text
                      maxLines={2}
                      class="w-full min-w-0 text-sm text-muted"
                    >
                      {description()}
                    </Text>
                  )}
                </Show>
              </View>
              {config.renderTrailing?.(item(), index())}
            </PrimitiveButton>
          );
        }}
      </VirtualList>
    </View>
  );
}
