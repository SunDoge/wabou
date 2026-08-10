import type { WabouElementProps } from "@wabou/core";
import "solid-js";

declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements {
      fractal: WabouElementProps & {
        cx?: string;
        cy?: string;
      };
    }
  }
}
