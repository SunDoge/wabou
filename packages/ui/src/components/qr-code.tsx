import { rgba, type WabouStyle } from "@wabou/core/style";
import { createMemo, type JSX, omit } from "solid-js";
import { encode, type QrCodeGenerateData } from "uqr";
import { Path, PathBuilder, View, type ViewProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";

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

function positive(name: string, value: number): number {
  if (!Number.isFinite(value) || value <= 0) {
    throw new RangeError(`${name} must be a positive finite number`);
  }
  return value;
}

function quietZone(value: number | undefined): number {
  const resolved = value ?? 4;
  if (!Number.isInteger(resolved) || resolved < 0) {
    throw new RangeError("QRCode quietZone must be a non-negative integer");
  }
  return resolved;
}

/** Encode with uqr while keeping its renderer-independent matrix contract. */
export function encodeQrCode(
  value: QrCodeGenerateData,
  errorCorrection: QrCodeErrorCorrection = "M",
): QrCodeMatrix {
  const encoded = encode(value, { border: 0, ecc: errorCorrection });
  return { size: encoded.size, data: encoded.data };
}

function rectangle(
  path: PathBuilder,
  x: number,
  y: number,
  width: number,
  height: number,
): void {
  path
    .moveTo(x, y)
    .lineTo(x + width, y)
    .lineTo(x + width, y + height)
    .lineTo(x, y + height)
    .close();
}

/**
 * Convert consecutive dark modules into one retained Vello path. Horizontal
 * runs keep the bridge traffic and native scene node count independent of the
 * number of QR modules.
 */
export function qrCodePath(
  matrix: QrCodeMatrix,
  renderedSize: number,
  quiet: number,
  foreground = 0x000000ff,
) {
  positive("QRCode size", renderedSize);
  const border = quietZone(quiet);
  const moduleSize = renderedSize / (matrix.size + border * 2);
  const path = new PathBuilder();

  for (let row = 0; row < matrix.size; row++) {
    const modules = matrix.data[row];
    for (let column = 0; column < matrix.size; ) {
      if (!modules[column]) {
        column++;
        continue;
      }
      const start = column;
      while (column < matrix.size && modules[column]) column++;
      rectangle(
        path,
        (border + start) * moduleSize,
        (border + row) * moduleSize,
        (column - start) * moduleSize,
        moduleSize,
      );
    }
  }
  return path.build({ fill: foreground });
}

/** A QR encoder from the JS ecosystem rendered as one native vector path. */
export function QRCode(props: QRCodeProps): JSX.Element {
  const rest = omit(
    props,
    "value",
    "size",
    "errorCorrection",
    "quietZone",
    "foreground",
    "background",
    "style",
    "class",
  );
  const renderedSize = () => positive("QRCode size", props.size ?? 180);
  const matrix = createMemo(() =>
    encodeQrCode(props.value, props.errorCorrection),
  );
  const source = createMemo(() =>
    qrCodePath(
      matrix(),
      renderedSize(),
      quietZone(props.quietZone),
      props.foreground,
    ),
  );

  return (
    <View
      {...rest}
      role="img"
      aria-label={props["aria-label"] ?? "QR code"}
      class={mergeClasses("relative flex-none overflow-hidden", props.class)}
      style={{
        ...props.style,
        width: renderedSize(),
        height: renderedSize(),
        "background-color": props.background ?? rgba(0xffffffff),
      }}
    >
      <Path
        aria-hidden="true"
        class="absolute inset-0"
        style={{ width: renderedSize(), height: renderedSize() }}
        source={source()}
      />
    </View>
  );
}
