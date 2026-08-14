import type { WabouElementProps } from "@wabou/core";
import "@wabou/solid-renderer";

declare module "@wabou/solid-renderer" {
  interface WabouIntrinsicElements {
    fractal: WabouElementProps & {
      cx?: string;
      cy?: string;
    };
  }
}
