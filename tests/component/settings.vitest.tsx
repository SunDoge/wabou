import { renderComponent } from "@wabou/test/component";
import { Input, SettingsGroup, SettingsSection } from "@wabou/ui";
import { expect, test } from "vitest";

test("settings sections own repeated hierarchy and responsive geometry", () => {
  const screen = renderComponent(() => (
    <SettingsSection
      title="Network"
      description="Defaults shared by every project."
      stacked
      contentClass="border-danger"
    >
      <SettingsGroup title="Proxy" description="Used by child processes.">
        <Input aria-label="Proxy URL" />
      </SettingsGroup>
    </SettingsSection>
  ));

  const section = screen.getByRole("group", { name: "Network" });
  const group = screen.getByRole("group", { name: "Proxy" });
  expect(section.className).toContain("flex-col");
  expect(section.text).toContain("Defaults shared by every project.");
  expect(group.text).toContain("Used by child processes.");
  expect(screen.getByRole("textbox", { name: "Proxy URL" })).toBeDefined();
  expect(group.parent?.className).toContain("border-danger");
});
