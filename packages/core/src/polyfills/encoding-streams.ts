import {
  TextDecoderStream,
  TextEncoderStream,
} from "@stardazed/streams-text-encoding";
import { installMissingGlobals } from "./globals";

const encodingStreamGlobals = { TextDecoderStream, TextEncoderStream };

/** Install the Encoding Standard stream transforms missing from QuickJS. */
export function installEncodingStreamsPolyfill(): void {
  installMissingGlobals(encodingStreamGlobals);
}

installEncodingStreamsPolyfill();
