import { renderComponent } from "@wabou/test/component";
import { SplitButton } from "@wabou/ui";
import { expect, test } from "vitest";

test("keeps primary and alternative split-button commands independent", () => {
  const actions: string[] = [];
  const screen = renderComponent(() => (
    <SplitButton
      label="Run"
      items={[
        { id: "debug", label: "Run with debugger" },
        { id: "profile", label: "Profile" },
      ]}
      onClick={() => actions.push("run")}
      onAction={(id) => actions.push(id)}
    />
  ));

  screen.getByRole("button", { name: "Run" }).click();
  expect(actions).toEqual(["run"]);
  screen.getByRole("button", { name: "Run alternatives" }).click();
  expect(screen.getByRole("menu", { name: "Run alternatives" })).not.toBeNull();
  screen.getByRole("menuitem", { name: "Profile" }).click();
  expect(actions).toEqual(["run", "profile"]);
});
