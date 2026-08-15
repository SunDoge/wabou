import {
  CodeEditor,
  type CodeEditorProps,
} from "@wabou/primitives";
import type { JSX } from "solid-js";

const join = (...values: Array<string | undefined | false>) =>
  values.filter(Boolean).join(" ");

export interface ConfigEditorProps extends CodeEditorProps {
  class?: string;
}

/**
 * Experimental native configuration editor. Its Wabou-owned props deliberately
 * hide the editor-core implementation so the backend can evolve independently.
 */
export function ConfigEditor(props: ConfigEditorProps): JSX.Element {
  return (
    <CodeEditor
      {...props}
      language="json"
      class={join(
        "min-h-48 w-full rounded-md border border-strong bg-input text-primary",
        props.class,
      )}
    />
  );
}
