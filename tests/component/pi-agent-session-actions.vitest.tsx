import { renderComponent } from "@wabou/test/component";
import { expect, test } from "vitest";
import { SessionActions } from "../../apps/pi-agent/ui/session-actions";

test("exposes Pi session maintenance as explicit menu actions", () => {
  const actions: string[] = [];
  const screen = renderComponent(() => (
    <SessionActions
      compact={() => actions.push("compact")}
      clone={() => actions.push("clone")}
      exportHtml={() => actions.push("export")}
    />
  ));

  const trigger = screen.getByRole("button", { name: "Session actions" });
  trigger.click();
  screen.getByRole("menuitem", { name: "Compact context" }).click();
  trigger.click();
  screen.getByRole("menuitem", { name: "Clone current branch" }).click();
  trigger.click();
  screen.getByRole("menuitem", { name: "Export as HTML" }).click();

  expect(actions).toEqual(["compact", "clone", "export"]);
});

test("disables maintenance until a session is idle", () => {
  const screen = renderComponent(() => (
    <SessionActions
      disabled
      compact={() => {}}
      clone={() => {}}
      exportHtml={() => {}}
    />
  ));

  screen.getByRole("button", { name: "Session actions" }).click();
  expect(screen.getAllByRole("menuitem").every((item) => item.disabled)).toBe(
    true,
  );
});
