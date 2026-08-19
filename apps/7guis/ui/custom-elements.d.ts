import type { WabouIntrinsicElements } from "@wabou/ui";

declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements extends WabouIntrinsicElements {}
  }
}
