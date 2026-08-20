import { renderComponent } from "@wabou/test/component";
import { Button, View } from "@wabou/ui";
import { expect, test } from "vitest";

test("component focus follows native focus and blur ordering", () => {
  const events: string[] = [];
  const screen = renderComponent(() => (
    <View>
      <Button
        aria-label="First"
        onFocus={() => events.push("first focus")}
        onBlur={() => events.push("first blur")}
      />
      <Button
        aria-label="Second"
        onFocus={() => events.push("second focus")}
        onBlur={() => events.push("second blur")}
      />
    </View>
  ));

  screen.getByRole("button", { name: "First" }).focus();
  screen.getByRole("button", { name: "Second" }).focus();
  screen.getByRole("button", { name: "Second" }).blur();

  expect(events).toEqual([
    "first focus",
    "first blur",
    "second focus",
    "second blur",
  ]);
});
