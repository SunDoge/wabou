import { renderComponent } from "@wabou/test/component";
import {
  Input,
  SettingsGroup,
  SettingsItem,
  SettingsSection,
  Switch,
} from "@wabou/ui";
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

test("settings items own orientation, control placement and disabled state", () => {
  const screen = renderComponent(() => (
    <SettingsGroup title="Runtime">
      <SettingsItem
        title="Automatic updates"
        description="Download updates in the background."
      >
        <Switch aria-label="Automatic updates" defaultChecked />
      </SettingsItem>
      <SettingsItem
        title="Proxy URL"
        description="Applied to package installation and model requests."
        orientation="vertical"
        disabled
        controlClass="w-full"
      >
        <Input aria-label="Proxy URL" disabled />
      </SettingsItem>
    </SettingsGroup>
  ));

  const updates = screen.getByRole("group", { name: "Automatic updates" });
  expect(updates.orientation).toBe("horizontal");
  expect(updates.className).toContain("justify-between");
  expect(
    screen.getByRole("switch", { name: "Automatic updates" }),
  ).toBeDefined();

  const proxy = screen.getByRole("group", { name: "Proxy URL" });
  expect(proxy.orientation).toBe("vertical");
  expect(proxy.disabled).toBe(true);
  expect(proxy.interactionBlocked).toBe(true);
  expect(proxy.style("opacity")).toBe("0.45");
  expect(screen.getByRole("textbox", { name: "Proxy URL" }).disabled).toBe(
    true,
  );
});
