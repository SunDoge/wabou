import { useClipboard } from "@wabou/core";
import { createSignal, type JSX, omit } from "solid-js";
import { Button, type ButtonProps } from "./button";

export interface CopyButtonProps extends Omit<ButtonProps, "onClick"> {
  value: string;
  idleLabel?: string;
  copiedLabel?: string;
  copiedChildren?: JSX.Element;
  onCopied?: () => void;
  onCopyError?: (error: unknown) => void;
}

export function CopyButton(props: CopyButtonProps): JSX.Element {
  const clipboard = useClipboard();
  const [copied, setCopied] = createSignal(false);
  const forwarded = omit(
    props,
    "value",
    "idleLabel",
    "copiedLabel",
    "copiedChildren",
    "onCopied",
    "onCopyError",
  );
  const copy = async () => {
    try {
      await clipboard.writeText(props.value);
      setCopied(true);
      props.onCopied?.();
    } catch (error) {
      props.onCopyError?.(error);
    }
  };
  return (
    <Button
      {...forwarded}
      aria-label={props["aria-label"] ?? "Copy"}
      onClick={copy}
    >
      {copied()
        ? (props.copiedChildren ?? props.copiedLabel ?? "Copied")
        : (props.children ?? props.idleLabel ?? "Copy")}
    </Button>
  );
}
