import {
  assertFocusOwnerCount,
  assertSingleSurfaceOwner,
  renderComponent,
} from "@wabou/test/component";
import { Editor, Input, PasswordInput, Text, TextArea, View } from "@wabou/ui";
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

  expect(name.className).toContain("h-8");
  expect(name.className).toContain("rounded-md");

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

test("reports the native textarea caret in JavaScript UTF-16 offsets", () => {
  let caret = -1;
  const screen = renderComponent(() => (
    <TextArea
      aria-label="Prompt"
      value="a😀b"
      onTextSelectionChange={(event) => {
        caret = event.head ?? -1;
      }}
    />
  ));

  screen.getByRole("textbox", { name: "Prompt" }).emit("textselectionchange", {
    anchor: 3,
    head: 3,
    text: null,
    kind: "simple",
  });
  expect(caret).toBe(3);
});

test("updates a controlled Editor through the component input contract", () => {
  const App = () => {
    const [source, setSource] = createSignal("initial");
    return (
      <Editor
        aria-label="Markdown source"
        value={source()}
        onInput={(event) => setSource(event.currentTarget.value)}
      />
    );
  };
  const screen = renderComponent(App);
  const editor = screen.getByRole("textbox", { name: "Markdown source" });
  const identity = editor.identity;

  editor.input("updated");
  expect(editor.value).toBe("updated");
  expect(
    screen.getByRole("textbox", { name: "Markdown source" }).identity,
  ).toEqual(identity);
});

test("exposes native Editor selection and submit events", () => {
  let selection: [number | undefined, number | undefined] = [
    undefined,
    undefined,
  ];
  let submit: [boolean, boolean] | undefined;
  const screen = renderComponent(() => (
    <Editor
      aria-label="Source"
      value="a😀中"
      onTextSelectionChange={(event) => {
        selection = [event.anchor, event.head];
      }}
      onSubmit={(event) => {
        submit = [event.secondary, event.shift];
      }}
    />
  ));
  const editor = screen.getByRole("textbox", { name: "Source" });

  editor.emit("textselectionchange", {
    anchor: 1,
    head: 3,
    text: "😀",
    kind: "simple",
  });
  editor.emit("submit", { secondary: false, shift: true });

  expect(selection).toEqual([1, 3]);
  expect(submit).toEqual([false, true]);
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

  expect(assertFocusOwnerCount(input, 1)).toHaveLength(1);
  expect(input.className).toContain("bg-surface-raised");
  expect(input.className).not.toContain("bg-input");
  expect(input.attribute("surfaceClass")).toBeNull();
});

test("removes all visual chrome from nested native editors", () => {
  const screen = renderComponent(() => (
    <View data-wabou-owns="surface">
      <Input aria-label="Inline name" chrome="none" />
      <TextArea aria-label="Inline notes" chrome="none" />
    </View>
  ));
  const root = screen.getByRole("textbox", { name: "Inline name" }).parent;
  expect(root).not.toBeNull();
  if (!root) throw new Error("nested editors require a surface parent");

  expect(assertSingleSurfaceOwner(root)).toEqual(root);
  for (const editor of screen.getAllByRole("textbox")) {
    expect(editor.attribute("data-wabou-owns")).toBe("native-editor");
    expect(editor.className).not.toContain("rounded");
    expect(editor.className).not.toContain("border");
    expect(editor.className).not.toContain("shadow");
    expect(editor.className).not.toContain("bg-");
  }
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
