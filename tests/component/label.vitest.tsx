import type { Handle } from "@wabou/core/renderer";
import { renderComponent } from "@wabou/test/component";
import { Field, FieldLabel, Input, Label, LabeledField } from "@wabou/ui";
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

test("labeled field binds its visible label to a native control", () => {
  const screen = renderComponent(() => (
    <LabeledField
      label="Provider"
      description="Used for new sessions"
      renderControl={(ref) => <Input ref={ref} aria-label="Provider input" />}
    />
  ));

  screen.getByRole("label", { name: "Provider" }).click();
  expect(screen.getByRole("textbox", { name: "Provider input" }).focused).toBe(
    true,
  );
  expect(JSON.stringify(screen.snapshot())).toContain("Used for new sessions");
});

test("labeled field owns validation feedback without losing its control binding", () => {
  const screen = renderComponent(() => (
    <LabeledField
      label="Proxy URL"
      errors={[{ message: "Enter an HTTP proxy URL" }]}
      renderControl={(ref) => <Input ref={ref} aria-label="Proxy input" />}
    />
  ));

  expect(screen.getByRole("alert").text).toContain("Enter an HTTP proxy URL");
  screen.getByRole("label", { name: "Proxy URL" }).click();
  expect(screen.getByRole("textbox", { name: "Proxy input" }).focused).toBe(
    true,
  );
});
