import type { WabouElementProps } from "@wabou/core";
import "@wabou/core/registry";

declare module "@wabou/core/registry" {
  interface WabouIntrinsicElements {
    fractal: WabouElementProps & {
      cx?: string;
      cy?: string;
    };
  }
}
