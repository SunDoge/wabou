import "solid-js";

declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements {
      "secure-input": {
        class?: string;
        placeholder?: string;
      };
    }
  }
}
