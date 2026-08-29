import { mergeClasses } from "@wabou/core/style";
import check from "lucide-static/icons/check.svg?raw";
import copy from "lucide-static/icons/copy.svg?raw";
import type { JSX } from "solid-js";
import { Icon, Text, View, type ViewProps } from "../primitives";
import { CopyButton } from "./copy-button";

export interface CodeBlockProps extends Omit<ViewProps, "children"> {
  code: string;
  language?: string;
  copyable?: boolean;
  copyLabel?: string;
  copiedLabel?: string;
}

export function CodeBlock(props: CodeBlockProps): JSX.Element {
  return (
    <View
      role="group"
      aria-label={props["aria-label"] ?? "Code block"}
      class={mergeClasses(
        "min-w-0 overflow-hidden rounded-xl border border-subtle bg-control",
        props.class,
      )}
    >
      <View class="h-8 flex flex-row items-center justify-between gap-3 pl-3 pr-1 bg-control">
        <Text class="min-w-0 font-mono text-xs text-muted">
          {props.language ?? "text"}
        </Text>
        {props.copyable !== false && (
          <CopyButton
            value={props.code}
            variant="ghost"
            size="icon"
            class="w-7 h-7 text-muted"
            idleLabel={props.copyLabel}
            copiedLabel={props.copiedLabel}
            aria-label={props.copyLabel}
            copiedChildren={<Icon source={check} size={13} />}
          >
            <Icon source={copy} size={13} />
          </CopyButton>
        )}
      </View>
      <Text class="min-w-0 p-3 font-mono text-sm text-primary whitespace-normal">
        {props.code}
      </Text>
    </View>
  );
}
