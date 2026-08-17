import type { WabouIntrinsicElements } from "@wabou/solid-renderer";

declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements extends WabouIntrinsicElements {}
  }
}
