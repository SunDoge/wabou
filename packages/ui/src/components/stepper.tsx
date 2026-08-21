import { createSignal, For, type JSX } from "solid-js";
import { Text, View, type ViewProps } from "../primitives";
import { Button } from "./button";
import { join } from "./class-names";

export interface StepperStep {
  id: string;
  label: string;
  description?: string;
}
export interface StepperProps extends Omit<ViewProps, "children"> {
  steps: readonly StepperStep[];
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
}

export function Stepper(props: StepperProps): JSX.Element {
  const [local, setLocal] = createSignal(
    props.defaultValue ?? props.steps[0]?.id ?? "",
  );
  const value = () => props.value ?? local();
  const currentIndex = () =>
    Math.max(
      0,
      props.steps.findIndex((step) => step.id === value()),
    );
  const select = (id: string) => {
    if (props.value === undefined) setLocal(id);
    props.onValueChange?.(id);
  };
  return (
    <View
      role="group"
      aria-label={props["aria-label"] ?? "Progress steps"}
      class={join("min-w-0 flex flex-row items-start", props.class)}
    >
      <For each={props.steps}>
        {(step, index) => (
          <View class="min-w-0 flex-1 flex flex-row items-start">
            <View class="min-w-0 flex-1 items-center gap-2">
              <Button
                aria-label={`Go to ${step.label}`}
                variant={index() <= currentIndex() ? "default" : "outline"}
                size="icon"
                onClick={() => select(step.id)}
              >
                {String(index() + 1)}
              </Button>
              <Text class="text-sm font-medium text-primary">{step.label}</Text>
              {step.description && (
                <Text class="text-xs text-muted whitespace-normal">
                  {step.description}
                </Text>
              )}
            </View>
            {index() < props.steps.length - 1 && (
              <View
                class={join(
                  "h-px flex-1 mt-4",
                  index() < currentIndex() ? "bg-accent" : "bg-subtle",
                )}
              />
            )}
          </View>
        )}
      </For>
    </View>
  );
}
