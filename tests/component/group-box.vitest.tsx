import { renderComponent } from "@wabou/test/component";
import { GroupBox, SettingsItem, Switch } from "@wabou/ui";
import { expect, test } from "vitest";

test("group box owns its title, description and surface variant", () => {
  const screen = renderComponent(() => (
    <GroupBox
      title="Updates"
      description="Choose how application updates are installed."
      variant="outline"
    >
      <SettingsItem title="Automatic updates">
        <Switch aria-label="Automatic updates" />
      </SettingsItem>
    </GroupBox>
  ));

  const group = screen.getByRole("group", { name: "Updates" });
  expect(group.className).toContain("gap-3");
  expect(screen.getByRole("heading", { name: "Updates" })).toBeDefined();
  expect(group.children.at(-1)?.className).toContain("border-subtle");
  expect(group.children.at(-1)?.className).toContain("p-4");
  expect(
    screen.getByRole("switch", { name: "Automatic updates" }),
  ).toBeDefined();
});

test("normal group boxes remain lightweight and infer no false label", () => {
  const screen = renderComponent(() => (
    <GroupBox title={<Switch aria-label="Title control" />}>
      <Switch aria-label="Content control" />
    </GroupBox>
  ));

  const group = screen.getAllByRole("group")[0];
  expect(group.name).toBe("");
  expect(group.className).toContain("gap-4");
  expect(group.children.at(-1)?.className).toContain("bg-transparent");
  expect(group.children.at(-1)?.className.split(/\s+/)).not.toContain("p-4");
});
