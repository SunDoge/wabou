import { mergeClasses } from "@wabou/core/style";
import type { JSX } from "solid-js";
import { CodeEditor, type CodeEditorProps } from "../primitives";

export interface ConfigEditorProps extends CodeEditorProps {
  class?: string;
}

/**
 * Configuration editor backed by DOM-free CodeMirror state and a controlled
 * native viewport. It is intentionally not a general-purpose IDE editor.
 */
export function ConfigEditor(props: ConfigEditorProps): JSX.Element {
  return (
    <CodeEditor
      {...props}
      language="json"
      class={mergeClasses(
        "min-h-48 w-full rounded-lg border border-strong bg-input text-primary",
        props.class,
      )}
    />
  );
}
