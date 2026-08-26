import { createTestHost, renderComponent } from "@wabou/test/component";
import { CalendarDate, DatePicker, Text, View } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

const hostFixture = () =>
  createTestHost(undefined, {
    intl: {
      locale: () => "en-US",
      timeZone: () => "UTC",
      today: () => ({ year: 2026, month: 8, day: 20 }),
    },
  });

test("changes month and commits a controlled date", () => {
  const fixture = hostFixture();
  const App = () => {
    const [date, setDate] = createSignal(new CalendarDate(2026, 8, 17));
    return (
      <View>
        <DatePicker
          aria-label="Deployment date"
          value={date()}
          minValue={new CalendarDate(2026, 1, 1)}
          maxValue={new CalendarDate(2027, 12, 31)}
          onValueChange={setDate}
        />
        <Text role="status" aria-label="Selected date">
          {date().toString()}
        </Text>
      </View>
    );
  };
  const screen = renderComponent(App, { host: fixture.host });
  const trigger = screen.getByRole("button", { name: "Deployment date" });

  trigger.click();
  expect(
    screen.getByRole("dialog", { name: "Deployment date" }),
  ).not.toBeNull();
  screen.getByRole("button", { name: "Next month" }).click();
  screen.getByRole("button", { name: "Thursday, September 17, 2026" }).click();

  expect(screen.getByRole("status", { name: "Selected date" }).text).toBe(
    "2026-09-17",
  );
  expect(trigger.text).toContain("Sep 17, 2026");
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("moves keyboard focus across unavailable dates and selects", async () => {
  const fixture = hostFixture();
  const screen = renderComponent(
    () => (
      <DatePicker
        aria-label="Appointment"
        defaultValue={new CalendarDate(2026, 8, 17)}
        isDateUnavailable={(date) => date.day === 18}
      />
    ),
    { host: fixture.host, clock: "fake" },
  );
  screen.getByRole("button", { name: "Appointment" }).click();
  const selected = screen.getByRole("button", {
    name: "Monday, August 17, 2026",
  });

  selected.focus();
  selected.press("ArrowRight");
  await screen.advanceTime(16);
  expect(
    screen.getByRole("button", {
      name: "Wednesday, August 19, 2026",
      focused: true,
    }),
  ).not.toBeNull();
  screen
    .getByRole("button", { name: "Wednesday, August 19, 2026" })
    .press("Enter");
  expect(screen.getByRole("button", { name: "Appointment" }).text).toContain(
    "Aug 19, 2026",
  );
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("uses the host calendar date for Today", () => {
  const fixture = hostFixture();
  const screen = renderComponent(
    () => (
      <DatePicker
        aria-label="Due date"
        defaultValue={new CalendarDate(2026, 8, 17)}
      />
    ),
    { host: fixture.host },
  );

  screen.getByRole("button", { name: "Due date" }).click();
  expect(
    screen.getByRole("button", { name: "Thursday, August 20, 2026" }).current,
  ).toBe("date");
  screen.getByRole("button", { name: "Select today" }).click();
  expect(screen.getByRole("button", { name: "Due date" }).text).toContain(
    "Aug 20, 2026",
  );
  expect(fixture.callsTo("intl.today").length).toBeGreaterThan(0);
});

test("derives the week start from locale data", () => {
  const fixture = hostFixture();
  const screen = renderComponent(
    () => (
      <DatePicker
        aria-label="British date"
        locale="en-GB"
        defaultValue={new CalendarDate(2026, 8, 17)}
      />
    ),
    { host: fixture.host },
  );

  screen.getByRole("button", { name: "British date" }).click();
  expect(
    screen.getByRole("button", { name: "Monday, 27 July 2026" }),
  ).not.toBeNull();
  expect(
    screen.queryByRole("button", { name: "Sunday, 26 July 2026" }),
  ).toBeNull();
});
