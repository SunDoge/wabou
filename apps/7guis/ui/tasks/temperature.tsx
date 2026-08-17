import { Input } from "@wabou/components";
import { Text, View } from "@wabou/primitives";
import { createSignal } from "solid-js";
import { FieldLabel, TaskPage } from "../shared";

const format = (value: number) =>
  Number.isInteger(value)
    ? String(value)
    : value.toFixed(2).replace(/0+$/, "").replace(/\.$/, "");

export function TemperatureTask() {
  const [celsius, setCelsius] = createSignal("0");
  const [fahrenheit, setFahrenheit] = createSignal("32");
  const updateCelsius = (value: string) => {
    setCelsius(value);
    const number = Number(value);
    if (value.trim() !== "" && Number.isFinite(number)) {
      setFahrenheit(format(number * (9 / 5) + 32));
    }
  };
  const updateFahrenheit = (value: string) => {
    setFahrenheit(value);
    const number = Number(value);
    if (value.trim() !== "" && Number.isFinite(number)) {
      setCelsius(format((number - 32) * (5 / 9)));
    }
  };
  return (
    <TaskPage
      number={2}
      title="Temperature Converter"
      summary="Two controlled fields update each other without creating an implicit reactive loop."
    >
      <View class="flex-1 flex items-center justify-center gap-4">
        <View class="w-44 flex flex-col gap-2">
          <FieldLabel>Celsius</FieldLabel>
          <Input
            aria-label="Celsius"
            value={celsius()}
            onInput={(event) => updateCelsius(event.currentTarget.value)}
          />
        </View>
        <Text class="mt-6 text-lg text-muted">=</Text>
        <View class="w-44 flex flex-col gap-2">
          <FieldLabel>Fahrenheit</FieldLabel>
          <Input
            aria-label="Fahrenheit"
            value={fahrenheit()}
            onInput={(event) => updateFahrenheit(event.currentTarget.value)}
          />
        </View>
      </View>
    </TaskPage>
  );
}
