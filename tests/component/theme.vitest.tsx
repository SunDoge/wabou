import { renderComponent } from "@wabou/test/component";
import {
  ComponentsProvider,
  Text,
  useComponentsTheme,
  View,
} from "@wabou/ui";
import { expect, test } from "vitest";

function ThemeProbe() {
  const theme = useComponentsTheme();
  return (
    <View role="status" aria-label="Resolved component theme">
      <Text>{theme()}</Text>
    </View>
  );
}

test("component theme reads the nearest provider and has a stable default", () => {
  const fallback = renderComponent(ThemeProbe);
  expect(
    fallback.getByRole("status", { name: "Resolved component theme" }).text,
  ).toBe("light");
  fallback.dispose();

  const nested = renderComponent(() => (
    <ComponentsProvider theme="dark">
      <ThemeProbe />
    </ComponentsProvider>
  ));
  expect(
    nested.getByRole("status", { name: "Resolved component theme" }).text,
  ).toBe("dark");
});
