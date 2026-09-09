import { match } from "ts-pattern";

/** Semantic color shared by compact component surfaces and indicators. */
export type ComponentTone = "neutral" | "accent" | "success" | "danger";

export type ComponentTonePresentation =
  | "text"
  | "surface"
  | "solid"
  | "indicator";

/**
 * Maps semantic intent to the built-in Wabou theme contract.
 *
 * Keeping this recipe in one place prevents badges, icon tiles and status
 * indicators from drifting into slightly different colors for the same state.
 */
export function componentToneClass(
  tone: ComponentTone,
  presentation: ComponentTonePresentation,
): string {
  return match({ tone, presentation })
    .with({ tone: "neutral", presentation: "text" }, () => "text-secondary")
    .with(
      { tone: "neutral", presentation: "surface" },
      () => "border-subtle bg-control text-secondary",
    )
    .with(
      { tone: "neutral", presentation: "solid" },
      () => "border-strong bg-control-pressed text-primary",
    )
    .with({ tone: "neutral", presentation: "indicator" }, () => "bg-muted")
    .with({ tone: "accent", presentation: "text" }, () => "text-accent")
    .with(
      { tone: "accent", presentation: "surface" },
      () => "border-accent bg-selected text-accent",
    )
    .with(
      { tone: "accent", presentation: "solid" },
      () => "border-accent bg-accent text-on-accent",
    )
    .with({ tone: "accent", presentation: "indicator" }, () => "bg-accent")
    .with(
      { tone: "success", presentation: "text" },
      () => "text-success-primary",
    )
    .with(
      { tone: "success", presentation: "surface" },
      () => "border-success-primary bg-success-surface text-success-primary",
    )
    .with(
      { tone: "success", presentation: "solid" },
      () => "border-success-primary bg-success-primary text-on-success",
    )
    .with(
      { tone: "success", presentation: "indicator" },
      () => "bg-success-primary",
    )
    .with({ tone: "danger", presentation: "text" }, () => "text-danger-primary")
    .with(
      { tone: "danger", presentation: "surface" },
      () => "border-danger bg-danger-surface text-danger-primary",
    )
    .with(
      { tone: "danger", presentation: "solid" },
      () => "border-danger bg-danger text-on-danger",
    )
    .with({ tone: "danger", presentation: "indicator" }, () => "bg-danger")
    .exhaustive();
}
