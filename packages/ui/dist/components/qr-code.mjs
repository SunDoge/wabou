import { Path, PathBuilder, View } from "../primitives/view.mjs";
import { mergeClasses, rgba } from "@wabou/core/style";
import { createMemo, omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { encode } from "uqr";
//#region src/components/qr-code.tsx
function positive(name, value) {
	if (!Number.isFinite(value) || value <= 0) throw new RangeError(`${name} must be a positive finite number`);
	return value;
}
function quietZone(value) {
	const resolved = value ?? 4;
	if (!Number.isInteger(resolved) || resolved < 0) throw new RangeError("QRCode quietZone must be a non-negative integer");
	return resolved;
}
/** Encode with uqr while keeping its renderer-independent matrix contract. */
function encodeQrCode(value, errorCorrection = "M") {
	const encoded = encode(value, {
		border: 0,
		ecc: errorCorrection
	});
	return {
		size: encoded.size,
		data: encoded.data
	};
}
function rectangle(path, x, y, width, height) {
	path.moveTo(x, y).lineTo(x + width, y).lineTo(x + width, y + height).lineTo(x, y + height).close();
}
/**
* Convert consecutive dark modules into one retained native path. Horizontal
* runs keep the bridge traffic and native scene node count independent of the
* number of QR modules.
*/
function qrCodePath(matrix, renderedSize, quiet, foreground = 255) {
	positive("QRCode size", renderedSize);
	const border = quietZone(quiet);
	const moduleSize = renderedSize / (matrix.size + border * 2);
	const path = new PathBuilder();
	for (let row = 0; row < matrix.size; row++) {
		const modules = matrix.data[row];
		for (let column = 0; column < matrix.size;) {
			if (!modules[column]) {
				column++;
				continue;
			}
			const start = column;
			while (column < matrix.size && modules[column]) column++;
			rectangle(path, (border + start) * moduleSize, (border + row) * moduleSize, (column - start) * moduleSize, moduleSize);
		}
	}
	return path.build({ fill: foreground });
}
/** A QR encoder from the JS ecosystem rendered as one native vector path. */
function QRCode(props) {
	const rest = omit(props, "value", "size", "errorCorrection", "quietZone", "foreground", "background", "style", "class");
	const renderedSize = () => positive("QRCode size", props.size ?? 180);
	const matrix = createMemo(() => encodeQrCode(props.value, props.errorCorrection));
	const source = createMemo(() => qrCodePath(matrix(), renderedSize(), quietZone(props.quietZone), props.foreground));
	return createComponent$1(View, mergeProps(rest, {
		role: "img",
		get "aria-label"() {
			return props["aria-label"] ?? "QR code";
		},
		get "class"() {
			return mergeClasses("relative flex-none overflow-hidden", props.class);
		},
		get style() {
			return {
				...props.style,
				width: renderedSize(),
				height: renderedSize(),
				"background-color": props.background ?? rgba(4294967295)
			};
		},
		get children() {
			return createComponent$1(Path, {
				"aria-hidden": "true",
				class: "absolute inset-0",
				get style() {
					return {
						width: renderedSize(),
						height: renderedSize()
					};
				},
				get source() {
					return source();
				}
			});
		}
	}));
}
//#endregion
export { QRCode, encodeQrCode, qrCodePath };

//# sourceMappingURL=qr-code.mjs.map