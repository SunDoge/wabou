import { Field, FieldError, FieldLabel, Input, Text, View } from "@wabou/ui";
import { createMemo, createSignal, Show } from "solid-js";
import * as v from "valibot";
import { TaskPage } from "../shared";
import { firstValidationError, temperatureSchema } from "./validation";

const format = (value: number) =>
  Number.isInteger(value)
    ? String(value)
    : value.toFixed(2).replace(/0+$/, "").replace(/\.$/, "");

export function TemperatureTask() {
  const [celsius, setCelsius] = createSignal("0");
  const [fahrenheit, setFahrenheit] = createSignal("32");
  const celsiusResult = createMemo(() =>
    v.safeParse(temperatureSchema, celsius()),
  );
  const fahrenheitResult = createMemo(() =>
    v.safeParse(temperatureSchema, fahrenheit()),
  );
  const celsiusError = () => firstValidationError(celsiusResult());
  const fahrenheitError = () => firstValidationError(fahrenheitResult());
  const updateCelsius = (value: string) => {
    setCelsius(value);
    const result = v.safeParse(temperatureSchema, value);
    if (result.success) setFahrenheit(format(result.output * (9 / 5) + 32));
  };
  const updateFahrenheit = (value: string) => {
    setFahrenheit(value);
    const result = v.safeParse(temperatureSchema, value);
    if (result.success) setCelsius(format((result.output - 32) * (5 / 9)));
  };
  return (
    <TaskPage
      number={2}
      title="Temperature Converter"
      summary="Two controlled fields update each other without creating an implicit reactive loop."
    >
      <View class="flex-1 flex items-center justify-center gap-4">
        <Field class="w-44" invalid={celsiusError() !== undefined}>
          <FieldLabel>Celsius</FieldLabel>
          <Input
            aria-label="Celsius"
            aria-invalid={celsiusError() !== undefined}
            value={celsius()}
            onInput={(event) => updateCelsius(event.currentTarget.value)}
          />
          <Show when={celsiusError()}>
            {(error) => <FieldError>{error()}</FieldError>}
          </Show>
        </Field>
        <Text class="mt-6 text-lg text-muted">=</Text>
        <Field class="w-44" invalid={fahrenheitError() !== undefined}>
          <FieldLabel>Fahrenheit</FieldLabel>
          <Input
            aria-label="Fahrenheit"
            aria-invalid={fahrenheitError() !== undefined}
            value={fahrenheit()}
            onInput={(event) => updateFahrenheit(event.currentTarget.value)}
          />
          <Show when={fahrenheitError()}>
            {(error) => <FieldError>{error()}</FieldError>}
          </Show>
        </Field>
      </View>
    </TaskPage>
  );
}
