import "@wabou/solid-renderer";

declare module "@wabou/solid-renderer" {
  interface WabouIntrinsicElements {
    fractal: import("@wabou/solid-renderer").WabouElementProps & {
      cx?: string;
      cy?: string;
    };
  }
}
