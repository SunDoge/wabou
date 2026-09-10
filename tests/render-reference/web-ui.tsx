import type { JSX } from "solid-js";

type WebPrimitiveProps = JSX.HTMLAttributes<HTMLDivElement> & {
  class?: string;
};

type WebTextProps = WebPrimitiveProps & { maxLines?: number };

/** Minimal DOM adapter for cross-render fixtures authored with Wabou View. */
export function View(props: WebPrimitiveProps): JSX.Element {
  return <div {...props} />;
}

/** DOM counterpart for Wabou's measured block text primitive. */
export function Text(props: WebTextProps): JSX.Element {
  const { maxLines: _maxLines, ...rest } = props;
  return <div {...rest} />;
}
