import { mergeClasses } from "@wabou/core/style";
import checkCircle from "lucide-static/icons/circle-check.svg?raw";
import ellipsisCircle from "lucide-static/icons/circle-ellipsis.svg?raw";
import xCircle from "lucide-static/icons/circle-x.svg?raw";
import clock from "lucide-static/icons/clock-3.svg?raw";
import wrench from "lucide-static/icons/wrench.svg?raw";
import { type JSX, omit } from "solid-js";
import { match } from "ts-pattern";
import { Icon, Text, View, type ViewProps } from "../primitives";
import { Badge, type BadgeVariant } from "./badge";
import { CodeBlock, type CodeBlockProps } from "./code-block";
import {
  Collapsible,
  CollapsibleContent,
  type CollapsibleContentProps,
  type CollapsibleProps,
  CollapsibleTrigger,
  type CollapsibleTriggerProps,
} from "./disclosure";

export type ToolStatus = "pending" | "running" | "success" | "failed";

export interface ToolProps extends CollapsibleProps {}

/** Composable disclosure root for an AI or automation tool invocation. */
export function Tool(props: ToolProps): JSX.Element {
  const rest = omit(props, "class");
  return (
    <Collapsible
      {...rest}
      class={mergeClasses(
        "w-full min-w-0 overflow-hidden rounded-lg border border-subtle bg-surface shadow-xs",
        props.class,
      )}
    />
  );
}

export interface ToolHeaderProps
  extends Omit<CollapsibleTriggerProps, "children" | "class"> {
  title: string;
  summary?: string;
  status?: ToolStatus;
  icon?: string;
  class?: string;
}

function statusPresentation(status: ToolStatus): {
  icon: string;
  label: string;
  variant: BadgeVariant;
} {
  return match(status)
    .with("pending", () => ({
      icon: ellipsisCircle,
      label: "Pending",
      variant: "secondary" as const,
    }))
    .with("running", () => ({
      icon: clock,
      label: "Running",
      variant: "secondary" as const,
    }))
    .with("success", () => ({
      icon: checkCircle,
      label: "Completed",
      variant: "success" as const,
    }))
    .with("failed", () => ({
      icon: xCircle,
      label: "Failed",
      variant: "destructive" as const,
    }))
    .exhaustive();
}

export function toolHeaderLabel(
  title: string,
  summary?: string,
  status?: ToolStatus,
): string {
  return [title, summary, status ? statusPresentation(status).label : undefined]
    .filter(Boolean)
    .join(": ");
}

/** Stable title, summary, status and disclosure geometry for one tool call. */
export function ToolHeader(props: ToolHeaderProps): JSX.Element {
  const presentation = () =>
    props.status ? statusPresentation(props.status) : undefined;
  const rest = omit(
    props,
    "title",
    "summary",
    "status",
    "icon",
    "class",
    "aria-label",
  );
  return (
    <CollapsibleTrigger
      {...rest}
      aria-label={
        props["aria-label"] ??
        toolHeaderLabel(props.title, props.summary, props.status)
      }
      class={mergeClasses(
        "min-h-10 px-3 py-1.5 bg-control text-left",
        props.class,
      )}
    >
      <View class="min-w-0 flex-1 flex flex-row items-center gap-2">
        <Icon
          source={props.icon ?? wrench}
          size={14}
          class="flex-none text-muted"
        />
        <Text class="min-w-0 max-w-2/5 truncate text-sm font-semibold text-primary">
          {props.title}
        </Text>
        {props.summary && (
          <Text class="min-w-0 flex-1 truncate text-xs text-muted">
            {props.summary}
          </Text>
        )}
        {presentation() && (
          <Badge
            variant={presentation()?.variant}
            weight="normal"
            class="flex flex-row items-center gap-1"
          >
            <Icon source={presentation()?.icon ?? wrench} size={11} />
            {presentation()?.label}
          </Badge>
        )}
      </View>
    </CollapsibleTrigger>
  );
}

export function ToolContent(props: CollapsibleContentProps): JSX.Element {
  const rest = omit(props, "class");
  return (
    <CollapsibleContent
      {...rest}
      class={mergeClasses(
        "min-w-0 border-t border-subtle bg-surface",
        props.class,
      )}
    />
  );
}

export interface ToolCodeSectionProps
  extends Omit<ViewProps, "children" | "class"> {
  code: string;
  label: string;
  language?: string;
  copyable?: boolean;
  class?: string;
  codeClass?: string;
  codeProps?: Omit<CodeBlockProps, "code" | "language" | "copyable">;
}

/** Labelled code payload used for tool parameters, results and errors. */
export function ToolCodeSection(props: ToolCodeSectionProps): JSX.Element {
  const rest = omit(
    props,
    "code",
    "label",
    "language",
    "copyable",
    "class",
    "codeClass",
    "codeProps",
  );
  return (
    <View {...rest} class={mergeClasses("min-w-0 p-3 gap-2", props.class)}>
      <Text class="text-xs font-medium tracking-wide text-muted">
        {props.label}
      </Text>
      <CodeBlock
        {...props.codeProps}
        code={props.code}
        language={props.language}
        copyable={props.copyable}
        class={mergeClasses("rounded-md", props.codeClass)}
      />
    </View>
  );
}

export interface ToolInputProps extends Omit<ToolCodeSectionProps, "label"> {
  label?: string;
}

export function ToolInput(props: ToolInputProps): JSX.Element {
  return <ToolCodeSection {...props} label={props.label ?? "Parameters"} />;
}

export interface ToolOutputProps extends Omit<ToolCodeSectionProps, "label"> {
  label?: string;
  error?: boolean;
}

export function ToolOutput(props: ToolOutputProps): JSX.Element {
  const rest = omit(props, "error");
  return (
    <ToolCodeSection
      {...rest}
      label={props.label ?? (props.error ? "Error" : "Result")}
      class={mergeClasses(
        "border-t border-subtle",
        props.error ? "bg-danger-surface" : undefined,
        props.class,
      )}
    />
  );
}
