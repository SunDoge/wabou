import { PrimitiveProps } from "@wabou/ui/primitives";
import { JSX } from "solid-js";
//#region generated/index.d.ts
export type PdfViewerConfig = {
  /**  Native filesystem path to the PDF document. */
  path: string;
  /**  One-based page number. */
  page?: number;
  /**  Raster pixels generated for each PDF point. */
  renderScale?: number;
  /**  Straight-alpha RGBA8 background behind transparent page content. */
  background?: [number, number, number, number];
};
export type PdfViewProps = Omit<PrimitiveProps, "children"> & PdfViewerConfig;
/** Typed Solid wrapper for the Rust `pdf-view` native widget. */
export declare function PdfView(props: PdfViewProps): JSX.Element;
//#endregion
//# sourceMappingURL=index.d.mts.map