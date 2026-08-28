import { renderComponent } from "@wabou/test/component";
import { Tabs, TabsContent, TabsList, TabsTrigger, Text } from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

const ExampleTabs = (props: {
  value?: string;
  defaultValue?: string;
  orientation?: "horizontal" | "vertical";
  onValueChange?: (value: string) => void;
}) => (
  <Tabs {...props}>
    <TabsList aria-label="Project sections">
      <TabsTrigger value="overview">Overview</TabsTrigger>
      <TabsTrigger value="activity" disabled>
        Activity
      </TabsTrigger>
      <TabsTrigger value="settings">Settings</TabsTrigger>
    </TabsList>
    <TabsContent value="overview">
      <Text>Overview panel</Text>
    </TabsContent>
    <TabsContent value="activity">
      <Text>Activity panel</Text>
    </TabsContent>
    <TabsContent value="settings">
      <Text>Settings panel</Text>
    </TabsContent>
  </Tabs>
);

test("selects tabs and publishes only the active panel", () => {
  const screen = renderComponent(() => <ExampleTabs defaultValue="overview" />);
  const overview = screen.getByRole("tab", { name: "Overview" });
  const settings = screen.getByRole("tab", { name: "Settings" });

  expect(
    screen.getByRole("tablist", { name: "Project sections" }).orientation,
  ).toBe("horizontal");
  expect(
    screen.getByRole("tablist", { name: "Project sections" }).className,
  ).toContain("rounded-lg");
  expect(overview.className).toContain("rounded-md");
  expect(overview.selected).toBe(true);
  expect(screen.getByRole("tabpanel").text).toBe("Overview panel");

  settings.click();

  expect(overview.selected).toBe(false);
  expect(settings.selected).toBe(true);
  expect(screen.getByRole("tabpanel").text).toBe("Settings panel");
});

test("moves focus and selection while skipping disabled tabs", () => {
  const screen = renderComponent(() => <ExampleTabs defaultValue="overview" />);
  const overview = screen.getByRole("tab", { name: "Overview" });
  const activity = screen.getByRole("tab", {
    name: "Activity",
    disabled: true,
  });
  const settings = screen.getByRole("tab", { name: "Settings" });

  overview.focus();
  overview.press("ArrowRight");

  expect(activity.focused).toBe(false);
  expect(settings.focused).toBe(true);
  expect(settings.selected).toBe(true);
  expect(screen.getByRole("tabpanel").text).toBe("Settings panel");
});

test("requests controlled changes without changing the active panel", () => {
  const changes: string[] = [];
  const screen = renderComponent(() => (
    <ExampleTabs
      value="overview"
      onValueChange={(value) => changes.push(value)}
    />
  ));

  screen.getByRole("tab", { name: "Settings" }).click();

  expect(changes).toEqual(["settings"]);
  expect(screen.getByRole("tab", { name: "Overview" }).selected).toBe(true);
  expect(screen.getByRole("tabpanel").text).toBe("Overview panel");
});

test("follows vertical arrow-key orientation", () => {
  const App = () => {
    const [value, setValue] = createSignal("overview");
    return (
      <ExampleTabs
        value={value()}
        orientation="vertical"
        onValueChange={setValue}
      />
    );
  };
  const screen = renderComponent(App);
  const overview = screen.getByRole("tab", { name: "Overview" });

  expect(screen.getByRole("tablist").orientation).toBe("vertical");
  overview.focus();
  overview.press("ArrowDown");

  expect(screen.getByRole("tab", { name: "Settings" }).selected).toBe(true);
});
