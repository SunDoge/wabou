import { mergeClasses } from "@wabou/core/style";
import { type JSX, Match, omit, Switch } from "solid-js";
import { Text, View, type ViewProps } from "../primitives";
import { Button } from "./button";
import { Spinner } from "./display";

export type ContentStateKind = "empty" | "loading" | "error";

export interface ContentStateProps
  extends Omit<ViewProps, "children" | "role"> {
  state: ContentStateKind;
  title: string;
  description?: string;
  /** Standard compact recovery or next-step action. */
  action?: {
    label: string;
    onAction(): void;
  };
  /** Lazily render media inside the state's reactive owner. */
  renderMedia?: () => JSX.Element;
  /** Lazily render custom actions inside the state's reactive owner. */
  renderAction?: () => JSX.Element;
}

/** Mutually exclusive loading, empty, or error state for a bounded region. */
export function ContentState(props: ContentStateProps): JSX.Element {
  const forwarded = omit(
    props,
    "state",
    "title",
    "description",
    "action",
    "renderMedia",
    "renderAction",
    "class",
  );
  const error = () => props.state === "error";
  return (
    <View
      {...forwarded}
      role={error() ? "alert" : "status"}
      aria-label={props["aria-label"] ?? props.title}
      class={mergeClasses(
        "w-full h-full min-w-0 min-h-0 flex-1 p-6 flex flex-col items-center justify-center gap-3 text-center",
        error() ? "text-danger-primary" : "text-secondary",
        props.class,
      )}
    >
      {props.renderMedia?.() ??
        (props.state === "loading" ? <Spinner decorative /> : null)}
      <View class="w-full max-w-sm min-w-0 flex flex-col items-center gap-1">
        <Text
          role="heading"
          class={mergeClasses(
            "w-full min-w-0 whitespace-normal text-sm font-medium",
            error() ? "text-danger-primary" : "text-primary",
          )}
        >
          {props.title}
        </Text>
        {props.description === undefined ? null : (
          <Text
            class={mergeClasses(
              "w-full min-w-0 whitespace-normal text-xs",
              error() ? "text-danger-primary" : "text-muted",
            )}
          >
            {props.description}
          </Text>
        )}
      </View>
      {props.renderAction?.() ??
        (props.action ? (
          <Button
            size="sm"
            variant="outline"
            aria-label={props.action.label}
            onClick={props.action.onAction}
          >
            {props.action.label}
          </Button>
        ) : null)}
    </View>
  );
}

export interface ResourceBoundaryProps
  extends Omit<ViewProps, "children" | "role"> {
  /** True while the resource is performing its initial load or refreshing. */
  loading: boolean;
  /** A rejected resource value. `undefined` and `null` mean no error. */
  error?: unknown;
  /** Whether a usable resource value currently exists, including an empty collection. */
  hasContent: boolean;
  loadingTitle: string;
  errorTitle: string;
  emptyTitle: string;
  loadingDescription?: string;
  emptyDescription?: string;
  retryLabel?: string;
  onRetry?: () => void;
  renderContent(): JSX.Element;
  renderEmptyMedia?: () => JSX.Element;
  renderErrorMedia?: () => JSX.Element;
}

/**
 * Mutually exclusive async resource boundary.
 *
 * Existing content remains mounted during a background refresh. This avoids
 * replacing a useful inspector or list with a loading spinner every time its
 * resource is refreshed.
 */
export function ResourceBoundary(props: ResourceBoundaryProps): JSX.Element {
  const forwarded = omit(
    props,
    "loading",
    "error",
    "hasContent",
    "loadingTitle",
    "errorTitle",
    "emptyTitle",
    "loadingDescription",
    "emptyDescription",
    "retryLabel",
    "onRetry",
    "renderContent",
    "renderEmptyMedia",
    "renderErrorMedia",
    "class",
  );
  const failed = () => props.error !== undefined && props.error !== null;
  return (
    <View
      {...forwarded}
      class={mergeClasses(
        "w-full min-w-0 min-h-0 flex-1 flex flex-col",
        props.class,
      )}
    >
      <Switch>
        <Match when={failed()}>
          <ContentState
            state="error"
            title={props.errorTitle}
            description={String(props.error)}
            renderMedia={props.renderErrorMedia}
            action={
              props.onRetry
                ? {
                    label: props.retryLabel ?? "Try again",
                    onAction: props.onRetry,
                  }
                : undefined
            }
          />
        </Match>
        <Match when={props.loading && !props.hasContent}>
          <ContentState
            state="loading"
            title={props.loadingTitle}
            description={props.loadingDescription}
          />
        </Match>
        <Match when={!props.hasContent}>
          <ContentState
            state="empty"
            title={props.emptyTitle}
            description={props.emptyDescription}
            renderMedia={props.renderEmptyMedia}
          />
        </Match>
        <Match when={true}>{props.renderContent()}</Match>
      </Switch>
    </View>
  );
}
