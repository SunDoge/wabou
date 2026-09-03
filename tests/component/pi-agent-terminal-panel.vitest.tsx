import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import { AgentTerminalPanel } from "../../apps/pi-agent/ui/terminal-panel";

test("reacts to retained panel visibility without remounting the terminal", () => {
  const [open, setOpen] = createSignal(true);
  const screen = renderComponent(() => (
    <AgentTerminalPanel
      cwd="/work/alpha"
      open={open()}
      close={() => setOpen(false)}
      dispose={() => {}}
    />
  ));
  const panel = screen.getByRole("region", { name: "Terminal panel" });
  const terminal = screen.getByRole("textbox", { name: "Terminal 1" });

  expect(panel.className.split(/\s+/u)).not.toContain("hidden");
  screen.getByRole("button", { name: "Close terminal panel" }).click();
  expect(panel.className.split(/\s+/u)).toContain("hidden");
  expect(screen.getByRole("textbox", { name: "Terminal 1" }).identity).toEqual(
    terminal.identity,
  );
});

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
  const terminal = screen.getByRole("textbox", { name: "Terminal 1" });
  expect(terminal.className).not.toContain("rounded");
  expect(terminal.parent?.parent?.className).not.toContain("p-1");
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
  expect(screen.getAllByRole("textbox")).toHaveLength(2);
  expect(screen.getByRole("textbox", { name: "Terminal 1" }).identity).toEqual(
    terminal.identity,
  );
  tabs[0]?.hover();
  expect(tabs[0]?.className).toContain("bg-slate-800");
  tabs[0]?.unhover();
  expect(tabs[0]?.className).not.toContain("bg-slate-800");
  tabs[0]?.click();
  tabs[0]?.press("ArrowRight");
  expect(tabs[1]?.selected).toBe(true);
  expect(screen.getByRole("textbox", { name: "Terminal 1" }).identity).toEqual(
    terminal.identity,
  );

  screen.getByRole("button", { name: "Close terminal 2" }).click();
  expect(screen.getAllByRole("tab")).toHaveLength(1);
  expect(screen.getAllByRole("tab")[0]?.name).toContain("alpha");
  expect(screen.getAllByRole("tab")[0]?.selected).toBe(true);
  expect(closed).toBe(0);

  screen.getByRole("button", { name: "Close terminal 1" }).click();
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

test("preserves the native terminal when its process updates the title", () => {
  const screen = renderComponent(() => (
    <AgentTerminalPanel
      cwd="/work/alpha"
      open
      close={() => {}}
      dispose={() => {}}
    />
  ));
  const terminal = screen.getByRole("textbox", { name: "Terminal 1" });
  const identity = terminal.identity;

  terminal.emit("terminaltitlechange", {
    title: "fish · alpha",
    subtitle: null,
  });

  expect(screen.getByRole("tab").name).toContain("fish · alpha");
  expect(screen.getByRole("textbox", { name: "Terminal 1" }).identity).toEqual(
    identity,
  );
});
