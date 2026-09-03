import { renderComponent } from "@wabou/test/component";
import {
  Tabs,
  TabsContent,
  TabsItem,
  TabsList,
  TabsTrigger,
  Text,
} from "@wabou/ui";
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
  expect(overview.className).toContain("flex-none");
  expect(overview.className).toContain("whitespace-nowrap");
  expect(overview.selected).toBe(true);
  expect(overview.focusOrder).toBe(0);
  expect(settings.focusOrder).toBe(-1);
  const disabled = screen.getByRole("tab", { name: "Activity" });
  expect(disabled.className).toContain("cursor-not-allowed");
  expect(disabled.className).toContain("opacity-60");
  const panel = screen.getByRole("tabpanel");
  expect(panel.text).toBe("Overview panel");
  expect(panel.className).toContain("w-full");
  expect(panel.className).toContain("min-w-0");
  expect(panel.className).toContain("flex-none");
  expect(panel.className).toContain("flex-col");

  settings.click();

  expect(overview.selected).toBe(false);
  expect(overview.focusOrder).toBe(-1);
  expect(settings.selected).toBe(true);
  expect(settings.focusOrder).toBe(0);
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
  expect(screen.getByRole("tabpanel").className).toContain("flex-1");
  expect(screen.getByRole("tabpanel").className).not.toContain("w-full");
  overview.focus();
  overview.press("ArrowDown");

  expect(screen.getByRole("tab", { name: "Settings" }).selected).toBe(true);
});

test("chooses the first enabled tab when no initial value is provided", () => {
  const changes: string[] = [];
  const screen = renderComponent(() => (
    <Tabs onValueChange={(value) => changes.push(value)}>
      <TabsList aria-label="Fallback selection">
        <TabsTrigger value="disabled" disabled>
          Disabled
        </TabsTrigger>
        <TabsTrigger value="available">Available</TabsTrigger>
      </TabsList>
      <TabsContent value="available">
        <Text>Available panel</Text>
      </TabsContent>
    </Tabs>
  ));

  expect(screen.getByRole("tab", { name: "Disabled" }).selected).toBe(false);
  expect(screen.getByRole("tab", { name: "Disabled" }).focusOrder).toBe(-1);
  expect(screen.getByRole("tab", { name: "Available" }).selected).toBe(true);
  expect(screen.getByRole("tab", { name: "Available" }).focusOrder).toBe(0);
  expect(screen.getByRole("tabpanel").text).toBe("Available panel");
  expect(changes).toEqual(["available"]);
});

test("keeps native tab panels mounted while hiding inactive content", () => {
  const App = () => {
    const [value, setValue] = createSignal("one");
    return (
      <Tabs value={value()} onValueChange={setValue}>
        <TabsList aria-label="Native sessions">
          <TabsTrigger value="one">One</TabsTrigger>
          <TabsTrigger value="two">Two</TabsTrigger>
        </TabsList>
        <TabsContent value="one" keepMounted>
          <Text role="status" aria-label="Native session one">
            First native session
          </Text>
        </TabsContent>
        <TabsContent value="two" keepMounted>
          <Text role="status" aria-label="Native session two">
            Second native session
          </Text>
        </TabsContent>
      </Tabs>
    );
  };
  const screen = renderComponent(App);
  const first = screen.getByRole("status", { name: "Native session one" });
  const firstIdentity = first.identity;
  const panels = screen.getAllByRole("tabpanel");

  expect(panels).toHaveLength(2);
  expect(panels[0]?.attribute("aria-hidden")).toBe("false");
  expect(panels[1]?.attribute("aria-hidden")).toBe("true");

  screen.getByRole("tab", { name: "Two" }).click();

  expect(
    screen.getByRole("status", { name: "Native session one" }).identity,
  ).toEqual(firstIdentity);
  expect(panels[0]?.attribute("aria-hidden")).toBe("true");
  expect(panels[1]?.attribute("aria-hidden")).toBe("false");
});

test("closeable tab items keep selection and close actions independent", () => {
  const [value, setValue] = createSignal("one");
  const closed: string[] = [];
  const screen = renderComponent(() => (
    <Tabs value={value()} onValueChange={setValue}>
      <TabsList unstyled aria-label="Documents">
        <TabsItem value="one" onClose={() => closed.push("one")}>
          <Text class="truncate">One</Text>
        </TabsItem>
        <TabsItem value="two" onClose={() => closed.push("two")}>
          <Text class="truncate">A very long second document</Text>
        </TabsItem>
      </TabsList>
    </Tabs>
  ));

  const second = screen.getByRole("tab", {
    name: "A very long second document",
  });
  expect(second.parent?.className).toContain("max-w-56");
  expect(second.parent?.className).toContain("overflow-hidden");

  screen.getByRole("button", { name: "Close two" }).click();
  expect(closed).toEqual(["two"]);
  expect(screen.getByRole("tab", { name: "One" }).selected).toBe(true);

  second.click();
  expect(second.selected).toBe(true);
});
