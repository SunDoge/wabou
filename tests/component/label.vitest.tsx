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

  const disabled = screen.getByRole("label", { name: "Disabled" });
  expect(disabled.disabled).toBe(true);
  expect(disabled.className).toContain("cursor-not-allowed");
  expect(disabled.className).toContain("opacity-60");
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

test("field owns required and horizontal label-column semantics", () => {
  const screen = renderComponent(() => (
    <Field orientation="horizontal" required invalid>
      <FieldLabel aria-label="Repository label">Repository</FieldLabel>
      <Input aria-label="Repository input" />
    </Field>
  ));
  const field = screen.getByRole("group");
  const label = screen.getByRole("label", { name: "Repository label" });

  expect(field.orientation).toBe("horizontal");
  expect(field.attribute("aria-required")).toBe("true");
  expect(field.attribute("aria-invalid")).toBe("true");
  expect(label.parent?.className).toContain("w-36");
  expect(label.className).toContain("text-danger-primary");
  expect(label.text).toBe("Repository");
  expect(label.parent?.text).toContain("*");
});

test("labeled field forwards its required state without changing the label name", () => {
  const screen = renderComponent(() => (
    <LabeledField
      required
      label="Provider"
      renderControl={(ref) => <Input ref={ref} aria-label="Provider input" />}
    />
  ));

  expect(screen.getByRole("group").attribute("aria-required")).toBe("true");
  expect(screen.getByRole("label", { name: "Provider" }).text).toBe("Provider");
});
