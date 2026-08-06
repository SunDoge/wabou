import "solid-js";

declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements {
      fractal: {
        class?: string;
        cx?: string;
        cy?: string;
      };
    }
  }
}
