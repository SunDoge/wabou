import { renderComponent } from "@wabou/test/component";
import {
  Checkbox,
  RadioGroup,
  RadioGroupItem,
  Text,
  Toggle,
  ToggleGroup,
  ToggleGroupItem,
  View,
} from "@wabou/ui";
import { createSignal } from "solid-js";
import { expect, test } from "vitest";

test("supports uncontrolled, controlled, mixed, and disabled checkboxes", () => {
  const App = () => {
    const [checked, setChecked] = createSignal(false);
    return (
      <View>
        <Checkbox label="Uncontrolled" />
        <Checkbox
          checked={checked()}
          onCheckedChange={setChecked}
          label="Controlled"
        />
        <Checkbox indeterminate label="Mixed" />
        <Checkbox disabled label="Disabled" />
      </View>
    );
  };
  const screen = renderComponent(App);
  const uncontrolled = screen.getByRole("checkbox", {
    name: "Uncontrolled",
  });
  const controlled = screen.getByRole("checkbox", { name: "Controlled" });

  expect(uncontrolled.checked).toBe(false);
  uncontrolled.click();
  expect(uncontrolled.checked).toBe(true);
  controlled.click();
  expect(controlled.checked).toBe(true);
  expect(screen.getByRole("checkbox", { name: "Mixed" }).checked).toBe("mixed");
  expect(
    screen.getByRole("checkbox", { name: "Disabled", disabled: true }),
  ).not.toBeNull();
});

test("selection controls separate compact indicators from stable targets and wrapping labels", () => {
  const screen = renderComponent(() => (
    <View>
      <Checkbox aria-label="Icon-only choice" />
      <Checkbox label="A deliberately long checkbox label" />
      <RadioGroup aria-label="Policy">
        <RadioGroupItem value="system" aria-label="Icon-only policy" />
        <RadioGroupItem
          value="automatic"
          label="A deliberately long radio label"
        />
      </RadioGroup>
    </View>
  ));

  expect(
    screen.getByRole("checkbox", { name: "Icon-only choice" }).className,
  ).toContain("w-10 h-7");
  expect(
    screen.getByRole("checkbox", {
      name: "A deliberately long checkbox label",
    }).className,
  ).toContain("items-start");
  expect(
    screen.getByRole("radio", {
      name: "A deliberately long radio label",
    }).className,
  ).toContain("items-start");
  expect(
    screen.getByRole("radio", { name: "Icon-only policy" }).className,
  ).toContain("w-10 h-7");
});

test("selects radio items and skips disabled choices during roving focus", () => {
  const App = () => {
    const [plan, setPlan] = createSignal("free");
    return (
      <View>
        <RadioGroup
          value={plan()}
          onValueChange={setPlan}
          aria-label="Subscription plan"
        >
          <RadioGroupItem value="free" label="Free" />
          <RadioGroupItem value="pro" label="Pro" disabled />
          <RadioGroupItem value="team" label="Team" />
        </RadioGroup>
        <Text role="status">{plan()}</Text>
      </View>
    );
  };
  const screen = renderComponent(App);
  const free = screen.getByRole("radio", { name: "Free" });
  const pro = screen.getByRole("radio", { name: "Pro", disabled: true });
  const team = screen.getByRole("radio", { name: "Team" });

  free.focus();
  free.press("ArrowDown");

  expect(pro.focused).toBe(false);
  expect(team.focused).toBe(true);
  expect(team.checked).toBe(true);
  expect(screen.getByRole("status").text).toBe("team");
});

test("keeps a controlled radio group stable until its owner accepts a change", () => {
  const changes: string[] = [];
  const screen = renderComponent(() => (
    <RadioGroup
      value="free"
      onValueChange={(value) => changes.push(value)}
      aria-label="Plan"
    >
      <RadioGroupItem value="free" label="Free" />
      <RadioGroupItem value="team" label="Team" />
    </RadioGroup>
  ));

  screen.getByRole("radio", { name: "Team" }).click();

  expect(changes).toEqual(["team"]);
  expect(screen.getByRole("radio", { name: "Free" }).checked).toBe(true);
});

test("renders a required horizontal radio choice as a segmented control", () => {
  const [locale, setLocale] = createSignal("en");
  const screen = renderComponent(() => (
    <RadioGroup
      appearance="segment"
      orientation="horizontal"
      loop
      value={locale()}
      onValueChange={setLocale}
      aria-label="Language"
    >
      <RadioGroupItem value="en" label="English" />
      <RadioGroupItem value="zh" label="Chinese" />
    </RadioGroup>
  ));

  const group = screen.getByRole("radiogroup", { name: "Language" });
  expect(group.className).toContain("flex-row");
  const english = screen.getByRole("radio", { name: "English" });
  const chinese = screen.getByRole("radio", { name: "Chinese" });
  expect(english.checked).toBe(true);
  expect(english.className).toContain("bg-selected");
  english.focus();
  english.press("ArrowRight");
  expect(chinese.focused).toBe(true);
  expect(chinese.checked).toBe(true);
  chinese.click();
  expect(chinese.checked).toBe(true);
});

test("toggles buttons and single-value groups through semantic state", () => {
  const App = () => {
    const [bold, setBold] = createSignal(false);
    const [alignment, setAlignment] = createSignal("left");
    return (
      <View>
        <Toggle pressed={bold()} onPressedChange={setBold} aria-label="Bold">
          B
        </Toggle>
        <Toggle size="sm" aria-label="Compact toggle">
          S
        </Toggle>
        <ToggleGroup
          type="single"
          value={alignment()}
          onValueChange={setAlignment}
          aria-label="Alignment"
        >
          <ToggleGroupItem value="left">Left</ToggleGroupItem>
          <ToggleGroupItem value="center" disabled>
            Center
          </ToggleGroupItem>
          <ToggleGroupItem value="right">Right</ToggleGroupItem>
        </ToggleGroup>
      </View>
    );
  };
  const screen = renderComponent(App);
  const bold = screen.getByRole("button", { name: "Bold" });
  const compact = screen.getByRole("button", { name: "Compact toggle" });
  const left = screen.getByRole("button", { name: "Left" });
  const center = screen.getByRole("button", {
    name: "Center",
    disabled: true,
  });
  const right = screen.getByRole("button", { name: "Right" });

  bold.click();
  expect(bold.pressed).toBe(true);
  expect(bold.className).toContain("h-8");
  expect(compact.className).toContain("h-7");
  expect(compact.className).not.toContain("h-6");
  left.focus();
  left.press("ArrowRight");

  expect(center.focused).toBe(false);
  expect(right.focused).toBe(true);
  expect(right.pressed).toBe(true);
});
