import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import { AgentTerminalPanel } from "../../apps/pi-agent/ui/terminal-panel";

test("keeps terminal tabs bound to the workspace where they were created", () => {
  const [cwd, setCwd] = createSignal("/work/alpha");
  let closed = 0;
  const screen = renderComponent(() => (
    <AgentTerminalPanel
      cwd={cwd()}
      open
      close={() => closed++}
      dispose={() => closed++}
    />
  ));

  expect(screen.getByRole("region", { name: "Terminal panel" })).toBeTruthy();
  expect(screen.getAllByRole("tab")).toHaveLength(1);
  expect(screen.getAllByRole("tab")[0]?.name).toContain("alpha");
  expect(screen.getAllByRole("tab")[0]?.selected).toBe(true);

  setCwd("/work/beta");
  screen.getByRole("button", { name: "New terminal" }).click();

  const tabs = screen.getAllByRole("tab");
  expect(tabs).toHaveLength(2);
  expect(tabs[0]?.name).toContain("alpha");
  expect(tabs[1]?.name).toContain("beta");
  expect(tabs[1]?.selected).toBe(true);

  screen.getByRole("button", { name: "Close terminal beta" }).click();
  expect(screen.getAllByRole("tab")).toHaveLength(1);
  expect(screen.getAllByRole("tab")[0]?.name).toContain("alpha");
  expect(screen.getAllByRole("tab")[0]?.selected).toBe(true);
  expect(closed).toBe(0);

  screen.getByRole("button", { name: "Close terminal alpha" }).click();
  expect(closed).toBe(1);
});

test("offers terminal lifecycle actions from a secondary click", () => {
  const screen = renderComponent(() => (
    <AgentTerminalPanel
      cwd="/work/alpha"
      open
      close={() => {}}
      dispose={() => {}}
    />
  ));

  screen.getByRole("group", { name: "Terminal 1 surface" }).contextMenu();
  screen.getByRole("menuitem", { name: "New terminal" }).click();
  expect(screen.getAllByRole("tab")).toHaveLength(2);

  screen.getByRole("group", { name: "Terminal 1 surface" }).contextMenu();
  screen.getByRole("menuitem", { name: "Close terminal" }).click();
  expect(screen.getAllByRole("tab")).toHaveLength(1);
});
