import { createTestHost, renderComponent } from "@wabou/test/component";
import { NumberField } from "@wabou/ui";
import { expect, test } from "vitest";

const defaultHost = () => createTestHost().host;

test("steps decimal values without floating-point drift", () => {
  const changes: Array<number | null> = [];
  const screen = renderComponent(
    () => (
      <NumberField
        aria-label="Opacity"
        defaultValue={0.2}
        min={0}
        max={1}
        step={0.1}
        onValueChange={(value) => changes.push(value)}
      />
    ),
    { host: defaultHost() },
  );

  const decrement = screen.getByRole("button", { name: "Decrease Opacity" });
  const increment = screen.getByRole("button", { name: "Increase Opacity" });
  expect(decrement.className).toContain("w-7");
  expect(decrement.className).toContain("h-7");
  expect(increment.className).toContain("w-7");
  expect(increment.className).toContain("h-7");
  increment.click();
  expect(changes).toEqual([0.3]);
  expect(screen.getByRole("spinbutton", { name: "Opacity" }).numericValue).toBe(
    0.3,
  );
});

test("supports Arrow, Page, Home, and End keyboard commands", () => {
  const screen = renderComponent(
    () => (
      <NumberField
        aria-label="Retries"
        defaultValue={5}
        min={0}
        max={20}
        step={1}
        largeStep={5}
      />
    ),
    { host: defaultHost() },
  );
  const input = screen.getByRole("spinbutton", { name: "Retries" });

  input.press("ArrowUp");
  expect(input.numericValue).toBe(6);
  input.press("PageUp");
  expect(input.numericValue).toBe(11);
  input.press("Home");
  expect(input.numericValue).toBe(0);
  input.press("End");
  expect(input.numericValue).toBe(20);
});

test("parses and formats locale-aware decimal input", () => {
  const fixture = createTestHost(undefined, {
    intl: { locale: () => "de-DE" },
  });
  const changes: Array<number | null> = [];
  const screen = renderComponent(
    () => (
      <NumberField
        aria-label="Preis"
        defaultValue={1}
        step={0.1}
        onValueChange={(value) => changes.push(value)}
      />
    ),
    { host: fixture.host },
  );
  const input = screen.getByRole("spinbutton", { name: "Preis" });

  input.focus();
  input.input("1,5");
  expect(changes).toEqual([1.5]);
  input.blur();
  expect(input.value).toBe("1,5");
});

test("preserves partial input and restores the canonical value on blur", () => {
  const changes: Array<number | null> = [];
  const screen = renderComponent(
    () => (
      <NumberField
        aria-label="Offset"
        defaultValue={4}
        onValueChange={(value) => changes.push(value)}
      />
    ),
    { host: defaultHost() },
  );
  const input = screen.getByRole("spinbutton", { name: "Offset" });

  input.focus();
  input.input("-");
  expect(input.value).toBe("-");
  expect(changes).toEqual([]);
  input.blur();
  expect(input.value).toBe("4");
  input.focus();
  input.input("not a number");
  expect(input.value).toBe("not a number");
  input.blur();
  expect(input.value).toBe("4");
  expect(changes).toEqual([]);
});

test("controlled values only change when their owner updates them", () => {
  const changes: Array<number | null> = [];
  const screen = renderComponent(
    () => (
      <NumberField
        aria-label="Workers"
        value={2}
        onValueChange={(value) => changes.push(value)}
      />
    ),
    { host: defaultHost() },
  );

  screen.getByRole("button", { name: "Increase Workers" }).click();
  expect(changes).toEqual([3]);
  const input = screen.getByRole("spinbutton", { name: "Workers" });
  expect(input.numericValue).toBe(2);
  expect(input.value).toBe("2");
});

test("disabled and read-only fields reject every stepping path", () => {
  for (const mode of ["disabled", "readOnly"] as const) {
    const changes: Array<number | null> = [];
    const screen = renderComponent(
      () => (
        <NumberField
          aria-label={mode}
          defaultValue={2}
          disabled={mode === "disabled"}
          readOnly={mode === "readOnly"}
          onValueChange={(value) => changes.push(value)}
        />
      ),
      { host: defaultHost() },
    );
    const input = screen.getByRole("spinbutton", { name: mode });
    if (mode === "readOnly") input.press("ArrowUp");
    else {
      expect(input.disabled).toBe(true);
      expect(
        screen.getByRole("button", { name: "Increase disabled" }).disabled,
      ).toBe(true);
    }
    expect(changes).toEqual([]);
    expect(input.numericValue).toBe(2);
    screen.dispose();
  }
});
