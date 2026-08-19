import {
  CodeEditor,
  type CodeEditorProps,
} from "../primitives";
import type { JSX } from "solid-js";
import { join } from "./class-names";

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
