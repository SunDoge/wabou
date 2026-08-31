import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { SessionTitle } from "../../apps/pi-agent/ui/session-title";

test("Pi Agent renames the current session from its header", async () => {
  let renamed = "";
  const screen = renderComponent(
    () => (
      <SessionTitle
        name="Initial session"
        rename={(name) => (renamed = name)}
      />
    ),
    { clock: "fake" },
  );

  screen.getByRole("button", { name: "Rename session" }).click();
  await screen.advanceTime(16);
  const name = screen.getByRole("textbox", { name: "Session name" });
  expect(name.focused).toBe(true);
  name.input("Readable title");
  screen.getByRole("button", { name: "Save" }).click();

  expect(renamed).toBe("Readable title");
  await Promise.resolve();
  screen.flush();
  await screen.advanceTime(200);
  expect(screen.queryByRole("dialog", { name: "Rename session" })).toBeNull();
});

test("Pi Agent keeps rename failures visible and retryable", async () => {
  let attempt = 0;
  const screen = renderComponent(() => (
    <SessionTitle
      name="Initial session"
      rename={async () => {
        attempt += 1;
        if (attempt === 1) throw new Error("storage unavailable");
      }}
    />
  ));

  screen.getByRole("button", { name: "Rename session" }).click();
  screen.getByRole("textbox", { name: "Session name" }).input("New title");
  screen.getByRole("button", { name: "Save" }).click();
  await screen.waitFor(() =>
    expect(
      screen.getByRole("alert", { name: "Could not rename the session" }).text,
    ).toContain("storage unavailable"),
  );
  expect(screen.getByRole("dialog", { name: "Rename session" })).toBeDefined();
  screen.getByRole("button", { name: "Save" }).click();
  await screen.waitFor(() =>
    expect(screen.queryByRole("dialog", { name: "Rename session" })).toBeNull(),
  );
  expect(attempt).toBe(2);
});
