import type { JSX } from "solid-js";
import { Text, View, type ViewProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";
import { CopyButton } from "./copy-button";

export interface CodeBlockProps extends Omit<ViewProps, "children"> {
  code: string;
  language?: string;
  copyable?: boolean;
}

export function CodeBlock(props: CodeBlockProps): JSX.Element {
  return (
    <View
      role="group"
      aria-label={props["aria-label"] ?? "Code block"}
      class={mergeClasses(
        "min-w-0 overflow-hidden rounded-lg border border-subtle bg-control",
        props.class,
      )}
    >
      <View class="h-9 flex flex-row items-center justify-between gap-3 px-3 bg-control">
        <Text class="min-w-0 text-xs text-muted">
          {props.language ?? "text"}
        </Text>
        {props.copyable !== false && (
          <CopyButton value={props.code} variant="ghost" size="sm" />
        )}
      </View>
      <Text class="min-w-0 p-3 font-mono text-sm text-primary whitespace-normal">
        {props.code}
      </Text>
    </View>
  );
}
