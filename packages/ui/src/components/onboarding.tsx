import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import { PageViewport, type PageViewportProps } from "./page";

export interface OnboardingProps
  extends Omit<PageViewportProps, "class" | "contentClass"> {
  class?: string;
  contentClass?: string;
}

/**
 * Full-height first-run boundary with native scrolling and a readable measure.
 * Content stays centered when it fits and remains reachable when it grows.
 */
export function Onboarding(props: OnboardingProps): JSX.Element {
  const forwarded = omit(props, "class", "contentClass", "children");
  return (
    <PageViewport
      {...forwarded}
      role={props.role ?? "region"}
      class={mergeClasses("h-full bg-canvas", props.class)}
      contentClass={mergeClasses(
        "min-h-full flex items-center justify-center",
        props.contentClass,
      )}
    >
      <View class="w-full max-w-2xl min-w-0 mx-auto px-8 py-12 flex flex-col gap-5">
        {props.children}
      </View>
    </PageViewport>
  );
}

export function OnboardingHeader(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "w-full min-w-0 flex flex-row items-center gap-3",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function OnboardingHeading(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses("min-w-0 flex-1 flex flex-col gap-1", props.class)}
    >
      {props.children}
    </View>
  );
}

export function OnboardingTitle(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      role={props.role ?? "heading"}
      class={mergeClasses(
        "w-full min-w-0 whitespace-normal text-lg font-semibold text-primary",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function OnboardingDescription(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={mergeClasses(
        "w-full min-w-0 whitespace-normal text-sm text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function OnboardingFooter(props: ViewProps): JSX.Element {
  return (
    <View {...props} class={mergeClasses("w-full min-w-0 pt-1", props.class)}>
      {props.children}
    </View>
  );
}
