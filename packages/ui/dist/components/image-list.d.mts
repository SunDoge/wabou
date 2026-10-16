import { ImageResourceErrorEvent, ImageResourceHandle, ImageResourceReadyEvent } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/image-list.d.ts
export interface ImageListProps<T> {
  /** Reactive backing collection. The renderer materializes only visible native rows. */
  items: () => readonly T[];
  getItemKey: (item: T, index: number) => string | number;
  getResource?: (item: T, index: number) => ImageResourceHandle;
  getLabel: (item: T, index: number) => string;
  getDescription?: (item: T, index: number) => string | undefined;
  itemHeight?: number;
  thumbnailWidth?: number;
  thumbnailHeight?: number;
  viewportHeight?: number;
  selectedKey?: string | number;
  onSelectionChange?: (item: T, index: number) => void;
  onResourceReady?: (item: T, index: number, event: ImageResourceReadyEvent) => void;
  onResourceError?: (item: T, index: number, event: ImageResourceErrorEvent) => void;
  accessibilityLabel?: string;
  class?: string;
  renderTrailing?: (item: T, index: number) => JSX.Element;
  /** Optional generated thumbnail; bypasses Image resource loading for this row. */
  renderThumbnail?: (item: T, index: number) => JSX.Element;
}
/**
 * A virtualized, selectable image list for page strips, albums and file pickers.
 * Resource creation remains owned by the caller; this component only borrows handles.
 */
export declare function ImageList<T>(props: ImageListProps<T>): JSX.Element;
//#endregion
//# sourceMappingURL=image-list.d.mts.map