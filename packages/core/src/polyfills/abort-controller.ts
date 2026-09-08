import AbortControllerPolyfill, {
  AbortSignal as AbortSignalPolyfill,
} from "abort-controller/dist/abort-controller";
import { installMissingGlobals } from "./globals";

/** Install cancellation primitives when the embedding runtime lacks them. */
export function installAbortControllerPolyfill(): void {
  installMissingGlobals({
    AbortController: AbortControllerPolyfill,
    AbortSignal: AbortSignalPolyfill,
  });
}

installAbortControllerPolyfill();
