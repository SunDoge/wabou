import { renderComponent } from "@wabou/test/component";
import { Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";
import { CommandPicker } from "../../apps/pi-agent/ui/command-picker";

test("discovers and inserts commands supplied by Pi", () => {
  const App = () => {
    const [draft, setDraft] = createSignal("");
    return (
      <>
        <CommandPicker
          commands={[
            {
              name: "fix-tests",
              description: "Fix failing tests",
              source: "prompt",
            },
            { name: "skill:review", source: "skill" },
          ]}
          choose={setDraft}
        />
        <Text role="status">{draft()}</Text>
      </>
    );
  };
  const screen = renderComponent(() => <App />);

  screen.getByRole("button", { name: "Commands" }).click();
  screen.getByRole("textbox", { name: "Search commands" }).input("review");
  screen.getByRole("option", { name: "/skill:review" }).click();

  expect(screen.getByRole("status").text).toBe("/skill:review ");
  expect(screen.queryByRole("listbox")).toBeNull();
});

test("enables the picker when Pi supplies commands after startup", () => {
  let supplyCommands!: () => void;
  const App = () => {
    const [commands, setCommands] = createSignal<
      Parameters<typeof CommandPicker>[0]["commands"]
    >([]);
    supplyCommands = () =>
      setCommands([
        {
          name: "subagents",
          description: "Administer isolated Pi subagents",
          source: "extension",
        },
      ]);
    return <CommandPicker commands={commands()} choose={() => undefined} />;
  };
  const screen = renderComponent(() => <App />);
  const trigger = screen.getByRole("button", { name: "Commands" });

  expect(trigger.disabled).toBe(true);
  supplyCommands();
  screen.flush();
  const enabledTrigger = screen.getByRole("button", { name: "Commands" });
  expect(enabledTrigger.disabled).toBe(false);
  enabledTrigger.click();
  expect(screen.getByRole("option", { name: "/subagents" })).not.toBeNull();
});
