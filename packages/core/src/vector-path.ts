/** Stable, renderer-independent vector path command stream. */
const PATH_MAGIC = 0x31504257; // `WBP1`, little endian
const PATH_VERSION = 1;
const HEADER_SIZE = 36;
const MAX_PATH_BYTES = 16 * 1024 * 1024;

const COMMAND = {
  MoveTo: 1,
  LineTo: 2,
  QuadTo: 3,
  CubicTo: 4,
  Close: 5,
} as const;

export type PathFillRule = "nonzero" | "evenodd";
export type PathLineCap = "butt" | "round" | "square";
export type PathLineJoin = "miter" | "round" | "bevel";

export interface VectorPathPaint {
  /** Packed RGBA (`0xRRGGBBAA`). Omit to disable filling. */
  fill?: number;
  /** Packed RGBA (`0xRRGGBBAA`). Omit to disable stroking. */
  stroke?: number;
  strokeWidth?: number;
  fillRule?: PathFillRule;
  lineCap?: PathLineCap;
  lineJoin?: PathLineJoin;
  miterLimit?: number;
}

/** Immutable path snapshot suitable for signals, memos, and component props. */
export interface VectorPath {
  readonly kind: "wabou-vector-path";
  readonly data: Uint8Array;
}

type Command = readonly [number, ...number[]];

function finite(name: string, values: readonly number[]): void {
  if (!values.every(Number.isFinite))
    throw new RangeError(`${name} requires finite coordinates`);
}

function rgba(value: number | undefined, fallback: number): number {
  if (value === undefined) return fallback;
  if (!Number.isInteger(value) || value < 0 || value > 0xffff_ffff)
    throw new RangeError("path colors must be packed 32-bit RGBA values");
  return value >>> 0;
}

function positive(name: string, value: number): number {
  if (!Number.isFinite(value) || value <= 0)
    throw new RangeError(`${name} must be a positive finite number`);
  return value;
}

export class PathBuilder {
  readonly #commands: Command[] = [];

  moveTo(x: number, y: number): this {
    finite("moveTo", [x, y]);
    this.#commands.push([COMMAND.MoveTo, x, y]);
    return this;
  }

  lineTo(x: number, y: number): this {
    finite("lineTo", [x, y]);
    this.#commands.push([COMMAND.LineTo, x, y]);
    return this;
  }

  quadTo(cx: number, cy: number, x: number, y: number): this {
    finite("quadTo", [cx, cy, x, y]);
    this.#commands.push([COMMAND.QuadTo, cx, cy, x, y]);
    return this;
  }

  cubicTo(
    c1x: number,
    c1y: number,
    c2x: number,
    c2y: number,
    x: number,
    y: number,
  ): this {
    finite("cubicTo", [c1x, c1y, c2x, c2y, x, y]);
    this.#commands.push([COMMAND.CubicTo, c1x, c1y, c2x, c2y, x, y]);
    return this;
  }

  close(): this {
    this.#commands.push([COMMAND.Close]);
    return this;
  }

  /** Create an immutable snapshot. Later builder mutations cannot alter it. */
  build(paint: VectorPathPaint = {}): VectorPath {
    const resolved = Object.freeze({
      fill: rgba(paint.fill, 0x00000000),
      stroke: rgba(paint.stroke, 0x00000000),
      strokeWidth: positive("strokeWidth", paint.strokeWidth ?? 1),
      fillRule: paint.fillRule ?? "nonzero",
      lineCap: paint.lineCap ?? "butt",
      lineJoin: paint.lineJoin ?? "miter",
      miterLimit: positive("miterLimit", paint.miterLimit ?? 4),
    });
    const byteLength = HEADER_SIZE + this.#commands.reduce(
      (size, command) => size + 4 + (command.length - 1) * 4,
      0,
    );
    if (byteLength > MAX_PATH_BYTES)
      throw new RangeError("vector path exceeds the 16 MiB protocol limit");
    const data = new Uint8Array(byteLength);
    const view = new DataView(data.buffer);
    view.setUint32(0, PATH_MAGIC, true);
    view.setUint16(4, PATH_VERSION, true);
    view.setUint16(6, 0, true);
    view.setUint32(8, this.#commands.length, true);
    view.setUint32(12, byteLength, true);
    view.setUint32(16, resolved.fill, true);
    view.setUint32(20, resolved.stroke, true);
    view.setFloat32(24, resolved.strokeWidth, true);
    view.setUint8(28, resolved.fillRule === "evenodd" ? 1 : 0);
    view.setUint8(29, resolved.lineCap === "round" ? 1 : resolved.lineCap === "square" ? 2 : 0);
    view.setUint8(30, resolved.lineJoin === "round" ? 1 : resolved.lineJoin === "bevel" ? 2 : 0);
    view.setUint8(31, 0);
    view.setFloat32(32, resolved.miterLimit, true);
    let offset = HEADER_SIZE;
    for (const command of this.#commands) {
      view.setUint8(offset, command[0]);
      offset += 4; // command byte + three reserved bytes
      for (let index = 1; index < command.length; index++) {
        view.setFloat32(offset, command[index], true);
        offset += 4;
      }
    }
    return Object.freeze({
      kind: "wabou-vector-path" as const,
      get data() {
        return data.slice();
      },
    });
  }
}

export function isVectorPath(value: unknown): value is VectorPath {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as { kind?: unknown }).kind === "wabou-vector-path" &&
    (value as { data?: unknown }).data instanceof Uint8Array
  );
}
