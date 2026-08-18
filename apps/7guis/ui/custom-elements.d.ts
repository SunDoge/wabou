import type { WabouIntrinsicElements } from "@wabou/core/registry";

declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements extends WabouIntrinsicElements {}
  }
}
