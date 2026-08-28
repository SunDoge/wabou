import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { SessionTitle } from "../../apps/pi-agent/ui/session-title";

test("Pi Agent renames the current session from its header", () => {
  let renamed = "";
  const screen = renderComponent(() => (
    <SessionTitle name="Initial session" rename={(name) => (renamed = name)} />
  ));

  screen.getByRole("button", { name: "Rename session" }).click();
  screen.getByRole("textbox", { name: "Session name" }).input("Readable title");
  screen.getByRole("button", { name: "Save" }).click();

  expect(renamed).toBe("Readable title");
  expect(screen.queryByRole("dialog", { name: "Rename session" })).toBeNull();
});
