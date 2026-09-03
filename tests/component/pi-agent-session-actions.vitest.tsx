import { renderComponent } from "@wabou/test/component";
import { expect, test, vi } from "vitest";
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

test("prevents duplicate session actions and reports failures", async () => {
  let rejectCompact: (error: unknown) => void = () => {};
  const compact = vi.fn(
    () =>
      new Promise<void>((_resolve, reject) => {
        rejectCompact = reject;
      }),
  );
  const onActionError = vi.fn();
  const screen = renderComponent(() => (
    <SessionActions
      compact={compact}
      clone={() => {}}
      exportHtml={() => {}}
      onActionError={onActionError}
    />
  ));
  const trigger = screen.getByRole("button", { name: "Session actions" });

  trigger.click();
  screen.getByRole("menuitem", { name: "Compact context" }).click();
  expect(compact).toHaveBeenCalledTimes(1);
  trigger.click();
  expect(
    screen.getByRole("menuitem", { name: "Compact context" }).disabled,
  ).toBe(true);

  const failure = new Error("compaction unavailable");
  rejectCompact(failure);
  await screen.waitFor(() => {
    expect(onActionError).toHaveBeenCalledWith("compact", failure);
  });
  expect(
    screen.getByRole("menuitem", { name: "Compact context" }).disabled,
  ).toBe(false);
});
