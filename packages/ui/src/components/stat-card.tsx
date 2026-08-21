import type { JSX } from "solid-js";
import { Text, View, type ViewProps } from "../primitives";
import { Card, CardContent } from "./card";
import { join } from "./class-names";

export interface StatCardProps extends Omit<ViewProps, "children"> {
  label: string;
  value: string;
  description?: string;
  trend?: string;
  indicatorClass?: string;
}

export function StatCard(props: StatCardProps): JSX.Element {
  return (
    <Card role="group" aria-label={props.label} class={props.class}>
      <CardContent class="gap-2">
        <View class="flex flex-row items-center justify-between gap-3">
          <Text class="min-w-0 text-xs font-medium text-muted">
            {props.label}
          </Text>
          <View
            aria-hidden="true"
            class={join("w-2 h-2 rounded-full bg-accent", props.indicatorClass)}
          />
        </View>
        <Text class="text-3xl font-semibold text-primary">{props.value}</Text>
        {(props.description || props.trend) && (
          <View class="min-w-0 flex flex-row items-center gap-2">
            {props.trend && (
              <Text class="flex-none text-xs font-medium text-success-primary">
                {props.trend}
              </Text>
            )}
            {props.description && (
              <Text class="min-w-0 text-xs text-muted whitespace-normal">
                {props.description}
              </Text>
            )}
          </View>
        )}
      </CardContent>
    </Card>
  );
}
