import type { JSX } from "solid-js";

type WebPrimitiveProps = JSX.HTMLAttributes<HTMLDivElement> & {
  class?: string;
};

/** Minimal DOM adapter for cross-render fixtures authored with Wabou View. */
export function View(props: WebPrimitiveProps): JSX.Element {
  return <div {...props} />;
}
