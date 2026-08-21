import { renderComponent } from "@wabou/test/component";
import { Input, PasswordInput, Text, TextArea, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("updates controlled single-line and multiline editors", () => {
  const App = () => {
    const [name, setName] = createSignal("");
    const [notes, setNotes] = createSignal("");
    return (
      <View>
        <Input
          aria-label="Name"
          value={name()}
          onInput={(event) => setName(event.currentTarget.value)}
        />
        <TextArea
          aria-label="Notes"
          value={notes()}
          onInput={(event) => setNotes(event.currentTarget.value)}
        />
        <Text role="status" aria-label="Form value">
          {`${name()}|${notes()}`}
        </Text>
      </View>
    );
  };
  const screen = renderComponent(App);
  const name = screen.getByRole("textbox", { name: "Name" });
  const notes = screen.getByRole("textbox", { name: "Notes" });

  name.input("Ada");
  expect(name.focused).toBe(true);
  expect(name.value).toBe("Ada");
  notes.input("first\nsecond");
  expect(notes.focused).toBe(true);
  expect(name.focused).toBe(false);
  expect(notes.value).toBe("first\nsecond");
  expect(screen.getByRole("status", { name: "Form value" }).text).toBe(
    "Ada|first\nsecond",
  );
});

test("blocks authored disabled and read-only editors", () => {
  const screen = renderComponent(() => (
    <View>
      <Input aria-label="Disabled" disabled />
      <Input aria-label="Read only" readOnly value="stable" />
    </View>
  ));
  const disabled = screen.getByRole("textbox", {
    name: "Disabled",
    disabled: true,
  });
  const readOnly = screen.getByRole("textbox", {
    name: "Read only",
    readOnly: true,
  });

  expect(disabled.readOnly).toBe(false);
  expect(readOnly.value).toBe("stable");
  expect(() => disabled.input("change")).toThrow(
    'cannot input into disabled component textbox "Disabled"',
  );
  expect(() => readOnly.input("change")).toThrow(
    'cannot input into read-only component textbox "Read only"',
  );
});

test("allows the input surface to be selected without conflicting backgrounds", () => {
  const screen = renderComponent(() => (
    <Input aria-label="Raised input" surfaceClass="bg-surface-raised" />
  ));
  const input = screen.getByRole("textbox", { name: "Raised input" });

  expect(input.className).toContain("bg-surface-raised");
  expect(input.className).not.toContain("bg-input");
  expect(input.attribute("surfaceClass")).toBeNull();
});

test("keeps password contents behind a Rust secret handle", () => {
  const screen = renderComponent(() => (
    <PasswordInput
      secret="account-master-password"
      aria-label="Master password"
      placeholder="Enter password"
    />
  ));
  const password = screen.getByRole("textbox", { name: "Master password" });

  expect(password.tag).toBe("password-input");
  expect(password.attribute("secret")).toBe("account-master-password");
  expect(password.attribute("placeholder")).toBe("Enter password");
  expect(password.value).toBeNull();
  expect(password.text).toBe("");
});
