import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { WabouStyle } from "@wabou/core/style";
import { JSX } from "solid-js";
import { QrCodeGenerateData } from "uqr";
//#region src/components/qr-code.d.ts
export type QrCodeErrorCorrection = "L" | "M" | "Q" | "H";
export interface QrCodeMatrix {
  readonly size: number;
  readonly data: readonly (readonly boolean[])[];
}
export interface QRCodeProps extends Omit<ViewProps, "children" | "style"> {
  value: QrCodeGenerateData;
  /** Logical-pixel width and height. The renderer keeps the code square. */
  size?: number;
  /** Error recovery level. Defaults to medium for application UI. */
  errorCorrection?: QrCodeErrorCorrection;
  /** Number of empty modules around the encoded matrix. */
  quietZone?: number;
  /** Packed RGBA (`0xRRGGBBAA`) used for dark modules. */
  foreground?: number;
  /** Background paint accepted by Wabou Style IR. */
  background?: WabouStyle["background-color"];
  style?: WabouStyle;
}
/** Encode with uqr while keeping its renderer-independent matrix contract. */
export declare function encodeQrCode(value: QrCodeGenerateData, errorCorrection?: QrCodeErrorCorrection): QrCodeMatrix;
/**
 * Convert consecutive dark modules into one retained native path. Horizontal
 * runs keep the bridge traffic and native scene node count independent of the
 * number of QR modules.
 */
export declare function qrCodePath(matrix: QrCodeMatrix, renderedSize: number, quiet: number, foreground?: number): import("@wabou/core").VectorPath;
/** A QR encoder from the JS ecosystem rendered as one native vector path. */
export declare function QRCode(props: QRCodeProps): JSX.Element;
//#endregion
//# sourceMappingURL=qr-code.d.mts.map