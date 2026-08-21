import type { Handle } from "@wabou/core/renderer";
import { renderComponent } from "@wabou/test/component";
import { Field, FieldLabel, Input, Label } from "@wabou/ui";
import { expect, test } from "vitest";

test("focuses an explicitly bound native control", () => {
  let control: Handle | undefined;
  const screen = renderComponent(() => (
    <Field>
      <FieldLabel control={() => control}>Project name</FieldLabel>
      <Input ref={(node) => (control = node)} aria-label="Project name input" />
    </Field>
  ));

  screen.getByRole("label", { name: "Project name" }).click();
  expect(
    screen.getByRole("textbox", { name: "Project name input" }).focused,
  ).toBe(true);
});

test("does not activate a disabled label or override a prevented click", () => {
  let control: Handle | undefined;
  const screen = renderComponent(() => (
    <>
      <Label disabled control={() => control}>
        Disabled
      </Label>
      <Label
        control={() => control}
        onClick={(event) => event.preventDefault()}
      >
        Prevented
      </Label>
      <Input ref={(node) => (control = node)} aria-label="Target" />
    </>
  ));

  expect(screen.getByRole("label", { name: "Disabled" }).disabled).toBe(true);
  screen.getByRole("label", { name: "Prevented" }).click();
  expect(screen.getByRole("textbox", { name: "Target" }).focused).toBe(false);
});
