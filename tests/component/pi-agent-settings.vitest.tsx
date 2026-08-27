import { renderComponent } from "@wabou/test/component";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import {
  SettingsPage,
  type AgentDefaults,
} from "../../apps/pi-agent/ui/settings";
import { createAgentWorkspace } from "../../apps/pi-agent/ui/workspace";

test("Pi Agent settings rename and explicitly confirm agent deletion", () => {
  const defaults: AgentDefaults = {
    proxy: "",
    noProxy: "127.0.0.1,localhost",
    provider: "",
    model: "",
  };
  const [agent, setAgent] = createSignal(createAgentWorkspace(1));
  let deleted = 0;
  const screen = renderComponent(() => (
    <SettingsPage
      value={defaults}
      update={() => {}}
      agent={agent()}
      updateAgent={(patch) =>
        setAgent((current) => ({ ...current, ...patch }))
      }
      close={() => {}}
      deleteAgent={() => deleted++}
    />
  ));

  screen.getByRole("textbox", { name: "Agent name" }).input("Build agent");
  expect(agent().name).toBe("Build agent");

  screen.getByRole("button", { name: "Delete agent" }).click();
  const dialog = screen.getByRole("alertdialog", { name: "Delete agent" });
  expect(dialog.text).toContain("Delete Build agent?");
  dialog.getByRole("button", { name: "Delete agent" }).click();
  expect(deleted).toBe(1);
});
